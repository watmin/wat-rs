//! `:wat::config::*` intrinsics — arc 255 Stone P6-c-W1, the P6-c campaign's
//! first wave.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-P6-c-W1-config.md`.
//!
//! The four nullary config readers (`dim-count`, `dim-capacity`, `global-seed`,
//! `noise-floor`), moved verbatim out of `runtime.rs`'s giant match — nothing is
//! renamed, no corpus file is touched.
//!
//! ★ **The one real change: the declared arity.** All four used to declare a
//! variadic `&[WatAST]` parameter they used only to reject via a hand-rolled
//! `check_nullary(op, args, list_span)` call — publishing a fictional
//! `Arity::Variadic` through `metadata-of` for a verb that is actually nullary
//! (the exact lie Stone H-1a spent 35 verbs correcting, and P2 fixed for `if`).
//! Homing them means declaring the REAL arity: no `args` parameter, no
//! `check_nullary`, and `#[wat_intrinsic]`'s generated shim owns the arity
//! check — raising the identical `RuntimeErrorKind::ArityMismatch { op, expected:
//! 0, got }` shape `check_nullary` used to raise by hand.
//!
//! The remaining tail is `(sym, list_span)` — a SUBSET of the old fixed
//! `(env, sym, list_span)` triple, in a non-canonical order, legal only because
//! Stone P6-c-1 taught the macro to honour whatever context params the handler
//! itself declares, in the order it declares them.

use wat_macros::wat_intrinsic;

use crate::runtime::require_encoding_ctx;
use crate::span::Span;
use crate::value::{EvalBreak, SymbolTable, Value};

/// `(:wat::config::dim-count)` -> `:wat::core::i64`. The program's committed
/// encoding dimension, set once at startup via `(:wat::config::set-dim-count!
/// n)`; defaults to [`crate::config::DEFAULT_DIM_COUNT`] (10000) when no
/// encoding ctx is attached (test harnesses bypassing the freeze pipeline).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Ambient
/// @ret     :wat::core::i64 the program's committed encoding dimension
/// @example (:wat::config::dim-count) #=> 10000
#[wat_intrinsic(":wat::config::dim-count")]
pub(crate) fn eval_config_dim_count_intrinsic(
    sym: &SymbolTable,
    _list_span: &Span, // rune:lint(unused-span) — infallible — no error path
) -> Result<Value, EvalBreak> {
    match sym.encoding_ctx() {
        Some(ctx) => Ok(Value::i64(ctx.dim_count as i64)),
        None => Ok(Value::i64(crate::config::DEFAULT_DIM_COUNT as i64)),
    }
}

/// `(:wat::config::dim-capacity)` -> `:wat::core::i64`. Hologram-slot count
/// for this program: `floor(sqrt(dim-count))`. Cached at freeze; reads from
/// `EncodingCtx`. Falls back to the default-derived value when no encoding
/// ctx is attached.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Ambient
/// @ret     :wat::core::i64 the program's committed hologram-slot capacity
/// @example (:wat::config::dim-capacity) #=> 100
#[wat_intrinsic(":wat::config::dim-capacity")]
pub(crate) fn eval_config_dim_capacity_intrinsic(
    sym: &SymbolTable,
    _list_span: &Span, // rune:lint(unused-span) — infallible — no error path
) -> Result<Value, EvalBreak> {
    match sym.encoding_ctx() {
        Some(ctx) => Ok(Value::i64(ctx.capacity as i64)),
        None => {
            let d = crate::config::DEFAULT_DIM_COUNT;
            // Arc 294.c.2a — the ONE capacity formula (no recompute).
            let cap = crate::holon::hologram::kanerva_capacity(d);
            Ok(Value::i64(cap as i64))
        }
    }
}

/// `(:wat::config::noise-floor)` -> `:wat::core::f64`. `1/sqrt(dim-count)` at
/// the program's committed `d`. Held for legacy callers; per-d noise-floor is
/// also computed internally by `presence?` / `coincident?` against the same
/// program-d.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Ambient
/// @ret     :wat::core::f64 `1/sqrt(dim-count)` at the program's committed dimension
/// @example (:wat::config::noise-floor) #=> 0.01
#[wat_intrinsic(":wat::config::noise-floor")]
pub(crate) fn eval_config_noise_floor_intrinsic(
    sym: &SymbolTable,
    _list_span: &Span, // rune:lint(unused-span) — infallible — no error path
) -> Result<Value, EvalBreak> {
    let d = match sym.encoding_ctx() {
        Some(ctx) => ctx.dim_count,
        None => crate::config::DEFAULT_DIM_COUNT,
    };
    Ok(Value::f64(1.0 / (d as f64).sqrt()))
}

/// `(:wat::config::global-seed)` -> `:wat::core::i64`. The committed
/// atom-seeding seed.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Ambient
/// @ret     :wat::core::i64 the program's committed atom-seeding seed
/// @example (:wat::config::global-seed) #=> 42
#[wat_intrinsic(":wat::config::global-seed")]
pub(crate) fn eval_config_global_seed_intrinsic(
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    let ctx = require_encoding_ctx(":wat::config::global-seed", sym, list_span)?;
    Ok(Value::i64(ctx.config.global_seed as i64))
}
