use crate::lake::LakeMeta;

#[must_use]
pub struct SandboxGuard<'a, T: LakeMeta> {
    pub lake: Option<&'a mut T>,
    pub base_offset: usize,
    pub committed: bool,
}

impl<'a, T: LakeMeta> SandboxGuard<'a, T> {
    #[inline(always)]
    pub fn view(&mut self) -> &mut T {
        self.lake.as_mut().unwrap()
    }

    /// Commit changes made inside the sandbox.
    #[inline(always)]
    pub fn commit(mut self) {
        let lake: &mut &mut T = self.lake.as_mut().unwrap();
        let delta: usize = lake.offset() - self.base_offset;
        lake.set_offset(self.base_offset + delta);
        self.committed = true;
    }

    /// Commit and get back the original `Lake`.
    #[inline(always)]
    pub fn commit_and_return(mut self) -> &'a mut T {
        let lake: &mut T = self.lake.take().unwrap();
        let delta: usize = lake.offset() - self.base_offset;
        lake.set_offset(self.base_offset + delta);
        self.committed = true;
        lake
    }
}

impl<'a, T: LakeMeta> Drop for SandboxGuard<'a, T> {
    fn drop(&mut self) {
        if !self.committed {
            if let Some(lake) = self.lake.as_mut() {
                lake.set_offset(self.base_offset);
            }
        }
    }
}
