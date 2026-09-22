# BRIEF — STONE 255.13: the heresy ledger

**Drawn 2026-09-22 against `main` @ `58d7c4b22`.** Floor 5989/5989, clippy 0, census `no STOP-8`.
Delta **3**, RECOVERY **0** (`scripts/replay/delta.sh`).

## ⭐⭐ THE BUILDER'S RULING, 2026-09-22

> *"the end state is that all call heads are symbols, not keywords, anything doing exact keyword
> matches should be considered heresy at this point… we need a dual support for the migration, but it
> feels like that dual support is nearing its terminal state"*

**Dual acceptance is a MIGRATION SCAFFOLD with an expiry.** This stone builds the instrument that
tells us when the expiry has arrived.

## ⛔ WHY NOT "MAKE KEYWORD CALL HEADS ILLEGAL" FIRST

Measured: **103 213** keyword heads in 2 208 tracked `.wat`, plus **8 447** wat forms embedded in
**363 `.rs` files (7 099 in `src/`)** that no `.wat` codemod can reach. The binary loads its own
stdlib, so the corpus must convert first.

⛔⛔ **And it would not make heretics identify themselves — it would break EVERYTHING AT ONCE.**
Two floors today read **3 747** and **416** failures; in that noise a heretic is indistinguishable
from ordinary breakage. ⭐ **Illegal-in-wat is the END of the migration. Illegal-in-RUST is a tool
FOR it.**

## What this stone builds

**A lint in the shape this repo already uses** (`tests/lint/no_bare_is_err.rs`,
`ignore_reason_justified.rs`, `every_ungated_wat_checks.rs`): a **banned shape** plus a **frozen
allowlist that can only shrink**.

⭐ **The ledger's number becomes the countdown to the terminal cut:** keyword call heads become
illegal when the ledger reads **0** *and* the corpus is converted. **Not before, and the stone that
does it cites this number.**

## ⛔⛔ THE DISCRIMINATOR IS THE WHOLE STONE — AND MY NUMBERS ARE A BAD LEAD

**Heresy is NOT "a keyword string literal".** Orchestrator-measured:

| | |
|---|---|
| `":wat::…"` literals in `src/` | **7 233** — ⛔ **mostly MESSAGE TEXT, docs and embedded fixtures** |
| of those, comparison/dispatch shapes | **~872** (`==` 302 · match arm 490 · `starts_with` 51 · `contains` 23 · `matches!` 6) |
| of a 354-line subset, **no normalizer within ±3 lines** | **340** |

⛔⛔ **THAT 340 IS FROM A ±3-LINE WINDOW — THE EXACT INSTRUMENT THAT GAVE 3 FALSE POSITIVES AND MISSED
THE REAL DEFECT IN 255.10, AND THAT MISSED `walk_for_restricted_call` IN 255.11.** **Do not inherit
it. Build a real discriminator and correct me.**

⭐ **The actual rule:** comparing an **already-canonicalized** name to a keyword constant is
**CORRECT** — the internal identity *is* keyword-spelled (`canonical_identity` → `:wat::core::X`).
The heresy is:

> **a decision made by comparing a NAME THAT CAN ARRIVE IN EITHER SPELLING against a keyword-spelled
> literal, WITHOUT passing it through the identity door first.**

## ⭐⭐ THE GATE WITH TEETH — THE LEDGER MUST CONTAIN THE KNOWN HERETICS

⛔ **A ledger that misses a heretic we already know about is not a ledger.** It MUST list, or its
allowlist must name, every site this arc has already convicted:

| site | convicted by |
|---|---|
| `src/collection/transform.rs:304-340` — comparator purity table, **468 log occurrences, ONE callee (`wat.core/<`)** | ⭐ **8d-ii 4th draw — STILL OPEN, and the 5th draw's target** |
| `edn/render.rs::edn_to_typed_value_inner` (hardcoded type table) | 255.12 (cured) |
| `function/subsume.rs::value_matches_type_by_name` | 255.12 (cured) |
| `types.rs::register_validated` raw `==` | 255.12 (cured) |
| `check.rs::walk_for_restricted_call` | 255.11 (cured) — ⛔ **a name grep MISSED this one** |
| `load/loader.rs::match_load_form` / `scan_for_setter` | 255.9 (cured) |
| `macros/eval.rs::validate_pure_total` | 255.10 (cured) |
| ⛔ `macros/expand.rs::is_quasiquote_form` | **STILL KEYWORD-ONLY, twice reported, builder's call** |

⭐ **Cured sites are the calibration set: the lint must NOT flag them** (they now normalize) —
⛔ **and it MUST flag `transform.rs:304`, which is still open.** **If your discriminator cannot tell
those two groups apart, it is not ready and you should say so rather than ship a number.**

## The work

1. **Build the discriminator.** Report what it can and cannot see.
2. **Run it. Report the number.** That number is the ledger.
3. **Freeze the allowlist**, one entry per site, ⭐ **each with a one-line reason** — not a bare path.
4. ⭐ **PROVE THE LINT FAILS:** add a new un-normalized keyword comparison, watch it go red, remove it.
   ⛔ **A gate that has never failed is not a gate.**

## The gate

- ⭐ The calibration table above: **cured sites not flagged, `transform.rs:304` flagged.**
- ⭐ **The lint has FAILED once, by construction, and you show it.**
- The allowlist is frozen with reasons; the number is recorded in the SCORE **and** in the lint.
- `scripts/floor.sh` green; clippy 0; census `no STOP-8`; delta **3**, ⛔ **RECOVERY non-zero is a STOP**.
- ⛔ **NOT ONE `.wat` CONVERTED.** This is a `src/` + `tests/` stone.
- ⛔ **CURE NOTHING.** ⭐ **This stone COUNTS. It does not fix.** Curing changes the number under
  your own instrument and you will not know which moved it.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** ⚠ Except in bookkeeping:
  `harvest_wrap_split` is diagnosed-unsound (`278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md`).
  **If it fires, CITE and REPORT — do not treat as a pass, do not re-run to clear.**
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Fifteen stones running.
- ⛔ **If this brief contradicts the code, THE CODE WINS.** ⭐ **Twenty-one corrections across
  nineteen stones.** The last stone refuted this orchestrator's central mechanism **and** caught it
  reporting a numerator larger than its denominator. ⛔ **The 340 above is the most likely thing here
  to be wrong. Assume a twenty-second.**
- ⚠ **Never `git add -A`, and never invoke cargo, while a conversion is writing `wat/`** — both cost
  this arc time TODAY.
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

8d-ii's 5th draw (the comparator purity cure — ⭐ **this stone FINDS it, the next one FIXES it**) ·
8d-iii · the terminal "keyword heads illegal" cut (**it becomes schedulable when this ledger reads
0**) · variant tags as bare symbols in declarations (**ruled, unbuilt**) · `is_quasiquote_form` ·
`Ngram`'s mixed-dialect collision.
