# DESIGN — Stone C5b: mixed-numeric ORDERING is exact, through ONE door

**Filed 2026-08-15. RULED:** builder, *"we fix the bug — c5 first."* Supersedes nothing; **completes C5**,
whose pinned contract this implementation has never delivered at the top of the i64 range.

> Read `NOTE-C5-mixed-compare-loses-precision-above-2-53.md` first — it is the grounding that opened this.
> This stone is what the census the NOTE demanded actually found, which is **more than the NOTE assumed.**

## The bug, in one line

**`(< 9007199254740992.0 9007199254740993)` returns `false`. The true answer is `true`.**

Re-grounded live this session, via the MCP eval, both directions:

```clojure
(:wat::core::< 9007199254740992.0 9007199254740993)   ⇒ false      ; TRUE is correct
(:wat::core::< 9007199254740993 9007199254740992.0)   ⇒ false      ; correct, BY ACCIDENT
```

2⁵³ = 9007199254740992 is the last integer f64 represents exactly. 2⁵³+1 is not representable and rounds
down to 2⁵³, so coercing the `i64` operand to `f64` makes the operands compare **equal** and `<` is `false`
in both directions. One accident, not two correct answers — **a one-direction test would be green over it.**

## ★ What the census found that the NOTE did not

The NOTE said *"census the mixed-numeric compare arms first; do not assume there is one."* Correct instinct.
There are **three independent hand-rolled ordering tables**, and they do not agree with each other:

| # | site | numeric tower it knows | mixed i64↔f64 | not-comparable | NaN |
|---|---|---|---|---|---|
| 1 | `src/runtime.rs:9793` `values_compare` | i64 · u8 · f64 · **BigInt · Rational** | lossy `as f64` | `None` → caller raises `TypeMismatch` | → `Equal` |
| 2 | `src/runtime.rs:13020` `walk_match_clause` `RawClause::Compare` | i64 · u8 · f64 | lossy `as f64` | **silent `false`** | → `Equal` |
| 3 | `src/rete/matcher.rs:954` `compare_values` | i64 · u8 · f64 | lossy `as f64` | `None` → constraint fails | → **`None`** |

Table 3's own doc comment says it outright — *"Mirrors the Compare arm in `walk_match_clause`
(`runtime.rs` ~:10615)"* — **naming its own duplication, and citing a line number that has since moved to
13020.** A comment that tracks a clone by line number is a clone that will drift, and it has.

So the precision defect is a *symptom*. The disease is **one ordering semantics with three implementations**,
of which one is authoritative and two are impoverished, stale copies.

## The reachability ruling — why ONE DOOR is behaviour-neutral for the rete tables

The obvious worry: routing tables 2 and 3 through the full tower **widens** them — a rete clause comparing a
bigint to an f64 would go from "silently no-match" to "compares". **It does not, and the disk says so.**

`src/check.rs:12548 check_comparison` — the gate for BOTH rete clause paths — requires
`unify(&l, &r, ...)`, i.e. **operands of the same type**. C5 widened `infer_equality` (the
`:wat::core::<` *call* path). It did **not** touch `check_comparison` (the `:wat::form::matches?` *clause*
path). Mixed-numeric operands are rejected at check before they can reach table 2 or table 3.

**Therefore:** the mixed arms in tables 2 and 3 are unreachable through the checked path. They exist as
unchecked-path defence (`eval_in_frozen` runs no checker — `values_equal`'s own comment at `runtime.rs:9552`
records exactly this). Making them exact is a **correctness repair of unreachable-but-wrong code**, not a
semantic widening. **No ruling needed; no rete behaviour changes.**

This is the check that dissolved a question before it was asked. It cost one `awk`.

## The fork the NOTE demanded be settled — SETTLED

The NOTE flagged that C5's two justifications part company at exactly this boundary, and refused to pick:

