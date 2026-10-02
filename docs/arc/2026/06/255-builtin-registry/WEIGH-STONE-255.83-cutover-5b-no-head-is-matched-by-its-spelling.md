# WEIGH — STONE 255.83: cutover 5b — no call head is decided by its keyword spelling — ACCEPTED

**Executor: grok via pulsare.** Commits `04cb3317f` (STOP-1), `e20099ad5` (amend 1), `e72efcba4`, `fbd326faa`,
`af6577c3e` (amend 2). Weighed by the orchestrator on 2026-10-02.

## Re-run by the orchestrator

- **At `e20099ad5`:** `.floor/2026-10-02T21-27-53Z` **red**, 6387/6388: `macros::tests::whole_body_templates_in_the_corpus_are_pure`
  found the two new negative fixtures. Grok's green (`.floor/2026-10-02T21-14-00Z`) had run before they were committed,
  and the scan enumerates tracked files: a different tree. Not re-run; cured in amend 2.
- **At `af6577c3e`** (its last commit is the SCORE only; grok's floor started after the last code commit):
  `.floor/2026-10-02T22-04-08Z` **6389 passed / 24 skipped**, exit 0. Clippy exit 0. Census no STOP-8 (grok).

## What landed

- **The keyword-heresy ledger's shapes A and B are 0** (146 → 64; the 64 left are shape E, the type-path class): every
  live Rust site that decided on a head by keyword text decides on its identity, each with a keyword/symbol pair as a
  driven test (rete `lower_call`, the legacy walkers, the verify/digest loaders, `eval-step!`, the purity classifier,
  `source_has_config_setter`, the quasiquote escape detector, the `where` head, the template purity walk).
- **The whole-body template hole is closed:** `is_quasiquote_form` reads the identity, and the template route checks its
  own unquote escapes, so a whole-body quasiquote that unquotes an impure call is refused in both spellings (the keyword
  spelling used to run it at expand time past the F5 default-deny gate). No corpus macro relied on it.
- **Wat has one door verb, `:wat::core::canonical-identity`,** and the wat-side identity decisions route through it
  (`Record.wat` unquote-splicing, `core.wat` agg-positional, `stratify.wat` exists, `lint.wat` `=`, and `fix.wat`'s
  `if`/`first`/`drop`/`Pure`/`Impure`, plus the five probed predicates). Markers that are not identities (`->`, `:-`, `&`,
  `:from`, `:locus`) stay text.

## On the record

- STOP-1's reading (that the spellings should keep the router's difference) was overruled: the keyword side was the
  defect.
- **A floor proves only the tree it ran on**, tracked set included (memory entry). Briefs now require a clean
  `git status` before the floor.
- **An instrument blind spot:** the ledger's literal walker does not see `== Some(":wat::…")` comparisons; new code was
  written in a shape it sees, and the gap is in the SCORE for the ledger's owner.

## Verdict

Accepted and pushed. 5c (the conversion) may proceed.
