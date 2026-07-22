# BRIEF — self-scheduling defservices (arc 278 item (c) substrate stone)

**Goal:** make the RED gate `tests/services/probe_arc278_self_scheduling.{wat,rs}` GREEN, **both loci**,
with the whole floor `cargo nextest run --release` **0-new failures** — by building the self-scheduling
capability exactly as `DESIGN-self-scheduling-defservices.md` specifies. Anchor: work only in
`/home/watmin/work/holon/wat-rs/`; `pwd` first; any path with `.claude/worktrees/` is illegal — re-cd.

## Read in order (the ground)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-self-scheduling-defservices.md` — the full spec + every ruling.
2. `tests/services/probe_arc278_self_scheduling.{wat,rs}` — the RED gate you make green (the acceptance).
3. `wat-scripts/scratch-pad/probe-self-scheduling-loop.wat` — **the proven exemplar**: the exact
   select'-over-a-mutating-set + buffer + arm-mid-loop + flush mechanism, green, that the generated
   serve loop must reproduce. Mirror its shape.
4. `wat/service.wat:48` (the `Outcome<S,R>` defenum) · `wat/service.wat:753` (`serve-op-arms` foldl) ·
   `wat/service.wat` serve loop (the `(serve self l clients state)` recursion + the
   `ServiceEvent::Message`/`Closed` arms + `proto-str::Op` dispatch).
5. `src/string_ops.rs:336` (`kebab_to_pascal_with_acronyms` — drops a leading `-` today).
6. `src/types.rs` `synthesize_surface_protocol` (where `<Surface>::Op`/`::Reply` are built — the seam
   the `<service>::Op` superset extends).

## The work, decomposed — VERIFY each step (`target/release/wat --check`, then the gate) before the next
1. **Types** (`wat/service.wat:48`) — grow `Outcome<S,R>` → **`Outcome<S,R,O>`**; add `NoReply [state]`,
   `ReplyAndArm [state reply arms<-Vector<Alarm<O>>]`, `NoReplyAndArm [state arms<-Vector<Alarm<O>>]`;
   add `(:wat::core::defrecord :wat::service::Alarm<O> [after <- :wat::time::Duration  op <- :O])`.
   (Proven legal: `--check` green on the phantom-`O` shape — existing `Reply`/`Stop` build bare, `O`
   inferred.) Update the 2 doc comments at `:40`/`:42`.
2. **`-`-marked internal ops + the `<service>::Op` superset + keyword-`:op`** — the defservice must:
   - recognize an `:impls` arm whose op name has a **leading dash** as INTERNAL (not paired with a
     surface op; a non-`-` arm with no surface match stays the existing compile error);
   - synthesize **`<service>::Op`** = `<Surface>::Op` variants **+** the internal `-`-op variants
     (kebab→pascal with the **leading dash PRESERVED** — fix `string_ops.rs:336` to prepend a leading
     `-`; `-flush-tick` → `-FlushTick`, proven a legal variant);
   - the serve loop dispatches `<service>::Op`; `Outcome`'s `O` = `<service>::Op`; the wire/listener/
     client peers stay `<Surface>::Op`, decoded-as-surface (**reject any non-surface tag** — a client
     cannot send an internal op) then embedded into `<service>::Op` for dispatch;
   - resolve the `Alarm`'s **`:op` KEYWORD** (`:op :-tick`) to the `<service>::Op` internal variant
     (NOT a hand-written variant — keeps `<service>::Op` invisible in user forms).
3. **The serve loop** (`wat/service.wat`) — rename/widen `clients` → **`selectables`** (one vec of
   connections + armed timers; `select'` takes one anyway). On `Message{idx, op}` → dispatch →
   `Outcome<State,Reply,<service>::Op>`, **three orthogonal effects**:
   - **reply** ← the Outcome variant (`Reply`/`ReplyAndArm` → send to `selectables[idx]`; `NoReply`/
     `NoReplyAndArm` → no send; `Stop` → send then stop);
   - **arm** ← `…AndArm` → for each `Alarm`, `conj (:wat::kernel::after own-kind alarm.after alarm.op)`
     into `selectables` (own-kind = env-grab `(:wat::program::Env/wat.peer-kind (:wat::program::env))`,
     per `wat-tests/timer-env-grab-parity.wat` → both loci);
   - **remove** ← the **op kind**: a fired `-`-internal op is a one-shot timer → `remove-at
     selectables idx`; a surface op is a persistent client → keep (even on a `NoReply` cast). Dead
     connections still reap via the existing `Closed{idx} → remove-at`.
   `serve-op-arms` (`:753`) must handle a **1-param `[s]`** internal arm (no `req-binder` — `first
   (rest param-ch)` currently blows up on it; that IS the RED gate's first failure).

## Blast radius (bounded)
- `wat/service.wat` (the def + the macro codegen) + `src/string_ops.rs` + `src/types.rs`
  (`synthesize_surface_protocol`'s superset seam) + whatever Rust the `<service>::Op` synthesis /
  keyword-`:op` resolution / wire-embed need. **NO other `.wat` corpus edits** — migration is ~zero
  (grep: `Outcome<` is 3 sites, all in `service.wat`; every existing defservice must stay behaviorally
  identical — default-empty timers, `O` phantom for `Reply`/`Stop`).

## STOP triggers (REJECTION criteria — ship nothing, surface the gap)
- If the `<service>::Op` superset / wire-embed / keyword-`:op` resolution needs a substrate primitive
  that does not exist → **STOP**, surface exactly what's missing. Do not improvise a Value-erasure or a
  string-typed op.
- If any step forces a change to an EXISTING defservice's behavior (a non-additive reshape) → **STOP**.
- If `select'`-over-`{connection + after-timer}` does not compose as the exemplar shows → **STOP** (it
  is proven; a divergence is a real finding, not a workaround target).
- Do NOT weaken the RED gate to pass it. Do NOT add a `tick`/periodic sugar (arc-292 forbids it).

## Done = (weighed by the orchestrator's OWN re-run, never your report)
- `tests/services/probe_arc278_self_scheduling` — both tests GREEN (count == 3, both loci).
- `cargo nextest run --release` — read the **Summary** line, **0 new failures** vs the standing floor.
- No `.wat` corpus edits beyond `service.wat`. Commit nothing — the orchestrator weighs + commits.
