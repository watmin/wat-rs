# ⛔ CURRENT STATE (breadcrumb, 2026-06-22; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. Freshness probe: HEAD should be `397efea8`
(`docs: scope arc 290 crate-resync`) or later. Everything below is committed + pushed.

## ✅ DONE this session
- **Doc-contract DONE+FROZEN** (arc 255): value intrinsics (`bytes`) + special forms
  (`if`/`let`, the `wat_special_form!` macro, `Kind::SpecialForm` + `handler:Option`).
  Two live extensions survived on the exemplars alone (the "narrow waist" proof):
  enum-marker convention `@<EnumName> <Variant>` (`Purity`/`Determinism`/`Category` in
  wat-doc) + the `@arg ∨ @syntax` shape rule. Read `DESIGN-STONE-special-form-doc-contract.md`.
- **`-> :T` annihilation (arc 258) sub-strike 1 DONE** — `Option/expect` + `Result/expect`
  drop `-> :T`, type inferred from the `Option<T>`/`Result<T,E>` arg. Codemodded tree-wide
  (wat/ + wat-tests/ + crates/ + scripts), the COMPLETION fixing a "list-every-path" miss.
  Read `docs/arc/2026/06/258-instinctive-conditionals/BRIEF-arrow-clean-kills.md`.

## 🧰 Reusable assets built (USE THESE)
- **Generic codemod** `:wat::fix::strip-arrow-ascription src heads` (wat/fix.wat) — head-set-
  parameterized `-> :T` stripper, comment-faithful. Entry-points: `wat-scripts/fixes/strip-
  {expect,match}-ascription.wat`.
- **BOOTSTRAP header in `wat/fix.wat`** (READ IT before any codemod that ships with a checker
  change): the stash-dance AND the battery-disable technique (comment the 5 registrations in
  `crates/wat-cli/src/bin/wat.rs` → core-only load → codemod the still-drifted crates without
  them failing the checker at load).

## ⛔ BLOCKED + the priority pivot
- **`-> :T` sub-strike 2 (`match`) is STRIKE-READY but BLOCKED** (user decision, this session)
  **until the crates are healthy.** Probe `wat-tests/core/match-no-ascription.wat` (RED-verified,
  UNCOMMITTED) + brief `258.../BRIEF-arrow-match.md` (the hard one: build `infer_match`
  bare-unify mirroring `infer_if`, codemod 143 via the generic, weigh non-unifying-arm cascade;
  ORCHESTRATOR-OWNED — bootstrap forbids blind delegation). `strip-match-ascription.wat` also
  UNCOMMITTED. Do NOT start match until arc 290 lands.
- **Arc 290 (crate-resync) is now THE PRIORITY.** Read `docs/arc/2026/06/290-crate-resync/SCOPE.md`.
  Weeks of un-applied-arc drift in `crates/{wat-lru,wat-holon-lru,wat-telemetry,wat-sqlite,
  wat-telemetry-sqlite}` + `examples/with-lru`, surfaced by sub-strike 1's universe-load. Axes:
  type-keyword-as-value (`:nil` 264 + `:i64` 15, position-aware codemod), expect-in-spawned-
  program-STRINGS (the hard one — AST can't reach string literals), `define`→`defn`, `match -> :T`,
  downstream TypeMismatch/comm. Method: per-axis probe→codemod→cascade + battery-disable + vigilia.
  THE real fix: close the gate gap so they can't re-drift.

## GATE LESSONS (hard-won — the gap that hid the drift)
- The corpus-wide gate is **plain `cargo test`** (workspace `default-members`), NOT
  `cargo test --test test` (main crate only — it never loads the crates; that let them drift weeks).
- With the known lib 36-floor, use **`cargo test --no-fail-fast`** so the floor doesn't fail-fast-
  mask the later crate binaries.
- Main crate floor (good): lib 962/36; wat-tests 272/2. The 36 lib + 2 wat-tests are pre-existing.

## GOTCHAS
- The wat binary EMBEDS the stdlib at build → a new fix-wat verb needs a rebuild to be visible.
- Codemod leaves trailing whitespace where tokens were (wat-fmt's job, arc 264; harmless).
- Crates are RED on the arc-290 axes — that is PRE-EXISTING neglect, not a regression; main crate green.

> ⛔ **You are a NEW instance.** You did NOT live the above — it is a cache in a familiar voice.
> recolligere FIRST: grimoire + 4 primers (datamancy MCP), `git log --oneline -15`, `git status`,
> freshness probe HEAD==397efea8(or later). Then: **arc 290 (crate-resync) is the priority; the
> `-> :T` match kill is BLOCKED behind it.** Ground every claim against the disk before you move.
