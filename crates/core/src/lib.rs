#![feature(try_trait_v2)]
#![feature(iter_next_chunk)]

// Public API exports
pub mod adapters;
pub mod application;
pub mod config;
pub mod domain;
pub mod ports;
pub mod prettyprint;

// Re-export key types for easy access
pub use application::*;
pub use domain::*;
pub use ports::*;
