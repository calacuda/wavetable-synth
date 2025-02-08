// #[cfg(feature = "desktop")]
// mod desktop;
// #[cfg(feature = "embeded", path = "embeded.rs")]
// mod embeded;
//
// #[cfg(feature = "desktop")]
// pub use desktop::*;
// #[cfg(feature = "embeded")]
// pub use embeded::*;

// mod configs {
// #[cfg_attr(feature = "embeded", path = "embeded.rs")]

// #[cfg(feature = "embeded")]
// #[path = "configs/embeded.rs"]
// pub mod configs;
// #[cfg(feature = "desktop")]
// #[path = "configs/desktop.rs"]
#[cfg_attr(feature = "embeded", path = "configs/embeded.rs")]
#[cfg_attr(feature = "desktop", path = "configs/desktop.rs")]
pub mod configs;

pub use configs::*;

// pub const MAX_LFOS: usize = 5;
