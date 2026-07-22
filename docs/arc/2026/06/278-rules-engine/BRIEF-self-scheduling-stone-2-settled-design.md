# BRIEF — Stone 2: the self-scheduling design ABOVE the multiplexer

> Substrate + macro strike. Full spec: `DESIGN-self-scheduling-defservices.md` (everything below its
> `✅ STATUS` block — the SETTLED design). Stone 1 (the timer-as-unified-`Peer'` foundation) is LANDED +
> green (`ca788849`), so a timer joins `poll'` by construction. This stone makes a `defservice`
> **arm itself** — the RED gate is `tests/services/probe_arc278_self_scheduling.{wat,rs}` (both loci).

## The work (one paragraph)

Grow the handler-result `Outcome<S,R>` → `Outcome<S,R,O>` (+ an `Alarm<O>` record) with three additive
variants so a handler can schedule a self-message; rename the serve loop's `clients` → `selectables`
(one vec holding client connections AND armed timers); synthesize a per-service `<service>::Op` superset
(surface ops + internal leading-dash `-`-ops) that the op-dispatch matches while the WIRE decode still
targets only the surface subset (the decode-gate that makes internals un-callable); preserve the leading
dash through `kebab→pascal`; resolve keyword-`:op` in an `Alarm` to the internal variant; and dispatch
the three orthogonal effects (reply / arm / remove) off the `Outcome` variant + the op kind. A `-tick`
internal op then arms via `NoReplyAndArm`, re-arms itself, advances a durable counter, and a client op
on the same service still replies between ticks — both loci.

## ⛔ STEP 0 — the disconfirming HOMOGENEITY probe (do this FIRST; HARD STOP)

