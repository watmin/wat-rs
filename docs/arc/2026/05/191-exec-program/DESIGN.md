# Arc 191 — `:wat::kernel::exec-program` (in-place universe replacement)

**Status:** stub opened 2026-05-13 per user direction. May be the first thing the user plays with after arc 170 resolves.
**Gates on:** arc 170 closure (services + closure-extraction + typed channels are the foundation this builds on).

## Motivation

**User direction 2026-05-13:**
> *"we have something shockingly close to an exec... can we do an exec... think of being in a repl... can we 'exec into' a new shell while not dropping the user?"*

The conversation that produced this stub lives in `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` — appended-entry framing "Gap I and the list of special things question" is the immediate parent; the exec discussion follows. The architectural recognition: **wat trades intra-program dynamism for cross-program composition with static guarantees per program.** spawn-process = fork+exec; this arc mints bare `exec` — replace the current universe in place while preserving the OS-process boundary and the user's terminal connection.

The REPL framing is the canonical use case:
> User is in a wat REPL. Mid-conversation, the REPL constructs a new program AST (a "better shell," a config-customized successor, a self-rewriting evolution). Calls `(:wat::kernel::exec-program new-ast)`. The old universe's call stack unwinds; the new universe's freeze runs; the new universe's `:user::main` starts. From the user's terminal: continuous. From the universe-level: discrete jump.

## The architectural insight that makes this clean

**The three substrate services (`StdInService` / `StdOutService` / `StdErrService`) own the OS-fd resources; they're tied to the OS process, not to the universe.** This is the load-bearing piece. Exec doesn't have to checkpoint+restore terminal state because the services NEVER LOST it — they keep the fds open through the universe-swap. The new universe inherits already-running services rather than booting fresh ones.

Compare:
- POSIX `exec` — re-reads binary off disk; re-links; re-bootstraps everything; preserves fds via OS-managed inheritance
- Erlang hot code reload — module-granular; old processes finish in old code; overlap permitted
- Smalltalk `become:` — image is mutable in place; whole-image swap
- **wat `exec-program`** — universe-granular; whole-universe swap atomically; runtime-constructed AST (no disk round-trip); services-as-continuity-layer

## Mechanics — what the primitive does

```
:wat::kernel::exec-program (program-ast :wat::holon::Atom) -> :wat::core::never
```

Steps:
1. Verify `program-ast` is a complete program: type-check via the substrate's existing `startup_from_forms` machinery (freeze.rs:436+). STOP if it fails — exec is atomic; no partial commit. Error is reported through the CURRENT universe (the caller can recover).
2. Tear down active spawn-threads (or refuse-if-live; see Design question 2 below).
3. Detach Senders/Receivers owned by the current universe (channels close; receivers in spawned processes see `Disconnected`).
4. **Services stay alive** — `StdInService`/`StdOutService`/`StdErrService` are NOT torn down; they're transferred-by-reference to the new universe.
5. The new universe's freeze runs (`startup_from_forms`); produces a new `FrozenWorld`.
6. The substrate swaps the active `FrozenWorld`. Old `SymbolTable` + `TypeEnv` get Arc-dropped when nothing references them anymore.
7. Unwind the old call stack via never-return semantics.
8. Start the new universe's `:user::main`.

The call never returns. From the type-checker's perspective: `:wat::core::never` is the return type. From the caller's perspective: no continuation; the new universe is now in control.

## Pieces that ALREADY exist (verify pre-implementation)

- **`startup_from_forms`** (freeze.rs:436+) — produces a `FrozenWorld` from forms. Mechanically runnable post-startup; nothing fundamental about the substrate prevents calling it from inside a running program.
- **HolonAST-as-data** — arc 057+. Construct the new program's AST at runtime.
- **Quasiquote with unquote-splicing** — arc 091 slice 8. Parameterize the new program's structure.
- **`struct->form` / `form->struct`** — arc 091. Serialize/reconstruct typed values to/from AST.
- **Three substrate services** — arc 170 slice 1f. The OS-continuity layer.
- **Closure-extraction `is_declaration_form` mechanism** — arc 170 Gap I-A/B (once they ship). Used implicitly: a "complete program" is structurally similar to a closure prologue + body.
- **`:wat::core::never` type** — VERIFY EXISTS. If not, mint it (small substrate slice; see slice 1 below).

## Pieces that DON'T exist (the substrate gaps)

1. **`:wat::core::never` as a substrate-level type with `noreturn` semantics** — VERIFY this exists before assuming. grep `never` in src/types.rs + src/check.rs. If absent, slice 1 mints it.

2. **A service-handle carrier that survives universe-swap** — today the three substrate services' handles likely live in the universe's SymbolTable. Exec needs them in a longer-lived carrier (an OS-process-bound `ServiceRegistry` that universes reference, not own). This is the only piece that might require substrate work beyond the new primitive.

