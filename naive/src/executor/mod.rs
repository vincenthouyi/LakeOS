use conquer_once::spin::OnceCell;
use crate::task::executor::Executor;

pub fn global_executor() -> &'static Executor {
    static GLOBAL_EXECUTOR: OnceCell<Executor> = OnceCell::uninit();

    GLOBAL_EXECUTOR.try_get_or_init(||
        Executor::new()
    ).unwrap()
}