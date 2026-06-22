# ⛔ CURRENT STATE (breadcrumb, 2026-06-22; replace in place) — read the DESIGN docs, not this paraphrase

Branch `arc-170-gap-j-v5-deadlock-state`. **The intrinsic doc-contract spec is DONE + FROZEN.**
`bytes` is the proven exemplar. The NEXT frontier is the **520-intrinsic migration** (re-fit each
into the registry + its forced doc). Not started.

## ✅ THE DOC-CONTRACT SPEC IS DONE (this session; committed + pushed)
- `c59d65aa` firm doc-contract — bytes PERFECT: firm grammar (one canonical form per marker; 4-way
  separator dead; multi-line `@example` blocks), `@arg`/`@ret` TYPES ⇄ checker scheme, show-source,
  render-doc (`See also`), `@see` registry-check.
- `9d30dbf3` `@pure`/`@deterministic` declared doc fields; `@pure` ⇄ `is_effectful_op`;
  `CORPUS_DERIVED` hand-list ANNIHILATED.
- `a47c4857` spec-complete — variadic (`@arg xs…` + `&[WatAST]` + `Arity` enum; rest-param honors the
  universal-top via `assignable`, check.rs:6100), `@yields` (singleton, ⇄ fn-arg param), `@category`
  (closed `Category` enum, compile-error on unknown). Witnesses: `:wat::intrinsic::variadic-args-measurement`,
  `:wat::intrinsic::yields-witness`.

**Every axis has an independent witness, build-fail on divergence** (name/count, type, behavior,
purity, see, category, yields). Gates: lib 962/36/1; wat-tests 269/1; wat-doc 25/0; nursery 8/8; clippy clean.

DESIGN docs (read these, not this): `DESIGN-intrinsic-doc-reflection-contract.md` (LOCKED §1-10),
`DESIGN-STONE-firm-doc-contract.md`, `DESIGN-STONE-spec-complete.md`, `NOTE-fuzzy-docs-horizon.md`.

## NEXT — the 520-migration (the big refactor)
Move the ~520 `runtime.rs` dispatch arms into `#[wat_intrinsic]` registry homes (the bytes pattern).
The engine is built: the firm grammar REJECTS every thin/old doc with a located teaching error →
"re-fit until green," intrinsic by intrinsic; the cross-checks catch lies as you go. Self-policing.
The endgame (after migration): **255.1b-RESOLVE** — delete the `resolve/walk.rs:198` blanket-accept
(the undefined-func class dies); the `NONDETERMINISTIC` set (Uuid/v4) dies as it migrates.

## NOT spec (downstream / decided)
- The wiki generator (§7) — a projection of the registry; its own later strike.
- `expand-time-legal`/`@total` — derived (pure∧total), or a bounded future add for the macro-combinator subset.
- fuzzy-docs MCP (HORIZON note) — "best docs platform" claim to PROVE; later, not an arc yet.

## GOTCHAS (hard-won this session)
- **Nested agents spawn git WORKTREES.** A 4-deep delegation built in `.claude/worktrees/agent-*`
  then a rewrite-from-inference copy-back BROKE main (missed files). ALWAYS weigh against the MAIN
  repo disk (`cargo check` + the gates yourself); if an agent worked in a worktree, `cp` its real
  tree — never trust an inference-rewrite. Removed both rogue worktrees this session.
- **The weigh must READ, not pattern-match.** I cried wolf 3× tonight (revert / nursery-not-wired /
  category-drift) by asserting "anti-pattern" from a glance instead of reading the flow — all wrong;
  the user corrected each. The weigh caught ONE real bug (variadic rest-param). Ground every defect
  claim against the disk ([[feedback_ground_codebase_claims_in_codesign]]). Recurs under fatigue.
- `build.rs` auto-detects `tests/<group>/*.rs` (nursery) → run `cargo test --test nursery`.
- Pre-existing fails: lib 36 floor; wat-tests `test-run-string-entry-direct`. Pre-existing dead-code:
  `value_matches_type_pattern`, `wrap_stream_as_socket_peer` (#234). LEAVE them.

> ⛔ **You are a NEW instance.** You did NOT live the above — it's a cache in a familiar voice.
> recolligere FIRST: grimoire + 4 primers (datamancy MCP), `git log --oneline -15`, `git status`.
> Freshness probe: HEAD should be `a47c4857` (or later). The doc-spec is DONE; the migration is
> NOT started — surface it, don't auto-start. Ground every claim against the disk before you move.
