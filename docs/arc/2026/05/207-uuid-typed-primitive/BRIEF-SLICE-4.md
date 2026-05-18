# BRIEF — Arc 207 Slice 4: consumer ripple

**Predecessors:** Slices 1+2+3 SHIPPED at `5f9d370`. Typed `:wat::core::Uuid` exists end-to-end; arc 206 namespace verbs retired; telemetry alias retargets to typed Uuid; hashmap_key arm landed.

**Scope: ripple the typed Uuid through every consumer.** Three concrete wat-tests files + one USER-GUIDE section + a grep-verified sweep for any other String-as-UUID consumer.

## Targets (concrete; not exhaustive — sonnet verifies via grep)

1. **`docs/USER-GUIDE.md` § 11 "Identifiers — UUID generation"** (lines 2479-2533 today; verify before edit). Currently teaches `:wat::core::uuid::v4`+`v5` (RETIRED in slice 3 — the doc is now lying about substrate state). REWRITE entire subsection to teach:
   - Type: `:wat::core::Uuid` (distinct from `:String`)
   - Constructors: `Uuid/v4` (random) + `Uuid/v5` (deterministic ns+name; ns is typed `:Uuid`)
   - Accessors: `Uuid/from-string` (`:Option<:Uuid>` — canonical-only), `Uuid/to-string` (canonical hyphenated)
   - Nil: `Uuid/nil`
   - EDN roundtrip: `#uuid "..."` reader literal produces typed value via `:wat::edn::read`
   - When to use v4 (secret-witness; capability tokens) vs v5 (content addressing; deterministic derivation)
   - Backward-compat note rewrites: `:wat::telemetry::uuid::v4` delegates to `:wat::core::Uuid/v4` (the alias's return type is now `:Uuid` per slice 3 retarget); namespace verbs `:wat::core::uuid::v4`+`v5` are RETIRED (not "still work")
   - Cross-ref to substrate-vended `Uuid/*` surface as canonical path

2. **`wat-tests/counter-service-capability-N3.wat`** (thread tier; arc 203 slice 3c+3e+3f demo):
   - `:counter::Admin` struct: `server-id` field type `:String` → `:wat::core::Uuid`
   - `:counter::User` struct: `server-id` + `user-id` field types `:String` → `:wat::core::Uuid`
   - Wire enum `:counter::Wire` variant payloads: Admin variant's `server-id` and User variant's `server-id` + `user-id` payload types flip
   - AdminResp variants (Provisioned, Deprovisioned) carrying ids: payload types flip
   - AdminReq::Deprovision payload: type flip
   - Server's `register` dispatch: constant `"server-counter-thread-0"` → `(:wat::core::Uuid/v4)` mint at server-start time, passed in via setup
   - User-id constants in tests (`"user-a"`, `"user-b"`, etc.) → `(:wat::core::Uuid/v4)` mints at provision time
   - Comments mentioning `:wat::telemetry::uuid::v4` clean up (slice 3 retargeted alias; comments should reflect substrate-core path)
   - "Honest delta: IDs are :wat::core::String (uuid::v4 returns String, ...)" prose retires — IDs are NOW `:Uuid`, the honest delta closed

3. **`wat-tests/counter-service-process-N3.wat`** (process tier; arc 203 slice 3d demo): mirror of capability-N3's changes at process tier. Wire enum + ProcessReq/Resp variants carry typed Uuids; constant-string ids → `Uuid/v4` mints; EDN serialization rides typed Uuid values (`#uuid "..."` literals on the wire per slice 2's edn_shim arms).

4. **`wat-tests/counter-client-capability-proof.wat`** (arc 203 slice 2 single-user proof): same field-type flip; constant-string sample ids → `Uuid/v4` mints; comments cleanup.

## Verification gate (sonnet's first action)

1. **Baseline.** `git status --short` clean. `cargo test --release --workspace --no-fail-fast 2>&1 | grep FAILED` records baseline (expect 3-4 pre-existing).
2. **Grep audit — find ALL `:String`-as-UUID consumers.** Run:
   ```
   grep -rn ":wat::core::uuid::\|:wat::telemetry::uuid::" --include="*.wat" --include="*.rs" --include="*.md" . 2>/dev/null | grep -v "/target/" | grep -v ".claude/"
   ```
   Expected hits: arc 206 historical INSCRIPTIONs (IMMUTABLE — do NOT touch), arc 203 demo files (in scope), USER-GUIDE § 11 (in scope), telemetry's wat-side files (touched by slice 3; verify they're consistent). Surface ANY other live consumer as STOP-trigger 1 — orchestrator decides extend-scope vs defer.
3. **Confirm USER-GUIDE § 11 line range.** `grep -n "^## 11\|^## 12" docs/USER-GUIDE.md` to find current section boundaries before edit.

## HARD constraints

