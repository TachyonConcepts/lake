#[cfg(test)]
mod tests {
    use lake::{
        force_static,
        lake::{force_static, LakeMeta, LakeSandboxExt, LakeSnapshot, LakeStats},
        sandbox::SandboxGuard
    };

    struct DummyLake {
        offset: usize,
        generation: usize,
        capacity: usize,
    }

    impl LakeMeta for DummyLake {
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
            self.capacity
        }
    }

    #[test]
    fn test_lake_stats() {
        let lake = DummyLake {
            offset: 100,
            generation: 2,
            capacity: 256,
        };
        let stats: LakeStats = lake.stats();
        assert_eq!(stats.used, 100);
        assert_eq!(stats.remaining, 156);
        assert_eq!(stats.capacity, 256);
        assert_eq!(stats.generation, 2);
    }

    #[test]
    fn test_lake_snapshot_equality() {
        let snap1 = LakeSnapshot { offset: 42 };
        let snap2 = LakeSnapshot { offset: 42 };
        let snap3 = LakeSnapshot { offset: 99 };
        assert_eq!(snap1, snap2);
        assert_ne!(snap1, snap3);
    }

    #[test]
    fn test_force_static_fn() {
        let a: Box<u32> = Box::new(123);
        let b = unsafe { force_static::<Box<u32>>(a) };
        assert_eq!(*b, 123);
    }

    #[test]
    fn test_force_static_macro() {
        let a: Box<u32> = Box::new(456);
        let b: Box<u32> = force_static!(a);
        assert_eq!(*b, 456);
    }

    #[test]
    fn test_sandbox_guard_creation() {
        let mut lake = DummyLake {
            offset: 10,
            generation: 1,
            capacity: 128,
        };
        let guard: SandboxGuard<DummyLake> = LakeSandboxExt::sandbox(&mut lake);
        assert!(!guard.committed);
        assert_eq!(guard.base_offset, 10);
        assert!(guard.lake.is_some());
    }
}
