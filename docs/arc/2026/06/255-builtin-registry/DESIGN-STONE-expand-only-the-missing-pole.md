# DESIGN — STONE 1 of 2: `ExpandOnly` — the missing pole of the ExpandTime axis

> **Builder, 2026-09-01:** *"so.. we need a new flavor for macro-error to be expand-time only?"* →
> *"if you think it's more tractable as 2 stepping stones, then it's 2."*
>
> **This is stone 1: the coordinate and its derived consequence. Green, no behaviour change.**
> Stone 2 is the mirror wall and is drawn separately.

## ⛔ THE ACTUAL ISSUE — it is not an exemption, it is a missing coordinate

`purity_mandated_examples` (`src/intrinsic/mod.rs:1305`) went RED:

```
pure+det intrinsic `:wat::core::macro-error` has no runnable @example (≥1 required by contract)
```

The tempting reading is *"a verb that always raises needs an exemption."* **That reading is wrong.**
The builder's question — *"how is this failing to be placed in an existing coordinate?"* — is what
found the real one.

`wat/runtime-meta.wat`, the `ExpandTime` axis:

```
:Legal        "May be called inside a `defmacro` body during expansion."   <- ALSO, not ONLY
:RuntimeOnly  "Needs state that does not exist yet at expand time."
:Preserving   its sub-forms'
:Unreviewed   nobody measured
```

`macro-error` declares `Legal` — the closest available coordinate. The disk asserts something
**stronger**, twice:

```
src/intrinsic/macro_error.rs:28   "Macro-body-only — legal ONLY where a `defmacro` body's …"
src/value/signal.rs:529           "Macro-body-only: evaluated at expand time, NEVER post-expansion"
```

★ **There is no variant for "expand-time ONLY."** `Legal` means *also*; the truth is *only*.
`RuntimeOnly` is the exact mirror — *"needs state that does not exist yet at expand time"* — and the
axis never minted the opposite pole.

★★ **That is the whole gate failure, and it explains it rather than excusing it:** a **runnable**
`@example` is evaluated at RUNTIME. The contract asks this verb to demonstrate itself at a tier
where it does not exist. `purity_mandated_examples` carries a hidden premise — *every Pure ∧
Deterministic verb has a runtime call site* — true for all 455 registrations until an expand-time-only
one appeared.

★★★ **And the shape is one this arc already knows.** `Totality` has four variants because collapsing
*"measured partial"* into *"nobody looked"* is a lie; every axis carries `Unreviewed` for that reason.
**Collapsing "only at expand time" into "also at expand time" is the same lie**, and a gate finding a
missing coordinate is a gate doing its job.

## ★ THE AXIS IS ENFORCED, AND ITS WALL IS HALF-BUILT

This is what makes the pole earn its place rather than be a documentation flavour. Measured:

```
macros/eval.rs:169   if is_expand_time_legal(head)   gates what a defmacro body MAY CALL
                     Legal | Preserving pass · RuntimeOnly | Unreviewed refused
outside a macro body                                 NOTHING. No consumer. No gate.
```

- **`RuntimeOnly`** — refused INSIDE a macro body. **Wall exists.**
- **`ExpandOnly`** — should be refused OUTSIDE a macro body. **Wall does not exist**, and there is no
  name to hang it on.

Measured consequence today: `(:wat::core::macro-error "x")` at top level passes `--check` (exit 0)
and raises at run. **Misuse is caught at runtime, by a raise — the bottom rung of the ladder.**

## THE ONE CONTRACT DECISION — pinned

**The doc gate's third branch is DERIVED from the coordinate, never a name.** An `ExpandOnly` verb
cannot carry a runnable example, so `@example-norun` becomes its **correct and required** form —
checkable in both directions, exactly as the existing two branches are. `ExpandOnly` + a runnable
example must be as unrepresentable as `Pure ∧ Deterministic` + no runnable example is today.

## What ships in STONE 1 — and what does NOT

