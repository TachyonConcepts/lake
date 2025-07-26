use crate::lake::lake::sandbox::SandboxGuard;

pub mod droplet;
pub mod lake;
pub mod utils;
pub mod memory;

pub use droplet::DropletBase;

pub trait LakeAllocatorExt: LakeMeta {
    fn alloc_struct<T>(&mut self) -> &mut T;
    fn alloc_slice<T>(&mut self, count: usize) -> &mut [T];
}

#[derive(Debug, Clone, Copy)]
pub struct LakeStats {
    pub used: usize,
    pub remaining: usize,
    pub capacity: usize,
    pub generation: usize,
}

pub trait LakeMeta {
    fn offset(&self) -> usize;
    fn offset_mut(&mut self) -> &mut usize;
    fn generation(&self) -> usize;
    #[inline(always)]
    fn set_offset(&mut self, val: usize) {
        *self.offset_mut() = val;
    }
    fn capacity(&self) -> usize;
    fn stats(&self) -> LakeStats {
        LakeStats {
            used: self.offset(),
            remaining: self.capacity() - self.offset(),
            capacity: self.capacity(),
            generation: self.generation(),
        }
    }
}

pub trait LakeSandboxExt: LakeMeta {
    fn sandbox(&mut self) -> SandboxGuard<'_, Self>
    where
        Self: Sized,
    {
        let offset: usize = self.offset();
        SandboxGuard {
            lake: Some(self),
            base_offset: offset,
            committed: false,
        }
    }
}

impl<T: LakeMeta> LakeSandboxExt for T {}

#[derive(Debug)]
pub enum LakeError {
    Overflow,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LakeSnapshot {
    pub offset: usize,
}

#[allow(dead_code)]
fn to_meta<'a, T: LakeMeta + 'a>(lake: &'a mut T) -> *mut (dyn LakeMeta + 'a) {
    lake as *mut T as *mut (dyn LakeMeta + 'a)
}

#[inline(always)]
pub unsafe fn force_static<'a, T>(val: T) -> T
where
    T: 'a,
{
    std::mem::transmute::<T, T>(val)
}

#[macro_export]
macro_rules! force_static {
    ($val:expr) => {{ unsafe { std::mem::transmute::<_, _>($val) } }};
}
