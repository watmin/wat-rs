//! Arc 109 Stone 1 — `src/numeric/`: the numeric tower's home.
//!
//! **Builder, `DESIGN-STONE-the-numeric-home.md`:** *"src/intrinsic/ is meant to wire up into
//! the registry… the edge of wat's kernel. The actual implementations live in some proper
//! home… impl and (registration, delegation) interface are not the same thing."* And the
//! requirement that shapes this module's internal layout: *"we will add direct support for
//! all of rust's numerics… prepare the file system such that these additions become trivial
//! once we're ready."*
//!
//! ## The one contract decision: split by CONCERN, never by TYPE
//!
//! Rust's numeric set is i8/i16/i32/i64/i128/isize · u8/u16/u32/u64/u128/usize · f32/f64, plus
//! this substrate's `BigInt` and `Rational` — ~16 types against today's 5 (i64, f64, bigint,
//! rational, u8). A per-type layout (`i64.rs`, `f64.rs`, …) grows one file per type and makes
//! adding a type look trivial while multiplying the surface — the opposite of the ask. A
//! per-CONCERN layout is four files today, and adding a type touches each concern once:
//!
//! ```text
//! src/numeric/arith.rs      arithmetic: eval_*_arith (AST door) + arith_*_*_inner (value door)
//! src/numeric/convert.rs    named to-<type> casts, plus the :u8 range-checked cast
//! src/numeric/compare.rs    the f64 NaN-correct ordering primitive
//! src/numeric/ops.rs        type-specific operations that do NOT cross the tower
//!                           (rational numerator/denominator, f64 round/unary/clamp)
//! ```
//!
//! ## EDGE vs IMPL — the architecture this module is one instance of
//!
//! `src/intrinsic/<domain>` is the EDGE — registration and delegation, the kernel's rim.
//! `src/<domain>/` is the IMPL — the actual work. Already built seven times before this stone:
//! `intrinsic/collection.rs → src/collection/`, `intrinsic/edn.rs → src/edn/`,
//! `intrinsic/holon/ → src/holon/`, `intrinsic/kernel/ → src/kernel/`, `rete`, `stream`,
//! `string`. This module is the numeric tower's turn: `src/intrinsic/{i64,f64,bigint,
//! rational}.rs` are the edge; this is the impl. **The impl must not reference its own edge
//! module** — that would create the exact cycle this architecture exists to avoid.
//!
//! ## What this stone shipped, and what it did not
//!
//! Stone 1 (this one): the 24 numeric implementation fns move out of `src/runtime.rs` into
//! this module, split by concern as above. Behaviour is unchanged — every numeric verb is
//! identical before and after; only the location moved.
//!
//! Stone 2 (not this one): the promotion lattice. Every numeric mechanism here is currently
//! written per ORDERED PAIR — `eval_i64_arith`/`eval_f64_arith`/`eval_bigint_arith`/
//! `eval_rational_arith` one per type, nine numeric conversion pairs for four types. That is
//! quadratic in the type count and unaffected by this relocation; replacing it with a
//! rank-based `promote(a, b) -> CommonRepr` so each concern becomes "promote once, then do the
//! op once" is what makes adding a type a linear edit. It is a behaviour-preserving rewrite of
//! real algorithms and deserves a red that points at one thing, not a relocation.
//!
//! `src/value/numeric_order.rs` — the tower's ordering door — stays where it is this stone;
//! moving it is a practitioner's call stone 2 makes, not this one.

pub(crate) mod arith;
pub(crate) mod compare;
pub(crate) mod convert;
pub(crate) mod ops;
