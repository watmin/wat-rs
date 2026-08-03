# BRIEF — the cosine family's outcome wall

Spec: `DESIGN-STONE-where-admits-only-rete-ops.md` §§ *"the match is the shield"*, *"The measurement
vocabulary"*, *"THE MEASUREMENT IS FULL; THE PREDICATE IS EXACT"*. **Read those three sections first —
the enums, their field names, and the two-enums-not-one reasoning are already RULED and intueri-cast.
Do not re-derive them and do not rename anything.**

## You are a rider, not the orchestrator

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Run every command in the
FOREGROUND and block on it. Your turn ends when the numbers are in your hands.

## The work in one paragraph

`:wat::holon::cosine` has two domain holes and both are currently dishonest: a **dimension mismatch**
raises `TypeMismatch` (and panics in the sibling crate underneath), and a **zero-magnitude operand**
returns a guarded `0.0` — which in cosine's codomain *means* "orthogonal, unrelated," a fabricated
answer that sails through `(f64::> … 0.9)` as a confident no-match. Convert both into matchable
variants so the verb becomes **total**: produces an ordinary value on every input. `dot` gets the same
treatment for its one hole. `coincident?` and `presence?` stay plain `bool` and their call sites do not
change.

## The classes — 64 sites, and only 30 of them move

| verb | sites | what changes |
|---|---|---|
| `presence?` | 17 | **NOTHING.** Already `total: true` (`purity.rs` — see the block naming it). Do not touch. |
| `coincident?` | 16 | **No call-site change.** Stays plain `bool`; becomes `total: true` once the shared guard stops raising. |
| `cosine` | 27 | returns `CosineOutcome` — these 27 must now match |
| `dot` | 3 | returns `DotOutcome` — 3 must match |
| `coincident-explain` | 1 | rides the same guard; keep its current return shape |

Counts measured 2026-08-03 by `grep -rhoF` (a `\b` after `?` cannot match — do not re-measure with a
word boundary). The stone's own table says 56/22; it is stale. **Report your own count before you
start.**

## Read in order

1. `src/runtime.rs:18725` `pair_values_to_vectors` — **THE ONE GUARD.** All four verbs route both
   operands through it; its dimension check (~`:18756`) is the single site the mismatch hole lives at.
2. `src/runtime.rs:18806` `eval_algebra_cosine` — calls the guard, then
   `Similarity::cosine(&vt,&vr).clamp(-1.0,1.0)`. The clamp stays.
3. `src/runtime.rs:19327` `eval_algebra_dot`.
4. `src/runtime.rs:18842` `eval_algebra_presence_q` and `:18896` `eval_algebra_coincident_q` — read
   them to confirm you are NOT changing their return type. `presence?` takes `require_holon` on both
   args and cannot reach the shared guard at all; that is why it is already total.
5. `src/runtime.rs:18942` `eval_algebra_coincident_explain`.
6. **The registration exemplar** — `SendOutcome`/`RecvOutcome` in `src/types.rs`. Six outcome walls
   already shipped in this arc (recv · send · close · accept · connect · spawn). Copy the shape of the
   nearest one; do not invent a registration pattern.
7. `src/rete/purity.rs` — the `intrinsic_meta` total block. `cosine`/`dot`/`coincident?` are currently
   `total: false` with a comment explaining exactly why; that comment is what your change retires.

## The enums — RULED, transcribe verbatim

```clojure
(:wat::core::defenum :wat::holon::CosineOutcome
  (:Similarity        [similarity <- :wat::core::f64])
  (:Degenerate        [side       <- :wat::holon::DegenerateSide])
  (:DimensionMismatch [expected   <- :wat::core::i64  got <- :wat::core::i64]))

(:wat::core::defenum :wat::holon::DotOutcome
  (:Computed          [product    <- :wat::core::f64])
  (:DimensionMismatch [expected   <- :wat::core::i64  got <- :wat::core::i64]))

(:wat::core::defenum :wat::holon::DegenerateSide :Target :Reference :Both)
```

**Two enums, not one:** `dot` performs no division, so a zero-magnitude operand yields an *honest*
`0.0` — a zero vector really does dot to zero. A shared enum would hand `dot` a `Degenerate` arm it can
never construct. (Same reasoning that split `TrySendOutcome` from `SendOutcome`.)

**`Degenerate` is ONE variant carrying a side, not three variants.** The caller acts identically on all
three; the side is a diagnostic, and `DimensionMismatch[expected, got]` is the in-family precedent for
a diagnostic as *fields*. `DegenerateSide` is three-valued rather than two bools precisely so
`(false,false)` — a `Degenerate` that is not degenerate — has no form.

