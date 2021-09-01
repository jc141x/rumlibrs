//! Available features:
//!
//! ```
//! default = ["library"]
//! pirate = ["library", "download"]
//! johncena141 = ["library", "database", "download"]
//! library = []
//! database = []
//! download = []
//! ```

pub mod config;
#[cfg(feature = "database")]
pub mod database;
#[cfg(feature = "download")]
pub mod download;
#[cfg(feature = "library")]
pub mod library;
pub mod util;
