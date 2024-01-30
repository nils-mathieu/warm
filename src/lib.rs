//! A thin wrapper around the Vulkan library, based on [`ash`].

mod library;
pub use library::*;

mod instance;
pub use instance::*;
