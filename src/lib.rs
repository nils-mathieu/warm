//! A lightweight rendering engine built on top of Vulkan.

#![warn(missing_docs, missing_debug_implementations)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod gpu;
pub mod render_pass;
pub mod surface;

mod error;

pub use self::error::*;

mod utility;
