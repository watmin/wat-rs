//! Arc 013 slice 5 + arc 018 slice 3 — reference binary composing
//! wat-rs + wat-lru + a user wat program into one runnable binary.
//!
//! The minimal-form consumer:
//! - `src/main.rs` — this one line.
//! - `wat/main.wat` — the entry wat source (implicit from `wat::main!`'s
//!   opinionated default).
//!
//! This is what a downstream consumer (holon-lab-trading) follows
//! when taking wat-lru as a Cargo dep and embedding a wat program.
//! Cargo's convention-over-configuration extended to wat consumers.

wat::main! {
    deps: [wat_lru],
}
