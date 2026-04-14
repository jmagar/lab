//! Upstream MCP server proxy — connects to external MCP servers and
//! exposes their tools through the lab gateway.
//
// Many items in pool and types are not yet called from outside the module
// (discovery, resource proxying, circuit breaker probing). They are exercised
// by tests and will be fully wired when `lab serve` gains `[[upstream]]` config
// support. The blanket allow prevents false-positive warnings on partially
// wired public APIs.
#[allow(dead_code)]
pub mod pool;
#[allow(dead_code)]
pub mod types;

