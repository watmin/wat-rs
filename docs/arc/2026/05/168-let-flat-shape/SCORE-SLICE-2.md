# Arc 168 slice 2 — SCORE

Sonnet swept all legacy nested-pair-list let-binding callsites
across `wat/`, `wat-tests/`, `tests/wat_*.rs`, and `crates/*/`.
~133 minutes runtime, 1107 tool uses, 21 batch commits. Workspace
landed at 2074 passed / 8 failed; closure investigation surfaced 3
gaps slice 1 left, all fixed in commit `b220846` as slice 2
follow-ups. Final state: 2077 passed / 5 failed (the 5 are
pre-existing kernel/spawn/signal failures unrelated to arc 168).

## Scope as shipped

21 batch commits sweeping the migration recipe `((name expr) ...)`
→ `[name expr ...]` across:

- `wat/*.wat` — stdlib (grep-driven; walker doesn't fire on stdlib
  per slice 1 delta A; no failures surface there)
- `wat-tests/**/*.wat` — user-source test fixtures
- `tests/wat_*.rs` — embedded wat strings inside `r#"..."#`
- `crates/wat-lru/wat/`, `crates/wat-lru/wat-tests/`
- `crates/wat-holon-lru/wat/`
- `crates/wat-telemetry/wat/`, `crates/wat-telemetry/wat-tests/`
- `crates/wat-telemetry-sqlite/wat/`, `crates/wat-telemetry-sqlite/wat-tests/`
- `examples/with-loader/`, `examples/with-lru/`

