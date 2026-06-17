//! Sans-IO client for the Gasmoxian OnlineCTR multiplayer protocol.
//!
//! The client reads Crash Team Racing shared memory from DuckStation,
//! drives a state machine that produces [`Effect`] values, and executes
//! those effects against the PS1 memory, an enet network connection,
//! and the terminal UI. No state logic performs I/O directly; all side
//! effects are deferred to [`io::exec_effects`].

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