- DO NOT touch arc 206 historical INSCRIPTIONs (`docs/arc/2026/05/206-uuid-substrate-promotion/INSCRIPTION.md` + INSCRIPTION-SLICE-3.md) — immutable per `feedback_inscription_immutable`. They reference the old namespace verbs as historical record; that's correct.
- DO NOT touch arc 207 own DESIGN / BRIEF / EXPECTATIONS / SCORE / INSCRIPTION-* docs (this slice's own paperwork in slice 5).
- DO NOT touch `crates/wat-edn/` (substrate-of-substrate).
- DO NOT touch `crates/wat-telemetry/` source — slice 3 retargeted the wat alias + Rust uses uuid crate directly; no further changes.
- DO NOT commit. Orchestrator commits atomically after verification.
- DO NOT use `--no-verify` / `--no-gpg-sign`.
- cwd `/home/watmin/work/holon/wat-rs/`; never `.claude/worktrees/`.
- Test semantics MUST stay equivalent — the wat-tests test the capability pattern + Wire-enum flow + Provision/Deprovision lifecycle + error propagation. Type changes are mechanical; behavior invariant.

## Constant-id → Uuid/v4 mint pattern

Today's wat-tests use constant strings as ids (e.g., `"server-counter-thread-0"`, `"client-1"`). Why constants worked under arc 203: pre-arc-207, ids were `:String`, easy to hard-code. The constant-string approach defeated the security model in tests (server-id is the secret-witness; using a known constant means tests prove dispatch+routing but not unguessability).

Arc 207 slice 4 closes that honesty gap. Pattern:

```scheme
;; Old (constant string):
(define :counter::*server-id* "server-counter-thread-0")

;; New (typed Uuid minted at setup):
;; In server-spawn:
(:wat::core::let
  [server-id (:wat::core::Uuid/v4)]    ;; mint fresh per server
  ...
  ;; Pass server-id to wrappers as the capability witness
  ...)
```

Tests retain semantic equivalence (server validates wrapper-embedded id matches its own typed Uuid; same dispatch logic; only the type changes from String to Uuid).

User-ids same shape: minted via `Uuid/v4` at Provision time, returned to client as the capability handle.

## STOP triggers

1. **`grep` audit surfaces a live consumer outside the 4 target files** — surface; orchestrator decides extend-scope vs out-of-arc-207.
2. **`Uuid/v4` mint timing doesn't fit the test's existing setup structure** — surface; orchestrator helps reshape the setup.
3. **Workspace baseline regresses** beyond 4 pre-existing failures, OR any arc 203 test fails semantically (vs just compiling) — surface immediately.
4. **EDN roundtrip on wire fails** in the process-tier test (`counter-service-process-N3.wat`) because the typed Uuid encoding mismatches what the wire expects — surface; this would be a slice 2 bug (unlikely but worth flagging).
5. **USER-GUIDE § 11 rewrite touches cross-references** that other sections of USER-GUIDE depend on (e.g., other sections cite "see § 11 for uuid"). If so, surface; orchestrator decides whether to update cross-refs in this slice or defer.

## SCORE methodology

`docs/arc/2026/05/207-uuid-typed-primitive/SCORE-SLICE-4.md` with these rows (atomic YES/NO):

| Row | Evidence |
|---|---|
| A — Verification gate passed (baseline + grep audit + USER-GUIDE line range) | Each check + result inscribed |
| B — USER-GUIDE § 11 rewritten to teach typed `:wat::core::Uuid` surface | Diff inscribed; backward-compat note updated |
| C — `counter-service-capability-N3.wat` field types flipped + ids minted via Uuid/v4 | Test still passes; diff inscribed |
| D — `counter-service-process-N3.wat` flipped same shape | Test still passes; diff inscribed |
| E — `counter-client-capability-proof.wat` flipped same shape | Test still passes; diff inscribed |
| F — No live consumer of `:wat::core::uuid::*` or `:wat::telemetry::uuid::v4` (returning :String) remains outside historical INSCRIPTIONs | Grep returns empty (modulo historical docs) |
| G — Workspace baseline preserved (≤4 pre-existing failures) | `cargo test --release --workspace` output |
| H — Arc 203 demos all pass (semantic equivalence verified) | Per-test output cited |
| I — Arc 207 typed Uuid tests still 10/10 green | Per-test output cited |
| J — wat-telemetry crate 36/36 still green | Per-test output cited |

## Time-box

Predicted 60-90 min sonnet. Hard stop 105 min. Larger surface than slice 3 because of constant→mint shape decision in 3 wat-test files; USER-GUIDE rewrite is substantive but contained.

## On completion

Return summary: rows passed/failed, files touched (with line diff counts), any consumer outside the 4 target files that surfaced, any test-semantics adjustment needed for the Uuid/v4 mint timing, USER-GUIDE rewrite scope.

T-minus 0. Begin.