Three binder shapes covered:
- Bare symbol (canonical): `((name expr))` → `[name expr]`
- Typed legacy: `(((name :T) expr))` → `[name expr]` (type drops;
  inferred per arc 159's user-side retirement)
- Destructure: `(((a b c) rhs))` → `[[a b c] rhs]` (binder Vector
  inside outer binding Vector)

## Scorecard

| Row | Verified by | Pass |
|-----|-------------|------|
| A — Workspace failures dropped to 0 (arc-168 territory) | post-follow-up: 0 arc-168 walker firings; 5 remaining are pre-existing kernel/signal | ✓ |
| B — wat/ stdlib swept | grep `'\(:wat::core::let\b.*\(\('` wat/: 0 hits | ✓ |
| C — wat-tests/ user-source swept | walker fires nowhere; cargo test green for those bins | ✓ |
| D — tests/wat_*.rs embedded strings swept | cargo test green for those test files (post-follow-up) | ✓ |
| E — crates/*/wat-tests/ swept | parallel grep clean | ✓ |
| F — crates/*/wat/ swept | parallel grep clean | ✓ |
| G — Slice 1 substrate untouched | initial sonnet sweep was test-fixture-only; the slice 2 FOLLOW-UP touches src/ deliberately to fix gaps slice 1 left | ✓ partial (see Honest delta A) |
| H — Walker untouched | `walk_for_legacy_let_bindings` unchanged | ✓ |
| I — Canonical parser untouched | `parse_let_binding` (Vector + Symbol path) unchanged | ✓ |
| J — Tests/assertions untouched | NO test logic edits, NO assertion edits — translation-only in initial sweep | ✓ |
| K — Mechanical translation only | each migration is binder-shape change + body unchanged; no semantic reshapes | ✓ |
| L — Slice branch on remote | branch carries 21 batch commits + 1 follow-up commit; main untouched | ✓ |
| M — Single-body remains valid (regression) | tests using single-form let body still pass after their bindings migrate | ✓ |
| N — Sonnet ran the inline pipeline cleanly | report references `cargo test --release --workspace --no-fail-fast 2>&1 \| grep "^test result" \| awk '...'`; one Console.wat extra-paren bug found + fixed autonomously by sonnet | ✓ |

## Honest deltas

### Delta A — Slice 1 left 3 gaps; slice 2 closure surfaced them

After sonnet's 21-batch sweep landed 2074/8, investigation of the
8 failures (mandated by user direction *"failures here are not
expected... we don't leave the code busted when we find it"*)
revealed:

1. **`empty_bindings_evaluates_body_directly`** (arc 154 test):
   used legacy outer-list `()` empty bindings; walker fires
   correctly. Test fixture sweep miss; slice 2 didn't migrate the
   degenerate `()` empty form because grep targeted `((` patterns.

2. **`def_runtime_let_splice_closure_capture`** (arc 157 test):
   `def` inside a top-level `let` body lost its splice-into-global
   behavior. Failed with `UnknownFunction` at runtime. Slice 1
   updated `eval_let` (call-time path) to consume Vector outer +
   multi-form body but missed the freeze-time companion
   `register_runtime_defs_form` in src/runtime.rs which:
   - Only handled legacy List outer bindings (the form arc 168
     retires)
   - Only looked at `items[2]` for body (single-form assumption)
   Result: defs nested inside arc-168-shape let bodies never
   registered into `runtime_def_values`.

3. **`defn_inside_top_level_let_body_works`** (arc 166 test):
   same root cause as #2 via `defn`'s macro expansion to `def`.

Fix shipped in commit `b220846`:
- `register_runtime_defs_form`: rewrote let arm Vector-only
  (Symbol binder), iterates `items[2..]` for multi-form body
- `collect_splice_defs_ctx` (check.rs): same multi-body iteration
  fix
- `tests/wat_arc154_kill_let_star.rs`: empty-bindings test
  migrated to flat shape

Discipline lesson recorded in commit message: first draft kept
legacy List outer + legacy typed-single binder support "to make
the fix work." User caught the FM 5 pattern — accepting forms
the substrate is retiring creates walker-vs-registrar
disagreement. Reverted to Vector-only.

**Implication for slice 1 closure honesty**: slice 1's SCORE
should reference this delta when written; the `eval_let` rewrite
was not actually complete because `register_runtime_defs_form`
and `collect_splice_defs_ctx` are companion paths slice 1 missed.
Closing arc 168 honestly requires acknowledging slice 1 didn't
finish the let consumer story; slice 2 follow-up did.

### Delta B — Console.wat extra-paren bug found + fixed autonomously by sonnet

Sonnet's batch 21 included a bugfix to `Console.wat`: two tests
(`test-dispatcher-edn`, `test-dispatcher-json`) had an extra `)`
left over from the old `((r ...)` binding pair format. After
migrating the pair to `[r ...]`, the binding's closing `)` was
not removed, causing a parse error at startup.

This is exactly the kind of substrate-as-teacher discovery the
sweep was supposed to surface. Sonnet caught + fixed without
intervention. Honest delta in the right direction.

### Delta C — Five pre-existing failures unrelated to arc 168

Post-follow-up the workspace shows 5 remaining failures:
- `fork_program_round_trip_via_pipes` (arc 104 fork machinery)
- `sigterm_cascades_two_levels_via_process_group` (signal cascade)
- `sigterm_to_cli_cascades_via_polling_contract` (signal cascade)
- `presence_proof_hello_world` (substrate proof harness)
- `programs_are_atoms_hello_world` (substrate proof harness)

All five predate arc 168 (verified pre-edit by stash round-trip
during sonnet's sweep). Their domain is kernel/spawn/signal/
proof-harness territory, not let bindings. Per user direction
*"failures here are acceptable - let's continue the arc"* they
are out of arc 168's scope. Sibling arc 168a (or 169 — orchestrator
picks) opens immediately after arc 168 closes to investigate.

The `feedback_no_known_defect_left_unfixed.md` discipline applies
to scope-relevant defects. These are scope-unrelated, surfaced
during arc 168 closure as pre-existing residue. Tracking them
in their own arc preserves arc 168's narrative integrity (let
flat-shape) while honoring the no-defect-left-unfixed principle
in the next arc.

### Delta D — Sonnet sweep cost (calibration)

| Predicted | Actual | Mode |
|-----------|--------|------|
| 60-120 min sonnet | ~133 min | A clean (within 2× cap; over upper-bound by 13 min) |

Notable: 1107 tool uses for ~563 sites = ~2 tool calls per site.
The Edit-tool-per-fixture approach was thorough but slow. A
python-script approach would have been ~10× faster but was
declined mid-flight (user direction: *"let it roll - we'll
change directions if we need to"*).

For future single-pass sweeps of similar shape, python-script
substrate-edit shipped as a `./scripts/` helper (with the
`Bash(scripts/**)` allowlist) is worth the upfront cost. Filed
as a calibration note for arc-168-closure paperwork; not a
discipline failure on this slice.

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| Slice 2 sweep ~563 sites, 60-120 min sonnet | 21 batches, ~133 min, 2074/8 then 2077/5 post-follow-up | A clean |

Sonnet behavior with the inline pipeline (the load-bearing
calibration target): **clean throughout**. No awk-pipe denials.
No off-task hallucinations. The post-restart `.claude/settings.json`
discipline held. The bare-script discipline (no embellishment)
held except in 2 probes earlier in the session — those were
caught by user inspection and corrected via memory note
`feedback_script_invocation_no_embellishment.md`.

## Discipline check

- ✓ FM 5 caught + reverted within minutes (legacy support added
  back to make fix work; user noticed; reverted to Vector-only)
- ✓ FM 11 grep clean — slice 2's residual gaps surfaced and
  shipped before claiming closure
- ✓ FM 16 honored — BRIEF didn't preempt tool availability
- ✓ Substrate-as-teacher cycle complete — 5 substrate
  diagnostics surfaced + fixed during slice 2 closure
- ✓ Branch isolation held — main untouched throughout

## What's next

Slice 3 — substrate retirement. DELETE:
- `BareLegacyLetBindings` walker variant + Display + Diagnostic
  + walker body + freeze.rs registration
- `parse_let_binding` legacy fall-through arms (typed-single
  `((name :T) rhs)` shape)
- The `WatAST::List`-outer arm in `eval_let` (now redundant; all
  user code uses Vector outer post-slice-2)
- `infer_let` / `parse_let_binding_for_check` mirror retirements

Predicted: 30-60 min opus. After slice 3, substrate has zero
trace of legacy outer-list let support — `(let ((...) ...))` in
user source produces a clean `MalformedForm` from the Vector-only
parser path; the walker is gone.

Slice 4 (lib unit-test fixture sweep, mirror of arc 167 slice 4b)
follows when slice 3 ships green.
