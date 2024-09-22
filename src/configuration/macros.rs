#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if $crate::configuration::SETTINGS.debug {
            println!($($arg)*);
        }
    }
}
