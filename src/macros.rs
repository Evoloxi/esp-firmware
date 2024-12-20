#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => (
        ::log::log!(target: $target, ::log::Level::Info, "{}\n", format!($($arg)+))
    );

    ($($arg:tt)+) => (
        ::log::log!(::log::Level::Info, "{}\n", format!($($arg)+))
    )
}

#[macro_export]
macro_rules! arc {
    ($e:expr) => {
        Arc::new(Mutex::new($e))
    };
}