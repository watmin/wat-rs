# Arc 170 Slice 6 BRIEF — spawn-process accepts program forms (wat-cli IPC contract)

**Task:** #323
**Phase:** Slice 6 (pivot — substrate redesign; supersedes part of 4c-α / 4b / 4c chain framing).
**Predecessors:** All of 4a chain + 4c-α-i + 4c-α-ii landed. The pivot rationale is fully captured in `INTERSTITIAL-REALIZATIONS.md` § 2026-05-15 "Substrate pivot: spawn-process accepts program forms" — read that first.

## Goal

Change `:wat::kernel::spawn-process` to accept a **wat program** (`Vec<WatAST>`) instead of a fn value. The new shape mirrors `wat some-file.wat` semantically:

- Program is a sequence of top-level forms (config setters, type declarations, helper defines, ending in `(define :user::main ...)`)
- `Config::from_source` collects top-level config at child parse time (set-capacity-mode!, set-dim-router!, set-redef!, etc.)
- `:user::main` runs at child runtime
- IPC contract: stdin = inputs; stdout = outputs; stderr = panics

This unifies spawn-process semantically with the wat-cli — same operation, different access paths. Restores the capability that arc 170 slice 1c's "fn-only" narrowing lost.

## Decay disclosure (orchestrator → sonnet)

The orchestrator has had multiple substrate-fact failures this session (documented in INTERSTITIAL § 2026-05-15). This BRIEF describes the TARGET SHAPE and the WAT-CLI CONTRACT model. **You (sonnet) have full authority on substrate-internal discovery** — which functions to modify in `src/spawn_process.rs`, how the child receives the program, how existing fn-shape callers update. Do NOT trust orchestrator claims about substrate internals; read the code.

## Target shape

```scheme
;; Before (current substrate)
(:wat::kernel::spawn-process fn) -> :wat::kernel::Process<I,O>
;; where fn is a Fn value (keyword path or expression evaluating to Fn).
;; Substrate implicitly wraps it as :user::main in the child.

;; After (this slice)
(:wat::kernel::spawn-process program) -> :wat::kernel::Process<I,O>
;; where program is :wat::core::Vector<wat::WatAST> — the top-level
;; forms of a wat program. The caller is responsible for including
;; a (:wat::core::define (:user::main -> :nil) ...) define somewhere
;; in the forms (typically last). The substrate ships the forms to
;; the child; the child parses them the same way wat-cli would when
;; reading a .wat file from disk.
```

## Migration impact (caller-side)

Every current caller of `:wat::kernel::spawn-process` updates. The migration pattern is uniform:

```scheme
;; Before (current macro expansion shape):
(:wat::kernel::spawn-process
  (:wat::core::fn [] -> :wat::core::nil <body>))

;; After:
(:wat::kernel::spawn-process
  (:wat::core::Vector :wat::WatAST
    '(:wat::core::define (:user::main -> :wat::core::nil)
       (:wat::core::fn [] -> :wat::core::nil <body>))))
;; Or equivalently — fn-form is allowed at runtime; pull the body out:
(:wat::kernel::spawn-process
  (:wat::core::Vector :wat::WatAST
    '(:wat::core::define (:user::main -> :wat::core::nil) <body>)))
```

(Sonnet — discover which form the substrate actually accepts. The substrate may need :user::main to BE a fn binding, OR may accept the body directly. The current fn-wrapping behavior tells you what the substrate expects under the hood.)

**Known callers to update** (sonnet — verify via grep before editing):

- `wat/test.wat:574-583` — `:wat::test::run-hermetic` macro
- `wat/test.wat:688-706` — `:wat::test::run-thread` macro (also expands to spawn-thread; only update the spawn-process branch if relevant)
- `wat/test.wat:916+` — `:wat::test::run-hermetic-with-io` macro
- Any other consumer found by `grep -rn ":wat::kernel::spawn-process" wat/ wat-tests/ tests/ crates/`

Macros stay user-facing-unchanged — the program-construction is internal to the macro expansion. 99% of users pass body forms to the canonical macros; the macros construct the spawn-process program shape internally.

## NEW macro for the prelude slot

Mint a new macro that exposes the prelude:

```scheme
(:wat::test::run-hermetic-with-config
  (CONFIG-FORMS)    ;; e.g., ((:wat::config::set-capacity-mode! :panic))
  BODY)             ;; the user's body
```

Expands to:

```scheme
(:wat::test::run-hermetic-with-config-driver
  (:wat::kernel::spawn-process
    (:wat::core::Vector :wat::WatAST
      ~@CONFIG-FORMS                                    ;; spliced as top-level
      '(:wat::core::define (:user::main -> :wat::core::nil) ~BODY))))
```

The driver mirrors `run-hermetic-driver` (no I/O channels; just drain stdio + join). The CONFIG-FORMS go at the top of the child program where `Config::from_source` can collect them at parse time.

Naming alternative: `run-hermetic-with-prelude` if "config" is too narrow. Sonnet — pick the name that best names what the slot holds. "config" suggests set-! family; "prelude" is more general (could hold any top-level forms).

## Out-of-scope for this slice

- DO NOT migrate the capability-losing tests from 4c-α-ii to the new variant (capacity-mode tests + scope tests) — that's a downstream stone (potentially Slice 6-γ if decomposition is needed).
- DO NOT touch the 4c-α-iii (check.rs fixtures) or 4c-α-iv (atomic delete) chain — those re-evaluate after this slice.
- DO NOT modify `wat-cli` or substrate Rust beyond the spawn-process change itself.
- DO NOT delete `wat/kernel/sandbox.wat` or `wat/kernel/hermetic.wat` — they may become redundant under the new substrate, but their retirement is a separate concern.

