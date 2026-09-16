# BRIEF — make every walking gate say how it knows it reached something

`tests/lint/` is where this arc's guarantees live, and a gate that walks an empty set asserts
nothing and reports PASS. Drive each walking gate to see what it actually visits, then add the lint
that keeps the property. Read `DESIGN.md` first — it explains why **no count appears in it** (F0),
and its ⚠ shows two legitimate guard shapes, so a syntactic grep is not the cure.

## Read in order

1. `tests/lint/no_ceiling_raise_in_rete.rs:92` — the reason written verbatim in the tree already.
2. `tests/lint/no_new_broken_doc_link.rs:236-250` — the **second** guard shape: the population is a
   diagnostic stream, "found ≥ N" is unwritable, and it proves its extractor still matches rustdoc's
   format instead. Your lint must accept this without a rune, or accept it *with* one — decide and
   say which.
3. `tests/lint/no_unknown_sequi_rune.rs` — the model for a closed-vocabulary rune check, named in
   the work list as the shape to copy.
4. `src/rete/purity.rs`, `KNOWN_UNREVIEWED` — the ratchet doc, if you need a seeded ledger. Its own
   account of why a cardinality check failed applies here too.
5. Any three of the flagged gates — read what they walk before deciding what "non-empty" means for
   them; the file set is not the same population in each.

## The order of work — the drive comes first

1. **Drive every walking gate** and record what it visits. A gate reaching **zero today** is a live
   defect and outranks everything else in this brief — report it before writing the lint.
2. Add the gate that requires a declared guard.
3. For each file it flags: add the real guard where the gate can have one, or the rune where it
   genuinely cannot — with a reason that names a mechanism.

## The rune

`rune:lint(vacuity-guard) — <what this gate does instead, and what would red first>`

A **declaration, not a suppression**. "N/A" or "not applicable" is a refused reason; the shape at
`no_new_broken_doc_link.rs:236` is the standard to meet.

## Blast radius

`tests/lint/` only. No `src/` change. Report the number of files you touch — do not take mine, and
note that **DESIGN deliberately contains no count** for you to anchor on.

## Traps named in advance — each with its step

1. **★ A syntactic grep for `assert!(n > 0)` flags correct gates.** DESIGN's ⚠ has the
   counter-example in the tree. **Step:** decide what *declares* a guard and state the rule in the
   lint's own header, the way `no_unknown_sequi_rune` states its vocabulary.
2. **"Walks a file set" is itself a judgement.** `read_dir`, a glob, a spawned process's output,
   `include_str!` over a directory — these are different populations. **Step:** define the
   population the lint recognises, in its header, and let anything outside it be out of scope rather
   than silently unchecked.
3. **The lint must not be vacuous ITSELF.** **Step:** it needs its own guard — assert it examined at
   least the gates you know exist — and you must drive that: mutate its walk to find nothing and
   confirm it reds.
4. **A rune with a hollow reason is worse than no rune.** **Step:** if you cannot state the
   mechanism for a file, add the real guard instead. Report any file where you could do neither.
5. **New test code trips `wat::lint`.** **Step:** run `binary_id(wat::lint)` before reporting — it
   has caught a red for six consecutive riders.
6. **Do not "fix" a gate's subject.** If one is vacuous, make it say so and report it; what it
   *should* assert is its own strike. **Step:** STOP-2.

## STOP triggers

- **STOP-1** — if a gate is reaching zero today, STOP and report it before writing any lint. That is
  a live defect and I want it in my hands first.
- **STOP-2** — if fixing a gate's vacuity would require changing what it asserts, STOP and report.
- **STOP-3** — if the flagged set is large enough that guarding them all exceeds this strike, STOP
  and propose the split. A seeded ledger is acceptable **only** if you first drove that none of the
  seeded gates is actually vacuous.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-doc-attribution/` — last strike, same directory, and its
gate is one of the files you will be assessing.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Twenty-one riders before you each returned a prescription of
mine that did not survive contact. The last found that my sketch reopened, one level down, the exact
failure my own stone had just warned against. If a step here is wrong, unnecessary, or impossible,
say it plainly.
