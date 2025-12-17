//! MCP (Model Context Protocol) server for CGT calculations.
//!
//! This crate provides an MCP server that exposes CGT calculation capabilities
//! to AI assistants via the Model Context Protocol.

mod error;
mod resources;
mod server;

pub use error::McpServerError;
pub use server::run_server;
