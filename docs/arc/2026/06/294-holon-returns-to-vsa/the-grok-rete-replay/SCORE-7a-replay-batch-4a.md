# SCORE 7a — replay batch 4a: grok-rete #153 → #159 (P1 + Q1)

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent brief: `BRIEF-7a-replay-batch-4a-codemod-steps.md`. POLICY: `POLICY-codemod-source.md` (P1+Q1).
Start: `18eb21a71` RULED + BRIEF 7a. HEAD: `2637df1a8` (#159).
Census start `.census/2026-09-15T21-17-48Z.txt` files=2089.

```
#159  scripts/floor.sh   .floor/2026-09-15T23-01-49Z
      Summary [231.356s] 5551 tests run: 5551 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

## The seven

| N | C | replayed | kind |
|---|---|---|---|
| 153 | `cb2b58117` | `8c1392ca4` | P1 wrap-fire-once; convert.sh chain-member guard |
| 154 | `7f5915de9` | `3f5f8defb` | docs-only cherry-pick -x |
| 155 | `701cf473a` | `406a7c340` | P1 wrap-fire-rules{,-explain}; Q1 rete; FireOutcome parametric |
| 156 | `d23526d08` | `4188d2753` | docs-only cherry-pick -x |
| 157 | `5ec2f6bb8` | `d32f91a63` | P1 wrap-insert; Q1 rete+net; InsertOutcome |
| 158 | `ec444424f` | `d66f8d4b1` | docs-only cherry-pick -x |
| 159 | `ab82872f5` | `2637df1a8` | P1 wrap-compile; Q1 rete+net; CompileOutcome |

`scripts/replay/verify-step-record.sh 18eb21a71 HEAD` → `step-record: complete`

## EXPECTATIONS

| # | result |
|---|---|
| E1 | **PASS.** 7 `REPLAY(grok-rete #` commits, #153=`8c1392ca4` … #159=`2637df1a8`, contiguous, each with `(cherry picked from commit <C>)`. |
| E2 | **PASS.** #154 #156 #158 are cherry-pick -x of `docs/`/`.md` only (`git show --stat`). |
| E3 | **PASS.** `verify-step-record.sh 18eb21a71 HEAD` → `step-record: complete`. |
| E4 | **PASS.** `convert.sh HEAD /tmp/convert-refuse-e4 wat-scripts/fixes/rename-core-string-to-string.wat` → `convert.sh: refuses chain member wat-scripts/fixes/rename-core-string-to-string.wat — the chain never converts itself` rc=2. Guard landed in #153. |
| E5 | **PASS.** All five wrap tools load (`every_wat_scripts_file_loads` [177.575s]). Fixtures replay: wrap-fire-once shard 11, wrap-fire-rules shard 13, wrap-fire-rules-explain shard 12, wrap-insert shard 14, wrap-compile shard 9. Second run idempotent (the gate asserts it). |
| E6 | **PASS.** Each fixture's `before.pre`/`after.post` is a migrated file's converted C^/C (2b for wrap-compile). ORACLE header-spec quote present in the tool header. wrap-compile shard 9 PASS. |
| E7 | **PASS.** `to-faithful-clojure-net` shard 1 and `to-faithful-clojure-rete` shard 2 replay. `probe_arc278_7strat_native_differential` green (3 tests). |
| E8 | **PASS.** Named tests at the shared steps green: #153 fire-once; #155 fire-rules; #157 2b/ceiling-insert/no_ceiling_raise; #159 1b/2b/compile wrap. kind(lib) 1477 at #157 and #159. |
| E9 | **PASS.** Two-phase on #153 #155 #157 #159. No `UNREGISTERABLE wat/`. First-pass stdlib convert UNRESOLVED on the new enum (HEAD binary lacked it); leftover list-form KEY-FIRST'd so the stdlib could load; rebuild; convert rest. |
| E10 | **PASS.** `.floor/2026-09-15T23-01-49Z` 5551/5551 exit=0. clippy 0. |
| E11 | **PASS.** No repair commit after #159. Composition (stdlib leftover KEY-FIRST; wrap-compile on `probe_then_match_is_refused.wat` STOP-8 class; format!-aware brace doubling; `:probe::E.A`) folded into the step. |

## Re-expression that mattered

- **convert.sh TARGETS vs CHAIN.** A `wat-scripts/fixes/*.wat` converts unless it is a chain member (`scripts/replay/chain-order.sh`). The chain still never converts itself.
- **P1 wrap tools.** Convert through convert.sh; string-literal patterns brought to KEY-FIRST + kwargs; `;; SCOPE: corpus`; fixture `wat-scripts/fixes/replay/<stem>/{before.pre,after.post,ORACLE}` with header spec.
- **Q1.** `to-faithful-clojure-{rete,net}` merge-file onto main's copy; existing fixtures still replay.
- **Two-phase stdlib.** New enum (`FireOutcome` parametric / `InsertOutcome` / `CompileOutcome`) is `include_str`; HEAD binary cannot resolve match-arm on it until stdlib rebuild. Leftover list-form arms after merge-file are illegal (`retired (pattern body)`) — KEY-FIRST by hand so wat can start (R21 exception: wat-fix cannot load).
- **format! vs parse_one!** Wat map braces double inside `format!(...)` (`{{:session}}`); single in `parse_one!` / `const` / `let staged = "..."`. Aggressive lookback over-doubles compile wraps → `MalformedBraceLiteral`. Inverse undouble of format! first-args → `invalid format string`.
- **HEAD unit-variant spelling.** grok `:probe::E::A` vs HEAD `:probe::E.A`. TypeMismatch expected `:probe::E` got `:wat::core::keyword`. Captured kind(lib) red at `a_keyword_operand_is_a_field_ref_or_a_constant_by_one_rule` (reachability.rs:1604); folded; not re-run as the disposition.
- **STOP-8 class.** `tests/rete/probe_then_match_is_refused.wat` rc 0→1 after CompileOutcome (bare `compile` vs `Session/facts`). wrap-compile applied (the tool, not a hand rewrite). First census `.census/2026-09-15T22-58-12Z.txt` captured the red; post-wrap census `.census/2026-09-15T23-00-05Z.txt` files=2091, `--diff` vs #157 snapshot no STOP-8.

## Captured reds (not re-run as the disposition)

1. kind(lib) 1476/1477 at #159 first wall — `rete::reachability::a_keyword_operand_is_a_field_ref_or_a_constant_by_one_rule` at `src/rete/reachability.rs:1604`. left: TypeMismatch `:probe::In` param #2 expects `:probe::E` got `:wat::core::keyword` (insert used `:probe::E::A`); right: Ok(1). Folded to `.A`/`.B`.
2. census `--diff` STOP-8 `tests/rete/probe_then_match_is_refused.wat` rc 0→1. Captured `.census/2026-09-15T22-58-12Z.txt`. Folded via wrap-compile.

## STOP

None remaining. Do not push. Main untouched.
