//! tests/archived/ — the archived test corpus (arc 252 test-surface reorg).
//!
//! These are SPENT design-probes: FM-2-bis disconfirming probes from CLOSED arcs
//! whose result is now a permanent property of the substrate. They did their job
//! at the arc — they proved a composition worked (or didn't) and steered the build.
//! We keep them RUNNABLE rather than delete them: if they still compile and pass,
//! they are zero-cost regression coverage, and they are the historical record of
//! how the substrate was proven. "If they work and were useful, don't lose them."
//!
//! NAMING: archived tests KEEP their arc-numbered names on purpose — the archive
//! IS the record of that arc, so the arc number is the right identifier here. The
//! behavior-renaming (`tests/<home>/<behavior>.rs`) applies only to LIVE tests.
//!
//! This is ONE leak-safe `[[test]]` binary (Cargo: `name="archived"`). Only PURE
//! (non-process) probes live here so `cargo test --test archived` never leaks;
//! living self-enforcing gates and process probes stay elsewhere.
//!
//! Run: `cargo test --release -p wat --test archived`

mod probe_arc214_slice4_stone1_program_env_typealias;
mod probe_arc216_stone1_hashset_roundtrip;
mod probe_arc216_stone2_vector_roundtrip;
mod probe_arc234_stone1_wat_record_variant;
mod probe_arc237_stone1_typeunion_substrate;
mod probe_diagnostic_bundle_result_compose;
