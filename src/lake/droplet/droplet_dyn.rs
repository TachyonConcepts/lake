use crate::lake::droplet::DropletDeserializeExt;
use crate::{
    guard,
    lake::{droplet::DropletBase, LakeMeta},
};
use std::ptr::NonNull;

/// A dynamically sized `Droplet` carved from a `Lake`.
///
/// Unlike the fixed-size variant, this one can hold any slice up to `SIZE`,
/// making it ideal for data generated at runtime (e.g. JSON, HTTP bodies).
///
/// The droplet keeps a raw pointer back to its lake (yes, Rust, we're adults),
/// enabling generation/offset-based validity checks and automatic rewind on drop.
///
/// It's not a smart pointer — it's a clever pointer.
#[must_use]
pub struct DropletDyn<const SIZE: usize> {
    /// Pointer to the beginning of the data (may not be aligned).
    pub ptr: NonNull<u8>,
    /// Actual length of the slice in use (≤ SIZE).
    pub len: usize,
    /// Offset into the lake buffer.
    pub(crate) offset: usize,
    /// Raw link back to the lake (don’t tell borrow checker).
    pub(crate) lake: *mut dyn LakeMeta,
    /// Generation to guard against stale reuse.
    pub(crate) generation: usize,
}

impl<const SIZE: usize> DropletBase for DropletDyn<SIZE> {
    #[inline(always)]
    fn d_as_ptr(&self) -> *const u8 {
        guard!(self);
        self.ptr.as_ptr()
    }
    #[inline(always)]
    fn d_as_mut_ptr(&mut self) -> *mut u8 {
        guard!(self);
        self.ptr.as_ptr()
    }
    #[inline(always)]
    fn d_len(&self) -> usize {
        guard!(self);
        self.len
    }
    #[inline(always)]
    fn d_as_mut_slice(&mut self) -> &mut [u8] {
        guard!(self);
        // Safety: ditto
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl<const SIZE: usize> DropletDyn<SIZE> {
    /// Leak the dynamic droplet into a `'static` slice.
    /// # Safety: you must guarantee that the backing memory lives forever.
    #[inline(always)]
    pub unsafe fn leak(self) -> &'static [u8] {
        std::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
    }

    /// Leak as mutable `'static` slice.
    #[inline(always)]
    pub unsafe fn leak_mut(self) -> &'static mut [u8] {
        std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
    }
    /// Returns a shared reference to the lake this droplet came from.
    /// Unsafe because we bypass lifetimes (with great power… etc).
    unsafe fn get_lake(&self) -> &dyn LakeMeta {
        &*(self.lake as *const dyn LakeMeta)
    }
    /// Returns a mutable reference to the lake (see above).
    #[allow(dead_code)]
    unsafe fn get_lake_mut(&mut self) -> &mut dyn LakeMeta {
        guard!(self);
        &mut *(self.lake)
    }
    /// Checks whether the droplet is still valid:
    /// - Same generation
    /// - Not yet overwritten in the lake
    pub fn is_valid(&self) -> bool {
        let lake: &dyn LakeMeta = unsafe { self.get_lake() };
        lake.generation() == self.generation && lake.offset() >= self.offset
    }
}

impl<const SIZE: usize> DropletDeserializeExt for DropletDyn<SIZE> {
    #[inline(always)]
    fn deserialize<T>(&self) -> Option<&T> {
        if self.d_len() < size_of::<T>() {
            return None;
        }
        let ptr = self.d_as_ptr() as *const T;
        unsafe { Some(&*ptr) }
    }

    #[inline(always)]
    fn deserialize_slice<T>(&self) -> Option<&[T]> {
        let size = size_of::<T>();
        if size == 0 || self.d_len() % size != 0 {
            return None;
        }
        let ptr = self.d_as_ptr() as *const T;
        let count = self.d_len() / size;
        unsafe { Some(core::slice::from_raw_parts(ptr, count)) }
    }
}
