use crate::lake::droplet::droplet::Droplet;
use crate::lake::droplet::droplet_dyn::DropletDyn;
use crate::lake::lake::lake_view::LakeView;
use crate::lake::utils::align_up;
use crate::lake::{LakeAllocatorExt, LakeError, LakeMeta, LakeSnapshot};
use std::{marker::PhantomData, ptr::NonNull};

/// A preallocated memory arena called `Lake`, from which fixed- or variable-sized droplets are carved.
///
/// The lake is conceptually a fast linear allocator with snapshot and rewind capabilities.
/// You can think of it as a memory pool — but with personality:
/// - It grows no further. What you get is a still, peaceful lake, not a swampy heap.
/// - Droplets carved from it remember where they came from and can safely rewind the lake if they were the last ones in.
/// - It supports dynamic output (`process`), zero-copy allocations (`alloc`), and scoped rollback (`mark`/`reset_to_mark`).
///
/// Designed for **blazing fast allocation** of temporary buffers in pipelines, encoders, or servers.
/// And unlike regular allocators, it doesn’t leave junk behind or call the OS crying.
pub struct Lake<const SIZE: usize> {
    /// Our "water reservoir" – preallocated and boxed for stable address.
    pub(super) buf: Box<[u8; SIZE]>,
    /// Current fill level of the lake (offset from the beginning).
    pub(super) offset: usize,
    /// Stack of marks for scoped rewinds.
    pub(super) mark_stack: Vec<usize>,
    /// Generation counter to guard against stale droplets.
    pub(super) generation: usize,
    /// Wipe data with 0u8 while reset
    pub(super) zeroing: bool,
}

