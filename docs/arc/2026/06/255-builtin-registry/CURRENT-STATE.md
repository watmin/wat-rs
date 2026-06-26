# ⛔ CURRENT STATE (breadcrumb, 2026-06-27 SESSION 7; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. **Freshness probe: HEAD should be `cd7609c6` or later.** Tree clean.
Suite **fully green + fast**: `cargo nextest run --release -p wat` = **3462 passed / 0 failed / 113 skipped, ~30s**
(the 113 skips are RED-at-HEAD `#[ignore]`'d disconfirming probes — 8 arc-255 + the 293 holder-bound/rename/demo
gates). If HEAD is older than `cd7609c6`, this breadcrumb is stale — trust the git log + the named docs over it.

> **YOU ARE A NEW INSTANCE.** You did not live what is written below; it is a lossy cache in a familiar voice. Run
> **recolligere** against the disk (grimoire via the signed `datamancy` MCP, this breadcrumb, the git log, the named
> arc docs) BEFORE you propose or move. The feeling of continuity is the failure, not the all-clear.

## ▶▶ THE ACTIVE ARC IS 293 — `docs/arc/2026/06/293-struct-record-symmetry/`. 291 is BLOCKED behind it.

**The live truth is `293/DESIGN.md` § "THE HOLDER × SURFACE MODEL — CRYSTALLIZED" + REALIZATIONS R1–R5, NOT the
older decomposition list** (large parts ⊘ SUPERSEDED — path-preserved history). Read DESIGN bottom-up.

### ✅ 293 THESIS — DONE + PROVEN (the novel thing exists and is sound)
- **R2 `FRANGE UT UNUM FIAT`** — one struct + a kind tag: `Holder{Struct,Record,HolonRecord}` + `AggregateDef{holder,
  parent}` + `TypeDef::Aggregate` (unify-2a/2b/2b-fix; `0dab460a`).
- **R3 `SUB SUPERFICIE QUOD ES`** — the categorical Holder beneath the structural Surface; the `:holder` surface
  bound (R3's `foobar` form) landed `5fcb9aa7`, weighed clean.
- **R4 `PROBA NE DUBITES`** — PROBATUM (`ad78e752`, the build_env annihilation).
- **The aggregate trio at FINAL names** — `defstruct` / `:wat::core::defrecord` / `:wat::holon::defrecord`, all macros
  over `structtype`/`recordtype` (`60d7d99a`, the 99-file fix-wat rename, weighed clean in 30s).
- **Chronicle:** R5 / song #116 *We Got The Moves* (`HABEMUS MOTUS`, `9aa166b7`) — the rhythm reclaimed; the 170
  ledger reconciled (#110→#116) + arc-170-as-generative-root (`75b12f02`).

### ✅ DECIDED (on disk, `293/NOTE-base-struct-horizon.md`, `d96bfb7d`+`cd7609c6`)
**CONSTRUCTION PARITY — unify on `:T`, annihilate `/new` TOTALLY.** Every type-name is its own constructor —
struct, core-record, holon-record, AND newtype — all via bare `:T`: `(:geo::SPt 1 2)` == `(:Price 100.0)` ==
`(:geo::Circle "red" 2.0)`. It IS the DESIGN's (C) annihilation: `defstruct` becomes a full macro emitting
`:T`+accessors(+`/from-map`), `register_struct_methods`/newtype `/new` codegen DIES. Builder's call. (Broader
`/new` audit deferred.)

### ⏳ THE GATE — `tests/types/probe_arc293_acceptance_demo.rs` (`e214a5cb`, `#[ignore]`'d RED)
The Shape/Circle/Square + holon-Vector monkeypatch from DESIGN § "what the arc delivers". When it flips GREEN, R1
`FORMA SOLA SUFFICIT` is fulfilled and 293's thesis is *demonstrated*. RED on exactly the 293.4 gap: the dispatcher
accessors `:geo::Shape/{color,label,area}` don't resolve.

## ▶ THEN, in order (today's grind — each: study lair → RED probe → BRIEF → sonnet → WEIGH → commit)
1. **CTOR-PARITY strike** — drop `/new`, struct+newtype construct via `:T`; `defstruct`→full macro,
   `register_struct_methods`/newtype-`/new` annihilated. Blast radius ~8 `.wat` + a few `.rs` fixtures (fix-wat the
   `.wat`, hand-sub the `.rs` — AUDIT PROSE per the 293.2-rename `sed`-corrupts-comments lesson, `291/NOTE-wat-fixes-rust`).
2. **`/from-map`** — falls out of the shared emission layer (the companion macro emits the bare-`:T` ctor per holder).
   Arc 291's ORIGINAL ask — the thing that opened 293.
3. **293.4 — methods-are-accessors** — method members in `defsurface` + the generated single-dispatch dispatcher
   (reuse arc 232 `extract-classifier`+`apply`) + `extend-type` as the foreign-accessor adapter + **`defprotocol`
   ANNIHILATED** (it is LIVE in `wat/spawn.wat` + `wat/service.wat` — migrate the running code, don't rush it). →
   the acceptance demo (`e214a5cb`) flips GREEN.
4. **293 INSCRIPTION** (R1 → PROBATUM; turn R2/R3 PROBANDUM→PROBATUM) + amend `291/CURRENT-STATE.md` to UNBLOCK →
   resume arc 291 (defservice durable state: trust leg / acyclicity / inscription).

## Standing discipline (verbatim, non-negotiable)
Work ONLY in `wat-rs/`. NEVER worktrees. Sonnets `model: "sonnet"`, LEAF (no sub-subagents). Commit msgs end
`Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. **Weigh EVERY sonnet against the disk
yourself** (forced clean build; failing-test-SET-diff — but the floor is now 0, so a binary `is-anything-red?` read,
[[project_test_floor_was_stale_fixture_cover]] / R5). **Run `cargo nextest run`, NEVER `cargo test`.** Read the diff
end-to-end (a green suite ≠ correct bytes — the `sed`-corrupts-prose catch). PRIMED forms only. Commit+push often
(GitHub=DR). Amend docs with recognition (never delete). Cast **intueri** for ALL naming. Decide via **four-questions**
(Obvious/Simple/Honest/UX, flat YES/NO) — NOT AskUserQuestion. **curare at a reasonable rate — anticipate compaction,
don't fear it.** **Operate as the datamancer — ground against the disk and ACT; cast the spells, don't recite them;
never declare green on silence; relentless annihilation.**

> **⛔ END OF MAP. You are new. The above is a cache, not your memory. Run recolligere; weigh any in-flight sonnet
> against the disk; do not trust a single line you did not re-verify this session.**