## Substrate edits IN scope

- `src/spawn_process.rs` — change `eval_kernel_spawn_process` signature/behavior to accept Vec<WatAST> instead of Fn. Update child-program-construction to ship the forms.
- Possibly `src/check.rs` — type-check arms for the new spawn-process shape (sonnet — verify; the existing arms expect a Fn; new arm expects Vec<WatAST>).
- Possibly `wat/kernel/channel.wat` or similar — if any type aliases need updates.

## Wat-side edits IN scope

- `wat/test.wat` — update `run-hermetic`, `run-thread`, `run-hermetic-with-io` macro expansions to construct program shape; mint `run-hermetic-with-config`
- Possibly `wat/test.wat`'s drivers (`run-hermetic-driver` etc.) — verify they still work under new shape

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- Operate ONLY in `/home/watmin/work/holon/wat-rs/` per FM 7-bis (worktree doctrine).
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / INTERSTITIAL / this BRIEF / EXPECTATIONS / past SCORE-SLICE-* docs.
- DO NOT use any path containing `.claude/worktrees/`.

## Scorecard (6 rows, YES/NO with evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | `:wat::kernel::spawn-process` substrate signature accepts `Vec<WatAST>` instead of Fn | `grep -nA 30 "fn eval_kernel_spawn_process" src/spawn_process.rs` shows the new signature parsing Vec<WatAST> |
| B | Canonical macros (`run-hermetic`, `run-thread`, `run-hermetic-with-io`) updated to construct program shape | grep wat/test.wat shows macro bodies use the new substrate shape |
| C | New macro `run-hermetic-with-config` (or `-with-prelude`) defined; exposes prelude slot | grep wat/test.wat shows the new defmacro |
| D | `cargo build --release --workspace --tests` clean | build output Finished |
| E | All canonical-macro consumer tests still pass under the new shape | targeted cargo test on the 5 hermetic-decorated deftests + Layer 2 readln-echo + wat_run_sandboxed.rs all green |
| F | Workspace test failure count ≤ baseline (post-4c-α-ii: 2 failed) | full workspace cargo test or targeted spot-checks per BRIEF's verification protocol |

## STOP triggers

- The new spawn-process signature doesn't fit cleanly into the existing substrate child-program-construction path → STOP and surface; sonnet's substrate discovery may reveal the redesign needs decomposition into sub-slices.
- A canonical macro's expansion can't reasonably construct the program shape (e.g., quasiquote splicing in macros has a substrate constraint) → STOP and surface; may need different macro construction approach.
- Build fails after the substrate change but before macros update → expected mid-sweep; complete the macro updates and re-test.
- Build STILL fails after macros update → STOP; surface; root-cause the broken path.
- > 5 unexpected substrate-finding surfaces → STOP; this slice's scope may need decomposition.

## Implementation protocol

Per `feedback_iterative_complexity` + `feedback_test_first`:

1. **Substrate first** — verify cwd; read `src/spawn_process.rs` to understand current shape; sketch the new shape; modify spawn-process. Build (will likely fail at the canonical-macro callers).
2. **Macros next** — update `run-hermetic`, `run-thread`, `run-hermetic-with-io` to construct program shape. Build (should pass).
3. **Targeted test** — run the canonical-macro-using deftests:
   - `wat-tests/run-thread.wat` (Ok-path + Err-path)
   - `wat-tests/test.wat` (5 hermetic-decorated deftests)
   - `wat-tests/kernel/services/ambient-stdio.wat` (Layer 2 readln-echo + the 4 println tests)
   - `wat-tests/core/option-expect.wat` / `result-expect.wat` / `struct-to-form.wat`
4. **Mint `run-hermetic-with-config`** — add the new macro variant. Add a simple deftest that uses it (proof of capability).
5. **Workspace verification** — full `cargo build --release --workspace --tests` + `cargo test --release --workspace --no-fail-fast`.
6. **Write SCORE.**

## Time-box

This is a substrate redesign with cascading macro updates. Predicted 60-120 min. Time-box 180 min hard stop. If approaching the stop, write a partial SCORE describing state-at-stop.

## On completion

Write `SCORE-SLICE-6-SPAWN-PROCESS-PROGRAM-SHAPE.md`. 6 rows YES/NO. Honest deltas — especially:

- Substrate-discovery findings (which functions changed; what the child-program-construction path actually looks like; any decay in orchestrator's hypothetical shape that didn't match reality)
- Macro construction patterns that worked / didn't work cleanly
- The new `run-hermetic-with-config` macro's verified shape + the proof-of-capability deftest
- Workspace test count + composition vs baseline (2 failed)
- Calibration record

## What this slice enables

After Slice 6 ships:
- `set-capacity-mode!` + other config setters become expressible from canonical macros (via the new variant)
- ScopedLoader containment recoverable via the same prelude slot (if loader config is at top-level)
- 4c-α-iii / 4c-α-iv / 4b / 4c chain re-evaluates under the new substrate; cleanup completes more cleanly
- arc 170 closure (Slice 5 INSCRIPTION) incorporates the unified spawn-process + wat-cli IPC contract as the canonical model

The substrate teaches; we listen; we ship.
