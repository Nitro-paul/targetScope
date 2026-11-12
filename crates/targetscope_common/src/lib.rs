//! Shared library of all Rust programs of TargetScope.
//!
//! Implements the language-neutral contract in `schemas/` for Rust: model types, message
//! enums and envelopes, the Redis bus, the iceoryx2 frame services, geometry, ring generation,
//! bootstrap configuration, settings access, logging, the clock and failpoints.
//! It contains no program logic. Design: doc/shared_libraries.md.

pub mod bus;
pub mod clock;
pub mod config;
pub mod frames;
pub mod geometry;
pub mod idempotency;
pub mod logging;
pub mod messages;
pub mod model;
pub mod settings;
pub mod targets;
