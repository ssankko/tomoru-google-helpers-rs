//! This library provides a way to access system information such as CPU load, mounted filesystems,
//! network interfaces, etc.

pub mod data;
pub mod platform;

pub use self::data::*;
pub use self::platform::Platform;
pub use self::platform::PlatformImpl as System;
