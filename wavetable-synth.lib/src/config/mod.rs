#[cfg_attr(feature = "embeded", path = "configs/embeded.rs")]
#[cfg_attr(feature = "desktop", path = "configs/desktop.rs")]
pub mod configs;

pub use configs::*;