**Ships:**
1. `:ExpandOnly` minted in `wat/runtime-meta.wat`'s `ExpandTime` defenum, with a doc sentence that
   states the mirror explicitly (`RuntimeOnly`'s opposite, not a synonym for `Legal`).
2. The Rust lift follows automatically — `wat_enum_from!` reads the `defenum` at BUILD time; the
   `wat-macros` arms that spell each variant gain one row.
3. `macro-error` re-declared `@ExpandTime ExpandOnly` (from `Legal`, which was wrong in one
   direction).
4. `purity_mandated_examples` gains its third branch, derived from the coordinate.
5. `is_expand_time_legal` (`macros/eval.rs:426`) must ACCEPT `ExpandOnly` — an expand-time-only verb
   is by definition legal inside a macro body. ⚠ **Miss this and stone 1 breaks macro-error at its
   ONLY legitimate call site.**

**Does NOT ship — stone 2:** the mirror wall. Nothing yet refuses an `ExpandOnly` head outside a
macro body. That is a **behaviour change**, it is where a red would be attributable, and bundling it
here would make stone 1's red un-attributable between "the coordinate is wrong" and "the wall is
wrong."

## ★ THE PREDICTION — falsifiable

```
purity_mandated_examples                RED today  ->  GREEN (third branch, derived)
macro-error inside a defmacro body      legal      ->  legal, UNCHANGED (is_expand_time_legal accepts)
macro-error at top level                --check 0, raises at run  ->  UNCHANGED. Stone 2's job.
every other verb's ExpandTime           169 Legal · 288 Unreviewed  ->  UNCHANGED
```

⚠ **Stone 1 changes NO runtime behaviour.** If any `.wat` program behaves differently, the coordinate
was mis-applied and that is a finding.

## Out of scope = REJECTED (not deferred)

- **The mirror wall.** Stone 2, drawn as its own design. Named here, not deferred vaguely.
- **Re-declaring any other verb's `@ExpandTime`.** ⚠ Population is **ONE**, measured:
  `macro-error` is the only verb on disk claiming macro-body-only. A second candidate is a finding,
  not a chore.
- **`macros/eval.rs`'s 58-name residue** — the homing backlog for verbs with no registration site.
  Untouched.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **mint the pole + derive the branch (stone 1 only)** | YES | YES | YES | YES | ✅ **ADMITTED** |
| pole + branch + mirror wall in one stone | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |
| exempt `macro-error` from the gate by name | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| a `rune:` at the site instead of a coordinate | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| tag it `@example-norun` and leave `Legal` | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| declare it non-Pure so the else-branch accepts norun | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **all-in-one Simple? NO** — a coordinate change and a new check-time refusal in one stone; a red
  could not be attributed to either.
- **exempt-by-name Honest? NO** — a hand-list keyed on one verb, which is the shape this whole
  campaign exists to retire. `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`
- **a-rune Honest? NO** — a rune says *"this looks like a finding but is correct."* This IS a
  finding: the axis is incomplete. A rune would suppress the evidence of a missing coordinate.
- **norun-with-`Legal` Honest? NO** — the gate would still demand a runnable example (the `if`
  branch), so it does not even work; and it leaves `Legal` asserting *also-callable-at-runtime*,
  which the disk contradicts twice.
- **declare-it-impure Honest? NO** — it raises; it does not *effect*. Purity is a property of the
  verb, and `Option/expect` is the standing precedent for a raising verb that is `Pure`. Lying about
  one axis to satisfy a gate on another is the exact defect this stone is fixing.

## Acceptance

| what | command | expected |
|---|---|---|
| the pole exists in wat | `wat/runtime-meta.wat` `ExpandTime` | `:ExpandOnly` present, mirror documented |
| the Rust lift carries it | `wat_doc::ExpandTime::ExpandOnly` | compiles; `wat-macros` spells it |
| `macro-error` declares it | its `@ExpandTime` | `ExpandOnly` |
| ★ the doc gate goes GREEN by DERIVATION | `purity_mandated_examples` | passes, third branch reads the coordinate |
| ★ `ExpandOnly` + runnable example is REFUSED | the new branch, tested by breaking it | the gate fires |
| ★ macro-error still legal in a macro body | `is_expand_time_legal` accepts `ExpandOnly` | UNCHANGED |
| ★ no runtime behaviour moves | top-level `macro-error` call | `--check` 0, raises at run — as today |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5110/5110, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
