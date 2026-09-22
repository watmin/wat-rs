# BRIEF — STONE 255.12: one identity question, four `==`s

**Drawn 2026-09-22 against `main` @ `d59634ff4`.** Floor 5968/5968, clippy 0, census `no STOP-8`.
Delta **4**, RECOVERY **0**, on the committed list
`docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt`. ⛔ **USE THAT FILE.**

## What this stone closes

⭐ **The 8d-ii BLOCKER and the LAST 2 DELTA FILES are the same question asked with `==`.** One door
already exists — `edn::render::type_denotation`, wired into 8 places by 255.8 — and **four sites ask
identity without it.**

## The four sites, measured here

| # | site | today | evidence |
|---|---|---|---|
| 1 | `types.rs:995` `register_validated` — `Some(e) if e == &def` | ⛔ **raw `==`** while ⭐ **`types::type_exprs_same` (`types.rs:142`) ALREADY consults `type_denotation`** | `wat/source.wat` converted → `duplicate type declaration: :wat::source::File` |
| 2 | `macros/registry.rs::ast_same_identity` | ⛔ **a DIFFERENT comparator** — cross-spelling arm requires `id.is_reference()` | `wat/holon/Ngram.wat` converted → `duplicate macro registration` |
| 3 | `edn/render.rs:2508` `edn_to_typed_value_inner` | ⛔ **`match p.as_str()` against a hardcoded literal table** keyed on `":wat::core::i64"` | ⭐ **this is the 8d-ii STOP** |
| 4 | `function/subsume.rs::value_matches_type_by_name` | ⛔ raw Path-string compare | same class |

**Reproduced by the orchestrator, minimal:**

```wat
(wat.core/defrecord my/R [n :- wat.core/i64])
(wat.core/defrecord my/R [n :- wat.type/i64])   ;; SEMANTICALLY IDENTICAL
```
→ ⛔ `rc=1  duplicate type declaration: :my::R`. **Annotation position is already fine** (255.8):
`wat.type/i64` and `wat.core/i64` both `--check` clean. **Only the equality sites disagree.**

⚠ **SITE 2 IS NOT SITE 1 AND THE PRIOR SCORE CONFLATED THEM.** `Ngram` is the **macro** registry.
⭐ **Measured lead (orchestrator, confirm it):** the converted body differs **only** by
**3 `<-` + 2 `->` → 5 `:-`** — a **Symbol→Keyword** flip — and `ast_same_identity`'s cross arm
requires `id.is_reference()`, which `<-` and `->` are not. **Confirm or refute before curing.**

## ⛔⛔ THIS CURE RUNS IN THE DANGEROUS DIRECTION

Every stone since 255.9 cured in the **restrictive** direction. ⛔ **This one makes walls MORE
PERMISSIVE** — more re-declarations counted "equivalent", more values counted "matching".
**That is the red → false-green direction.** 255.11's own rule applies: *a wall that stops refusing
is indistinguishable from a wall that never fired.*

**Measured semantics you must preserve** (orchestrator):

| | |
|---|---|
| identical re-declaration | **rc=0** — a permitted no-op |
| **DIVERGENT** re-declaration (`i64` vs `String`) | ⭐ **rc=1 `DuplicateType`** |

⛔ **NON-VACUITY, MANDATORY, IN ONE TEST:**
1. cross-spelling **identical** → **accepted** (the cure).
2. ⛔ **DIVERGENT → STILL `DuplicateType`** — and ⛔⛔ **a divergence that `type_denotation` might
   COLLAPSE must still be refused.** Construct one deliberately. **This row is the stone.**
3. Same two rows for the **macro** registry (site 2).
4. Sites 3/4: a value of the right type still coerces; ⛔ **a value of the WRONG type is still
   REFUSED** — with the diagnostic, not merely non-zero.

## ⭐ THE TWO INSTRUMENT DEBTS — THREE STONES OLD. CLOSE THEM HERE.

**A. RECOVERY as a standing column.** Printed by hand for three consecutive stones. 255.11: *"one
`awk` clause, and it belongs in `scripts/`."* ⛔ **It is the ONLY gate that can see a green-forging
conversion** (255.9 caught one; the delta's NEW column was blind). **Make it a script that reports
`orig-clean / conv-clean / NEW / RECOVERY`, and say non-zero RECOVERY is a STOP.**

**B. A GATE ON PASS ORDERING.** ⛔⛔ **Recommended three times. Its absence is what let 255.9 AND
255.10 mark a LIVE capability wall "unreachable".** Both wrote *"post-step-7 ⇒ unreachable"* and
applied only its **CODE** half; a DATA position is never normalized, and the wall was open.
**Pin the order so a re-ordering goes RED instead of silently re-opening every "unreachable" row.**
⚠ **If you cannot build this small, say why and what it would take — do not pad it.**

## The work

1. Confirm/refute the site-2 arrow lead. **Say which.**
2. Cure all four through the **existing** doors (`type_denotation` / `type_exprs_same`). ⛔ **No new
   comparator.** Each cure names the probe that fails without it.
3. The non-vacuity rows above, in one test.
4. **A** and **B**.

## The gate

- ⭐ **Delta `< 4`** on the committed list — **the 2 remaining stdlib files should close.**
  ⛔ **REPORT RECOVERY; non-zero is a STOP until explained.**
- ⭐ **8d-ii's STOP probe:** site 3 is what killed the converted stdlib. **Show a `wat.type/`-spelled
  value decoding through the coerce path.** ⚠ **If you cannot reach it without converting the
  stdlib, SAY SO** — that is 8d-ii's job, not this stone's.
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` 0; census `no STOP-8`.
- ⛔ **NOT ONE `.wat` CONVERTED** in the live tree.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole, never re-run to green, name the arm.
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Thirteen stones running.
- ⛔ **If this brief contradicts the code, THE CODE WINS.** ⭐ **Nineteen corrections across seventeen
  stones.** 255.11 found a live capability escape **at the row the orchestrator marked
  `do not re-litigate`** — ⛔ **so treat every "already verified" line here as SCOPED TO ITS PROBE,
  not settled.** Assume a twentieth.
- ⚠ **Reformat in its own commit, or not at all.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

The 416-test rete remainder · the shape-B/D read-site sweep · `:wat::keyword::canonical-identity` ·
255.8's wrong-join acceptance · `is_quasiquote_form`'s residual asymmetry (**builder's call**) ·
the two mutation heads `set-redef!`/`set-eval-redef!` (**builder's call**) · 8d-ii · 8d-iii.
