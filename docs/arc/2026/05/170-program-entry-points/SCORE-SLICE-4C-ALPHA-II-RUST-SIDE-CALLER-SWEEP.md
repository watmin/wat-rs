# Arc 170 Slice 4c-α-ii SCORE — Rust-side caller sweep of `:wat::kernel::run-sandboxed*`

**BRIEF:** `BRIEF-SLICE-4C-ALPHA-II-RUST-SIDE-CALLER-SWEEP.md`
**EXPECTATIONS:** `EXPECTATIONS-SLICE-4C-ALPHA-II-RUST-SIDE-CALLER-SWEEP.md`
**Task:** #320
**Predecessor tip:** `8adf62b` (working tree carried prior sub-Agent's partial migrations on 6 of 7 files; this run completed `wat_run_sandboxed.rs` and surfaced/folded in the `set-capacity-mode!` body-context posture).

## Outcome — 6/6 rows PASS

| Row | What | Result | Evidence |
|-----|------|--------|----------|
| A | Zero `:wat::kernel::run-sandboxed\b` callers in `tests/` (string entry) | YES | `grep -rEn ':wat::kernel::run-sandboxed' tests/ \| grep -vE '//'` → 0 active lines |
| B | Zero `:wat::kernel::run-sandboxed-ast\b` callers in `tests/` | YES | Same grep → 0 active lines (only `//`-prefixed migration markers remain) |
| C | Zero `:wat::kernel::run-sandboxed-hermetic-ast\b` callers in `tests/` | YES | Same grep → 0 active lines |
| D | Canonical macros appear in migrated sites | YES | `grep -rcE '^\s*\(:wat::test::run-(hermetic\|thread)' tests/<7 files>` shows 16 sites: 2 thread + 14 hermetic (plus 1 pre-existing hermetic in `wat_core_forms.rs` line 155 outside the migration set) |
| E | `cargo build --release --workspace --tests` clean | YES | `Finished` with 3 pre-existing lib warnings (dead_code on `parse_fn_signature_for_check`, `eval_kernel_process_send`, `eval_kernel_process_recv`); zero errors |
| F | Workspace test failure count ≤ 11 | YES | 2271 passed / 2 failed (`deftest_wat_tests_tmp_totally_bogus`, `startup_error_bubbles_up_as_exit_3`) — both pre-existing rotation members. `lifeline_pipe_zero_orphans_across_100_trials` was named in another aggregator block but the totals row counted 2; well within the band of ≤ 11 |

## Per-file site distribution (final)

| File | Thread | Hermetic | Notes |
|---|---|---|---|
| `tests/probe_plain_panic_produces_structured_edn.rs` | 0 | 1 | Body sets `set-dim-count!` + `set-capacity-mode!` at OUTER startup (the file's outer source — NOT inside the macro body); body triggers Bundle panic. Rule 3 of FM 7-ter fires. |
| `tests/wat_arc113_cross_fork_cascade.rs` | 0 | 1 | Cross-fork cascade — process boundary intrinsic. Hermetic. |
| `tests/wat_arc113_raise_round_trip.rs` | 1 | 0 | Body raises; outer reads only `RunResult/failure`. No stdio-slot reads. Thread is the cheaper, correct destination per three-rule. |
| `tests/wat_core_forms.rs` | 0 | 1 (migration) + 1 pre-existing | Migrated `forms_composes_with_run_sandboxed_ast` to hermetic (body println + outer reads stdout slot). `test_run_ast_via_test_program_roundtrips_hello` at line 155 predates this slice. |
| `tests/wat_hermetic_round_trip.rs` | 0 | 2 | Already-hermetic legacy verb (`run-sandboxed-hermetic-ast`) — process-boundary preserved. |
| `tests/wat_run_sandboxed_ast.rs` | 1 | 1 | First test reads stdout slot → hermetic; second test reads only failure → thread. Per-three-rule classification. |
| `tests/wat_run_sandboxed.rs` | 0 | 8 | File-header doctrine fixed every site at hermetic. 6 normal + 2 scope-using; all rearchitected per substrate finding (below). |
| **Total migrations** | **2** | **14** | 16 active sites (matches BRIEF), 1 pre-existing hermetic outside migration set. |

## Reclassifications mid-sweep

None at the thread→hermetic level — the prior agent and this run respected the file-header doctrine in `wat_run_sandboxed.rs` ("every site lands on `:wat::test::run-hermetic`") and the three-rule classification across the other files. No site originally aimed at thread had to be re-aimed at hermetic.

## Honest deltas — substrate findings (load-bearing)

### Finding 1 — `set-capacity-mode!` is a startup-time setter, not a runtime verb

The BRIEF's posture had several rule-3-of-FM-7-ter classifications grounded in "body mutates runtime config (`set-capacity-mode!`)". On migration, `:wat::config::set-capacity-mode!` inside the canonical-macro body fires:

```
unknown function: :wat::config::set-capacity-mode!
```

at evaluation time. Root cause (verified in `src/config.rs:331`): `set-capacity-mode!` is parsed by `Config::from_source` as a **top-level startup setter** — it must appear before any `define` form at parse time. It is NOT registered as a runtime verb and cannot be evaluated as an expression inside a macro body.

**Consequence for the migration:** all 6 non-scope sites in `wat_run_sandboxed.rs` had their body's `(:wat::config::set-capacity-mode! ...)` form REMOVED (would crash the child with "unknown function"). The child runs with the default `CapacityMode::Error` (per `DEFAULT_CAPACITY_MODE` in `src/config.rs:59`).

**Three-rule classification remains correct** — rules 1 + 2 (outer reads stdio slots + body calls stdio verbs) still fire across these sites, so hermetic destination is still mandatory. Only the rule-3 rationale ("body mutates runtime config") was misapplied; the body cannot mutate capacity-mode through the canonical macro path.

Sites affected by this finding (body had to drop `set-capacity-mode!`): 1, 2, 3, 4, 5, 6, 7, 8 of `wat_run_sandboxed.rs` (all of them).

### Finding 2 — `:wat::kernel::raise!` requires `:wat::holon::HolonAST`, not `:wat::core::String`

First-pass migrations of sites 4 and 5 used `(:wat::kernel::raise! "string")`. The type-checker rejected with:

```
TypeMismatch { callee: ":wat::kernel::raise!",
               param: "#1",
               expected: ":wat::holon::HolonAST",
               got: ":wat::core::String" }
```

Fixed by wrapping in `:wat::holon::leaf` per the canonical pattern (`wat_arc113_raise_round_trip.rs` line 64: `(:wat::kernel::raise! (:wat::holon::leaf 42))`).

### Finding 3 (LOAD-BEARING — corrects BRIEF) — `scope :Option<String>` was FUNCTIONAL plumbing, not "never functional"

The BRIEF stated:

> **scope :Option<String>** — DROP entirely (leaked substrate plumbing; never functional in legacy).

**This claim is wrong.** Direct read of substrate sources confirms:

- `spawn-process` and the canonical `:wat::test::run-hermetic` macro hardcode `InMemoryLoader::new()` for the child. The parent's loader does NOT propagate.
- The legacy `:wat::kernel::run-sandboxed src stdin scope` `scope` parameter, when set to `(:Some "<path>")`, drove a `ScopedLoader` for the child's `eval-file!` / load mechanism. The legacy `scoped_file_eval_inside_scope_succeeds` test relied on this — its `(:Some <scope-path>)` argument actually configured a `ScopedLoader` containment check that ALLOWED in-scope reads and BLOCKED out-of-scope reads.

The canonical macros (`run-thread` / `run-hermetic`) have NO surface for per-call scope override. So when the two scope-using sites (`scoped_file_eval_inside_scope_succeeds`, `scoped_file_eval_outside_scope_surfaces_as_err`) migrate to `:wat::test::run-hermetic`, the `ScopedLoader` containment semantic is LOST. Both tests now exercise canonical-macro behavior under a default empty `InMemoryLoader` — the child has no loader entry for ANY path, so `:wat::eval-file!` always routes through the Err arm regardless of in-scope or out-of-scope.

**Mechanical migration accepted per user policy (accumulate-tests-rearchitect-not-delete).** Both tests retained, both now assert "Err arm → eprintln" (post-migration symmetric shape). Inline doc comments in both tests document the semantic loss explicitly.

**Substrate gap surfaced:** the `ScopedLoader` containment test surface is now uncovered. A future follow-up (outside this slice) needs to either:
1. Reach `ScopedLoader` through a non-spawn-sandbox path (test the loader directly at the Rust API level), or
2. Add a canonical-macro variant that accepts a scope/loader override.

This is NOT a regression introduced by the slice — `ScopedLoader` containment coverage was already going to be lost when the substrate verb retires under #310. The slice surfaces the gap honestly rather than burying it.

### Finding 4 — `parse_error_in_source_surfaces_as_failure` test purpose is unreachable

The legacy verb accepted a source STRING and parsed it inside the child; an unterminated string in the inner source surfaced as a startup parse error captured into `Failure`. The canonical macros accept BODY FORMS (already parsed at the outer Rust level), so the "inner source has lexer error" surface cannot be exercised through this path.

Rearchitected: body uses `raise!` to surface a runtime failure; outer asserts non-empty Failure message. Same shape (failure surfaces in `Failure.message`), different mechanism. Stdio-empty asserts on stderr had to be loosened (`raise!` routes structured EDN through stderr as part of canonical failure capture).

### Finding 5 — `missing_user_main_surfaces_as_failure` test purpose is unreachable

The legacy verb required a `:user::main` definition in the inner source and failed at startup if omitted. The canonical macro body IS the entry — there is no `:user::main` requirement to violate. Rearchitected to assert a different specific-message-in-Failure shape (raise! with a sentinel payload).

### Finding 6 — `sandboxed_panic_caught_into_failure_and_partial_output_preserved` original mechanism unreachable

Original mechanism: body sets `capacity-mode :panic`, writes "before panic" to stdout, then triggers a raw Rust panic via `:wat::holon::Bundle` exceeding capacity. With `set-capacity-mode!` not body-callable AND the child defaulting to `CapacityMode::Error`, the Bundle would return an Err Result (not panic).

Rearchitected to use `raise!` after the println — same shape: partial stdout survives + Failure carries the payload — but no longer exercises the raw-Rust-panic-via-Bundle path. Original panic surface needs separate coverage outside this slice.

## Stderr-empty asserts relaxed

In sites 4 and 5 (the raise!-based rearchitectures), `assert!(stderr.is_empty())` had to drop. Under the canonical macro, `raise!` routes structured EDN through stderr as part of failure capture. The legacy verb produced no stderr noise from `raise!`. Documented inline.

## File-header doc-comment refreshes

3 of the 7 files needed (and received, per the prior sub-Agent + this run) line-1 doc-comment refresh:

1. `wat_hermetic_round_trip.rs` — line 1 now says "Integration: `:wat::test::run-hermetic` round trip (arc 170 slice 4c-α-ii)" instead of naming the legacy verb.
2. `wat_run_sandboxed_ast.rs` — line 1 now reads "Integration coverage for the canonical body-AST entry path — historically `:wat::kernel::run-sandboxed-ast`, now exercised through `:wat::test::run-hermetic` / `:wat::test::run-thread` per arc 170 slice 4c-α-ii."
3. `wat_run_sandboxed.rs` — line 1 now reads "End-to-end tests for canonical hermetic body-AST entry — historically `:wat::kernel::run-sandboxed` (arc 007 slice 2a), now exercised through `:wat::test::run-hermetic` per arc 170 slice 4c-α-ii."

## File-rename deferral

Per BRIEF: `wat_run_sandboxed.rs` and `wat_run_sandboxed_ast.rs` retain their legacy names. Rename to descriptive canonical names (e.g., `wat_run_hermetic_body_string.rs`, `wat_run_thread_body_ast.rs`) deferred to post-109 cleanup per accumulate-tests-rearchitect-not-delete policy.

## Layer 2 escalations

ZERO. The BRIEF predicted 0–1; outcome 0. No site needed `:wat::test::run-hermetic-with-io` (no `readln` driving from outer stdin).

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 30–60 min | ~60 min (prior sub-Agent ~30 min + this run ~30 min on `wat_run_sandboxed.rs` 7 sites + 6 substrate-finding fixes + SCORE) |
| Scorecard rows | 6/6 PASS | 6/6 PASS |
| Workspace fail count | ≤ 11 | 2 (well within band; below baseline of 3 pre-slice) |
| Thread destinations | 11–13 | **2** (significantly below — most legacy sites had stdio rules firing, pushing them to hermetic) |
| Hermetic destinations | 3–5 | **14** (significantly above — same root cause) |
| Layer 2 escalations | 0–1 | 0 |
| Doc-comment files refreshed | 3 | 3 |
| Mode | A (clean) | **B (substrate finding surfaced + folded in mid-sweep)** — three load-bearing findings (capacity-mode body-context, raise! HolonAST signature, scope/ScopedLoader functional plumbing) corrected BRIEF posture without exceeding STOP triggers |

**Distribution calibration note:** the thread-vs-hermetic prediction (11–13 thread / 3–5 hermetic) inverted. The actual distribution is 2 thread / 14 hermetic. Root cause — the prediction underweighted how many legacy bodies wrote to stdio AND had outer assertions on `RunResult/stdout` / `RunResult/stderr`. Both stdio rules fired far more often than predicted (per-site three-rule analysis confirmed hermetic for each). Recalibrate future predictions: legacy `run-sandboxed*` callers default to "writes stdio + outer reads stdio slot" because that's why they USED the sandboxed verb in the first place — to capture inner stdio.

## Doctrine compliance

- No `src/`, `wat/`, `wat-tests/`, `crates/`, `examples/` edits.
- No INSCRIPTION / past SCORE / DEFERRAL / SUPERSEDED / AUDIT / recovery doc / INTERSTITIAL / BRIEF / EXPECTATIONS edits.
- No test deletions, no file renames.
- All file operations via absolute paths under `/home/watmin/work/holon/wat-rs/`. Worktree at `.claude/worktrees/agent-<id>/` ignored per FM 7-bis.
- No commit (orchestrator commits atomically).
- Failure-engineering: substrate findings 1–6 surfaced as honest deltas with concrete grep / type-checker evidence; no "future fix is open" deferrals, no "investigate later" hedges. Findings 3 + 6 explicitly identify the load-bearing capability lost in migration and the future-work shape that recovers it.

## Next stone

After this slice: zero Rust-side callers of `:wat::kernel::run-sandboxed*`. Next stone (#321 4c-α-iii) audits the 2 `check.rs` embedded wat fixtures per the BRIEF predecessor sequence.

Two surfaces newly UNCOVERED by this slice (substrate-gap audit list, NOT this slice's responsibility to fix):

1. `ScopedLoader` containment behavior under sandbox-style entry (Finding 3).
2. Inner-source lexer/parse-error capture (Finding 4) + missing-`:user::main` startup capture (Finding 5) + raw-Rust-panic-via-Bundle capture (Finding 6) — three substrate-verb-only surfaces that the legacy `:wat::kernel::run-sandboxed` retains until #310. After #310 lands, these surfaces become unreachable; canonical-macro coverage gaps need closing first.
