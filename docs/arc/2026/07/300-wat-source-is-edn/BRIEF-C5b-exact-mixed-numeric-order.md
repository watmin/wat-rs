# BRIEF — C5b: mixed-numeric ordering becomes exact, through one door

> Read `DESIGN-STONE-C5b-exact-mixed-numeric-order.md` first — **it governs.** This brief is the strike path.
> Baseline: `HEAD = 1ec22cba`. ⚠ **The tree is NOT clean** — it carries 40 uncommitted `#[ignore]` deletions
> in `tests/` from a parked campaign, and the floor is deliberately red there. **That is not yours. Do not
> touch `tests/wat_lang/` or `tests/types/`, do not revert it, do not `git checkout --` anything.**

## The work, in one paragraph

`(< 9007199254740992.0 9007199254740993)` returns `false`; `true` is correct. The mixed-numeric ordering
arms coerce the exact operand **down** to `f64`, and above 2⁵³ that rounds two different numbers onto the
same float. Three separate hand-rolled ordering tables have this defect. Mint **one exact ordering door**,
route all three through it, and preserve each caller's distinct policy for the non-`Ordering` outcomes.

## Read in order — the rooms, and why you are being sent there

1. **`src/runtime.rs:9793–9913`** — `values_compare`. The authoritative table, and the only one reachable
   with mixed operands through the checked path. **Six lossy arms** (`:9802`, `:9805`, `:9819`, `:9822`,
   `:9846`, `:9849`) — every one coerces via `as f64` / `to_f64()`. Note the five arms *above* them
   (`i64↔BigInt`, `Rational↔i64`, `Rational↔BigInt`) already promote **up** and are exact. You are making
   the sixth family match the five, not inventing a policy.
2. **`src/runtime.rs:13011–13048`** — `walk_match_clause`'s `RawClause::Compare`. A hand-rolled clone that
   knows nothing of BigInt/Rational and returns `Ok((false, env))` for anything it does not know.
3. **`src/rete/matcher.rs:946–967`** — `compare_values`. A second clone. **Its doc comment names its own
   duplication and cites `runtime.rs ~:10615`, which is now `:13020`.** Fix that comment as part of this.
4. **`src/check.rs:12548`** — `check_comparison`. Read it to see *why* rooms 2 and 3 cannot receive mixed
   operands through the checked path: it `unify`s the two operand types. This is the fact that makes the
   collapse behaviour-neutral. You are not widening anything.
5. **`src/value/mod.rs`** — where the new module gets wired in.
6. **`tests/value/probe_rational_C5_mixed_compare.rs` + `.wat`** — the existing C5 surface; the gate lands here.

## Implementation sketch — fill it in, do not invent the shape

New file `src/value/numeric_order.rs`:

```rust
use num_traits::FromPrimitive;   // brings BigRational::from_f64 into scope

pub(crate) enum NumOrd { Ord(std::cmp::Ordering), Incomparable, NotNumeric }

/// Exact ordering over the numeric tower. Promotes to the narrowest EXACT common
/// representation — never down to f64. `Incomparable` means NaN was involved;
/// `NotNumeric` means at least one side is not a number. Callers own the policy
/// for those two — they deliberately differ.
pub(crate) fn numeric_order(a: &Value, b: &Value) -> NumOrd {
    // 1. FAST PATHS FIRST — same-type, native, zero allocation. Load-bearing for
    //    the rete hot loop; keep these ahead of everything else.
    //    (i64,i64) (u8,u8) (f64,f64) (BigInt,BigInt) (Rational,Rational)
    //
    // 2. EXACT INTEGER PAIRS — already correct today, keep as-is:
    //    (i64,BigInt) (BigInt,i64) -> BigInt
    //    (Rational, i64|BigInt) and mirrors -> BigRational
    //
    // 3. THE FIX — any exact numeric vs f64, both directions:
    //    - NaN            -> Incomparable
    //    - +inf / -inf    -> Ord(Greater) / Ord(Less) vs any finite exact value
    //    - finite f64     -> BigRational::from_f64(x).unwrap() and compare exactly
    //
    // 4. _ => NotNumeric
}
```

**The one contract decision, pinned:** the door returns **three** states, never two. Squashing
`Incomparable` into `NotNumeric` is what let the three tables drift apart in the first place.

### The exact-conversion call, and the trap beside it

```rust
let xr = num_rational::BigRational::from_f64(x).expect("finite checked above");
```

- `from_f64` → `Ratio::<BigInt>::from_float` → **`integer_decode()`**: mantissa × 2^exp, the float's *exact*
  value. Verified in the vendored source, `num-rational-0.4.2/src/lib.rs:287`.
- ⛔ **`Ratio::approximate_float` is NOT it.** Same file, ~1000 lines down, iterative with a max-error
  bound. Using it silently reintroduces this exact bug and every gate row above 2⁵³ becomes meaningless.
  **STOP-3 fires if `approximate_float` appears anywhere in your diff.**
- `from_f64` returns `None` for NaN/±∞ — handle both **before** the call, per the sketch.

### The three call sites — policies, which DIFFER, and all three are preserved exactly

| room | `Ord(o)` | `Incomparable` (NaN) | `NotNumeric` |
|---|---|---|---|
| `values_compare` | `Some(o)` | `Some(Ordering::Equal)` ← preserves today's NaN→Equal | fall through to the **non-numeric arms** (String/bool/keyword/Instant/Duration/Vec/Tuple/Option/Result/Vector), which must keep working |
| `walk_match_clause` | `o` | `Ordering::Equal` | `return Ok((false, env))` ← preserve the silent-false; it is Clara no-error semantics |
| `matcher.rs compare_values` | `Some(o)` | **`None`** ← this one differs from the other two, deliberately | `None` |

