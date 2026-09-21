# SCORE-AMEND — STONE 251.8d-i-b (3): the detector's own input

Folded into the stone landing (was `e57e04f52`). **Not pushed.**
Parent: `AMEND-251.8d-i-b-3-the-detectors-own-input.md`.
Floor / workspace clippy / census: orchestrator. Crate clippy + the named lints run here.

## The 2 reds

They were the generator gate's **self-test fixtures**, not leftover binds.

| site | wall | disposition |
|---|---|---|
| `binding_name_before` `strip_suffix("(:wat::core::")` | `no_inlined_edn` | **runed** — parser prefix over a wat source fragment; not a golden; a `.edn` file would have to parse and this must not |
| `qvars.contains("fact-sym")` (two self-tests) | `no_loose_string_assert` | **tightened** — `assert_eq!` on the whole `HashSet`, matching the adjacent `assert_eq!(uses, vec![(2, "fact-sym")])` |

No file-wide suppression. The one rune names why that string is detector input, not inlined data.

## Gate still non-vacuous

- `rete_bind_generators` — **4 passed** (walk + 3 detector tests)
- `every_walking_gate_declares_non_vacuity` — **15 passed**
- `tests_carry_no_inlined_edn` — **ok**
- `tests_carry_no_loose_string_assert` — **ok**
- crate clippy `-p wat --all-targets -D warnings` — **0**

`cargo nextest list --release -p wat`: **5324** (unchanged).

Floor + workspace clippy + census: orchestrator. Do not push. Do not start 8d-ii.
