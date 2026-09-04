# BRIEF — teach the resolver that a declared record mints its accessors

`:wat::rete::DerivationNode/via` is live — the record is declared, the field is there, a program
using it runs. The gate rejects it, because accessors are synthesized at freeze and never appear
textually where `attested()` looks. 66 accessors are unwritable in `wat-scripts/` for that reason.

## Read in order

1. `probe-c15.wat.txt` beside this brief — four lines that turn the gate RED at HEAD. **Run it
   first, then DELETE it.** Leaving it in the tree reds the floor.
2. `tests/lint/rete_names_in_wat_scripts_resolve.rs:383` — `attested()`, the textual scan of `src/`
   and `wat/`. This is why a synthesized name cannot resolve: it is not text anywhere.
3. `tests/lint/rete_names_in_wat_scripts_resolve.rs:411-430` — the test body and the universe it
   builds (`rows ∪ attested`), plus its own **non-vacuity floor** (`rows >= 70 && attested >= 250`).
   Your fourth source needs the same treatment: a floor, or it can silently become empty.
4. `tests/lint/rete_names_in_wat_scripts_resolve.rs:265` — `registry_rows`, the existing example of
   "a source of truth parsed out of a file." Copy its shape; do not invent a second idiom.
5. `wat/rete.wat:374-393` — the two records the probe uses, and the nested-type field
   (`via <- (:wat::core::PersistentVector :- [...])`) that broke the orchestrator's first count.
6. `tests/lint/every_walking_gate_declares_non_vacuity.rs` — house style for a gate that must
   declare how it knows it reached something.

## Driven by the orchestrator at HEAD `b5c068ebd`

- The probe type-checks, runs, and REDs the gate at `:4`.
- **19 `defrecord`s in `wat/rete.wat`, 66 synthesized accessors**, counted by balanced-bracket parse.
- ⚠ **A first count said 46.** The regex stopped at the first `])` and dropped every last-field with
  a nested type — including `DerivationNode`'s `via`, the very accessor the probe drives. **Anchor
  your parse on `DerivationNode` = 3 (`fact rule via`) and `DerivationStep` = 4
  (`supporting pattern bindings constraints`) before trusting any total.**

## The change

Add a fourth resolution source: `:wat::rete::<Type>/<field>` resolves when `<Type>` is a
`defrecord` declared in `wat/rete.wat` **and** `<field>` is one of its declared fields.

**The field half is not optional.** A rule that accepts any `<Type>/<anything>` cannot refuse a typo,
and a resolver that cannot refuse a typo has stopped resolving. Mutation 2 is exactly this.

Give the new source a **non-vacuity floor** the way the existing universe has one — a parse that
silently returns an empty set would make every accessor unresolvable again, and the gate would look
green while the corpus was blocked.

## Blast radius

`tests/lint/rete_names_in_wat_scripts_resolve.rs` only. **No `src/` change, no `wat/` change** — the
accessors are generated correctly; the resolver's model of what exists is what is short.

## STOP triggers

1. **If a `defrecord` in `wat/rete.wat` uses a form your parser cannot read**, stop and report which
   — a parser that silently skips a record under-populates the universe and re-blocks its fields.
2. **If the field check would reject a name currently in the corpus**, stop and report it: that is
   either a real typo already in the tree (a finding) or a shape your parser mis-reads.
3. **If this requires touching `src/` or `wat/`**, stop and report.

## Mutation proofs — run all three, report all three

1. **The probe file present, cure in place** → gate PASSES. Remove the cure → RED at `:4`, naming
   `:wat::rete::DerivationNode/via`.
2. ★ **A typo'd accessor** (`:wat::rete::DerivationNode/vai`) → gate **REDs**. This is the row that
   proves the fix still refuses; without it the cure is a hole shaped like a slash.
3. **Empty the new source** (make the record parse return nothing) → the non-vacuity floor REDs.
   A source that silently empties must not read as "everything resolved".

Delete the probe file after each. Verify restores by **hash** — `git checkout <sha> -- <path>` STAGES.

## What to report

- The gate's result with the probe present, before and after.
- All three mutation results.
- Your accessor count, and the anchor you used to trust it.
- Whether any name already in `wat-scripts/` newly resolves or newly fails.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Four consecutive strikes here had their ★ be
  a false claim in a file the brief said to trust — twice it was the orchestrator's own artifact.
  Assume there is a fifth.

Do not commit.
