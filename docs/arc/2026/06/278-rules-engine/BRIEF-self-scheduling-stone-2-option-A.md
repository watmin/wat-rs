# BRIEF — Stone 2 (Option A): the `<service>::Op` superset + the self-scheduling design

> Substrate + macro strike. Full spec: `DESIGN-self-scheduling-defservices.md` (the `✅ STATUS` +
> `✅ O-SIDE RULED` blocks + everything below them). Foundations LANDED + DR'd: Stone 1 (timer =
> unified `Peer'`, `ca788849`) and the `Never` bottom / STEP-0 I-side (`a392fd40`). This stone builds
> the O-side (Option A — ruled over `Value`) + the settled design. RED gate:
> `tests/services/probe_arc278_self_scheduling.{wat,rs}` (both loci).

## The work (one paragraph)

Make a `defservice` arm ITSELF. The I-side homogeneity is done (`Never`). For the O-side: synthesize a
per-service superset enum `<service>::Op` = the surface's `<S>::Op` variants + the service's internal
leading-dash `-ops`; type `selectables` as `Vector<Peer'<Reply, <service>::Op>>`; and — the ONE novel
mechanism — **re-tag** a client's wire-decoded `<surface>::Op::X` value into its `<service>::Op::X`
counterpart at the decode boundary, so both clients (re-tagged) and timers (deliver `<service>::Op`
internal variants in-process) present ONE `<service>::Op`, dispatched by an exhaustive `match` (the free
coverage check preserved). Then the rest: grow `Outcome<S,R>`→`<S,R,O>` + `Alarm<O>`; `clients`→
`selectables`; the leading-dash marker (scoped `kebab→pascal` preservation); keyword-`:op`; 1-param
internal arms; the arm/reply/remove dispatch.

## ⛔ STEP 0 — prove the RE-TAG first (HARD STOP; the one novel mechanism)

Before ANY macro work, prove the re-tag on the REAL decode path (a hand-rolled `poll'` loop, per the
STEP-0 homogeneity probe you can copy: `wat-scripts/scratch-pad/probe-selectables-homogeneity.wat`).
The composition: a real client sends a `<surface>::Op::X` wire frame; the service's peer expects
`<service>::Op` (the superset); the decode produces a `<service>::Op::X` value (re-tagged), which then
matches a `<service>::Op::X` pattern. Write it in `wat-scripts/scratch-pad/probe-optA-retag.wat` and
prove BOTH tiers (thread + process). **The load-bearing question: how does the decode know to map
`<surface>::Op::X` → `<service>::Op::X`?** Candidates to ground (pick the cleanest, do not force):
- keyed on the peer's **expected O** — when a peer's expected receive type is a `<service>::Op` superset
  and the wire tag is a member surface variant (same variant name), re-tag to the superset counterpart;
