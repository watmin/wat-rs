# EXPECTATIONS — a declared record mints its accessors

> ⚠ **This strike makes 66 accessors WRITABLE, not written.** A report claiming the rete data model
> is now used from `wat-scripts/` is wrong.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,403 plus every arm you drive.**

## The scorecard — every pre-value driven at HEAD `b5c068ebd`

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ a live accessor resolves | ⛔ **RED**: `zz-c15-probe.wat:4 :wat::rete::DerivationNode/via` | the same probe **PASSES** |
| 2 | ★ a typo'd accessor still fails | — | mutation 2: `DerivationNode/vai` **REDs**. Without this the cure is a hole shaped like a slash |
| 3 | ★ the new source cannot silently empty | existing universe has a floor (`rows >= 70 && attested >= 250`) | a floor on the record source too; mutation 3 REDs |
| 4 | the population is known | **19 `defrecord`s, 66 accessors** (balanced parse) | rider's own count, **with its anchor stated** |
| 5 | no rune is minted for a live name | — | **zero** `rete-name-unminted` runes added. That spelling about a minted accessor is a lie |
| 6 | no allowlist | — | the DECLARATION is the authority, parsed — not a list |
| 7 | corpus unchanged otherwise | — | no name already in `wat-scripts/` newly fails; any that newly resolves is named |
| 8 | blast radius | — | `tests/lint/rete_names_in_wat_scripts_resolve.rs` only; **zero `src/` and `wat/` diff** |
| 9 | floor / lints / clippy | **`5403 tests run: 5403 passed, 21 skipped`** (440.9 s, 0 FAIL rows), lints **254**, clippy rc=0 | ≥ 5403 + arms, 0 FAIL, lints ≥ 254, rc=0 |
| 10 | the probe does not survive | — | **no `zz-c15-probe.wat` in the tree at hand-back** |

## Runtime prediction

**50–75 minutes.** The parse is the work; the field check and the floor are what make it a gate.

## Trap doors named in advance

- **⛔ ANCHOR THE COUNT.** The orchestrator's first parse said 46 because it stopped at the first
  `])`, dropping every last-field with a nested type — including `DerivationNode`'s `via`, the very
  accessor this strike is about. Check `DerivationNode` = 3 and `DerivationStep` = 4 by eye first.
- **A probe left in `wat-scripts/` REDs the floor.** The orchestrator has pushed a red floor exactly
  this way before. Row 10 exists because of it.
- **Accepting `<Type>/<anything>` is not a cure.** It converts a false RED into a permanent blind
  spot, which is worse: nobody notices a gate that never fires.
- **`git checkout <sha> -- <path>` STAGES.** Verify restores by hash.

## What would make this strike a failure even if every test passes

**A resolver that accepts any slashed token.** The gate's whole value is refusing a name that does
not exist; trading a false RED for a false GREEN across 66 names is a worse trade than leaving C15
open. Row 2 is the strike.

**And a fourth source with no floor.** If the record parse silently yields nothing, every accessor is
unresolvable again — but the *gate* stays green, because nothing in `wat-scripts/` uses one today.
The blockage would return invisibly, and the next hand would rediscover C15 from scratch.
