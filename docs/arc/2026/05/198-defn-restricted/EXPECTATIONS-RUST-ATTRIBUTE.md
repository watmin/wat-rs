# Arc 198 Slice 2 EXPECTATIONS

**BRIEF:** `BRIEF-RUST-ATTRIBUTE.md`

## Independent prediction

**Runtime band:** 180-300 minutes sonnet.

Reasoning:
- Proc-macro attribute parsing + codegen in wat-macros: ~80-150 LOC + tests
- `RestrictionEntry` struct + inventory wiring in wat crate: ~30-50 LOC + setup-iteration hook
- 2 fn annotations: trivial (2 lines added above each fn)
- Stone B rule deletion: ~30-60 LOC removed
- Stone B test updates: ~20-40 LOC changed
- Cargo.toml updates (add inventory dep): ~3-5 LOC
- New attribute-mechanism test: ~80-120 LOC
- Substantive work; novel substrate infrastructure (first proc-macro attribute beyond `#[wat_dispatch]`)

**Time-box:** 360 min hard stop.

## SCORE methodology

7 rows YES/NO per BRIEF; per-row evidence patterns:

- **Row A** (attribute defined): `grep -rn "restricted_to" crates/wat-macros/src/` shows the attribute proc-macro definition + variadic parsing
- **Row B** (inventory wiring): grep `RestrictionEntry\|inventory::iter\|inventory::submit` across src + crates shows the full pipeline
- **Row C** (attribute applied): `grep -B 2 "fn eval_kernel_thread_join_result\|fn eval_kernel_process_join_result" src/runtime.rs` shows `#[restricted_to(...)]` above each
- **Row D** (iteration populates): grep shows setup-time iteration; smoke test or debug print confirms HashMap has the right entries
- **Row E** (Stone B rule deleted): `grep "validate_join_result_user_namespace\|JoinResultUserNamespace" src/check.rs` returns ZERO matches
- **Row F** (tests pass): cargo test on the named test files all green
- **Row G** (workspace baseline): cargo test summed failed ≤ 4 (current baseline)

## Honest deltas to watch for

- **Sub-decision (a) vs (b) — attribute syntax shape.** The Rust fn doesn't naturally know its wat name. Options:
  - **(a)** Attribute carries wat name: `#[restricted_to(wat_name = ":wat::kernel::Thread/join-result", from = [":wat::"])]` — full self-contained but redundant with dispatch arm declaration
  - **(b)** Declarative macro at registration site: `register_restricted!(env, ":wat::kernel::Thread/join-result", scheme, [":wat::"])` — not an attribute on fn body
  - User stated preference: attribute on fn (so (a)). But (a) requires the wat_name redundancy. Sonnet decides which reads cleanest given the existing dispatch + registration architecture; document the choice + rationale in SCORE.

- **Inventory crate interaction with freeze flow.** The `inventory` crate uses linkme-style auto-registration; entries are collected at link time. Need to verify this works with wat-rs's freeze sequence. Setup iteration must run after `env.register` calls complete and before frozen-world starts.

- **wat-macros codegen patterns.** `#[wat_dispatch]` exists as precedent for proc-macro attribute infrastructure. Sonnet should extend or mirror those patterns rather than greenfield.

- **CheckEnv ↔ SymbolTable mirror.** Per arc 198 slice 1, `env.defined_value_restrictions` mirrors from `sym.defined_value_restrictions`. Need to verify that Rust-side restriction population happens at the right phase — either populate `sym` (so mirror copies it) or populate `env` directly after mirror (so the inventory-iteration happens AFTER the mirror clone).

- **Empty whitelist semantics.** Per arc 198 slice 1: `[]` matches nothing — every caller fails. Should `#[restricted_to()]` (no args) match this semantic, or be a parser error? Sonnet decides; either is defensible (parser error is more conservative).

- **Stone B test error-message updates.** Stone B's 4 tests `assert!(err.contains("drain-and-join"))` — arc 198's `DefRestrictedCallerNotAllowed` error message doesn't have "drain-and-join" wording. Need to either: update test assertions to grep arc 198's wording (e.g., "callable whitelist" / "DefRestrictedCallerNotAllowed"), OR update arc 198's error message to mention drain-and-join when the rejected verb is a *_join-result. Sonnet decides; latter is more teaching-flavored.

## Workspace baseline (commit `24d3b0d`)

- `cargo build --release --workspace --tests`: clean per arc 198 slice 1 SCORE
- `cargo test --release --workspace --no-fail-fast`: 4 pre-existing target failures (lifeline flake, t6 unquote, totally_bogus, startup_error)

Post-slice-2 target:
- ≥ baseline + 1+ passed (new attribute-mechanism test minimum; Stone B's 4 tests stay green via the new mechanism)
- ≤ 4 failed (no regressions; arc 198 slice 2 is additive + replaces equivalent enforcement)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 180-300 min | TBD |
| Scorecard rows | 7/7 PASS | TBD |
| Workspace fail count | ≤ baseline (4) | TBD |
| Sub-decision chosen | (a) attribute-with-name OR (b) callsite-macro | TBD |
| New test count | 1+ | TBD |
| Stone B test updates | 4 | TBD |
| Substrate-discovery surprises | 0-3 | TBD |
| Mode | Substantial (proc-macro + inventory + migration + rule deletion) | TBD |