| | rule | consequence |
|---|---|---|
| **EXACT** | compare mathematical values | correct at every magnitude; **diverges from clj** above 2⁵³ |
| **CLJ-FAITHFUL** | keep coerce-to-f64 | matches clj; **keeps the wrong answers**, and C5's wording must be corrected |

**Ruled EXACT.** The builder called it a bug and said fix it; CLJ-FAITHFUL is the reading under which it is
*not* a bug but a wording defect, and that reading was declined. C5's pinned contract already says
**"the numeric-value comparison"** — EXACT is the option that makes the shipped contract true rather than
the one that edits the contract down to the implementation.

⚠ **Two consequences that must ship with the fix, not after it:**
1. **This is a deliberate, documented divergence from Clojure**, in an arc whose whole thesis is Clojure
   familiarity. It must be written down as such — see "The record" below. An undocumented divergence is
   how the next reader mints a defect report against correct code.
2. The NOTE's claim *"clj returns false here too"* was recorded last session and **has not been re-verified
   this session** (no JVM in this loop — builder: *"i do not wish to have the jvm requirement in our CI
   tooling"*). It is carried as **unverified**, and the divergence claim in the record must say so.

## Two corrections to the NOTE

1. **The affected family is FOUR ops, not six.** The NOTE says *"the family is six ops and the census above
   covers five."* `=` and `not=` route through `values_equal`, which is **category-aware** (C4): an i64 and
   an f64 are different categories → `Some(false)`, *no coercion happens at all*. They are structurally
   immune to this defect. The family is **`< > <= >=`**.
2. **The lossy pairs are six, not two** — `values_compare` coerces to f64 for `i64↔f64`, `BigInt↔f64`, AND
   `Rational↔f64`, in both directions. `(< 1N 2.0)` and `(> 3.0 1/2)` are the same defect one type over.

## ⚠ FLAGGED, NOT FOLDED — the NaN policy is separately wrong

`values_compare` maps NaN → `Ordering::Equal` (`unwrap_or`, documented at `runtime.rs:9949`). Because
`<=` is `order != Greater`, this means:

```clojure
(<= 1 ##NaN)   ⇒ true      ; IEEE 754 says every NaN comparison is false except !=
(>= 1 ##NaN)   ⇒ true
```

That is a **second, independent defect** sitting one line from the first. It is **NOT in this stone** —
changing it alters same-type f64 behaviour across the whole substrate, which is a separate contract
question with its own blast radius. C5's own posture governs: *flag, don't fold.* This stone
**preserves NaN→Equal exactly**, and the flag is recorded so the next reader finds it as a known open
question rather than as a fresh discovery.

Table 3 already disagrees (NaN → `None`). **That disagreement is real and must survive the collapse** —
it is a per-caller *policy*, not a table difference. This is the whole reason the door returns three states.

## The contract

**One door owns the ordering table. Each caller owns its own policy for the two non-`Ordering` outcomes.**

```rust
/// Three outcomes, because the three callers have three different policies for the
/// last two — and conflating "NaN" with "not a number type" is precisely why the
/// three tables diverged.
pub(crate) enum NumOrd {
    Ord(std::cmp::Ordering),
    Incomparable,   // both numeric, but NaN was involved
    NotNumeric,     // at least one side is not a numeric Value
}
```

**Exactness rule — promote to the narrowest EXACT common representation, never down to f64:**

| operands | promote to | note |
|---|---|---|
| same type (`i64·i64`, `u8·u8`, `f64·f64`) | native | **fast path, zero alloc — must stay first** |
| integer × integer (`i64 · BigInt`) | `BigInt` | already exact today; unchanged |
| any × `Rational` | `BigRational` | already exact today; unchanged |
| **any exact integer × `f64`** | **`BigRational`** | **the fix** — the f64 converts *exactly* |
| **`Rational` × `f64`** | **`BigRational`** | **the fix** |

The substrate's own established pattern is *promote up*: five of six mixed pairs already do. Only the f64
pairs promote **down**. This stone makes the sixth consistent with the five — it is not a new idea, it is
the existing idea finished.

## The mechanism — and the trap sitting next to it

`num_rational::BigRational` (`Ratio<BigInt>`) has **`Ratio::<BigInt>::from_float` / `FromPrimitive::from_f64`**.
Verified this session by reading the vendored source
(`~/.cargo/registry/.../num-rational-0.4.2/src/lib.rs:287`): it uses **`integer_decode()`** — mantissa ×
2^exponent — so a finite f64 converts to its **exact** rational value, no approximation. It returns `None`
only for non-finite input (NaN, ±∞).

> ⛔ **THE TRAP.** `Ratio::approximate_float` sits ~1000 lines below in the same file and is an **iterative
> approximation with a max-error bound**. It is the wrong function and it would silently reintroduce this
> exact bug in a form no test above 2⁵³ would catch. **Use `from_float`/`from_f64`. Never `approximate_float`.**

Neither is used anywhere in the tree today — `grep -rn "from_f64\|FromPrimitive" src/` returns nothing.
This composition is **new**, which is why it was probed before this stone was written and why the first
row of the gate is a direct unit assertion on the conversion itself.

**±∞ must be handled before the conversion** (`from_float` returns `None` for it): `+∞` is `Greater` than
every finite exact value, `−∞` is `Less`. Today the f64 fast path gets this right via `partial_cmp`;
the mixed path must not lose it. **`(< 1 ##Inf)` must stay `true`.**

## Rooms

```clojure
{:door   "NEW src/value/numeric_order.rs — the enum + `numeric_order(a,b) -> NumOrd`. New module; wire
          into src/value/mod.rs. Home chosen because BOTH src/runtime.rs and src/rete/matcher.rs already
          depend on src/value, so neither import creates a cycle."
 :call-1 "src/runtime.rs:9793 values_compare — replace the 6 lossy f64-cross arms. Policy: Ord(o)->Some(o),
          Incomparable->Some(Equal) (PRESERVE NaN->Equal), NotNumeric->fall through to the NON-numeric arms
          (String/bool/keyword/Instant/Duration/Vec/Tuple/Option/Result/Vector) which MUST keep working."
 :call-2 "src/runtime.rs:13020 walk_match_clause RawClause::Compare — replace the numeric arms.
          Policy: Ord(o)->o, Incomparable->Equal (preserve), NotNumeric->return Ok((false, env)) (preserve
          the silent-false; it is Clara no-error semantics, NOT a defect to fix here)."
 :call-3 "src/rete/matcher.rs:954 compare_values — replace the numeric arms.
          Policy: Ord(o)->Some(o), Incomparable->None (PRESERVE — table 3 differs from 1 and 2 here and
          that difference is deliberate), NotNumeric->None. DELETE the stale '~:10615' line reference in
          the doc comment; point at the door instead."
 :tests  "tests/value/probe_rational_C5_mixed_compare.rs (+ .wat) — the C5 surface already exists; the new
          gate rows land here."}
```

## Out of scope — affirmatively cut, not deferred

- **NaN ordering policy** (`(<= 1 ##NaN)` ⇒ `true`). Flagged above; separate contract, separate blast
  radius. This stone preserves today's behaviour byte-for-byte at all three callers.
- **`=` / `not=`.** Category-aware per C4; structurally immune. Untouched.
- **`u8` mixed arms.** No mixed-`u8` arm exists at any of the three tables today; `u8` vs anything else is
  `NotNumeric` and stays so. Adding them is a widening this bug does not ask for.
- **Widening `check_comparison`** to admit mixed-numeric rete clauses. That is the C5 widening applied to a
  second gate — a real question, and not this one.
- **Collapsing the three callers' policies.** They differ deliberately; the door exists precisely so they
  can differ *visibly*.

## STOP triggers

- **STOP-1 — a same-type comparison changes behaviour.** `i64·i64`, `f64·f64`, `String`, `bool`, `keyword`,
  `Instant`, `Duration`, `Vec`, `Tuple`, `Option`, `Result`, `Vector` must be byte-identical. The fast path
  is load-bearing for the rete hot loop; if it moved, stop.
- **STOP-2 — NaN behaviour changed at ANY of the three callers.** All three policies are preserved as they
  are today, including the fact that they disagree.
- **STOP-3 — `approximate_float` appears anywhere in the diff.** It is the trap; its presence means the
  exactness is fake and every gate row above 2⁵³ is meaningless.
- **STOP-4 — a `.wat` corpus change is needed.** It is not. If it appears to be, that is a finding.
- **STOP-5 — the floor moves off 0 failures for any reason you cannot name.** A red is a red; capture it
  whole and verbatim, name the arm, do not re-run.

## RED spec — the gate. Both directions on every row.

The load-bearing rows are the ones that are RED at HEAD. The rows that are GREEN at HEAD are pinning the
**accidents**, and they are not optional — one of them passes today for the wrong reason.

| # | assertion | at HEAD |
|---|---|---|
| 1 | `BigRational::from_f64(9007199254740993.0)` is exact (unit-level, proves the mechanism) | n/a — new |
| 2 | `(< 9007199254740992.0 9007199254740993)` ⇒ **true** | **RED** ← the bug |
| 3 | `(< 9007199254740993 9007199254740992.0)` ⇒ **false** | green **by accident** — pins it |
| 4 | `(> 9007199254740993 9007199254740992.0)` ⇒ **true** | **RED** |
| 5 | `(<= 9007199254740992.0 9007199254740993)` ⇒ **true** | green by accident |
| 6 | `(>= 9007199254740992.0 9007199254740993)` ⇒ **false** | **RED** |
| 7 | `(= 9007199254740992.0 9007199254740993)` ⇒ **false** (category-aware, unchanged) | green |
| 8 | `(< 1N 2.0)` ⇒ true · `(> 3.0 1/2)` ⇒ true — the other two lossy pairs still right | green |
| 9 | a bigint above 2⁵³ vs an f64 orders exactly (the `BigInt↔f64` mirror of row 2) | **RED** |
| 10 | a rational above 2⁵³ vs an f64 orders exactly (the `Rational↔f64` mirror) | **RED** |
| 11 | `(< 1 ##Inf)` ⇒ true · `(> 1 ##-Inf)` ⇒ true — ±∞ survives the exact path | green — must stay |
| 12 | `(< 1 ##NaN)` ⇒ false · `(<= 1 ##NaN)` ⇒ **true** — NaN policy PRESERVED, wart and all | green — must stay |
| 13 | ordinary small mixed numerics unchanged: `(< 1 2.0)` ⇒ true, `(< 2.0 1)` ⇒ false | green |

**Row 12 pins a behaviour this stone calls wrong.** That is deliberate: it is the negative control for
STOP-2, it is keepable as ordinary test code, and per `docs/DUNGEON-CRAWL.md` Phase 3 a keepable control is
**kept as a test**, not performed and discarded. It also makes the flagged NaN defect *falsifiable later* —
when that stone is drawn, this row is the thing that has to change, and it will be found.

## The record — ships WITH the fix, not after

- **`DESIGN-STONE-rational-C5-mixed-compare.md`** gets an amendment note: its *"the numeric-value
  comparison"* line is now **true as implemented** (it was not, from the day it landed), and the
  **deliberate divergence from clj above 2⁵³** is stated there, with the clj claim marked unverified.
- **`NOTE-C5-mixed-compare-loses-precision-above-2-53.md`** gets its two corrections (four ops not six; six
  lossy pairs not two) and a pointer to this stone as the settled disposition.

## Kin

- `NOTE-C5-mixed-compare-loses-precision-above-2-53.md` — the grounding, and the fork this stone settles.
- `DESIGN-STONE-rational-C5-mixed-compare.md` — the contract this defect violated; correctly superseded
  237.8a, and is **not** in question.
- `docs/arc/2026/06/255-builtin-registry/SEAM.md` — the live breadcrumb; names this as RULED-not-built.
