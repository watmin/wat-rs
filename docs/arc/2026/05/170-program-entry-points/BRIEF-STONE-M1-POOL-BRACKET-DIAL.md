# BRIEF — Arc 170 M1-pool: granted bracket workers dial a granted service (single-service)

**You are extending `wat/bracket.wat` so a granted process-pool worker can DIAL a granted service.**
Today the bracket grants each worker's pid to `:grants` services (grant-boot, proven) but the worker has no
way to *reach* them — a capability (`Address'`) crosses the trusted **wire**, never as closure data (ocap).
So the worker receives the service address over its runner wire as a **`Setup` message**, dials-and-holds
the typed peer (admitted, because grant-boot already granted its pid), and the work-fn uses it. This is the
`defservice` `:init`/`:ephemeral` pattern lifted onto the bracket. SCOPE: **single service** (one dialed
service, context = one peer). The heterogeneous N-service context is an explicit follow-on — do NOT build it.

## Read in order (rooms — every one grounded)

1. **`scratchpad/probe-m1-worker-setup.wat`** (GREEN — run it: `./target/release/wat scratchpad/probe-m1-worker-setup.wat` → `"echo:a echo:b"`). THE runtime reference. It hand-rolls exactly the runner you're baking: a worker `recv'`s a `:Pure` `Msg` enum, `match`es `Setup(addr) → (connect' addr)` held as `(Option Peer')`, `Work(s) → echo via the held peer`. **Copy this shape into the baked runner.**
2. **`wat/spawn.wat:190-196`** — `ServiceEvent<I,O,A>`, a **parametric `:enum`** that crosses the wire and is `match`ed in `collect-loop`. Your `PoolMsg<D,I>` mirrors it — parametric enums are already how this file works; this is NOT new machinery.
3. **`wat/bracket.wat:51-59`** — `process-runner` (baked, reserved), the loop you extend: today `recv' (i64,I) → send' (i64, work-fn item)`.
4. **`wat/bracket.wat:116-147`** — the process `spawn-runner`: `fn-forms` the work-fn, then AST-walks it (`:126-140`) to derive concrete `I`/`O` and build the `self-peer` tuple-type keywords. You extend this for a **2-param** work-fn `Fn(context,I)->O` + the `PoolMsg` recv type.
5. **`wat/bracket.wat:169-216`** — `collect-loop` (the result drain; it sends `Work` items via the peer — sends must now wrap as `PoolMsg::Work`).
6. **`wat/bracket.wat:226-284`** — `map-worker`: grant-boot at `:249-260` (unchanged), then you add the **`Setup` send** (after grant, before the first `Work`); revoke-shutdown at `:270-283` (unchanged).
7. **`wat/spawn.wat`** — `ProcessOpts` + `process/grants` builder: add a parallel **`:dials`** field + `process/dials` builder carrying the dial-target `Address'` (mirror how `:grants` is done, `wat/spawn.wat` — the `grants` defclause + ProcessOpts field). ThreadOpts/RemoteOpts → empty (the firm boundary; a thread worker shares memory, no wire dial).

## The build (single-service)

1. **`PoolMsg<D,I>` enum** (`:wat::enum::Pure` — proven to cross by probe-m1-worker-setup):
   `:Setup [deps <- :D]` | `:Work [pair <- :(wat::core::i64,I)]`. `D` = the dial-target address type; `I` = item.
2. **`process-runner`** threads a context `(Option C)` (None until Setup), `C` = the dialed peer type:
   - `(PoolMsg::Setup deps)` → recurse with `(Some (:wat::kernel::connect' deps))` — DIAL-and-HOLD.
   - `(PoolMsg::Work pair)` → `(Option/expect ctx …)`, `work-fn ctx (second pair)`, `send' (Tuple (first pair) out)`, recurse. (Copy the probe's serve-loop.)
3. **work-fn signature** `Fn(C,I)->O`. The AST-walk (`:126-140`) derives `I` off the **2nd** param and `C` off the 1st (or derive `C` = `Peer'<S,R>` from the `:dials` address type `Address'<S,R>` — `connect'`'s return). `O` off the return. Build the `self-peer` recv type as `PoolMsg<D,I>` (concrete `D` from `:dials`), send type `(i64,O)`.
4. **`map-worker`** — after grant-boot per worker, read `(:wat::spawn::dials locus)`, `send' p (PoolMsg::Setup <the-address>)` BEFORE the first `Work` item. The `Work` sends (in `map-worker`'s prime and in `collect-loop`) wrap the pair as `PoolMsg::Work`.

## Blast radius
`wat/bracket.wat` + `wat/spawn.wat` (the `:dials` field) ONLY. No `src/` Rust. No new intrinsics (`connect'`, grant, `peer-pid`, enums all exist). Do NOT touch the thread-tier runner-loop beyond what a shared `PoolMsg` requires (thread workers need no dial — the firm boundary; `:dials` is empty for thread).

## STOP triggers (halt + surface; do NOT thrash)
- **STOP-1** — if the 2-param monomorphization AST-walk (`:126-140`) cannot cleanly derive `C`/`I`/`O` for a `Fn(C,I)->O` work-fn, STOP and report the exact blocker. This is the delicate spot; the checker teaches — one located error at a time (RVINA ERVDIT). Do NOT invent a workaround that ships types unbound into the child.
- **STOP-2** — if `PoolMsg` (a `:Pure` enum carrying an `Address'`) won't cross the wire, STOP (it SHOULD — `probe-m1-worker-setup.wat` proved a `:Pure` enum with an `Address'` payload crosses; if it doesn't here, that's a real finding).
- **STOP-3** — if the single-service scope forces a heterogeneous-context design decision, STOP — that's the explicit follow-on, out of scope.

## How to work / iterate
Build without `--features simd`. The gate is a scratchpad probe first, then the test. Iterate the probe with the CLI (fast), the test FOREGROUND-blocking (never background-and-poll):
```
./target/release/wat scratchpad/probe-m1-pool-dial.wat      # your gate probe (see below)
cargo nextest run --release -p wat --test kernel -E 'test(bracket)' --test-threads=1   # bracket regressions
```

## Gate / Expectations (report each with its real result)
| what | command | expected |
|---|---|---|
| granted pool dials a service | a probe: `echo'` service (process), `(bracket/map (process/grants [eh] with :dials ea) items work-fn)` where work-fn `Fn(peer,String)->String` echoes | `["echo:a" "echo:b" "echo:c"]` |
| existing bracket unbroken | `cargo nextest run --release -p wat --test kernel -E 'test(bracket)' --test-threads=1` | all pass (no regression — the non-dialing brackets still work; `:dials` empty = the old path) |
| whole floor | `cargo nextest run --release` | 0 NEW failures (only the known `no_inlined_wat` lint + the `sigterm` flake, which passes isolated) |

Return: the gate-probe output, the bracket-suite result, the floor summary line, the final `bracket.wat`/`spawn.wat` diffs, and any STOP you hit. Do NOT commit — the orchestrator weighs by its own re-run.
