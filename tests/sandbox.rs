use lake::sandbox::SandboxGuard;
use lake::Lake;

#[test]
fn test_sandbox_guard_commit_sets_offset() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let _ = lake.alloc::<8>().unwrap(); // offset = 8

    {
        let mut sandbox = SandboxGuard {
            base_offset: lake.get_offset(), // = 8
            lake: Some(&mut lake),
            committed: false,
        };

        let _ = sandbox.view().alloc::<16>().unwrap(); // view offset = 16
        sandbox.commit(); // total = 8 + 16 = 24
    }

    assert_eq!(lake.get_offset(), 24);
}

#[test]
fn test_sandbox_guard_drop_resets_offset() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let _ = lake.alloc::<8>().unwrap(); // offset = 8

    {
        let mut sandbox = SandboxGuard {
            base_offset: lake.get_offset(), // = 8
            lake: Some(&mut lake),
            committed: false,
        };

        let _ = sandbox.view().alloc::<16>().unwrap(); // offset = 8 + 16
        // no commit: drop should reset
    }

    assert_eq!(lake.get_offset(), 8); // rolled back
}

#[test]
fn test_sandbox_guard_commit_and_return() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let _ = lake.alloc::<4>().unwrap(); // offset = 4

    let lake: &mut Lake<64> = {
        let mut sandbox = SandboxGuard {
            base_offset: lake.get_offset(),
            lake: Some(&mut lake),
            committed: false,
        };

        let _ = sandbox.view().alloc::<12>().unwrap();
        sandbox.commit_and_return()
    };

    assert_eq!(lake.get_offset(), 4 + 12);
}

#[test]
fn test_sandbox_guard_nested_sandboxes() {
    let mut lake: Lake<64> = Lake::<64>::new();
    let _ = lake.alloc::<8>().unwrap(); // offset = 8

    {
        let mut outer = SandboxGuard {
            base_offset: lake.get_offset(),
            lake: Some(&mut lake),
            committed: false,
        };

        let _ = outer.view().alloc::<8>().unwrap(); // inner offset = 8

        {
            let mut inner = SandboxGuard {
                base_offset: outer.view().get_offset(),
                lake: Some(outer.view()),
                committed: false,
            };

            let _ = inner.view().alloc::<8>().unwrap(); // inner inner offset = 8
            inner.commit();
        }

        outer.commit();
    }

    assert_eq!(lake.get_offset(), 8 + 8 + 8); // = 24
}
