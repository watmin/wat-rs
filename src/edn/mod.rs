//! `:wat::edn::*` — the EDN home. Arc 255 HOME #5.
//!
//! Five loose files at `src/` root were all EDN — ~7,000 lines — while every sibling domain
//! (`string/`, `rete/`, `value/`, `types/`, `collection/`, `comms/`, `kernel/`) already had a
//! named directory. This is that directory.
//!
//! ```text
//! render    <- edn_shim.rs           "shim" is a name that says TEMPORARY, over 5,016 lines.
//!                                    It renders any wat value as EDN/JSON text. It renders.
//! bridge    <- wat_edn_bridge.rs     WatAST <-> plain EDN, both directions
//! contract  <- to_edn.rs             the ONE serialization contract (`ToEdn`) every
//!                                    error/diagnostic type implements
//! error     <- runtime_error_edn.rs  errors-as-EDN
//! derive_tests <- to_edn_derive_tests.rs   `#[cfg(test)]`; see below
//! ```
//!
//! ## ⛔ NO RE-EXPORTS HERE. This file is declarations only.
//!
//! `ToEdn` is named ~51 times, so `crate::edn::ToEdn` is shorter than
//! `crate::edn::contract::ToEdn` and a `pub use contract::ToEdn;` is the obvious, tempting
//! addition. It would mint a **second path to one item** — two ways to say the same thing.
//! A call site that reads badly wants a `use` at the top of ITS file, not a synonym minted
//! here. The extra segment is the price of not creating one.
//!
//! ## Why this is not `crates/wat-edn`
//!
//! `crates/wat-edn` and `crates/wat-to-edn-derive` already exist, so "move it to the crate" is
//! the obvious next thought. It cannot happen yet: `render` names `Value`, `TypeEnv`, `WatAST`,
//! `scope` and `stream`; `bridge` names `WatAST` and `scope`. A leaf crate cannot name the root
//! crate's types. `src/edn/` is the intermediate step — the same one `src/string/` was — and it
//! is what makes the crate question askable later.
//!
//! ## Why `derive_tests` lives in `src/` and must stay
//!
//! It looks like tests in the wrong place and it is not. `#[derive(ToEdn)]` generates
//! `impl crate::edn::contract::ToEdn for <T>`, which only resolves INSIDE the `wat` crate, so an
//! integration test under `tests/` cannot host the toy types. It is `#[cfg(test)]`-gated and
//! compiles only in test builds.
//!
//! ★ `render` and `bridge` are mutually recursive (each names the other). They are siblings for
//! that reason and cannot be layered one under the other.

pub mod render;
pub mod bridge;
pub mod contract;
pub mod error;
#[cfg(test)]
mod derive_tests;
