//! `:wat::holon::` intrinsic registry entries — arc 255 Stone HOME-8, the second half of the
//! VSA-surface carve (the first half, `d43f758870`, lifted the pure algebra out of `runtime.rs`
//! into `src/holon/`).
//!
//! **A pure RE-REGISTRATION, not a rename.** `:wat::holon::` was already a top-level namespace —
//! 3,058 corpus sites, zero legacy `:wat::core::holon::` spellings — so unlike every prior home
//! (which moved a NAME and needed a three-phase register/codemod/retire sequence), this one moves
//! nothing: the 95 `":wat::holon::…" =>` arms that lived in `runtime.rs`'s dispatch match become
//! `#[wat_intrinsic(":wat::holon::…")]` handlers under the SAME names, here. No codemod, no
//! `RetirementEntry` rows, no dual-spelling window, no corpus churn — see the brief's
//! `⛔⛔ CORRECTED 2026-08-27` section for the full record of why the six-stone phase-order
//! boilerplate does not apply here.
//!
//! Clustered by receiver, mirroring the directory-home precedent
//! (`intrinsic/io/`, `intrinsic/kernel/`):
//!
//! - [`atom`] — the 60 bare ops: the algebra combinators (`Bind`, `Bundle`, `Permute`, `Blend`,
//!   `Thermometer`), the classified-collection constructors (`Map`/`Set`/`Vector`/`List`/`Tuple`),
//!   the `Value ⇄ HolonAST ⇄ WatAST` conversions, classifier predicates/projections, the
//!   Thermometer/term surface, the measurement primitives (`cosine`/`dot`/`presence?`/
//!   `coincident?`/`coincident-explain`/`simhash`), the `eval-*-coincident?` family, and the raw-
//!   `Vector` mirrors.
//! - [`hologram`] — `Hologram/*` (7 verbs): the therm-routed coordinate-cell store.
//! - [`engram`] — `Engram/*` + `EngramLibrary/*` (10 verbs): learned pattern snapshots and their
//!   nearest-match index.
//! - [`subspace`] — `OnlineSubspace/*` (10 verbs): the CCIPCA online-PCA anomaly tracker.
//! - [`reckoner`] — `Reckoner/*` (8 verbs): directional-evidence accumulation and calibration.
//!
//! **The bodies do not change here** — each handler is the SAME body its pre-carve `runtime.rs`
//! dispatch fn had, with its `list_span` parameter moved to the END to match
//! `#[wat_intrinsic]`'s variadic calling convention (`args, env, sym, span`); every handler still
//! delegates to the pure algebra in [`crate::holon`] (the first-strike home) or to the external
//! `holon` VSA crate directly, exactly as it did before this carve.
//!
//! **Provenance:** `atom::eval_holon_from_holon` (`from-holon`) is the one PRODUCER among all 95 —
//! the only verb pre-carve that was hoisted into `dispatch_keyword_head`'s `TrackedValue`-returning
//! fast path, stamping `Provenance::RuntimeBuilt`. It keeps that return type here, so Stone G's
//! `sniff_return` forwards it un-rewrapped instead of downgrading it to `Provenance::Unknown`. Every
//! other verb returned bare `Value` before this carve and still does.
//!
//! **The rete seam is untouched.** `src/rete/purity.rs:647` classifies exactly four `:wat::holon::`
//! verbs (builder-ruled, 2026-08-01); this carve moves what that ruling requires and classifies
//! nothing new (STOP-7).
mod atom;
mod engram;
mod hologram;
mod reckoner;
mod subspace;
