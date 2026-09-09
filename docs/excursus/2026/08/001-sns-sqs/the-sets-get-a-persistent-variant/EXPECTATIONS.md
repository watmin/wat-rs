# EXPECTATIONS — the sets get a persistent variant

Written **before** the strike. Substrate stone; `src/` plus a corpus probe.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **insert shares structure** | read the diff | `conj` builds from `rpds::HashTrieSetSync`; **no `(**s).clone()`** anywhere on the `:wat::set::` path |
| 2 | ★ **the five verbs exist and type-check** | a new `wat-scripts/scratch-pad/` probe | `conj` `disj` `contains?` `empty?` `length`, each driven and asserted |
| 3 | ★ **it round-trips through EDN** | the same probe | a `:wat::set::` value renders and re-reads; the `Value` variant is not write-only |
| 4 | **`:wat::hashset::` is untouched** | `git diff` | its four verbs, its cloning `conj`, and its marked name all unchanged |
| 5 | **the unmarked name is the persistent one** | read the diff | `:wat::set::` is `HashTrieSetSync`, matching `:wat::map::`'s documented rationale |
| 6 | **no ordered-set surface leaked in** | `git diff` | `RedBlackTreeSetSync` absent |
| 7 | **the new variant is handled everywhere it must be** | `cargo build` + the probe | it compiles with no `_ =>` arm added to a previously-exhaustive `Value` match to silence it |
| 8 | **the corpus still loads** | `every_wat_scripts_file_loads` | PASS — the new probe parses and type-checks on the current runtime |
| 9 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5235 + the tests you add (state the number), 22 skipped, **0 FAIL, 0 TIMEOUT** |

## The rows that carry it

★ **Row 1 is the stone.** A persistent set that clones on insert is the current set with a new name.
The property is *no whole-collection clone on the write path* — grep the diff for it rather than trusting
that the right type was reached for.

★★ **Row 7 is the trap a new `Value` variant sets.** Adding a variant makes every exhaustive `match`
fail to compile, and the cheap way out is a catch-all arm. **A `_ =>` added to silence the compiler
hides the sites that genuinely need the new case** — rendering, freezing, observation. If a site truly
does not care, it should say so at that site, not be swept up by a wildcard.

★ **Row 3 exists because a collection that cannot round-trip is not a value in this language.** EDN
expressibility is what makes it storable in a `defrecord` and shippable across a boundary.

⚠ **Row 4 keeps the migration honest.** `:wat::hashset::` is not being retired here; its callers move
in a later mechanical pass, through `wat/fix.wat`, not by hand.

## Runtime prediction

**90–150 minutes.** A new intrinsic home, five verbs, a `Value` variant with a compiler-driven ripple,
TypeSchemes, EDN rendering, and a corpus probe. The variant ripple is the unknown — it is why row 7
exists.

## Trap-doors

- **`HashTrieSetSync`, not `HashTrieSet`** — the `Sync` alias is what the other two use, and the value
  must cross threads.
- **`Value` must stay `Hash + Eq`** for set membership; the existing `HashSet` path already relies on
  that and guards opaque handles at insert. Mirror that guard.
- **`disj` on an absent element returns the set unchanged**, not an error.
- **Do not add `get`/`keys`/`values`.** Five verbs; a set has no map shape.
- **The probe belongs in `wat-scripts/scratch-pad/`**, where the corpus gate type-checks it — not in the
  session scratchpad.

## What this stone does NOT claim

⚠ It does **not** migrate any existing `:wat::hashset::` caller.
⚠ It does **not** fix `seen-ids` — the right fix there is to count the visibility transition, not to
swap the set.
⚠ It does **not** retire `:wat::hashset::` or add an ordered set.
