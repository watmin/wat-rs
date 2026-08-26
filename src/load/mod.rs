//! The load home — Arc 255 HOME #6. Four loose files at `src/` root were all about **getting
//! source text into the runtime**, while every sibling domain (`edn/`, `resolve/`, `check/`,
//! `types/`, `collection/`, `rete/`, `kernel/`, ...) already had a named directory. This is that
//! directory. (`crate::load` here is the Rust module; it is unrelated to the wat-language keyword
//! namespace `:wat::load::*`, which is a retired interface — see `loader.rs`'s own history.)
//!
//! ```text
//! loader   formerly `load.rs` — recursive `load!` resolution, the LoadSpec family,
//!          SourceLoader/FsLoader/InMemoryLoader, the Load*Error types
//! stdlib   formerly `stdlib.rs` — the bundled wat stdlib, baked into the binary via `include_str!`
//! source   formerly `source.rs` — the user-facing surface for wat-source contribution
//! ```
//!
//! ## `sandbox.rs` — DELETED, not moved
//!
//! A fourth loose file, `src/sandbox.rs`, sat beside these three and carried **zero code**:
//! thirteen lines, every one a `//!` comment. Its own header recorded that pre-arc-105 this module
//! hosted `eval_kernel_run_sandboxed` / `eval_kernel_run_sandboxed_ast` — substrate Rust impls
//! that collected stdio as `Vec<String>` buffers — and that arc 105c retired both. The wat-level
//! reimplementation in `wat/kernel/sandbox.wat` (atop arc 105a's spawn-program-returns-Result and
//! arc 105b's `:wat::kernel::ThreadDiedError/message` accessor) is the canonical sandbox loader
//! today; `Vec<String>` survives only inside that wat-level helper, where it's the test-assertion
//! target. `resolve_sandbox_loader` (and the corresponding `src/spawn.rs` callers) were retired in
//! arc 298, after which the module remained only as a namespace anchor — anchoring nothing. Moving
//! an empty anchor into this new home would have minted exactly the kind of home-nothing-earned
//! this stone exists to avoid, so it is deleted instead. A reader looking for the sandbox loader:
//! it is `wat/kernel/sandbox.wat`, not here.
//!
//! ## ⛔ NO RE-EXPORTS HERE. This file is declarations only.
//!
//! Same ruling as `src/edn/mod.rs` (HOME #5) and for the same reason: `wat::load::InMemoryLoader`
//! is shorter than `wat::load::loader::InMemoryLoader` and is the tempting addition. It would mint
//! a second path to one item — two ways to say the same thing. A call site that reads badly wants
//! a `use` at the top of ITS file, not a synonym minted here.

pub mod loader;
pub mod source;
pub(crate) mod stdlib;
