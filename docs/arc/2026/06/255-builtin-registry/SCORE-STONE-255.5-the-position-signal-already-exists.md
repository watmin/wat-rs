# SCORE — STONE 255.5: the POSITION SIGNAL already exists

Branch: `main`. **Committed, not pushed.** Drawn against `8df49e008` (draw `4e4e53b2e`).
Parent: `BRIEF-STONE-255.5-the-position-signal-already-exists.md`.
Floor / workspace clippy / census: orchestrator. Crate clippy + lint suite run here.
**No corpus `.wat` converted.** New fixtures only. 8d-ii not started.

## Gate: the 179-file delta — ONE tree

Same spread as 255.1–255.4: every 12th of the 8d-i census `files.txt` (**179**).
Originals: **live tree**. Converted: `/tmp/8d-i/census/tree-after-1`.
Timed one file first: annotation fixture **0.264 s**.

The brief's baseline **77** is the orchestrator's 255.4 WEIGH tree. This stone's
before is **this tree at 255.4 (81)**. Like-for-like.

| | this tree at 255.4 | this stone |
|---|---|---|
| originals clean | **161 / 179** | **161 / 179** |
| still clean after conversion | 80 | **95** |
| ⛔ **REGRESSIONS** | **81** | **66** |

**15 closed. 0 newly_broken. Net −15.**

### Classification (66)

| cause | n | vs this tree at 255.4 (81) |
|---|---|---|
| `UnresolvedReference` | **23** | 48 → 23 |
| `ReteCheckErrors` | **21** | 16 → 21 (five left UR and hit rete) |
| `MalformedDecl` | 13 | |
| `CheckErrors` / OTHER / `UnknownNamedType` | 9 | |

`:wat::WatAST` (**65 occurrences**) is **gone**. So are Instant / Overlay / HolonAST
as annotation URs. Remaining UR paths are not the annotation class:

| path | n | kind |
|---|---|---|
| `:probe-homog::serve-proc` | 9 | declaration **and** later calls; `defn` is **not** `role = declare` |
| `:wat::query::mem-store::start` · `:wat::telemetry::journal::start` | 12 | **functions**, not types |
| `:wat::rete::core::defn` | 4 | reconstruct of a nested ns |
| `:wat::core::not-a-special-form` | 3 | ⚠ deliberate negative-test name — **stays** |
| `:probe-homog::{Op,Reply}` · `:probe::same-shape?` | 6 | declaration names |
| `:wat::enum::Pure` | 2 | purity marker in a declare form, not items[1] |
| `:wat::core::i64` | 2 | leftover, not the 65-count class |

## What landed — the flag, not a new question

Type slots are derived from the **same predicates the parsers already use**:

- after `is_param_annotation_arrow` (`<-` or `:-`) in a Vector — argspec triples
- after `is_return_arrow` (`->` or `:-`) in an Ordinary list — return slot
- nested List/Vector in a type slot — parametric args / tuple elements
- type-binder heads already had `also_accept_type = true` (Stone ②)

`argspec::parse_triple` now calls `is_param_annotation_arrow` (one door).

`normalize.rs:525-528` no longer claims *"normalize carries no position context
and needs none."* That was true for `wat.type` and false for every mixed
namespace.

### Declaration names (step 3)

`items[1]` of a `role = declare` form rewrites to the identity keyword without
asking `is_resolvable_call_head`. **`defn` is not `role = declare`** (it is a
macro/special form without that row), so `probe-homog/serve-proc` as a `defn`
name still URs. `:restricted-to` resolution **did not close**.

## Negative control

`(wat.time/Instant)` in **call** position still `UnresolvedReference`. Fixture
`tests/resolve/probe_arc255_5_position_signal.wat.bad`. The widening did not
loosen call heads.

## What the door cannot express

`also_accept_type` answers *"is this a known type?"* It cannot:

