## CONFERRE — Cast Report (TARGET 3 — the 43 Clara twins against their contract)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## SCOPE

**Read in full:** `CLARA-TRANSLATIONS.md` (422 lines — the contract) and `REMAINING-CLARA-MOUTHS.md` (91 lines, closure record). Directory listing confirmed: 54 `.wat` axes, **43 `.clj` twins** (= 54 minus the 11 with a `gen-*.sh` generator instead).

**Sampling rule:** complete `.wat`/`.clj` bodies for **8 pairs**, chosen to cover (a) every axis the contract documents that actually has a static `.clj` here, (b) the newest additions post-dating the contract's last edit (2026-09-03), and (c) the highest-risk shapes named in the cast: `parametric-erasure` (the contract's own "most important axis"), `where-accum-group`, `where-not-and-bound`, `accum-over-derived`, `userfn-head`, `retract-multiplicity`, `where-not-derived-in-query`, `where-exists`.

I then grep-verified the pattern found in that sample across the full 43-pair corpus rather than reading all 43 bodies. **Not read in full:** the remaining ~35 pairs — I sampled their headers by grep for the specific pattern, not their rule bodies.

## The contract's translation rules, as I understand them

`CLARA-TRANSLATIONS.md` documents 8 axes in depth (A2, A3, A5, A6, A7, A8, A9, A10) with per-axis "same final derived set" verdicts and caveats. ⛔ **Only A10 (`parametric-erasure`) of those 8 has a static `.clj` in this directory** — the other seven are `gen-*.sh` axes with no static twin to check the contract against. The remaining 35+ pairs are **not individually discussed in the contract at all**; only 7 `where-*` shapes are covered, in `REMAINING-CLARA-MOUTHS.md`, a separate and now-closed document.

## FINDING 1 — Clara's `:or` duplicate-insert semantics is a load-bearing translation caveat the contract never states, anywhere

**Code side (mechanism confirmed):** `where-or-conditions.clj:2` states it outright: *"n= is DISTINCT locations (Clara insert!s twice when both arms match)."* This is Clara's genuine behaviour — its `:or` compiles to independent activation paths, so a rule fires its RHS **once per matching disjunct**; two arms matching means two `insert!`s of the same logical fact. To compensate, `:45` computes `n-locs` as `(count (set (map :?loc …)))`, not a raw row count.

That `(count (set …))` collapse recurs in **16 of the 43 `.clj` twins** — `where-accum-group`, `where-accum-lead`, `where-accum-where`, `where-exists`, `where-not-and-bound`, `where-not-and`, `where-not-and-not`, `where-not-fact`, `where-not-not`, `where-not-or`, `where-not-where`, `where-not-windy`, `where-or-and`, `where-or-conditions`, `where-or-inline` — while their `.wat` twins compute the paired `n-*` as a **raw** `(:wat::core::length (:wat::rete::query …))`. Only `where-or-conditions.clj` states *why*; the other 15 assert *that* their `n=` is "distinct" without the mechanism, and it is not needed for most of those shapes (`:not`/`:exists` fire at most once per token on both engines, per the contract's own A5/A9 analysis) — it appears to have been copied defensively rather than derived per file.

**Spec side (the gap):** `CLARA-TRANSLATIONS.md:6-7` states its own scope as covering, per axis, *"whether the two compute the same final derived set, and any semantic caveat that could make an accuracy/speed differential misleading."* Grepped in full: the document discusses `:or` **zero times** as a Clara-vs-wat semantic point. Its one `:or`-adjacent line (`:179`) is about wat's native **query** masking token multiplicity via `production_delta` — an entirely different mechanism, in the A9 section.

**Which is right:** the *code* is right and safe as constructed; the *contract* is incomplete. It documents caveats for 8 axes that mostly have no static twin here, while staying silent on the one Clara-specific quirk that reaches **16 of the 43 real twins**. A future rider authoring a new `:or`-shaped `.clj`, reading only the contract as its stated grounding source, would have no warning that Clara doubles the insert — exactly the failure mode the ward watches for: **a translator misled by the document that exists to prevent that mistake.**

**Severity:** Medium. Not a live accuracy bug (the 16 sites already compensate), but a real gap in the corpus's single point of translation authority, on its most-repeated Clara-specific idiom.

## Observation (not a canonical finding — one coordinate only, noted transparently)

`CLARA-TRANSLATIONS.md:363` cites *"Rule 4 of this document (mirror the OPERATION, not the vocabulary)"* as the authority for striking A10's original "no twin" reasoning. I read the document's full text and its complete git history (3 commits: `5fcf4bda0`, `372e65f66`, `545771b2f`); **no numbered rule list ever existed in this file at any point**, and none exists in `docs/arc/2026/06/278-rules-engine/DESIGN-clara-grid.md` either. The citation is self-referential and dangling. **I report this as an aside rather than a conferre finding because it has no code-side coordinate** — it is a spec-internal broken citation, not a spec/code divergence — but it is worth attention, since it is the sole justification given for treating A10 as resolved.

## Runes

None. `grep -rn "rune:conferre"` over the target returned nothing.

## Prior art respected

All 8 pairs read in full were **structurally faithful rule-for-rule** to their stated intent, several with exemplary self-documentation of deliberate deviations (e.g. `where-not-derived-in-query`'s stated Clara-vs-wat acceptance asymmetry; `userfn-head`'s *"if mk-rate ever computes, this file… must be rewritten"* fidelity guard).

**FINDINGS: 1** plus one non-canonical aside.
