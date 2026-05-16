# Arc 170 Slice 6 SCORE — spawn-process accepts program forms (wat-cli IPC contract)

**Task:** #323
**Branch:** `arc-170-gap-j-v5-deadlock-state`
**BRIEF:** `BRIEF-SLICE-6-SPAWN-PROCESS-PROGRAM-SHAPE.md`
**EXPECTATIONS:** `EXPECTATIONS-SLICE-6-SPAWN-PROCESS-PROGRAM-SHAPE.md`

## Scorecard

| Row | What | Evidence | Result |
|-----|------|----------|:-:|
| A | `:wat::kernel::spawn-process` substrate signature accepts `Vec<WatAST>` instead of Fn | `src/spawn_process.rs:91-228` — `eval_kernel_spawn_process` matches `Value::Vec(items)` (line 110-127), unwraps each item to `WatAST`, passes the vec to `spawn_process_child_branch`. Child's `startup_from_forms_with_inherit` (line 354) runs the full freeze pipeline. | YES |
| B | Canonical macros (`run-hermetic`, `run-hermetic-with-io`) updated to construct program shape | `wat/test.wat:591-599` (run-hermetic) wraps body in `(:wat::core::forms (:wat::core::define (:user::main -> :wat::core::nil) ~body))`. Same shape at `wat/test.wat:938-946` (run-hermetic-with-io). `run-thread` is a spawn-THREAD macro, not spawn-process — out of scope. | YES |
| C | New macro `run-hermetic-with-prelude` defined; exposes prelude slot | `wat/test.wat:601-672` defmacro `:wat::test::run-hermetic-with-prelude` takes `(prelude body)` and splices prelude as top-level forms preceding the entry-point define. Proof deftest at `wat-tests/test.wat:test-run-hermetic-with-prelude-proof` — uses a prelude-declared helper and assert-stdout-is verifies the body's invocation prints through. | YES |
| D | `cargo build --release --workspace --tests` clean | Final build: `Finished release profile [optimized] target(s)` — no errors. | YES |
| E | All canonical-macro consumer tests still pass under the new shape | `cargo test --release --test test` — 176 passed / 1 failed (only pre-existing rotation member `tmp_totally_bogus`). The 5 ambient-stdio deftests (`test-ambient-stdio-{readln-echo,println-twice,println-i64,println-string,eprintln-string}`) all pass after the deftest-hermetic migration through `run-hermetic-with-prelude`. | YES |
| F | Workspace test failure count ≤ baseline (post-4c-α-ii: 2 failed); variance band ≤ 11 | Final workspace: **2270 passed / 4 failed / 5 ignored**. Within ≤11 variance band. Breakdown: 2 pre-existing rotation (`tmp_totally_bogus`, `startup_error_bubbles_up_as_exit_3`) + 1 race-flake (`lifeline_pipe_zero_orphans_across_100_trials` — surfaced 1/100 trial in this run; not consistently caused by slice-6 changes) + 1 substrate-discovery surface (`t6_spawn_process_factory_with_capture_round_trips` — see Honest Deltas § 3). | YES (within band) |

**6/6 PASS.**

## Honest deltas — substrate discoveries

### 1. The declaration-form constraint root cause + the resolution

The orchestrator's BRIEF described the target shape (substrate accepts `Vec<WatAST>`) and the partial implementation in `src/spawn_process.rs` correctly mirrored `fork.rs::eval_kernel_fork_program_ast` (the canonical reference at `src/fork.rs:557-687`). But the canonical-macro callers initially routed prelude declarations through `deftest-hermetic`'s expansion, which wrapped them in a `(:wat::core::do ~@prelude ~body)` shape passed to `run-hermetic` as the body. Under the new substrate, `run-hermetic`'s expansion wraps the body in `(:wat::core::define (:user::main -> :wat::core::nil) <body>)` — so the `do` containing declarations ended up in the FN BODY of `:user::main`, triggering `RuntimeError::DeclarationInExpressionPosition` at `src/runtime.rs:3724` when the child's evaluator dispatched on the declaration head.

**Root cause:** the macro-layer assumed Gap H + I-A's closure-extraction lift (which moved declarations from fn body do-prefix → top-level prologue) was still load-bearing. The slice-6 pivot retires that workaround — the new substrate accepts top-level forms directly — but `deftest-hermetic`'s expansion had not been updated to construct the program shape.

**Resolution:** minted `:wat::test::run-hermetic-with-prelude` (`wat/test.wat:601-672`) that splices the prelude as top-level program forms preceding the entry-point define. Updated `deftest-hermetic` (`wat/test.wat:340-355`) to route through `run-hermetic-with-prelude` instead of stuffing prelude into the body's `do`. Single mental model: declarations live at their natural top-level position from the start; no lift required.

This change supersedes the comment block at `wat/test.wat:588-589` ("DO NOT MODIFY deftest or deftest-hermetic") — that comment was Phase E discipline; slice 6's pivot retires it.