- or a registered surface→superset relationship recorded by the superset synthesis.
**HARD STOP-0:** if the decode cannot cleanly produce a `<service>::Op` value from a `<surface>::Op`
wire frame (the mapping is ambiguous, the decode can't see the expected O, or it needs a Value-erasure),
STOP and report the exact blocker — do NOT fall back to `Value`-O (ruled out: it loses the free coverage
check). This is the crux the whole option rests on; prove it before the macro.

## Read in order (the rooms — grounded 2026-07-21/22)

1. `src/edn_shim.rs` — the self-describing wire decode: `tagged_to_value` (`:2427`),
   `reconstruct_enum_tagged` (`:2891`), `decode_trusted_wire`. The re-tag hooks here or just after
   (post-decode, keyed on the expected O). Ground how `poll'`/`recv'` obtain the expected type.
2. `src/runtime.rs` — `eval_poll_prime` (`:27366`): how a client frame is decoded (process tier uses
   `select_raw` + `decode_trusted_wire`). This is where a client peer's `<service>::Op` O flows in.
3. `wat/service.wat:509-516` — where `<S>::Op`/`<S>::Reply` are named under `proto-str`, and
   `synthesize_surface_protocol` (`src/types.rs`) is referenced. The `<service>::Op` SUPERSET synthesis
   sits here (mint the superset = surface variants + internal `-ops`; record the surface→superset map if
   STEP 0 needs it).
4. `wat/service.wat:40-80` — `Outcome<S,R>` (`:48`). Grow to `<S,R,O>` + add `Alarm<O>` +
   `NoReply`/`ReplyAndArm`/`NoReplyAndArm` (exact shapes in DESIGN § "The one contract decision").
   `Outcome<` grep → 3 sites (def + 2 comments), no handler annotates → phantom `O`, localized.
5. `wat/service.wat:743-827` — `serve-op-arms`. `:760-764` binds `s`/`req` (`first (rest param-ch)`) —
   a **1-param `-`-arm breaks here**. `:766` `op-pascal = kebab->pascal-in surface-kw op-str` — the
   **leading dash drops here**. `:797-825` the `outcome-match` (`Reply`/`Stop`) + the #16.2 budget guard
   — ADD `NoReply`/`ReplyAndArm`/`NoReplyAndArm`; the internal-op path skips the #16.2 guard (no `req`)
   + the reply-send.
6. `wat/service.wat:848-925` — `serve-body`: `poll'` (`:848`), `conj` on `Connection` (`:851`), the
   `Message idx op` dispatch (`:908-911` → `(match op ~@serve-op-arms)`), `Closed`/`Lost` `remove-at`.
   Rename `clients`→`selectables`. On `…AndArm`: `conj (after own-kind alarm.after alarm.op)` into
   `selectables`. On a fired `-`-internal op (a one-shot timer): `remove-at` its idx (a surface op / a
   client cast keeps its idx).
7. `src/string_ops.rs:336` `kebab_to_pascal_with_acronyms` — `split('-')` drops a leading dash. Do NOT
   change the GLOBAL fn (called everywhere). Preserve the dash SCOPED at the internal-op synthesis (strip
   `-`, kebab→pascal, re-prepend `-`; a `-FlushTick` variant is already `--check`-legal).
8. `wat-tests/timer-env-grab-parity.wat` — the `own-kind` env-grab for `after`. `after` now returns a
   unified `Peer'<Never, O>` (Stone 1 + Never); arming `(after own-kind alarm.after alarm.op)` where
   `alarm.op : <service>::Op` internal variant → the timer delivers a `<service>::Op` value directly
   (in-process, no wire, no re-tag).
9. `tests/services/probe_arc278_self_scheduling.{wat,rs}` — the RED gate. Un-ignore the two `.rs` tests
   when green. (The `.rs` ignore-reason was re-pointed at this stone in `ca788849`.)

## Sub-step order (after STEP 0 proves the re-tag)

1. `<service>::Op` superset synthesis (+ the surface→superset map STEP 0 established) + the decode re-tag.
2. Grow `Outcome<S,R,O>` + `Alarm<O>`.
3. `clients` → `selectables` (pure rename; floor stays green — a checkpoint).
4. The leading-dash marker (parse a `-op` in `:impls`, off the surface — NOT a coverage error; a non-`-`
   arm with no surface match stays a compile error) + the scoped dash preservation.
5. 1-param `-`-arms in `serve-op-arms` (no `req-binder`, no #16.2 guard, no reply-send; → the `NoReply`
   family).
6. keyword-`:op` (`:op :-tick`) → the `<service>::Op` internal variant.
7. The arm / reply / remove dispatch (the 3 orthogonal effects, per the DESIGN table).

## Blast radius

`wat/service.wat` (the defservice macro — the bulk: superset synth + serve loop), `src/edn_shim.rs` /
`src/runtime.rs` (the decode re-tag), `src/check.rs`/`src/types.rs` (the superset type + surface→superset
map), a small scoped dash-preservation seam (NOT the global `kebab_to_pascal_with_acronyms`), and the RED
gate. **No change to** `poll'`'s multiplex, the timer construction (Stone 1), `select'`, or the `Never`
foundation. Any `.wat` corpus form-change → a wat-fix codemod, not hand edits.

## STOP triggers (halt + surface; never a Value-erasure or a mode flag)

- **STOP-0** (above) — the re-tag can't cleanly turn a `<surface>::Op` wire frame into a `<service>::Op`
  value. The crux; do not force it, do not fall to `Value`.
- **STOP-1** — the `<service>::Op` superset (surface variants + internal `-ops`) is not cleanly
  synthesizable in the macro.
- **STOP-2** — the dash preservation can only be done by changing the GLOBAL `kebab_to_pascal_with_acronyms`.
- **STOP-3** — the 1-param `-`-arm can't cleanly skip the #16.2 budget guard (which assumes `req`).
- **STOP-4** — `own-kind` env-grab for `after` is not reachable in the serve-loop context.
- **STOP-5 (the exhaustiveness guard)** — if you find yourself wanting a **wildcard** or an
  `assertion-failed!`/`unreachable`-style dead arm in the `<service>::Op` op-dispatch, STOP and
  re-examine: A's whole point is that the superset match is exhaustive with **every arm reachable** (a
  surface op from clients, an internal `-op` from timers, each with an `:impl`) — no papering. The
  re-tag maps `<surface>::Op` → `<service>::Op` **exhaustive over the SURFACE variants** (never a
  service→service match that would demand dead internal arms). The wall is a **decode REJECTION** — a
  client frame tagged with an internal `-op` is a located decode error (the client cannot call `-tick`),
  NOT a dead match arm.

## Done criteria (RED → GREEN, weighed by the ORCHESTRATOR's own re-run)

- STEP-0's re-tag probe committed + green (both tiers).
- The op-dispatch match over `<service>::Op` stays **exhaustive** (a missing `:impl` → compile error —
  the free coverage check preserved; NO wildcard papering).
- `tests/services/probe_arc278_self_scheduling.{wat,rs}` un-ignored + GREEN, **both loci** (count ==
  target after re-arming; a client `poll` still replies between ticks).
- The `-tick` internal op is NOT client-callable (the wire is `<surface>::Op`; a client cannot construct
  a `-op`) — assert or note it.
- **Floor:** `cargo nextest run --release` → 0 NEW failures (Summary line; the known `wat-cli sigterm…`
  flake passes isolated).