- bind a **name** (`defn` items[1] is not a type slot and `defn` is not declare-role)
- treat a **function** (`mem-store/start`) as resolved just because it sits near `:-`
- keep a **deliberate negative-test** name (`not-a-special-form`) from UR — and must not

## Test count

Predicted **+2**. `cargo nextest list --release -p wat`: **5319** (was 5317).

## Walls I ran

- crate clippy `-p wat --all-targets -D warnings` — **0**
- `one_variant_separator` / `one_param_spec` / `no_loose_string_assert` — pass
- `probe_arc255_5_position_signal` (accept + call-position refuse) — pass

Floor / workspace clippy / census: orchestrator. Do not push. Do not start 8d-ii.

---

# ORCHESTRATOR'S WEIGH — independent re-run, 2026-09-21. **ACCEPTED.**

| row | result |
|---|---|
| `scripts/floor.sh` | ✅ **5941/5941 passed**, exit 0 |
| clippy `-D warnings --all-targets --workspace` | ✅ **0** |
| `census.sh --diff` | ✅ `no STOP-8` |
| no corpus `.wat` converted | ✅ |
| ⭐ **THE DELTA — my tree, like-for-like** | **77 → 64. NET −13.** |

**Arc: 104 → 97 → 80 → 77 → 64.** Two trees again agree on the direction and closely on the net
(executor 81 → 66 = −15; orchestrator 77 → 64 = −13).

**Classification (64):** `unresolved reference` **21** (was 44) · `ReteCheckErrors` 21 ·
`defsurface` 8 · `ProgramBodyEvalFailed` 3 · `UnknownNamedType` 1.

⭐ **`:wat::WatAST` — 65 occurrences, the single largest path in the arc — is GONE**, along with
`Instant` / `Overlay` / `HolonAST` as annotation URs. `unresolved reference` has more than halved.

## ⭐ THE NON-VACUITY CONTROL HELD — the widening did not leak

This stone **loosens** a check, so the row that mattered was proof it did not loosen everything.
Measured on the built binary, **call** position:

```
(wat.time/Instant)  → UnresolvedReference
(wat/WatAST)        → UnresolvedReference
(wat.core/i64)      → UnresolvedReference
```

⭐ **A type is now accepted where a type belongs and still refused where it does not.** That is the
whole claim of the stone, and it is measured rather than asserted.

## ⭐ THE METHOD IS WHAT MADE THIS ARC WORK — recorded for the next one

The class closed here is the one 255.2 attacked with **three predicates over leaves**, each of which
**raised** the count (116 · 108 · 114). The cure was not a cleverer predicate — it was noticing that
**the signal already existed and was set for exactly one slot**.

| stone | what it added | delta |
|---|---|---|
| 255.1 | identity is the pair; `wat.type` gets members | 104 → 97 |
| 255.2 | the arrow door — **and three measured dead ends** | 97 → 97 |
| 255.3 | the registry decides the join — *needed 255.1's members* | 97 → 80 |
| 255.4 | one member join; retirement teaches | 80 → 77 |
| **255.5** | **wire the existing position flag to the slots that need it** | **77 → 64** |

⛔ **255.2 looks like the stone that did nothing and is the one that made 255.5 possible.** Its
*negative* result — "a predicate over leaves cannot do this" — is what stopped a fourth predicate
being tried and pointed at the layer where the answer was.

## Residue (64) — what 8d still needs

| cause | n | note |
|---|---|---|
| `ReteCheckErrors` | **21** | ⚠ **now the largest class.** Never classified — *"classify before assuming"* still stands |
| `unresolved reference` | 21 | declaration names (`probe-homog`), functions (`mem-store::start`), `rete::core::defn`, and ⚠ `not-a-special-form` ×3 which is a **deliberate negative-test name and should stay** |
| `defsurface` | 8 | the `:messages` completeness slot, not the arrow |
| `ProgramBodyEvalFailed` · `UnknownNamedType` | 4 | gap-2 tail |

**VERDICT: ACCEPTED.**