Before ANY macro work, prove the load-bearing composition the whole stone rests on:
**can a client peer and a timer coexist in ONE `poll'` set and both dispatch?** Hand-roll (in
`wat-scripts/scratch-pad/probe-selectables-homogeneity.wat`) a `poll'` loop whose `selectables` vec holds
**both** a real accepted client `Peer'` (delivering a surface op) **and** a Stone-1 timer `Peer'`
(delivering an internal op) — and confirm `poll'` type-checks the mixed vec AND delivers each as a
`ServiceEvent::Message`. The seam: Stone 1's `after` yields `Peer'<nil, O>` (I=`nil`), but a client peer
is `Peer'<Reply, Op>` (I=`Reply`); and a client delivers `<surface>::Op` while the timer delivers the
internal `<service>::Op`. For the homogeneous vec to type, BOTH the I-side (`nil` vs `Reply`) and the
O-side (`<surface>::Op` vs `<service>::Op`) must resolve — via the superset embedding (O) and whatever the
I-side needs (a phantom `Reply` on the timer, or an existential I). **HARD STOP-0:** if a client peer and
a timer cannot share one `poll'`-typed `selectables` vec, STOP and surface the exact I-side/O-side gap —
this is a fork to resolve (like the poll'/timer fork was), NOT something to force. Do not proceed to the
macro until this composition is proven on the REAL `poll'`/`Peer'` path.

## Read in order (the rooms — grounded 2026-07-21)

1. `wat/service.wat:40-80` — the `Outcome<S,R>` defenum (`:48`). GROW to `<S,R,O>`; add `Alarm<O>` +
   `NoReply`/`ReplyAndArm`/`NoReplyAndArm` (the exact shapes are in the DESIGN § "The one contract
   decision"). Grep `Outcome<` → 3 sites (def + 2 comments), no handler annotates → phantom `O`, localized.
2. `wat/service.wat:509-512` — where `<S>::Op`/`<S>::Reply` are named under `proto-str`. This is where the
   `<service>::Op` SUPERSET is synthesized (surface variants + the internal `-`-ops). The op-dispatch
   matches `<service>::Op`; the wire decode targets `<surface>::Op` (subset) + rejects non-surface tags —
   the "internals un-callable" wall.
3. `wat/service.wat:743-827` — `serve-op-arms` (the per-op foldl). `:760-764` binds `s-binder` +
   `req-binder` (`first (rest param-ch)`) — a **1-param `-`-arm breaks here** (rest is empty). `:766`
   `op-pascal = kebab->pascal-in surface-kw op-str` — the **leading dash drops here**. `:797-825` the
   `outcome-match` (`Reply`/`Stop`) + the #16.2 budget guard — ADD the `NoReply`/`ReplyAndArm`/
   `NoReplyAndArm` arms; the internal-op path skips the #16.2 guard (no `req`) + skips the reply-send.
4. `wat/service.wat:848-925` — `serve-body`. `:848` `poll' self l clients`; `:850-851` conj on
   `Connection`; `:908-911` the `Message idx op` → `(match op ~@serve-op-arms)` dispatch; `:912-913`
   `Closed` → `remove-at clients idx`; `:922-925` `Lost` → `remove-at`. Rename `clients`→`selectables`
   throughout. On a `…AndArm` outcome: `conj` each `(after own-kind alarm.after alarm.op)` into
   `selectables`. On a fired `-`-internal op (a one-shot timer): `remove-at` its idx (a surface op / a
   client cast keeps its idx).
5. `src/string_ops.rs:336` `kebab_to_pascal_with_acronyms` — `s.split('-')`, so a leading `-` → an empty
   first segment that gets DROPPED. **Do NOT globally change this fn** (it is called everywhere). Preserve
   the dash SCOPED to the internal-op synthesis: at the defservice site, if `op-str` starts with `-`,
   strip it → kebab→pascal → re-prepend `-` (`-flush-tick` → `-FlushTick`). Ground the cleanest seam
   (a scoped wat helper, or handle it in `serve-op-arms`); a `-FlushTick` variant is already `--check`-legal.
6. `wat/service.wat:551-553` — `clients` typing (`Vector<Peer'<proto::Reply, proto::Op>>`). The
   `selectables` element type must admit both a client and a timer (STEP 0's resolution).
7. `wat-tests/timer-env-grab-parity.wat` — the `own-kind` env-grab pattern for `after` (the service's own
   tier → both loci) the arming uses.
8. `tests/services/probe_arc278_self_scheduling.{wat,rs}` — the RED gate. The `.wat` is a `Ticker` surface
   + a `ticker'` defservice with a `-tick` internal op arming via `ReplyAndArm`/`NoReplyAndArm`. Un-ignore
   the two `.rs` tests when green. (The `.rs` ignore-reason was re-pointed at this stone in `ca788849`.)

## Sub-step order (after STEP 0 proves the composition)

1. Grow `Outcome<S,R,O>` + `Alarm<O>` (localized def edit).
2. `clients` → `selectables` (pure rename; floor stays green — a checkpoint).
3. The `<service>::Op` superset synthesis + the wire decode-gate (surface subset).
4. The leading-dash marker: parse a `-op` in `:impls` (off the surface — NOT a coverage error; a non-`-`
   arm with no surface match stays a compile error) + the scoped dash-preservation.
5. 1-param `-`-arms in `serve-op-arms` (no `req-binder`, no #16.2 guard, no reply-send; → the `NoReply` family).
6. keyword-`:op` (`:op :-tick`) → the `<service>::Op` internal variant (kebab→pascal + dash-preserve).
7. The arm / reply / remove dispatch (the 3 orthogonal effects, per the DESIGN table).

## Blast radius

`wat/service.wat` (the defservice macro — the bulk), a small scoped dash-preservation seam (a wat helper
or `src/string_ops.rs` + a new scoped entry, NOT the global fn), possibly the `Alarm`/`Outcome` def, and
the RED gate `.wat`/`.rs`. **No change to `poll'`** (Stone 1 + the runtime are done), the timer
construction, or `select'`. CRUX-A (the macro binding `O` to the synthesized `Op`) is wiring resolved
in-strike, RED-gated.

## STOP triggers (halt + surface; never improvise a Value-erasure / a mode flag)

- **STOP-0** (above) — client + timer cannot share one `poll'`-typed `selectables` vec. The primary risk.
- **STOP-1** — the `<service>::Op` superset (surface variants + internal `-`-ops, matched by dispatch,
  gated on the wire) is not cleanly synthesizable in the macro.
- **STOP-2** — the leading-dash preservation cannot be done SCOPED (only by changing the global
  `kebab_to_pascal_with_acronyms`, which breaks other callers). Do NOT change the global fn.
- **STOP-3** — the 1-param `-`-arm cannot cleanly skip the #16.2 budget guard (which assumes `req`).
- **STOP-4** — `own-kind` env-grab for `after` is not reachable in the serve-loop context.

## Done criteria (RED → GREEN, weighed by the ORCHESTRATOR's own re-run)

- STEP-0's homogeneity probe committed + green (both tiers).
- `tests/services/probe_arc278_self_scheduling.{wat,rs}` un-ignored + GREEN, **both loci** (count ==
  target after re-arming; a client `poll` still replies between ticks).
- The `-tick` internal op is NOT client-callable (the wire decode-gate rejects it) — assert or note it.
- **Floor:** `cargo nextest run --release` → 0 NEW failures (Summary line; the known `wat-cli sigterm…`
  flake passes isolated).
- Any `.wat` corpus form-change goes through a wat-fix codemod, not hand edits.
