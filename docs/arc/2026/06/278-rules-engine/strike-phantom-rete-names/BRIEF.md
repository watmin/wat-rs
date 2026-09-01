# BRIEF — delete a rename that cannot exist, and make `wat-scripts/` prove its names

The gate that justifies the whole scratch-pad convention proves a file *parses*, not that its names
*exist* — an invented head in a `def` body type-checks and runs. Two phantom rete names live in the
tree because of it, one pair of them inside the codemod `CLAUDE.md` mandates for every `.wat`
migration. Read `DESIGN.md` first — its ⛔ explains why those rows must be **deleted rather than
corrected**, and its ⚠ shows that most of what a naive scan flags is noise.

## Read in order

1. `wat-scripts/fixes/rete-where-per-type-spelling.wat`, the `Tuple` table — 41 pairs, two of which
   target names that do not exist. Note the file's own comment at `:83` recording `foldr`'s
   retirement: **that is prose and must survive.**
2. `src/rete/vocabulary.rs:965-975` — why rete took `mapv`/`filterv` and not `map`/`filter`, in the
   tree's own words.
3. `src/rete/expr_ir/eval.rs:586` and its `eager_items` — *"a `Stream` is deliberately absent."*
   Together with (2) this is why no rename is valid.
4. `wat-scripts/scratch-pad/probe-arc278-57-round1b-parametric-and-hof.wat:44-66` — the two dead
   `def`s, and the file's own header saying it *"drops from five HOF probes to four."*
5. `tests/lint/wat_scripts_fixes_load.rs` — the gate that reads every file here and cannot see this.
6. `tests/lint/every_walking_gate_declares_non_vacuity.rs` — landed last strike; **your new gate is
   in its population and must carry a declaration.**

## The work

1. **Delete the two rename pairs.** State at the table why no correct target exists, so nobody
   re-adds them by symmetry with the 39 that are pure spelling changes.
2. **Fix the scratch probe.** Its own header already says there is no rete-spelled right fold to
   probe; decide what the map/filter probes become and say why. **Not a rename to `mapv`/`filterv`
   unless you can show the probe still probes what it claims.**
3. **The gate**: every `:wat::rete::` name in **code** under `wat-scripts/` resolves to a `RETE_OPS`
   row or a known form.
4. **Narrow or vindicate `CLAUDE.md`'s claim.** The gate makes *"all wat stays correct"* true for
   rete names; the sentence should say what is actually proven. That file's own header rule applies:
   **state only what this repo can check.**

## Traps named in advance — each with its step

1. **★ Prose must survive.** `foldr` and `nth` appear only in comments, correctly recording
   absences. A gate that flags them demands the deletion of accurate history. **Step:** strip `;;`
   comments before scanning, and drive it — confirm those two files stay green.
2. **`defn` is a FORM, not a row.** 15 files use `:wat::rete::core::defn` legitimately. **Step:**
   the gate needs a known-forms set beside `RETE_OPS`; enumerate it from the tree, not from memory.
3. **Tokenizing is where a naive scan breaks.** `enum::=` and `f64::` fragments came out of my regex
   because it stopped before `=`. **Step:** include the operator characters rete names actually use,
   and report any token you cannot classify rather than dropping it.
4. **The codemod is idempotent and mandated.** **Step:** after deleting the rows, confirm no `.wat`
   in the corpus still needs them — `grep` for the source spellings inside a rete fence.
5. **Your new gate is itself a walking gate.** **Step:** it must carry a `NON-VACUITY` marker with a
   real floor, or the gate landed last strike reds. Run `binary_id(wat::lint)`.
6. **Do not fix the general `def`-is-never-resolved problem.** DESIGN cuts it. **Step:** if you find
   the textual approach cannot decide a case, report it rather than reaching for forcing.

## STOP triggers

- **STOP-1** — if any corpus `.wat` still requires the deleted rename pairs, STOP and report: that
  would mean a live program needs a semantic migration, which is a different strike.
- **STOP-2** — if any currently-green test goes red, STOP and report which.
- **STOP-3** — if the classifier cannot separate code from prose for some construct, STOP and name
  it. Guessing there deletes history.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-vacuity-guard/` — last strike, same directory, and its gate
will judge yours.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Twenty-two riders before you each returned a prescription of
mine that did not survive contact. The last found that the gate it was building parsed its own
documentation as an answer and was one run from vouching for itself. If a step here is wrong,
unnecessary, or impossible, say it plainly.
