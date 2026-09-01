# DESIGN-STONE — a gate that reaches nothing passes

> **Origin (2026-09-01).** Class **F1**, row 5, found by `complectens`. Driven at HEAD `82029517a`.
> The first Class F strike, and it is drawn under **F0**: *a number in prose is replaced by the
> command that derives it.*

## Why

`tests/lint/` is where this arc's guarantees live. Most of those gates **walk a file set** and
assert something about what they find. A gate that walks a set which comes back **empty** asserts
nothing and reports PASS — and every verdict downstream of it inherits that silence.
`no_ceiling_raise_in_rete.rs:92` already writes the reason verbatim; `complectens` found the
property missing across much of the directory.

**This is the whole `tests/lint/` suite's credibility**, so it is the right first Class F strike:
every other row in this arc has been proven by a gate, and a vacuous gate proves nothing.

## ⛔ THE COUNT IS NOT IN THIS STONE, AND THAT IS THE POINT

The work-list row says *"10 of 15"*. My own audit grep said *"16 of 24"*. **Both are prose counts and
at least one is wrong** — mine demonstrably is: it reported `no_new_broken_doc_link.rs` as unguarded,
and that file's own `:236` says *"A vacuity guard of the usual shape … is unavailable here"* and then
implements a different one (an extractor self-check, driven last strike).

F0 is the rule and this is its first application: **do not correct the count — build the instrument
that derives it.** The lint IS the count. Whatever it reports is the number, and it stays true
without anyone re-deriving it.

## ⚠ THERE IS MORE THAN ONE LEGITIMATE GUARD SHAPE

A lint that greps for `assert!(n > 0)` would flag correct gates. At least two shapes are already in
the tree:

- **the usual one** — assert the walk visited at least N files;
- **an extractor self-check** — `no_new_broken_doc_link.rs`, where the population is a *diagnostic
  stream* rather than a file set, so "found ≥ N" cannot be written; it proves instead that its
  parser still matches the format it parses, and **that self-check reds first** if the format moves.

So the lint must accept a **declared** guard, not a syntactic one.

## ★ THE ONE CONTRACT DECISION

**A gate that walks a set states how it knows the set was not empty — in code, or in a rune that
names the shape it uses instead.** After this strike a new file-walking gate cannot land without
answering that question, and the answer is mechanical rather than prose.

Rune: `rune:lint(vacuity-guard) — <what this gate does instead, and what would red first>`. It is a
**declaration, not a suppression**: the reason must name a mechanism, the way
`no_new_broken_doc_link.rs:236` does.

## The live question this strike must answer FIRST

A missing guard is a risk; **a vacuous gate is a defect.** Before any lint lands, **drive each
walking gate and record what it actually visits.** If any reaches zero today, that gate has been
reporting PASS while proving nothing, and that is a finding that outranks the lint.

## Blast radius

`tests/lint/` — one new gate, plus a guard or a rune on each file it flags. No `src/` change.

## Out of scope — AFFIRMATIVELY CUT

- **F1's other four lints.** Each is its own strike with its own instances. This one is first
  because the other four would themselves be gates whose credibility depends on it.
- **F2's rotted claims.** They ship WITH F0's rule, not as corrections, and several need the
  `bare *.rs filename` lint (F1 row 2) that does not exist yet.
- **Rewriting any gate's subject.** If a gate is found vacuous, this strike makes it *say so*; what
  it should have been asserting is that gate's own strike.
