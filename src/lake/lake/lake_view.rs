use crate::lake::{droplet::{droplet::Droplet, droplet_dyn::DropletDyn}, LakeAllocatorExt, LakeError, LakeMeta};
use std::{marker::PhantomData, ptr::NonNull};
use crate::lake::utils::align_up;

/// A view into a section of the lake — a *temporary tributary* or shallow basin
/// that lives within the larger memory lake but has its own offset and capacity.
///
/// `LakeView` behaves just like a full `Lake`, but its buffer is a borrowed slice:
/// - No allocation happens here — it’s all reused water.
/// - You can use `alloc` and `process` exactly like with a full lake.
/// - It can even `split` into subviews — yes, rivers can fork.
/// - Perfect for recursive algorithms, parser sub-states, or context-local arenas.
#[must_use]
pub struct LakeView<'a, const SIZE: usize> {
    /// Pointer to the beginning of the view’s water.
    pub buf: *mut u8,
    /// Total amount of water this view controls.
    pub(super) capacity: usize,
    /// How deep we’ve gone into this section.
    pub(super) offset: usize,
    /// Stack of memory marks for scoped rewinding.
    pub(super) mark_stack: Vec<usize>,
    /// Borrow marker – makes sure we don't outlive the parent lake.
    pub(super) _marker: PhantomData<&'a mut [u8]>,
    /// Generation counter to detect expired droplets.
    pub(super) generation: usize,

    pub(super) zeroing: bool,
}

impl<'a, const SIZE: usize> LakeView<'a, SIZE> {
    /// Create a new `LakeView` from a mutable slice.
    /// This is like dipping a ladle into the lake: you're not taking memory,
    /// you're just forming a smaller handle to it.
    #[inline(always)]
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self {
            buf: buf.as_mut_ptr(),
            capacity: buf.len(),
            offset: 0,
            mark_stack: Vec::new(),
            _marker: PhantomData,
            generation: 0,
            zeroing: false,
        }
    }
    /// Allocate a fixed-size droplet from this view. Just like in `Lake`,
    /// but bounded by the view’s own capacity.
    #[inline(always)]
    pub fn alloc<const N: usize>(&mut self) -> Option<Droplet<N, LakeView<'a, SIZE>>> {
        if self.offset + N > self.capacity {
            return None;
        }
        let ptr: *mut [u8; N] = unsafe { self.buf.add(self.offset) as *mut [u8; N] };

        let droplet = Droplet {
            ptr: NonNull::new(ptr)?,
            offset: self.offset + N,
            lake: self as *mut Self,
            generation: self.generation,
        };
        self.offset += N;
        Some(droplet)
    }
    /// Same idea as `Lake::process` — create a droplet dynamically
    /// by invoking a closure and copying its result into the lake.
    /// Useful for one-shot encoders, parsers, and temporary transformations.
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
        let dst: *mut u8 = unsafe { self.buf.add(offset) };

        let data: Vec<u8> = f(remaining);
        let len: usize = data.len();

        if len > remaining {
            return Err(LakeError::Overflow);
        }

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
    /// Fork this view into a subview.
    /// If `Lake` is a lake, then `LakeView` is a river — and this method is a tributary.
    #[inline(always)]
    pub fn split(&mut self, len: usize) -> Option<LakeView<'_, SIZE>> {
        if self.offset + len > self.capacity {
            return None;
        }

        let new_ptr: *mut u8 = unsafe { self.buf.add(self.offset) };
        let view = LakeView {
            buf: new_ptr,
            capacity: len,
            offset: 0,
            mark_stack: Vec::new(),
            _marker: PhantomData,
            generation: 0,
            zeroing: self.zeroing,
        };
        self.offset += len;
        Some(view)
    }
    /// Returns how much of the view’s buffer is currently used.
    #[inline(always)]
    pub fn used(&self) -> usize {
        self.offset
    }
    /// Returns how much space remains in this view.
    #[inline(always)]
    pub fn remaining(&self) -> usize {
        self.capacity - self.offset
    }
    /// Full capacity of the view.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    /// Clear the view — wipe offset and mark stack, increment generation.
    /// Think of this as letting the river run dry and rerouting it anew.
    #[inline(always)]
    pub fn reset(&mut self) {
        if self.zeroing {
            unsafe {
                std::ptr::write_bytes(self.buf, 0, self.offset);
            }
        }
        self.offset = 0;
        self.mark_stack.clear();
        self.generation += 1;
    }
    /// Push a mark to rewind to later.
    #[inline(always)]
    pub fn mark(&mut self) {
        self.mark_stack.push(self.offset);
    }
    /// Rewind to the most recent mark.
    #[inline(always)]
    pub fn reset_to_mark(&mut self) {
        if let Some(mark) = self.mark_stack.pop() {
            self.offset = mark;
        }
    }
    /// Update the latest mark to the current position.
    #[inline(always)]
    pub fn move_mark(&mut self) {
        if let Some(last) = self.mark_stack.last_mut() {
            *last = self.offset;
        }
    }
    /// Reset and call it a day.
    #[inline(always)]
    pub fn clear(&mut self) {
        self.reset();
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

impl<'a, const N: usize> LakeAllocatorExt for LakeView<'a, N> {
    fn alloc_struct<T>(&mut self) -> &mut T {
        let align: usize = align_of::<T>();
        let size: usize = size_of::<T>();
        let offset: usize = align_up(self.offset, align);

        if offset + size > self.capacity {
            panic!("LakeView overflow");
        }

        let ptr = unsafe { self.buf.add(offset) as *mut T };
        self.offset = offset + size;
        unsafe { &mut *ptr }
    }

    fn alloc_slice<T>(&mut self, count: usize) -> &mut [T] {
        let align = align_of::<T>();
        let size = size_of::<T>() * count;
        let offset = align_up(self.offset, align);

        if offset + size > self.capacity {
            panic!("LakeView overflow");
        }

        let ptr = unsafe { self.buf.add(offset) as *mut T };
        self.offset = offset + size;
        unsafe { core::slice::from_raw_parts_mut(ptr, count) }
    }
}

impl<const N: usize> LakeMeta for LakeView<'_, N> {
    fn offset(&self) -> usize {
        self.offset
    }
    fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
    }
    fn generation(&self) -> usize {
        self.generation
    }
    fn capacity(&self) -> usize {
        self.capacity()
    }
}
