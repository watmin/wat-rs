//! `:wat::rand::*` — a threaded PRNG, and an ambient wrapper over it.
//!
//! wat had no randomness interface. Chaos needs one that is **replayable**:
//! same seed, same sequence, so a red can be re-derived. That is the
//! threaded form. The ambient form exists for scripts and demos, and it is
//! a **different verb** because it sits in a different purity class.
//!
//! ```text
//! (:wat::rand::int      lo hi)        -> i64                ambient  · Pure, NOT Deterministic
//! (:wat::rand::int-from state lo hi)  -> (Tuple i64 i64)    threaded · Pure AND Deterministic
//! ```
//!
//! Both `[lo, hi)`, matching `:wat::core::range`. The tuple is
//! `(new-state, draw)` so the state threads as `first`.
//!
//! ★ They are NOT one name at two arities. `(int 0 6)` and `(int 0 6 seed)`
//! would hide that they classify differently; a service arm does not demand
//! `Deterministic`, so the ambient form would compile inside a chaos reactor
//! and an unreproducible red would have nothing to catch it. The name
//! carries the class.
//!
//! Algorithm: SplitMix64, then reject-and-redraw so `n` does not bias the
//! low residues. One function, two verbs — the ambient form is a wrapper
//! that seeds from `uuid::v4`'s entropy and discards the new state.

use std::sync::Arc;

use wat_macros::wat_intrinsic;