**Do not "harmonise" these.** The stone's whole point is that the *table* is shared and the *policy* is not.

## Blast radius

`src/value/numeric_order.rs` (new) · `src/value/mod.rs` (one line) · `src/runtime.rs` (two functions) ·
`src/rete/matcher.rs` (one function + its doc comment) · `tests/value/probe_rational_C5_mixed_compare.{rs,wat}`.

**No `.wat` corpus changes. No `src/check.rs` changes. No new types in the language. No new wat verbs.**

## The gate — every row, both directions

Rows 2, 4, 6, 9, 10 are **RED at HEAD** — they are the bug. Rows 3, 5 are **green at HEAD by accident** and
must stay green; they pin the accident. Rows 11, 12, 13 are **regression guards that must not move.**

| # | assertion | at HEAD |
|---|---|---|
| 1 | `BigRational::from_f64(9007199254740993.0)` is exact (unit-level; proves the mechanism) | new |
| 2 | `(< 9007199254740992.0 9007199254740993)` ⇒ **true** | **RED** |
| 3 | `(< 9007199254740993 9007199254740992.0)` ⇒ false | green by accident |
| 4 | `(> 9007199254740993 9007199254740992.0)` ⇒ **true** | **RED** |
| 5 | `(<= 9007199254740992.0 9007199254740993)` ⇒ true | green by accident |
| 6 | `(>= 9007199254740992.0 9007199254740993)` ⇒ **false** | **RED** |
| 7 | `(= 9007199254740992.0 9007199254740993)` ⇒ false — category-aware, untouched | green |
| 8 | `(< 1N 2.0)` ⇒ true · `(> 3.0 1/2)` ⇒ true | green |
| 9 | bigint > 2⁵³ vs f64 orders exactly | **RED** |
| 10 | rational > 2⁵³ vs f64 orders exactly | **RED** |
| 11 | `(< 1 ##Inf)` ⇒ true · `(> 1 ##-Inf)` ⇒ true | green — **must stay** |
| 12 | `(< 1 ##NaN)` ⇒ false · `(<= 1 ##NaN)` ⇒ **true** | green — **must stay** |
| 13 | `(< 1 2.0)` ⇒ true · `(< 2.0 1)` ⇒ false | green — **must stay** |

**Row 12 pins behaviour the stone calls WRONG** (IEEE says every NaN comparison is false). That is
deliberate and is not yours to fix: it is the negative control for STOP-2, and it makes the flagged NaN
defect falsifiable when its own stone is drawn. **Write the row; add a comment saying it pins a known,
separately-flagged wart.** Per `docs/DUNGEON-CRAWL.md` Phase 3, a keepable control is **kept as a test.**

If you find a row you cannot express through the wat surface, express it as a Rust unit assertion on
`numeric_order` directly and **say which rows you had to move**, and why.

## Negative controls — keep the keepable ones

For each control you build to prove a row can still fail: **is it expressible as test code or a fixture?**
If yes, **bank it as a test.** If it needs an `src/` mutation, report it with the reason. Discarding is a
declared exception with a stated reason, never the default.

## STOP triggers — these REJECT; none of them is permission to ship less

- **STOP-1** — any *same-type* comparison changes behaviour (`i64·i64`, `f64·f64`, String, bool, keyword,
  Instant, Duration, Vec, Tuple, Option, Result, Vector). The fast path is load-bearing for the rete loop.
- **STOP-2** — NaN behaviour changes at any of the three callers, including the fact that they disagree.
- **STOP-3** — `approximate_float` appears in the diff.
- **STOP-4** — the work seems to need a `.wat` corpus change or a `src/check.rs` change. It does not.
  If it appears to, that is a finding: report it and stop.
- **STOP-5** — the floor shows a failure you cannot name. **Do NOT re-run** — a re-run that goes green
  destroys the evidence. `scripts/floor.sh` has already kept the untruncated log: copy the failing test's
  whole stdout+stderr block **verbatim** (never `| head`, never a summary), name the exact assertion that
  fired, and report. Only then may anything be re-run. **There is no such thing as a known flake.**

## Verify — in this order

```
cargo build --release --tests
cargo clippy --workspace --all-targets --release -- -D warnings      # must be 0
scripts/floor.sh                                                      # read the SUMMARY LINE
```

Never a piped exit code — `cargo nextest … | tail` returns `tail`'s exit, not nextest's.

**Expected floor:** `4606 run / 4566 passed / 40 failed / 82 skipped` — the **same 40** that are red at
HEAD, in `tests/wat_lang/` and `tests/types/`, from the parked campaign. Plus your new gate rows passing.
**If a 41st test is red, or a different test is red, that is yours and STOP-5 governs.**

## How to work

You are a **rider, not the orchestrator. Ending your turn ENDS you** — it does not suspend you, and nothing
will wake you. There is no notification coming. ⛔ **Run every build and test in the FOREGROUND and block on
it.** Do not use `run_in_background`. Do not set a Monitor. Do not poll and end your turn. Four riders on
these arcs died exactly that way. Your turn ends when the numbers are in your hands.

Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. **Leave the work uncommitted.** Never `git commit`,
`push`, `stash`, `revert`, or `checkout --` — `stash@{0}` holds unrelated work and the tree holds a parked
campaign.

## Report

- the door's final shape, and any place the sketch was wrong
- the gate table, one row per line, with its **before** and **after** result
- **every row that was green at HEAD and is still green** — name them; that is the regression evidence
- the clippy count and the floor **Summary line, verbatim**, with the arithmetic
- negative controls: which you kept as tests, which you did not and why
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.** Every count in this arc's
  briefs has been wrong at least once; re-measure what you act on, and say so when I was wrong.