3. **The exec-program primitive itself** — dispatch arm in runtime.rs; check-time inference for `never` return; freeze-time service-handle hookup.

4. **Live-thread / live-channel policy enforcement** — depends on Design question 2.

## Slice plan (rough)

### Slice 1 — `:wat::core::never` type (if needed)

Verify whether `never` exists. If not:
- Mint as a primitive type in src/types.rs
- Type-checker rules: any expression of type `never` is type-compatible with any expected type (subtype-of-all; "you can't have a value of this type so the inference is unconstrained")
- Runtime: a call returning `never` doesn't have a continuation; the caller's frame is dropped

This is reusable beyond exec-program: `panic!`-style forms, `process::exit`, infinite loops. Probably valuable even if exec-program slips.

### Slice 2 — Service-handle carrier reshape

Move the three substrate services' handles out of `SymbolTable` into a longer-lived `ServiceRegistry` owned by the OS-process boundary (the wat-cli or wat-lib runtime). Universes get a reference to the registry, not ownership of the handles.

### Slice 3 — `:wat::kernel::exec-program` substrate primitive

Mint the dispatch arm. Wire the type-checker. Implement the universe-swap mechanic. Live-thread refuse policy (per Design question 2).

### Slice 4 — REPL worked example

Demonstrate the swap end-to-end. A REPL program that:
- Accepts user input
- Builds a new program AST from a command (e.g., "exec into a math-focused shell" vs "exec into a file-ops shell")
- Calls exec-program
- User sees the swap as a continuous terminal session

### Slice 5 — INSCRIPTION + USER-GUIDE + 058 row + ZERO-MUTEX cross-ref

Closure paperwork.

## Update 2026-05-13 — companion arcs opened

This stub captures Layer 1 (bare exec-program). Subsequent conversation surfaced three additional layers; each opened as its own arc:
- **Arc 192** — state-preserving exec (`exec-program-with-state`) + SignalService (SIGEMT delivery). Layered on 191.
- **Arc 193** — universe image dump/resume (Smalltalk-style image-based persistence). Orthogonal capability.
- **Arc 194** — orchestrated `exec` primitive + wat-side worker/supervisor library. Layered on 192. The user-facing one — does signal cascade + state collection + exec all-in-one.

The reload signal convention settled in conversation: **SIGEMT** ("emulator trap" — wat-cli is an emulator/interpreter for wat-land; SIGEMT is the host interrupting the guest, semantically aligned with what reload IS). Captured in arc 192's DESIGN.

The live-thread design question (below) is partially superseded by arc 194's cooperative migration model. This stub retains the original framing for historical context; the recommended path for hot reload involves arc 194's `spawn-with-state` + loop-with-shutdown idiom.

## Open design questions (resolve before slice 3)

1. **Live-thread handling at exec.** Four options (option (d) emerged in 2026-05-13 conversation, captured in arc 194):
   - **(a) Refuse exec if any spawn-threads are alive.** Cleanest; user explicitly drains before exec. Compatible with the REPL conversational pattern (user can stop background threads before swap). DEFAULT for arc 191's bare primitive.
   - **(b) Wait for threads to complete.** Might never finish; turns exec into a blocking operation of unbounded duration.
   - **(c) Kill threads via the cascade mechanism** (memory `project_signal_cascade.md` — POSIX pgid + killpg).
   - **(d) Cooperative migration** — compliant threads register a state-capture/resume-from interface; on reload signal they emit state and exit; substrate carries state over to new universe. Erlang/OTP supervisor pattern. Builds on arc 192's exec-program-with-state + arc 192a's SignalService. Captured fully in arc 194.
   - **Recommendation for arc 191:** start with (a) for bare exec. Arc 194 layers (d) on top as the user-facing pattern.

