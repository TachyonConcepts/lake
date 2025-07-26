use crate::{
    guard,
    lake::{
        droplet::{DropletBase, DropletDeserializeExt},
        LakeMeta,
    },
};
use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

/// A `Droplet` is a fixed-size memory fragment allocated from a `Lake`.
///
/// It retains a pointer to the allocated data and maintains a link back to the lake it came from.
/// This link allows it to verify its validity (`generation`, `offset`) and, if it's the most recent allocation,
/// automatically rewind the lake's offset on drop.
///
/// Think of it as a persistent memory "droplet" carved out of a larger "lake" —
/// it doesn’t own memory, but lives temporarily inside it with context-aware lifecycle management.
///
/// Safety and lifetime tracking are manual — the lake governs allocation, the droplet respects the flow.

#[must_use]
#[repr(C)]
pub struct Droplet<const N: usize, TARGET: LakeMeta> {
    pub(crate) ptr: NonNull<[u8; N]>,
    pub(crate) offset: usize,
    pub(crate) lake: *mut TARGET,
    pub(crate) generation: usize,
}

unsafe impl<const N: usize, TARGET: LakeMeta> Send for Droplet<N, TARGET> {}
unsafe impl<const N: usize, TARGET: LakeMeta> Sync for Droplet<N, TARGET> {}

impl<TARGET: LakeMeta, const N: usize> Droplet<N, TARGET> {
    /// Leak the droplet and obtain a `'static` reference.
    /// # Safety: you must guarantee that this will not outlive the Lake.
    #[inline(always)]
    pub unsafe fn leak(self) -> &'static [u8; N] {
        &*self.ptr.as_ptr()
    }
    /// Leak as mutable `'static` reference.
    #[inline(always)]
    pub unsafe fn leak_mut(self) -> &'static mut [u8; N] {
        &mut *self.ptr.as_ptr()
    }

    /// Access immutable reference to the lake.
    /// Yes, it's `unsafe`, because we're using raw pointers like it’s C again.
    unsafe fn get_lake(&self) -> &dyn LakeMeta {
        &*(self.lake as *const dyn LakeMeta)
    }
    /// Access mutable reference to the lake.
    /// Double `unsafe` bingo, hope you know what you’re doing.
    pub unsafe fn get_lake_mut(&mut self) -> &mut dyn LakeMeta {
        &mut *(self.lake)
    }
    /// Check whether the droplet is still valid.
    /// A tiny bit of sanity-checking in the middle of a memory free-for-all.
    pub fn is_valid(&self) -> bool {
        let lake: &dyn LakeMeta = unsafe { self.get_lake() };
        lake.generation() == self.generation && lake.offset() >= self.offset
    }

    #[inline(always)]
    pub fn get_lake_ptr(&self) -> *mut TARGET {
        self.lake
    }
}

impl<const N: usize, T: LakeMeta> DropletBase for Droplet<N, T> {
    #[inline(always)]
    fn d_as_ptr(&self) -> *const u8 {
        guard!(self);
        self.ptr.as_ptr().cast()
    }
    #[inline(always)]
    fn d_as_mut_ptr(&mut self) -> *mut u8 {
        guard!(self);
        self.ptr.as_ptr().cast()
    }
    #[inline(always)]
    fn d_len(&self) -> usize {
        guard!(self);
        N
    }
    #[inline(always)]
    fn d_as_mut_slice(&mut self) -> &mut [u8] {
        println!("is_valid = {:?}", self.is_valid());
        guard!(self);
        unsafe { &mut *self.ptr.as_ptr() }
    }
}

impl<const N: usize, T: LakeMeta> Deref for Droplet<N, T> {
    type Target = [u8; N];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        debug_assert!(self.is_valid(), "Droplet is outlive");
        unsafe { &*self.ptr.as_ptr() }
    }
}
impl<const N: usize, T: LakeMeta> DerefMut for Droplet<N, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        debug_assert!(self.is_valid(), "Droplet is outlive");
        unsafe { &mut *self.ptr.as_ptr() }
    }
}

impl<const N: usize, TARGET: LakeMeta> DropletDeserializeExt for Droplet<N, TARGET> {
    #[inline(always)]
    fn deserialize<T>(&self) -> Option<&T> {
        if size_of::<T>() > N {
            return None;
        }
        let ptr: *const T = self.as_ptr() as *const T;
        unsafe { Some(&*ptr) }
    }

    #[inline(always)]
    fn deserialize_slice<T>(&self) -> Option<&[T]> {
        let size: usize = size_of::<T>();
        if size == 0 || N % size != 0 {
            return None;
        }
        let ptr: *const T = self.as_ptr() as *const T;
        let count: usize = N / size;
        unsafe { Some(core::slice::from_raw_parts(ptr, count)) }
    }
}