impl<const SIZE: usize> Lake<SIZE> {
    /// Create a new, pristine lake. Surface like glass, zero offset.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            buf: Box::new([0u8; SIZE]),
            offset: 0,
            mark_stack: Vec::with_capacity(100),
            generation: 0,
            zeroing: false,
        }
    }
    /// Take a snapshot of the lake's current water level.
    #[inline(always)]
    pub fn snapshot(&self) -> LakeSnapshot {
        LakeSnapshot {
            offset: self.offset,
        }
    }
    /// Rewind to a previous snapshot (rollback to known-safe state).
    #[inline(always)]
    pub fn rewind(&mut self, snapshot: LakeSnapshot) {
        self.offset = snapshot.offset;
    }
    /// Split off a `LakeView` — a sub-lake with its own internal memory.
    #[inline(always)]
    pub fn split(&mut self, len: usize) -> Result<LakeView<'static, SIZE>, LakeError> {
        if self.offset + len > SIZE {
            return Err(LakeError::Overflow);
        }

        let view = LakeView {
            buf: unsafe { self.buf.as_mut_ptr().add(self.offset) },
            capacity: len,
            offset: 0,
            mark_stack: Vec::new(),
            _marker: PhantomData,
            generation: 0,
            zeroing: self.zeroing
        };

        self.offset += len;
        Ok(view)
    }
    /// Invoke a closure to produce data into the lake, returning a dynamic droplet.
    /// Perfect for encoding, compression, or other on-the-fly data shaping.
    #[inline(always)]
    pub fn process<F>(&mut self, f: F) -> Result<DropletDyn<SIZE>, LakeError>
    where
        F: FnOnce(usize) -> Vec<u8>,
    {
        let remaining: usize = SIZE - self.offset;
        if remaining == 0 {
            return Err(LakeError::Overflow);
        }

        let offset: usize = self.offset;
        let generation: usize = self.generation;
        let dst: *mut u8 = unsafe { self.buf.as_mut_ptr().add(offset) };

        let data: Vec<u8> = f(remaining);

        let len: usize = data.len();

        if len > remaining {
            return Err(LakeError::Overflow);
        }

        // We trust the closure not to lie. Now copy the result into the lake.
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), dst, len);
        }

        self.offset += len;

        let lake: *mut dyn LakeMeta = self as *mut Self as *mut dyn LakeMeta;

        Ok(DropletDyn {
            ptr: unsafe { NonNull::new_unchecked(dst) },
            len,
            offset,
            lake,
            generation,
        })
    }
    /// Allocate a fixed-size droplet from the lake.
    /// No copying. No allocations. Just pure, raw, delicious speed.
    #[inline(always)]
    pub fn alloc<const N: usize>(&mut self) -> Option<Droplet<N, Lake<SIZE>>> {
        if self.offset + N > SIZE {
            return None;
        }
        let ptr: *mut [u8; N] = unsafe { self.buf.as_mut_ptr().add(self.offset) as *mut [u8; N] };
        let droplet = Droplet {
            ptr: NonNull::new(ptr)?,
            offset: self.offset + N,
            lake: self as *mut Self,
            generation: 0,
        };
        self.offset += N;
        Some(droplet)
    }
    /// Wipe the lake clean and start a new generation. Fresh waters.
    #[inline(always)]
    pub fn reset(&mut self) {
        if self.zeroing {
            unsafe {
                std::ptr::write_bytes(self.buf.as_mut_ptr(), 0, self.offset);
            }
        }
        self.offset = 0;
        self.mark_stack.clear();
        self.generation += 1;
    }
    /// Returns used capacity.
    #[inline(always)]
    pub fn used(&self) -> usize {
        self.offset
    }
    /// Returns remaining capacity.
    #[inline(always)]
    pub fn remaining(&self) -> usize {
        SIZE - self.offset
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.offset == 0
    }
    #[inline(always)]
    pub fn is_full(&self) -> bool {
        self.offset == SIZE
    }
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.offset]
    }
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buf[..self.offset]
    }
    /// Preview what the next allocation would look like.
    #[inline(always)]
    pub fn peek<const N: usize>(&self) -> Option<&[u8; N]> {
        if self.offset + N > SIZE {
            return None;
        }
        let ptr: *const [u8; N] = unsafe { self.buf.as_ptr().add(self.offset) as *const [u8; N] };
        Some(unsafe { &*ptr })
    }
    /// Rewind by N bytes. Careful: not validated.
    #[inline(always)]
    pub fn reset_to(&mut self, n: usize) {
        self.offset = self.offset.saturating_sub(n);
    }
    #[inline(always)]
    pub fn get_offset(&self) -> usize {
        self.used()
    }
    /// Mark the current position (you can come back to it later).
    #[inline(always)]
    pub fn mark(&mut self) {
        self.mark_stack.push(self.offset);
    }
    /// Roll back to last mark (if any).
    #[inline(always)]
    pub fn reset_to_mark(&mut self) {
        if let Some(mark) = self.mark_stack.pop() {
            self.offset = mark;
        }
    }
    /// Move the most recent mark to the current offset.
    #[inline(always)]
    pub fn move_mark(&mut self) {
        if let Some(last) = self.mark_stack.last_mut() {
            *last = self.offset;
        }
    }
    /// Wipe everything. Same as `reset`, but sounds more decisive.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.reset();
    }
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        SIZE
    }
    #[inline(always)]
    pub fn set_zeroing(&mut self, state: bool) {
        self.zeroing = state;
    }
    #[inline(always)]
    pub fn get_zeroing(&mut self) -> bool {
        self.zeroing
    }
}

impl<const N: usize> LakeAllocatorExt for Lake<N> {
    fn alloc_struct<T>(&mut self) -> &mut T {
        let align = align_of::<T>();
        let size = size_of::<T>();
        let offset = align_up(self.offset, align);
        if offset + size > N {
            panic!("Lake overflow");
        }
        let ptr: *mut T = self.buf[offset..].as_mut_ptr() as *mut T;
        self.offset = offset + size;
        unsafe { &mut *ptr }
    }

    fn alloc_slice<T>(&mut self, count: usize) -> &mut [T] {
        let align = core::mem::align_of::<T>();
        let size = core::mem::size_of::<T>() * count;
        let offset = align_up(self.offset, align);
        if offset + size > N {
            panic!("Lake overflow");
        }

        let ptr = self.buf[offset..].as_mut_ptr() as *mut T;
        self.offset = offset + size;
        unsafe { core::slice::from_raw_parts_mut(ptr, count) }
    }
}

impl<const N: usize> LakeMeta for Lake<N> {
    fn offset(&self) -> usize {
        self.offset
    }
    fn capacity(&self) -> usize {
        self.capacity()
    }
    fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
    }
    fn generation(&self) -> usize {
        self.generation
    }
}
