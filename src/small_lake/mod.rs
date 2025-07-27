use crate::lake::memory::LakeTools;

#[repr(C)]
#[derive(Clone)]
pub struct SmallLake<const N: usize> {
    pub buf: [u8; N],
    pub pos: usize,
}

impl<const N: usize> SmallLake<N> {
    #[inline(always)]
    pub const fn build() -> Self {
        Self {
            buf: [0u8; N],
            pos: 0,
        }
    }
    #[inline(always)]
    pub fn reset_pos(&mut self) {
        self.pos = 0;
    }
    #[inline(always)]
    pub unsafe fn write_byte(&mut self, c: u8) {
        *self.buf.as_mut_ptr().add(self.pos) = c;
        self.pos += 1;
        // Ring behavior
        if self.pos >= self.buf.len() {
            self.pos = 0;
        }
    }
    #[inline(always)]
    pub fn freeze_ref(&mut self) -> &Self {
        &*self
    }
    #[inline(always)]
    pub fn freeze_ptr(&self) -> *const Self {
        self as *const Self
    }
    #[inline(always)]
    pub unsafe fn as_slice(&self) -> &[u8] {
        &self.buf[..self.pos]
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.pos
    }
    #[inline(always)]
    pub unsafe fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }
    #[inline(always)]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buf.as_mut_ptr().add(self.pos)
    }
    #[inline(always)]
    pub unsafe fn write(&mut self, src: *const u8, len: usize) {
        let buf_len: usize = self.buf.len();
        if len > buf_len {
            panic!("DataLake overflow: trying to write {len}, but buffer is only {buf_len}");
        }
        let dst: *mut u8 = if len <= (buf_len - self.pos) {
            self.buf.as_mut_ptr().add(self.pos)
        } else {
            self.pos = 0;
            self.buf.as_mut_ptr()
        };
        LakeTools::write_to(dst, src, len);
        self.pos += len;
    }
    #[inline(always)]
    pub unsafe fn write_num_str(&mut self, mut value: usize) {
        let mut tmp: [u8; 20] = [0u8; 20];
        let mut curr: usize = tmp.len();

        while value >= 10 {
            let rem: usize = value % 10;
            value /= 10;
            curr -= 1;
            *tmp.get_unchecked_mut(curr) = (rem as u8) + b'0';
        }
        curr -= 1;
        *tmp.get_unchecked_mut(curr) = (value as u8) + b'0';

        let len = tmp.len() - curr;
        self.write(tmp.as_ptr().add(curr), len);
    }
    #[inline(always)]
    pub unsafe fn write_num_str_fixed(&mut self, mut value: usize, len: usize) {
        let dst: *mut u8 = self.buf.as_mut_ptr().add(self.pos + len);
        let mut ptr: *mut u8 = dst;

        for _ in 0..len {
            ptr = ptr.offset(-1);
            *ptr = ((value % 10) as u8) + b'0';
            value /= 10;
        }

        self.pos += len;
    }
    #[allow(dead_code)]
    #[inline(always)]
    fn into_raw_parts(mut self) -> (*mut u8, usize) {
        let ptr: *mut u8 = self.buf.as_mut_ptr();
        let len: usize = self.pos;
        core::mem::forget(self);
        (ptr, len)
    }
}