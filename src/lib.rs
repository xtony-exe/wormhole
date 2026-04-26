//! # Wormhole
//!
//! A fast, modern TCP tunnel in Rust that punches through NAT firewalls,
//! exposing local ports to a remote server.
//!
//! **By THINKING TEAM — authored by XTONY**
//!
//! ## Quick Start
//!
//! ```shell
//! # Start the server
//! wormhole server
//!
//! # On your local machine, expose port 8080
//! wormhole local 8080 --to your-server.com
//! ```
//!
//! Both the client and server are public and can be driven
//! programmatically with a Tokio 1.x runtime.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod auth;
pub mod client;
pub mod server;
pub mod shared;