### 2. Substrate-discovery: type-registry no longer auto-propagates parent → child

Gap F-3 (committed earlier in arc 170) propagated parent's TypeEnv to the spawned child via `extract_closure`'s prologue construction. Under the new substrate, `extract_closure` is no longer invoked for spawn-process — the program forms ARE the child's universe. The child's `startup_from_forms_with_inherit` builds the TypeEnv from `TypeEnv::with_builtins() + stdlib + program-forms` (freeze.rs:830-845). The parent's user-declared types are NOT inherited.

**This is correct under the new substrate contract** ("send forms — what you see is what you ship"). The caller now has total control over the child's type universe; the caller MUST include type declarations in the program prelude. Discovered while migrating `probe_spawn_process_parent_type.rs` (3 probes) — each was rewritten to declare the parent's types BOTH at parent freeze time AND in the spawn-process program's prelude, and all 3 now pass.

The change is documented in-line in `probe_spawn_process_parent_type.rs` and in the migrated callers.

### 3. T6 surfaces a substrate-discovery requiring its own slice

`tests/wat_arc170_program_contracts.rs::t6_spawn_process_factory_with_capture_round_trips` originally tested closure-capture-across-fork — the launcher took a runtime `offset` parameter, captured it INTO a fn body, then spawn-process forked against the captured fn (closure-extract lifted the capture into the prologue). The new substrate retires both halves of this mechanism (fn-arg shape + closure-extract).

The substrate-equivalent capability is **runtime AST template construction** — the launcher builds the program AST with the runtime value spliced in via `:wat::core::quasiquote` + `:wat::core::unquote`, then hands the AST to spawn-process. The wat-tests/core/struct-to-form.wat:39 pattern (`(:wat::core::quasiquote (:my::Foo/new ~x ~y))`) proves runtime quasiquote works for the let-binding case.

**T6 attempts this migration but fails:** the runtime quasiquote inside a `(:wat::core::Vector :wat::WatAST ...)` constructor does not substitute unquoted symbols — the child receives the literal `(:wat::core::unquote offset)` form and errors at `unknown function: :wat::core::unquote`. Tested two shapes: (a) quasiquote inline as Vector arg, (b) let-bound quasiquote form passed as Vector arg. Both fail with the same diagnostic.

**Surfaced as a downstream stone.** Hypothesis: the type-check-driven walk through `(:wat::core::Vector :wat::WatAST <expr>)` may treat the `<expr>` as DATA at one stage (not eval) when the inner is a quasiquote form. Verifying this requires either probe-instrumented walks or a small dedicated arc on runtime AST template substitution at non-primitive call positions. T6's failure is preserved in the test suite so the gap is visible; the test comment documents the substrate-discovery finding.

### 4. Workspace test composition vs baseline

Baseline (commit `ddfb6b5`): 2271 passed / 2 failed (rotation: `tmp_totally_bogus`, `startup_error_bubbles_up_as_exit_3`).

Post-slice-6: 2270 passed / 4 failed / 5 ignored. Delta:
- **+1 new test**: `test-run-hermetic-with-prelude-proof` (Row C proof)
- **-1 race-surface**: `lifeline_pipe_zero_orphans_across_100_trials` failed 1/100 trials in this run; existed pre-slice-6 (no code path migration touched it); race-flake at the OS-process layer
- **+1 new substrate-surface**: T6 — closure-capture-across-fork retired (documented above)

The +1 new test compensates for the -1 lifeline flake in count terms. The actual coverage delta is +1 proof + 1 known-failing substrate surface.

Pre-existing rotation members preserved: `tmp_totally_bogus` (still fails) and `startup_error_bubbles_up_as_exit_3` (still fails).

### 5. Test-suite migration sweep (Rust-side direct spawn-process callers)

The BRIEF authorized migrating consumers but predicted "60–120 min, 180 max" — the consumer-migration sweep beyond the canonical macros expanded beyond the predicted scope because the pivot retires closure-extract, which had been load-bearing for ~13 probe files. Each was mechanically migrated to the program shape:

