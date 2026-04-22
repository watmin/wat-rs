//! Arc 013 slice 5 — reference binary that composes wat-rs +
//! wat-lru + a user wat program into one runnable binary.
//!
//! The entire main() is the `wat::main!` macro expansion. The
//! macro parses (source: ..., deps: [...]), pulls each dep's
//! `stdlib_sources()` and `register()`, and routes through
//! `wat::compose_and_run` with real OS stdio.
//!
//! This is what a downstream consumer (holon-lab-trading,
//! eventually) will look like when it takes wat-lru as a
//! Cargo dep and embeds a wat program.

wat::main! {
    source: include_str!("program.wat"),
    deps: [wat_lru],
}
