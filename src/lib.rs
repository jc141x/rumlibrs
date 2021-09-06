//! Available features:
//!
//! ```toml
//! default = ["library"]
//! pirate = ["library", "download"]
//! johncena141 = ["library", "database", "download"]
//! library = []
//! database = []
//! download = []
//! default = ["library"]
//! chad = ["library", "download"]
//! johncena141 = ["library", "database", "download"]
//! admin = ["database", "scraper"]
//! library = []
//! database = []
//! download = []
//! scraper = []
//! ```

#[cfg(feature = "banner")]
pub mod banner;
pub mod config;
#[cfg(feature = "database")]
pub mod database;
#[cfg(feature = "download")]
pub mod download;
#[cfg(feature = "library")]
pub mod library;
#[cfg(feature = "scraping")]
pub mod scraper;
pub mod util;