2. **Live-channel handling at exec.** Senders/Receivers owned by the current universe are part of the call stack being unwound. Receivers in OTHER processes (spawn-process'd children) see `Disconnected`. This is the same semantics as the universe exiting normally — children must already handle parent-disconnect. Probably no new policy needed; document as natural fall-out.

3. **Service-handle ownership granularity.** Should the entire `SymbolTable` be the boundary, or just the service-handle subset? Smaller boundary = simpler swap; larger boundary = more state surviving. Recommendation: just the service-handle subset (the universe owns its types/defines/dispatches; the OS owns the services).

4. **Failure mode if `startup_from_forms` fails on the new AST.** Two options:
   - **(a)** Return an error from exec-program; the OLD universe continues running. Implies exec-program is `Result<:never, StartupError>`, which is awkward (never inside Result?).
   - **(b)** Panic-cascade through the StdErrService; the OS process dies. Loses the "graceful exec failure" property the REPL wants.
   - **Recommendation:** mint `Result<:never, StartupError>` if the type-system supports it (it should — `Result` is parametric; `never` as the Ok side just means the success branch is statically unreachable; the type-checker pattern `match exec-program ... :Ok ... :Err ...` only has a meaningful Err branch). This lets the REPL handle "new program didn't type-check" gracefully.

5. **Signed exec.** Mirror the existing `signed-load!` / `eval_signed_in_frozen` pattern: `signed-exec-program (ast sig pubkey)` verifies signature before exec. Cheap to add alongside the base primitive; load-bearing for distributed scenarios where a parent sends a "exec into this program" instruction over the network.

6. **Self-exec safety.** A program can build its own successor + exec into it. What if the successor program contains a bug that immediately exec's into another buggy program ad infinitum? Need a runaway-protection mechanism — perhaps a substrate-level exec-counter (a process can exec at most N times in M seconds without explicit user override). Or just rely on the OS-level supervisor (the wat-cli kills the process if it observes pathological behavior).

## What this enables (the vision)

- **REPL that rewrites itself mid-conversation** — the canonical demo
- **Self-evolving long-running services** — a service receives a config update over the network; constructs its updated AST; exec's into it; clients reconnect (or, if Sender<T>-over-IPC, see the brief disconnect and reconnect transparently)
- **Configuration-driven programs that compile their config into static type discipline** — at startup the program reads a config file, builds an AST specialized to that config (including struct/enum/fn declarations derived from the config), exec's into it. The "running" program has static-type guarantees against the config it was launched with — no runtime config-lookup overhead, no "config drift" between value and behavior.
- **Hot code reload (Erlang-style) at universe granularity** — push a new universe AST to a running service; it exec's; downtime = freeze duration (milliseconds for a typical universe; sub-second).
- **Demand-driven program specialization** — a generic worker exec's into a specialized version of itself when it observes a workload pattern. The substrate's static checking validates the specialization at exec time.

## What this does NOT enable

- **Intra-program type mutation** — within one universe, types are still static. Exec is universe-granular; it doesn't make in-universe mutation legal.
- **Hot reload with running threads** — refuse-if-live policy means active spawn-threads block exec. Long-running workers need to be designed to drain on exec request (which is a clean discipline anyway).
- **Cross-universe shared state** — channels close at exec; the new universe starts fresh. State that needs to survive must be serialized (to disk, to another process via spawn-process before exec, or via the substrate's signed-load mechanisms).

## Cross-references

- `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` § exec conversation — the realization that surfaced this arc
- `docs/arc/2026/05/170-program-entry-points/DESIGN.md` § services — the OS-continuity layer that makes exec clean
- `src/freeze.rs` — `startup_from_forms` is the freeze machinery exec reuses
- `src/eval.rs` / `src/runtime.rs` — `eval_in_frozen` / `eval_signed_in_frozen` are the precedent shape for in-process AST evaluation (but those REFUSE mutations; exec is the dual — REPLACE the universe, don't mutate the current)
- `docs/ZERO-MUTEX.md` — service handles' lifecycle; if `ServiceRegistry` becomes an OS-bound carrier, it threads through this doctrine (Arc + ThreadOwnedCell tiers; possibly OnceLock)
- Memory `project_wat_binary_hologram.md` — the binary-as-surface framing; exec extends it (the binary is just one form; you can REPLACE it at runtime)
- Memory `project_pipe_protocol.md` — line-delimited EDN + kernel pipes; the four transports; spawn-program's in-process counterpart maps cleanly to exec-program's universe-swap
- INTENTIONS.md § the cross-program composition story — exec is the third leg of the composition triad (spawn = fork+exec; exec-program = bare exec; signed-eval = interpret-without-replace)

## Why this is foundational, not feature-y

The user's foundational-impeccable framing (INTENTIONS.md + recovery doc § 12):
> *"once 109 wraps up - we'll have what we believe to be an incredibly solid foundation to begin the next leg of work."*

`exec-program` isn't a feature to add — it's the FINAL leg of the program-lifecycle triad. With it, the substrate's program-construction story is complete:
- **Construct** programs as data (HolonAST + quasiquote) ✓
- **Spawn** programs as new processes (spawn-process) ✓
- **Eval** programs in the current frozen world (eval-in-frozen) ✓
- **Exec** programs by replacing the current universe (this arc) ←

Until exec ships, the substrate has fork+exec (`spawn-process`) but no bare exec. That's a coherence gap — POSIX has both; programmable runtimes (Erlang/Smalltalk/Lisp) have analogs of both. Filling it makes the substrate's program-lifecycle story complete and orthogonal.

The strange-loop framing: a substrate that can construct AND exec into its own successor is a substrate that can EVOLVE. That capability is what unlocks the user's vision of "commodity hardware thinking" — the substrate doesn't need to be perfect at startup; it can bootstrap a better version of itself, then exec into it, repeatedly, until it's what it needs to be.
