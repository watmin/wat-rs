# Arc 243 — conformare: error-shape class elimination

**Status:** ACTIVE. Opened 2026-05-30 immediately after Stone 241.18a SHIPPED (`4d9b963e`). Conformare is the disciplined response to the catastrophic-failure class surfaced during Stone 241.18a's vigilia.

## Why this arc

Stone 241.18a's vigilia surfaced a class of substrate-level diagnostic-quality failure:

- `ParseStep::ArityMismatch { actual: usize }` — no span field; the variant lies about being a structurally-spanned error type
- `TypeError::CyclicSubtype { child: String, parent: String }` — no span field; same class
- `TypeError` has no `.span()` accessor — every consumer must exhaustively match across all variants (parse.rs has a 16-arm match for this exact reason)
- `parse_fn_signature` API takes a bare `&[WatAST]` slice with no head-span parameter — callers (eval_fn, infer_fn) HAVE span context but the parser boundary discards it
- Other error types (`RuntimeError`, `CheckError`, `StartupError`, `ArgSpecError`) may have similar gaps — uncatalogued

**Rust's type system has no opinion on "errors must carry span."** Each error type's adherence to the span discipline is by hand-written convention (the argspec home documented this explicitly at `src/argspec/error.rs:6`). Without a trait + audit spell + convention doc, future error types continue to silently lack spans.

Per `scratch/FAILURE-ENGINEERING.md` discipline: **eliminate the CLASS by making the wrong shape STRUCTURALLY UNAVAILABLE.** Catching instance-by-instance via vigilia is reactive vigilance; minting the trait + spell + doctrine makes the class structurally impossible going forward.

## What this arc delivers

Multi-layer skill (precedent: ZERO-MUTEX.md):

1. **`docs/CONFORMARE.md`** — substrate convention doc; the WHY + the discipline statement; lists existing exemptions (`CyclicSubtype` legitimately has no source location — registry cycle, not source error) with reasoning.

2. **`datamancy.dev/conformare/SKILL.md`** — audit spell; casts on error-type enum definitions + `From` impl boundaries + parser API signatures. Flags non-conformance at L1/L2. Joins vigilia default set.

3. **`trait Conformare`** (Rust trait in `src/error.rs` or similar) — type-level guard. Every error type implements; the trait's bound enforces what Rust CAN enforce ("you must have a span accessor"). The audit spell catches what Rust can't (e.g., "your span is always Span::unknown" — the value-level dishonesty the type system doesn't see).

4. **Per-error-type retrofit** — every existing error type implements `Conformare`; variants gain span fields where structurally available; `Span::unknown()` is acceptable ONLY when documented (CyclicSubtype: no source location; registry-cycle context).

5. **Parser-API sister-walk** — every parser/check API that takes a bare slice without span context gains a `head_span: &Span` parameter. Closes the ArityMismatch class at the API boundary.

## Stone chain

Provisional; each stone gets its own DESIGN-STONE-N.md before strike.

| Stone | Scope |
|---|---|
| **243.1** | `docs/CONFORMARE.md` doctrine. Orchestrator-direct authorship (substrate convention doc, sibling to ZERO-MUTEX.md). Establishes the WHY, the discipline statement, the exemption framework. |
| **243.2** | Mint `datamancy.dev/conformare/SKILL.md` spell + first-cast audit on the entire substrate. Catalog every error-type variant + From impl + parser-API signature that violates the discipline. Output: a substrate-wide non-conformance manifest. |
| **243.3** | Mint `trait Conformare` in Rust. Choose the right home (`src/error.rs` new file? In an existing module?). Pure mint stone (no usage yet). |
| **243.4..N** | Per-error-type retrofit stones — each major error type gets a stone: implement `Conformare`, add span fields to variants, update From impls to preserve span. Likely stones: TypeError, ArgSpecError, RuntimeError, CheckError, StartupError, ParseStep (Stone 241.18a's leftover NEW-2 closes here). |
| **243.M** | Parser-API sister-walk — every parser/check API taking a bare slice gains `head_span: &Span`. Closes the ArityMismatch-style defensive-class at the boundary. |
| **243.M+1** | Add `conformare` to vigilia default defensive set for code files. |
| **243.Z** | INSCRIPTION — class structurally eliminated. |

## Trap-doors

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | `CyclicSubtype` (and possibly others) genuinely have no source location — registry-cycle errors fire at registration time, not at a source position | The audit spell at Stone 243.2 catalogs; per-variant judgment at Stone 243.4+ | `trait Conformare`'s `span() -> Span` returns `Span::unknown()` for these; the convention doc lists them as documented exemptions; rune mechanism at the variant level captures the WHY |
| **T2** | The retrofit cascade is potentially very large (every error type touches every consumer of its variants) | Stone 243.2's audit gives the size | Per-error-type stones; substrate-as-teacher discipline handles the cascade per stone |
| **T3** | `From<E1> for E2` impls that drop span data | Stone 243.2's audit + Stone 243.4+ per-type review | Update From impls; the audit spell catches them going forward |
| **T4** | Parser APIs taking bare slices are numerous; the sister-walk could be its own arc | Stone 243.2's audit reveals the count | Stone 243.M handles; if scope creeps, spawn arc 244 for the parser-API discipline specifically |
| **T5** | `trait Conformare` design (associated types? generic over Span? do we want Result<Span, Reason> for documented-exempt cases?) | Stone 243.3 design phase | Four-questions cast at Stone 243.3 prep; intueri on the trait method names |

## What this arc DOES NOT do

- Does not retrofit `wat` source-level error reporting (that's a separate concern: how errors RENDER to users at the command line)
- Does not change the substrate's error semantics (no error-merging, no error-recovery; just span-discipline)
- Does not address non-error types that might have similar span-discipline gaps (Bundles? AST nodes? — separate audit, separate arc if needed)

## Cross-references

- `scratch/FAILURE-ENGINEERING.md` — the principle this arc embodies
- `docs/ZERO-MUTEX.md` — the multi-layer skill precedent
- `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.18a.md` § Phase B — the vigilia rounds that surfaced this class
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 11 — deferral-language discipline (the orchestrator-doctrine sibling)
- `datamancy.dev/exigere/SKILL.md` — sibling spell for deferral-language (minted during Stone 241.18a; same authoring pattern as conformare will follow)
- Memories: `feedback_correctness_makes_honesty`; `feedback_runes_illegal_when_solvable`; `feedback_dont_document_non_fixes` — doctrines that landed during the realization

## What's deferred until arc 243 lands

- arc 241 Stone 241.18b-g (def-family migration: src/def/ + defmacro/defstruct/defenum/defclause/defalias retrofits) — these continue after conformare's class-elimination work completes. Conformare may inform their error-type designs.

## Status header (live)

- **Opened:** 2026-05-30
- **Trigger:** Stone 241.18a vigilia surfaced the substrate-wide error-shape class
- **Spell name verdict:** `conformare` (intueri cast 2026-05-30; runners-up `normare`, `redigere`, `respuere`)
- **Failure-engineering frame:** catastrophic class; cannot proceed with other substrate work until the structural fix lands
- **First stone:** 243.1 (CONFORMARE.md doctrine; orchestrator-direct)
