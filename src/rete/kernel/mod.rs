//! Arc 278 Stone P1 — native `FireSession` + fire kernel.
//!
//! Split: `wm` types, `fire` loop, `arm` intern, `stratify` driver,
//! `census` instrument, `insert` overlay. Tests are `tests.rs`.
//!
//! The mutable mirror of a `:wat::rete::Session` that the fire kernel (P2–P5) mutates
//! during a fire pass. `to_transient` converts a frozen `Session` value into a native
//! `FireSession`; `to_persistent` rebuilds the frozen `Session` from it. The boundary
//! is lossless: `to_persistent(to_transient(s)) == s` for every compiled / fired session.
//!
//! Both functions are `pub(crate)` — the transient mutation is sealed in Rust; no
//! mutation primitive is exposed to the wat language surface. The user calls `fire`
//! (P5), never the transient.
//!
//! ## Session record (8 fields, declaration order — `wat/rete.wat` `defrecord Session`)
//! ```text
//! network           <- :wat::core::PersistentMap
//! rules             <- :wat::core::PersistentVector<wat::rete::Rule>
//! alpha-memory      <- :wat::core::PersistentMap
//! beta-memory       <- :wat::core::PersistentMap
//! production-memory <- :wat::core::PersistentMap
//! facts             <- :wat::core::PersistentVector
//! next-id           <- :wat::core::i64
//! query-memory      <- :wat::core::PersistentMap
//! ```

mod wm;
pub(crate) use wm::*;
mod census;
pub(crate) use census::*;
mod arm;
pub(crate) use arm::*;
mod fire;
pub(crate) use fire::*;
mod stratify;
pub(crate) use stratify::*;
mod insert;
pub(crate) use insert::*;

#[cfg(test)]
mod tests {
    include!("tests.rs");
}