**`dot` needs no `Degenerate` and no `:undefined`:** it sums `i8 × i8` products, bounded by
`d × 127²`, so reaching ±Inf needs `d ≈ 10³⁰⁴`. This closes the stone's one open question.

## The two-repo split — holon-rs is ONE LINE

- **holon-rs** (`src/kernel/similarity.rs` + wherever the const belongs): add
  `pub const DEGENERATE_EPSILON: f64 = 1e-10;` and point `cosine`'s existing guard at it. **No
  signature change, no behaviour change, nothing returned differently.** The `assert_eq!` dimension
  panics STAY — they become unreachable from wat and remain a backstop for other Rust callers.
- **wat-rs** does the rest. `holon::Vector::norm()` is already `pub` (`src/kernel/vector.rs:68`), so
  wat-rs can test each operand's norm itself, decide the side, and return `Degenerate` **before**
  calling `Similarity::cosine` — which is why the sibling never needs to return the distinction.

**Consume the const; do NOT copy `1e-10` into wat-rs.** A duplicated threshold that can drift is how
the mask comes back: wat says "not degenerate," calls cosine, holon says "degenerate" and returns
`0.0`.

## Implementation sketch

The guard stops raising and starts reporting:

```rust
// pair_values_to_vectors — the dimension arm returns the fact instead of a TypeMismatch.
// Shape it so each of the five callers decides what to DO with a mismatch: cosine/dot build
// their DimensionMismatch variant; coincident?/presence?/explain answer `false` (the predicate
// contract — an undefined comparison is not below the floor).
```

```rust
// eval_algebra_cosine, after the guard yields (vt, vr):
let (na, nb) = (vt.norm(), vr.norm());
let degenerate = match (na < holon::DEGENERATE_EPSILON, nb < holon::DEGENERATE_EPSILON) {
    (true,  true)  => Some(Side::Both),
    (true,  false) => Some(Side::Target),
    (false, true)  => Some(Side::Reference),
    (false, false) => None,
};
// Some(side) -> CosineOutcome::Degenerate[side]; None -> Similarity[clamped cosine]
```

## Why `coincident?`/`presence?` stay `bool` — do not "improve" this

Ruled, and the line is written into the stone so it is never re-litigated. A **measurement** may not
absorb its own undefined case (a fabricated `0.0` propagates into arithmetic that treats it as data). A
**predicate** may, *because absorbing it is the predicate's stated job*: `coincident?` asks "is the
distance below the floor?", and an undefined distance is not below the floor. NaN's `false` is an IEEE
accident nobody declared; a predicate's `false` is a documented total contract on a named verb.

## STOP triggers — rejection criteria. Ship nothing, report the gap.

1. **STOP-1 — your call-site count disagrees materially with 27/3/16/17/1.** Report the numbers and
   stop; the migration's scope is the brief's load-bearing claim.
2. **STOP-2 — a `cosine` call site cannot be expressed as a match** (e.g. it sits somewhere a match
   form is illegal). Report the site; do not weaken the enum to fit it.
3. **STOP-3 — `presence?`'s totality moves.** It is already `total: true` and its path does not reach
   the shared guard. If your change makes it non-total, you have altered a path you were not asked to.
4. **STOP-4 — the `_` wildcard on an enum scrutinee is doctrine-illegal.** Name every variant. The
   exhaustiveness error offers `"(or include `_` wildcard)"`; taking it is a rejected strike.
5. **STOP-5 — scope.** Do NOT mint the `:wat::rete::holon::*` fallback surface (that is #57's mint
   round). Do NOT arm any fence. Do NOT touch `coincident-explain`'s return shape.

## Gates — foreground, report every result line

```
cargo build --release --all-targets            # exit 0, ZERO warnings
cargo clippy --release --all-targets           # likewise
cargo test --release --test lint               # includes every_wat_scripts_file_loads
./target/release/wat --check <each touched .wat>
```

Plus, in **holon-rs**: `cargo test` and `cargo test --features simd` — the const is consumed by a
`cfg`-gated arm, so the default run does not compile the code you changed. Both configurations.

**Do NOT run `cargo nextest run`** — the orchestrator weighs the whole floor centrally once your tree
is quiescent.

Two lint traps that have bitten repeatedly in this arc: a doc comment or assert message that **parses
as a wat list** trips `no_inlined_wat_in_tests`; a `contains(...)` on a rendered error trips
`no_loose_string_assert`. Fix at the root — **no `rune:lint`**.

## Do not

Do not commit, push, stash, or revert anything you did not write. Do not add `#[allow(dead_code)]` to
silence a signal. Do not re-derive the enum names or field names — they are cast and ratified.
