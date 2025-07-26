use std::cell::RefCell;
use crate::lake::lake::lake::Lake;

pub const DEFAULT_SIZE: usize = 65536;

thread_local! {
    pub static THREAD_LAKE: RefCell<Option<Lake<DEFAULT_SIZE>>> = RefCell::new(None);
}

pub fn thread_lake_init() {
    THREAD_LAKE.with(|slot| {
        *slot.borrow_mut() = Some(Lake::new());
    });
}

#[macro_export]
macro_rules! with_lake {
    ($block:expr) => {{
        $crate::thread_lake::THREAD_LAKE.with(|slot| {
            let mut opt = slot.borrow_mut();
            let lake = opt
                .as_mut()
                .expect("Lake not initialized. Call `thread_lake_init()` first.");
            $block(lake)
        })
    }};
}
