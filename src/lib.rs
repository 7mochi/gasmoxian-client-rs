#![deny(clippy::correctness)]
#![warn(clippy::complexity, clippy::perf, clippy::style)]

pub mod console;
pub mod effect;
pub mod enet;
pub mod filter;
pub mod io;
pub mod protocol;
pub mod ps1_memory;
pub mod ps1_snapshot;
pub mod server;
pub mod state;
