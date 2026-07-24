# NOTE — full enum matching is MANDATORY: a `_` wildcard ARM never satisfies exhaustiveness

> **Deferred design decision (builder, 2026-07-24, arc 278).** Surfaced during the `run-hermetic`/
> `run-thread` IPC de-prime (wave 2b): a reduce-fixup rider, unsure which `:wat::kernel::LociDiedError`
> variant a process capacity-panic produced, collapsed the death match to
> `((… ::Panic message _failure) message) (_ "LOST-NON-PANIC")` — a bare `_` arm swallowing the other
> seven death variants. Builder: *"this is an illegal syntax, right? we impose mandatory full matching?
> … full enum matching is always mandatory — the verbosity is our shield."* Recorded per the arc-109
> `NOTE-*.md` convention; kin to `NOTE-io-boundary-outcome-enum.md` and `NOTE-underscore-hygiene-leak-cleanup.md`.

## The rule
A `:wat::core::match` on an **enum** scrutinee must **name every variant**. A bare `_` **arm** —
the *coverage wildcard* that collapses all remaining un-named variants into one handler — does **NOT**
satisfy exhaustiveness. Exhaustiveness is met *only* by full enumeration (every variant has its own arm).

This is **always** — not only for the failure-report enums (`LociDiedError`, the `*Outcome` family)
where it bites hardest, but for every enum. The verbosity is the shield.

## The distinction that keeps it precise (two different uses of `_`)
1. **`_` / `_name` as a FIELD-BINDING placeholder INSIDE a named-variant arm** — e.g.
   `(:wat::kernel::LociDiedError::Panic message _failure)`. The variant is explicitly named; `_failure`
   merely declines to bind a field. **This stays LEGAL** — nothing is hidden; you handled `Panic`
   explicitly and skipped a field you don't use.
2. **`_` as a whole ARM** — e.g. `(_ "LOST-NON-PANIC")`. A coverage wildcard collapsing every remaining
   un-named variant into one arm nobody wrote a name for. **This becomes ILLEGAL on an enum scrutinee.**

The two are the wart and its exception; the rule targets #2 only.

## Why — the no-hidden-failures LAW (R52 `QVOD LEX ACCENDIT`)
A `_` arm lumps un-named variants into one hidden handler: a peer that writes `_` is no longer
explicitly handling every way a loci can die (`LociDiedError`'s whole reason to exist —
*"we never know what locus a service/bracket-worker is on; we measure that every loci is handled"*).
The deeper property is the **ablaze**: when a new variant is added to an enum, every match that used a
`_` arm keeps compiling **silently** — swallowing the new case — whereas every *fully-enumerated* match
lights up RED and forces the author to handle it. Mandatory full matching turns "add a variant" into a
checker-driven worklist instead of a silent gap. That is the shield.

## The fix (deferred) — a surgical checker addition
The checker already carries the two signals SEPARATELY, so the rule is small and precise
(`src/check.rs`, the `:wat::core::match` coverage machinery, ~6820–6970):
- `covered_enum_variants` (~6829) — the set of explicitly-named variant arms.
- `wildcard_seen` / `Coverage::Wildcard` (~6821, ~6927) — set by a bare `_` **arm** (and the
  hash-destructure arm), NOT by a field-`_` inside a named-variant arm.

**The change:** for an enum scrutinee, require `covered_enum_variants == all variants`; a `wildcard_seen`
arm must NOT count toward exhaustiveness — instead it is a located error ("name every variant of
`<enum>`; a `_` catch-all is not permitted — verbosity is the shield"). Field-`_` bindings are untouched
(they never set `wildcard_seen`). Decide at draw whether the rule extends to `Option`/`Result` (also
enums) or is scoped to user/kernel `defenum`s — the builder's ruling was "always"; confirm the
`Option`/`Result` edge against the corpus.

## Corpus blast radius (the R52 ablaze this rule lights)
~50 `.wat` files currently use a bare-`_` **arm** (`grep -rlE '^\s*\(_ ' --include=*.wat`). Making the
rule real is a corpus migration — a wat-fix codemod expanding each `_`-arm on an enum into the named
variants it was standing in for (the checker names each violator once the rule lands). Not all 50 are
enum matches (some are `Option`/`Result`/other); the checker rule scopes the true set.

## Status
**DEFERRED.** A no-hidden-failures checker rule + its corpus migration, ruled by the builder this arc.
Grounded: `src/check.rs` ~6821 (`wildcard_seen`), ~6829 (`covered_enum_variants`), ~6927
(`Coverage::Wildcard`), ~6969 (`exhaustive`). Kin: `NOTE-io-boundary-outcome-enum.md`,
`NOTE-underscore-hygiene-leak-cleanup.md`; doctrine: 278 R52 `QVOD LEX ACCENDIT`, the LociDiedError
stone (`DESIGN-loci-died-error.md`), *"all exception paths explicitly managed — the verbosity is our shield."*
