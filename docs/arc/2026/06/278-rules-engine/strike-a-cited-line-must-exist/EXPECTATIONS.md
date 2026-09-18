# EXPECTATIONS — a cited line must exist

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,411 plus every arm you drive.**

## The scorecard — pre-values driven at HEAD `c22cfe6e3`

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ a cited line is checked | ⛔ **never** — `no_stale_path_in_doc` checks path existence only | out-of-range REDs, naming cited line **and** real length |
| 2 | ★ `wat/` and `wat-tests/` are scanned | ⛔ roots are `src/rete` alone (`:88`) | both added |
| 3 | ★ the six citations cured | `rete.wat:1508` ×5 across 4 files; file is **533** lines | live symbol, **no line number** |
| 4 | the boundary is exact | — | mutation 2 (len+1) REDs, mutation 3 (len) passes |
| 5 | not vacuous | — | mutation 4: empty scan FAILS |
| 6 | retired names in prose stay legal | `rete_names_in_wat_scripts_resolve` rules prose may name a retired form | **unchanged** — this gate checks locations, not names |
| 7 | the codemod is untouched | `rete-oracle-sigil.wat` correctly records the retirement | **zero diff** |
| 8 | floor / lints / clippy | **`5411 tests run: 5411 passed, 21 skipped`** (425.9 s, 0 FAIL), lints **258**, clippy rc=0 | ≥ 5411 + arms, 0 FAIL, lints ≥ 258, rc=0 |

## Runtime prediction

**35–60 minutes.** The depth check is a length lookup; the widened scope's fallout is the unknown.

## Trap doors named in advance

- **⛔ WIDENING SCOPE MAY SURFACE A PILE.** STOP-1 exists so a corpus-wide count reaches the
  orchestrator as a *finding* rather than becoming an unbounded fixing session inside this strike.
- **A gate that rejects valid citations gets disabled.** Mutation 3 (cite the last line exactly →
  passes) is as load-bearing as mutation 2.
- **Do not gate retired names.** Row 6. That ruling is deliberate and this strike is orthogonal to it.
- **`git checkout <sha> -- <path>` STAGES.** Restore by hash.

## What would make this strike a failure even if every test passes

**Fixing six citations without the gate.** They were true when written; the next six will rot the same
way. F0's whole thesis is that a corrected number buys weeks and a check buys the class.

**And a gate that only checks existence.** That is what exists today and it is what let a
1508-in-533 citation survive five times across four files. **Row 1 is the strike.**
