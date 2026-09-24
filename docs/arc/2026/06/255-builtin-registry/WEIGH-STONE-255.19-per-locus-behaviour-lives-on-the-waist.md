# WEIGH — STONE 255.19: per-locus behaviour lives on the waist — ACCEPTED

**Executor commit `59fb869f3`.** Weighed by the orchestrator against disk on 2026-09-24.

## Re-measured

| row | measured | result |
|---|---|---|
| tree · floor | `git status`; `.floor/2026-09-24T05-24-46Z/clean.log` | clean; `6037 tests run: 6037 passed (9 slow), 22 skipped` |
| defclauses retired | `git grep 'defclause :wat::spawn::(with-label\|runner-count)'` | 0 |
| a process locus passed through `with-label` and claimed `Shared` | `…_with_label_process_claimed_shared.wat.bad`: pre-stone binary (`7eb624a0a`) vs new | **old 0 → new 1** |
| its twin, claimed `Wire` | `…_with_label_process_claimed_wire.wat` | 0 / 0 |
| a process locus started via the abstract start and claimed `Shared` | `…_start_process_claimed_shared.wat.bad` | **old 0 → new 1**, a second real hole |
| its twin | `…_start_process_claimed_wire.wat` | 0 / 0 |

Taken from the report without re-running: clippy 0, census `no STOP-8` (213 → 213), delta NEW 3 /
RECOVERY 0, ledger 220. The codemod `locus-methods-on-the-waist.wat` was dry-run and diffed, its apply
is byte-identical, it is idempotent, and its replay gate was shown able to go red. No bare `Locus`
remains in a live type position; the survivors are replay fixtures, the declaration, comments, and the
C-b5 checker arm.

## Where the brief was wrong

1. **`runner-count` has no `extend-type` arm.** An arm collides (`DuplicateDefine`) with the record's
   own field accessor `<Type>/runner-count`, which already satisfies the method through the flat
   `<Type>/<method>` key.
2. **`with-label :- [R]` could not type-check.** A method-level `R` is rigid, so `r <- :wat::core::Record`.
   The `:R` existed only for defclause runtime dispatch.
3. **Call-site counts** again included comments: 10 + 8 head strings + 3 code sites, not 26 and 7.

## New findings, not fixed

- ⛔ **A call to an undeclared surface method type-checks and dies at run time.** The orchestrator
  reproduced it: `(:wat::spawn::Locus/bogus-xyz (:wat::spawn::process) 1 2 3)` gives `--check` **rc=0**,
  then `UnknownFunction ":wat::spawn::Locus/bogus-xyz"` at run time. The control, a non-surface head
  `:wat::spawn::NoSuchThing/bogus-xyz`, is refused at check (rc=1). This is the class 255.16 and D2
  belong to: the checker passes, the runtime fails. It is also why a pre-stone binary accepts any
  fixture naming a new surface method (so those rows could not tell the two binaries apart). It needs
  its own stone.
- The macros' head-name compare (`service.wat` ×4, `bracket.wat` ×4) recognises only the keyword
  spelling; a faithful-spelled head falls through to the abstract impl. That result is honest now
  (correctly typed), but it is the string-compare hazard CLAUDE.md names. It dies with step 3's generic
  start, which retires the per-locus routing.