use crate::value::{EvalBreak, RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

const OP_FROM: &str = ":wat::rand::int-from";
const OP_INT: &str = ":wat::rand::int";

/// SplitMix64 — 64-bit state, one add, two xor-shift-multiply rounds.
/// Returns `(new_state, mixed_bits)`.
fn splitmix64(state: u64) -> (u64, u64) {
    let state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (state, z ^ (z >> 31))
}

/// Uniform draw on `[lo, hi)`. Returns `(new_state, value)`.
/// Empty range (`hi <= lo`) is a domain error — Partial, like `vector::set`.
fn int_from(state: i64, lo: i64, hi: i64) -> Result<(i64, i64), String> {
    let width = (hi as i128) - (lo as i128);
    if width <= 0 {
        return Err(format!("empty range [{lo}, {hi})"));
    }
    let n = width as u64;
    let mut s = state as u64;
    // 2^64 % n — values below this are the incomplete last cycle.
    let threshold = n.wrapping_neg() % n;
    loop {
        let (s2, x) = splitmix64(s);
        s = s2;
        if x >= threshold {
            let v = (lo as i128 + ((x % n) as i128)) as i64;
            return Ok((s as i64, v));
        }
    }
}

fn require_i64(op: &'static str, v: &Value) -> Result<i64, EvalBreak> {
    match v {
        Value::i64(n) => Ok(*n),
        other => Err(RuntimeError::new(
            crate::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn malformed(op: &'static str, reason: String) -> EvalBreak {
    RuntimeError::new(
        crate::rust_caller_span!(),
        RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason,
        },
    )
    .into()
}

fn pair(state: i64, value: i64) -> Value {
    Value::Tuple(Arc::new(vec![Value::i64(state), Value::i64(value)]))
}

fn fresh_seed() -> i64 {
    // Same entropy source as `:wat::uuid::v4`. No process-global RNG cell.
    let bytes = wat_edn::new_uuid_v4().into_bytes();
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[..8]);
    i64::from_le_bytes(buf)
}

/// `(:wat::rand::int-from state lo hi)` → `(Tuple new-state draw)` on `[lo, hi)`.
///
/// Threaded SplitMix64. Same `state` produces the same sequence. Empty
/// range (`hi <= lo`) is a located error.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Partial
/// @Category      Transform
/// @arg     state :wat::core::i64 the PRNG state (the seed, then each new-state)
/// @arg     lo    :wat::core::i64 inclusive lower bound
/// @arg     hi    :wat::core::i64 exclusive upper bound
/// @ret     (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) `(new-state, draw)`
/// @example (:wat::core::second (:wat::rand::int-from 1 0 6)) #=> 5
/// @see     :wat::rand::int
#[wat_intrinsic(":wat::rand::int-from")]
pub(crate) fn eval_rand_int_from(
    state: &Value,
    lo: &Value,
    hi: &Value,
) -> Result<Value, EvalBreak> {
    let state = require_i64(OP_FROM, state)?;
    let lo = require_i64(OP_FROM, lo)?;
    let hi = require_i64(OP_FROM, hi)?;
    let (s2, v) = int_from(state, lo, hi).map_err(|reason| malformed(OP_FROM, reason))?;
    Ok(pair(s2, v))
}

/// `(:wat::rand::int lo hi)` → a draw on `[lo, hi)`.
///
/// Ambient wrapper over [`eval_rand_int_from`]: a fresh seed from the same
/// entropy as `:wat::uuid::v4`, then the threaded draw, then the value
/// only. Pure, **not** Deterministic — same class as `uuid::v4`. For
/// anything that must replay (services, chaos), use `int-from`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Total         Partial
/// @Category      Entropic
/// @arg     lo :wat::core::i64 inclusive lower bound
/// @arg     hi :wat::core::i64 exclusive upper bound
/// @ret     :wat::core::i64 a draw in `[lo, hi)`
/// @example-norun (:wat::rand::int 0 6) #=> 4
/// @see     :wat::rand::int-from
#[wat_intrinsic(":wat::rand::int")]
pub(crate) fn eval_rand_int(lo: &Value, hi: &Value) -> Result<Value, EvalBreak> {
    let lo = require_i64(OP_INT, lo)?;
    let hi = require_i64(OP_INT, hi)?;
    let (_s2, v) = int_from(fresh_seed(), lo, hi).map_err(|reason| malformed(OP_INT, reason))?;
    Ok(Value::i64(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draw_n(seed: i64, lo: i64, hi: i64, n: usize) -> Vec<i64> {
        let mut s = seed;
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            let (s2, v) = int_from(s, lo, hi).expect("non-empty range");
            s = s2;
            out.push(v);
        }
        out
    }

    #[test]
    fn same_seed_same_sequence() {
        let a = draw_n(1, 0, 6, 100);
        let b = draw_n(1, 0, 6, 100);
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_diverge() {
        let a = draw_n(1, 0, 6, 100);
        let b = draw_n(2, 0, 6, 100);
        assert_ne!(a, b, "S and S+1 must not produce the same 100 draws");
    }

    #[test]
    fn draws_are_in_range() {
        for v in draw_n(1, 0, 6, 10_000) {
            assert!((0..6).contains(&v), "draw {v} outside [0, 6)");
        }
    }

    #[test]
    fn below_six_is_unbiased() {
        // 100k draws, 6 buckets, expected ≈ 16667. A few percent of even.
        const N: usize = 100_000;
        let mut buckets = [0u32; 6];
        for v in draw_n(1, 0, 6, N) {
            buckets[v as usize] += 1;
        }
        let expected = N as f64 / 6.0;
        for (i, &c) in buckets.iter().enumerate() {
            let rel = (c as f64 - expected).abs() / expected;
            assert!(
                rel < 0.03,
                "bucket {i} has {c} / {N} (relative error {rel:.4}); expected ~{expected:.0}"
            );
        }
    }

    #[test]
    fn empty_range_is_a_domain_error() {
        assert!(int_from(1, 5, 5).is_err());
        assert!(int_from(1, 6, 0).is_err());
    }

    #[test]
    fn int_from_handler_replays() {
        let a = eval_rand_int_from(&Value::i64(1), &Value::i64(0), &Value::i64(6)).unwrap();
        let b = eval_rand_int_from(&Value::i64(1), &Value::i64(0), &Value::i64(6)).unwrap();
        assert_eq!(a, b);
        match a {
            Value::Tuple(xs) => {
                assert_eq!(xs.len(), 2);
                match &xs[1] {
                    Value::i64(5) => {}
                    other => panic!("seed 1, [0, 6) must draw 5; got {other:?}"),
                }
            }
            other => panic!("int-from must return a 2-tuple; got {other:?}"),
        }
    }

    #[test]
    fn ambient_int_is_in_range() {
        for _ in 0..32 {
            match eval_rand_int(&Value::i64(0), &Value::i64(6)).unwrap() {
                Value::i64(v) => assert!((0..6).contains(&v), "ambient draw {v} outside [0, 6)"),
                other => panic!("int must return i64; got {other:?}"),
            }
        }
    }

    #[test]
    fn unit_width_is_always_lo() {
        let (_, v) = int_from(1, 7, 8).unwrap();
        assert_eq!(v, 7);
    }
}
