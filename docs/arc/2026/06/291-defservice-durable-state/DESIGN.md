# Arc 291 — defservice durable state: `init` / `stop` / `hibernate` / `resume`

**Status:** SCOPED (2026-06-22). Surfaced by arc 290's lru migration: the cache's
state is a thread-owned, non-serializable Rust `LruCache`, which defservice today
**cannot host** because it ships the State value over the wire (down as `state0`,
back on `:Stop`). The HORIZON note
(`272/NOTE-service-final-state-return.md`) already recorded the constraint:
*"a service's return value IS its final state; the symmetry forces state to be
wire-serializable."* This arc removes that forcing.

## The realization

defservice conflates two different things — **the State** and **what crosses the
wire**. gen_server keeps them separate. There are exactly THREE concepts:

| concept | must be EDN? | always present? | role |
|---|---|---|---|
| **`resp`** (return value) | yes | yes | the caller's answer from any op, incl. stop |
| **State** | **no — any type** | yes | the live thing; lives only in-locus |
| **snapshot** (hibernate/resume) | **only if State is EDN** | **only then** | durable hibernation + migration |

The wire only ever carries: **init-args in**, **resp out** (both always EDN), and
— *when the type allows* — a **Snapshot** for hibernate/resume. The live State
never crosses the wire by accident.

gen_server has three core callbacks; defservice built two:

| gen_server | defservice today | this arc |
|---|---|---|
| `handle_call/3 → {reply,R,S}` | ✅ `Outcome::Reply` | unchanged |
| `terminate` (+ state back) | ✅ `:Stop` ships `StopResponse[state]` | **split** → `stop → resp` |
| `init/1 → {ok,State}` | ❌ missing | **add** the `init` callback |
| (hibernation) | ❌ ships raw state always | **add** type-gated `hibernate`/`resume` |

## The lifecycle (target)

```
fresh:        start(locus, init-args) ─→ [running] ─→ stop ─→ resp        ;; ANY state type
hibernation:  start(locus, init-args) ─→ [running] ─→ hibernate ─→ Snapshot
                                                                  │ (EDN; persist anywhere)
                              later / another process: resume(locus, Snapshot) ─→ [running]
```

- **`init`** — a callback `(init-args → State)` that runs **in the service's locus**
  (thread: in the spawned thread; process: child-side, after it `recv'`s the EDN
  args it already receives). Builds the live State — including non-serializable
  resources (LruCache, sockets, DB handles) — where it lives. `start` takes
  **EDN `init-args`**, not a pre-built `state0`.
- **`stop`** — terminates, returns a serializable **`resp`** (may be `nil`). The
  return value is **decoupled from the State**. This is how a caller gets a final
  value regardless of what the State is made of.
- **`hibernate`** — terminates and hands the State out **as a Snapshot** (the State,
  EDN-encoded). Emitted/valid **only when State is EdnRepresentable**.
- **`resume`** — the dual of `start`: a **fresh spawn** whose initial State is the
  **deserialized Snapshot**, **bypassing `init`** (a snapshot-able state is pure
  data — no resources to rebuild). `resume : snapshot :: start : init-args`. NOT
  injection into a live service (no hot state-swap).

**The headline:** hibernate → kill the holding process → `resume` in a new process
→ **the service cannot tell the difference**. Service code is byte-identical across
`start` and `resume`; it only ever observes "I hold State X and I'm serving." That
makes a defservice location- and time-independent: durable actors + transparent
process migration, achieved purely by the EDN-only-on-the-wire discipline.

## Sub-strikes (sequenced)

0. **DESIGN** (this doc) — the contract.
1. **RED probe** — a counter that `start`s, increments, `hibernate`s, has its
   process killed, then `resume`s in a fresh process and continues — asserting the
   resumed value. RED at HEAD (no init/hibernate/resume exists). Commit before build.
2. **`init` callback** (THE KEYSTONE, unblocks arc 290) — add `:init [args] <body→State>`
   to the defservice macro; `start [locus init-args]` runs it in-locus on BOTH the
   thread launch (`wat/spawn.wat` ThreadOpts) and the process launch (child runs init
   after `recv'`ing the EDN args). After this lands, the lru/holon-lru cache migration
   (arc 290 Class A) is unblocked — non-serializable state is constructible in-locus.
3. **`stop → resp` decouple** — split today's `:Stop[state:S reply:R]` /
   `StopResponse[state]` into `stop → resp` (always EDN, no raw state). Preserve
   "return value IS final state" *for EDN state* by letting the author project the
   state into `resp`; non-EDN services return an honest serializable summary (or nil).
4. **`hibernate` / `resume`** (the durable-actor capability) — type-gated on
   `EdnRepresentable`. **RISK / PROBE:** there is no wat-level `EdnRepresentable`
   protocol to bound on today (only the Rust trait in `edn_shim.rs`). Decide the gate:
   - (a) **type-checker gate** (preferred) — `hibernate`/`resume` are always emitted,
     but their bodies EDN-encode/decode the State; if EDN-encode of a non-EDN type is
     a **compile error**, the gate is automatic (calling `cache/hibernate` won't
     type-check). **Probe first:** does encoding / sending a non-EDN value (e.g. an
     `LruCache`) over a peer fail at **compile** time or **runtime**? That answer
     decides whether the gate is clean (compile) or needs a protocol bound.
   - (b) **macro-time gate** — emit `hibernate`/`resume` only when all state-field
     types are EDN-able. Harder (macro lacks type info); fallback only.

## Out of scope (affirmative cuts)
- Hot state-swap / live-injection into a running service (different, hairier op).
- Snapshot persistence format / storage — Snapshot is just EDN; persistence is the caller's.
- Generic `<K,V>` defservice — orthogonal (arc 290 stays monomorphic).

## Done = the gate
The RED probe (sub-strike 1) goes GREEN: counter hibernate → process-kill → resume →
continue, across processes, asserted. `service-locus-parity.wat` still green. And the
arc 290 cache migration compiles against the new `init` surface (non-serializable
state hosted, no `Option`/`ensure-cache` hack).
