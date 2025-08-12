#![feature(async_closure)]
#![feature(try_trait_v2)]
#![feature(iter_next_chunk)]

// Public API exports
pub mod ports;
pub mod domain;
pub mod application;
pub mod adapters;
pub mod config;
pub mod prettyprint;

// Re-export key types for easy access
pub use ports::*;
pub use domain::*;
pub use application::*;