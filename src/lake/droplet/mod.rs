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
    #[inline(always)]
    fn d_as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.d_as_ptr(), self.d_len()) }
    }
    #[inline(always)]
    fn d_as_str(&self) -> Option<&str> {
        std::str::from_utf8(self.d_as_slice()).ok()
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
}

#[macro_export]
macro_rules! guard {
    ($self:ident) => {
        if !$self.is_valid() {
            panic!("Droplet is outlive generation or no longer valid");
        }
    };
}
