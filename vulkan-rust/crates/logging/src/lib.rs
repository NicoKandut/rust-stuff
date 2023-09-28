use colored::Colorize;
use lazy_static::lazy_static;

#[macro_export]
macro_rules! log {
    ($scope: expr, $fmt_string:expr $(, $arg:expr )*) => {
        println!("[{}] {}", $scope, format!($fmt_string, $( $arg ),*));
    };
}

#[rustfmt::skip]
lazy_static! {
    pub static ref LOG_RENDER: String = "RENDER".blue() .to_string();
    pub static ref LOG_VULKAN: String = "VULKAN".red()  .to_string();
    pub static ref LOG_WORLD : String = "WORLD ".green().to_string();
    pub static ref LOG_ENGINE: String = "ENGINE".yellow().to_string();
}

#[cfg(test)]
mod tests {
    use crate::{LOG_RENDER, LOG_VULKAN, LOG_WORLD};

    #[test]
    fn works() {
        let nico = "Nico";

        log!(*LOG_WORLD, "Hello {}!", nico);
        log!(*LOG_RENDER, "Hello {}!", 17);
        log!(*LOG_VULKAN, "Hello!");
    }
}