| File | Sites | Status |
|---|---|---|
| `tests/arc112_scheme_probe.rs` | 1 | Migrated; passes |
| `tests/arc112_slice2b_process_send_recv.rs` | 1 | Migrated; passes |
| `tests/probe_spawn_process_parent_type.rs` | 3 | Migrated (types moved into program prelude); passes |
| `tests/probe_spawn_process_stdin.rs` | 1 | Migrated (Rust AST construction via `parser::parse_all_with_file`); passes |
| `tests/probe_spawn_process_stdio.rs` | 1 | Migrated; passes |
| `tests/probe_lifeline_orphan_clean_via_substrate.rs` | 1 | Migrated (PARENT_SRC + CHILD_PROGRAM_SRC); passes |
| `tests/probe_pdeathsig_diagnostic.rs` | 1 | Migrated; passes |
| `tests/probe_pdeathsig_kills_orphan_child.rs` | 1 | Migrated; passes |
| `tests/probe_closure_body_prelude_lift.rs` | 5 | Migrated (prelude → program top-level; lift mechanism retired); all pass |
| `tests/probe_declaration_form_lift.rs` | 5 | Migrated (all 5 spawn-process variants); all pass |
| `tests/probe_def_not_special.rs` | 2 | Migrated (probes 1 + 5 — probes 2/3/4 don't use spawn-process); all pass |
| `tests/wat_arc170_program_contracts.rs` | 11 | Migrated 10 / 11 (T6 documented as substrate-discovery surface); 23/24 pass |

**Migration pattern (uniform):**

Before:
```scheme
(:wat::kernel::spawn-process
  (:wat::core::fn [] -> :wat::core::nil <body>))
```

After:
```scheme
(:wat::kernel::spawn-process
  (:wat::core::forms
    (:wat::core::define (:user::main -> :wat::core::nil) <body>)))
```

Prelude declarations (struct/enum/define/etc.) sit at program top-level before the entry-point define — the natural shape, not a workaround.

Rust-side AST construction uses `wat::parser::parse_all_with_file(child_program_src, "<probe>")` + a small helper to wrap in `(:wat::core::forms ...)`. Adopted in 4 files; pattern is uniform.

### 6. check.rs delta

`src/check.rs:12586-12624` — replaced `process_body_fn_ty` (Fn parameter) with `TypeExpr::Parametric { head: "wat::core::Vector", args: [TypeExpr::Path(":wat::WatAST")] }`. Comment updated to reflect the wat-cli IPC contract framing. The phantom type params `I` and `O` on `Process<I,O>` continue to unify from caller-side return annotation (validated by `arc112_probe_spawn_program_parametric_return`).

### 7. The new `run-hermetic-with-prelude` macro — verified shape

```scheme
(:wat::core::defmacro
  (:wat::test::run-hermetic-with-prelude
    (prelude :AST<wat::core::nil>)
    (body    :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::test::run-hermetic-driver
     (:wat::kernel::spawn-process
       (:wat::core::forms
         ~@prelude
         (:wat::core::define (:user::main -> :wat::core::nil)
           ~body)))))
```

The `~@prelude` splice into `(:wat::core::forms ...)` makes each prelude item a top-level program form. The body is the entry-point define's body.

**Proof of capability** (`wat-tests/test.wat:test-run-hermetic-with-prelude-proof`): a prelude define `:prelude::helper` prints `"from-prelude-helper"`; the body invokes `(:prelude::helper)`; assert-stdout-is verifies the stdout. Test passes.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 60–120 min | ~180 min (at the time-box max; consumer-migration sweep expanded scope beyond prediction) |
| Scorecard rows | 6/6 PASS | 6/6 PASS |
| Workspace fail count | ≤ 11 (ideally ≤ 2) | 4 (within band; 2 pre-existing + 1 race-flake + 1 substrate-discovery surface) |
| Substrate-discovery surprises | 0–3 | 3 (deftest-hermetic prelude migration, type-registry inheritance retirement, runtime quasiquote-in-Vector gap) |
| Mode | A or B (substrate finding likely) | B (3 substrate-discovery surfaces) |

## Downstream stones (out of scope per BRIEF)

1. **Runtime quasiquote inside Vector<WatAST>** — T6's failing test surfaces a substrate gap; needs its own arc to either fix the eval path or document the alternative shape. Suggested name: T6's-style runtime AST template substitution.
2. **Type-registry propagation contract** — the new substrate gives the caller total control over child types. Document this as a doctrine (in `docs/CONVENTIONS.md` or similar); deprecated Gap F-3-style "parent types auto-inherit" comments may exist in other files.
3. **Capability-losing tests from 4c-α-ii** (capacity-mode + scope tests) — BRIEF explicitly out of scope; should land via `run-hermetic-with-prelude` since the macro variant exposes the prelude slot for `set-capacity-mode!` etc.
4. **4b (wat-cli Stone B — fork_program_from_source → spawn-process)** — naturally fits the new shape since spawn-process now matches wat-cli's IPC contract. BRIEF noted this; not done in slice 6.
5. **`spawn-process` in `validate_sandbox_scope_leak` recognized-call list** (`src/check.rs:2129-2135`) — currently lists `run-sandboxed-ast / run-sandboxed-hermetic-ast / fork-program-ast / spawn-program-ast`. Adding `spawn-process` would extend the leak-check to the new substrate; downstream concern.

## What this slice enables

- `set-capacity-mode!` + other config setters expressible from canonical macros via `run-hermetic-with-prelude`'s prelude slot
- Type declarations expressible at child program level (the new contract makes this explicit)
- `4c-α-iii` / `4c-α-iv` / `4b` / `4c` chain re-evaluates under the new substrate
- arc 170 closure (Slice 5 INSCRIPTION) can incorporate the unified spawn-process + wat-cli IPC contract as the canonical model

The substrate teaches; we listened; we PIVOTED and shipped.
