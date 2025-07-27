use std::backtrace::Backtrace;
use crate::lake::memory::LakeTools;
use std::ops::Add;

pub mod droplet;
pub mod droplet_dyn;

pub trait DropletDeserializeExt {
    fn deserialize<T: Copy>(&self) -> Option<&T>;
    fn deserialize_slice<T: Copy>(&self) -> Option<&[T]>;
}

pub trait DropletBase {
    fn d_as_ptr(&self) -> *const u8;
    fn d_as_mut_ptr(&mut self) -> *mut u8;
    fn d_len(&self) -> usize;
    fn d_as_mut_slice(&mut self) -> &mut [u8];
    fn d_offset_mut(&mut self) -> &mut usize;
    fn d_offset(&self) -> usize;
    fn d_reset(&mut self) {
        *self.d_offset_mut() = 0;
    }
    fn d_remaining(&self) -> usize {
        self.d_len() - self.d_offset()
    }
    #[inline(always)]
    fn d_as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.d_as_ptr(), self.d_len()) }
    }
    #[inline(always)]
    fn d_as_str(&self) -> Option<&str> {
        std::str::from_utf8(self.d_as_slice()).ok()
    }
    #[inline(always)]
    fn freeze_ref(&mut self) -> &Self {
        &*self
    }
    #[inline(always)]
    fn freeze_ptr(&self) -> *const Self {
        self as *const Self
    }
    #[inline(always)]
    fn d_as_slice_of<T>(&self) -> Option<&[T]> {
        let ptr = self.d_as_ptr();
        let align = align_of::<T>();
        let size = size_of::<T>();
        let len = self.d_len();
        if size == 0 || len % size != 0 || ptr.align_offset(align) != 0 {
            return None;
        }
        Some(unsafe { std::slice::from_raw_parts(ptr as *const T, len / size) })
    }

    // Utils
    #[inline(always)]
    fn d_write(&mut self, src: *const u8, len: usize) {
        let buf_len: usize = self.d_len();
        let remaining: usize = self.d_remaining();
        if len > buf_len || len > remaining {
            panic!("Droplet overflow: trying to write {len}, but buffer is only {buf_len} and remaining {remaining}\n{}", Backtrace::capture());
        }
        let dst: *mut u8 = unsafe { self.d_as_mut_ptr().add(self.d_offset()) };
        unsafe { LakeTools::write_to(dst, src, len) };
        *self.d_offset_mut() += len;
    }

    #[inline(always)]
    fn write_num_str(&mut self, mut value: usize) {
        let mut tmp: [u8; 20] = [0u8; 20];
        let mut curr: usize = tmp.len();
        while value >= 10 {
            let rem: usize = value % 10;
            value /= 10;
            curr -= 1;
            unsafe {
                *tmp.get_unchecked_mut(curr) = (rem as u8) + b'0';
            }
        }
        curr -= 1;
        unsafe {
            *tmp.get_unchecked_mut(curr) = (value as u8) + b'0';
        }
        let len: usize = tmp.len() - curr;
        unsafe { self.d_write(tmp.as_ptr().add(curr), len) };
        *self.d_offset_mut() += len;
    }

    #[inline(always)]
    fn write_num_str_fixed(&mut self, mut value: usize, len: usize) {
        unsafe {
            let dst: *mut u8 = self.d_as_mut_ptr().add(self.d_offset() + len);
            let mut ptr: *mut u8 = dst;
            for _ in 0..len {
                ptr = ptr.offset(-1);
                *ptr = ((value % 10) as u8) + b'0';
                value /= 10;
            }
        }
        *self.d_offset_mut() += len;
    }
    #[inline(always)]
    fn write_byte(&mut self, c: u8) {
        unsafe {
            *self.d_as_mut_ptr().add(self.d_offset()) = c;
            *self.d_offset_mut() += 1;
        }
    }
}

#[macro_export]
macro_rules! guard {
    ($self:ident) => {
        if !$self.is_valid() {
            panic!("Droplet is outlive generation or no longer valid");
        }
    };
}
