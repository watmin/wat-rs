# BRIEF — every client call has a deadline

Extract the deadline pattern into one parametric stdlib helper, then put all four of the
worker's client calls behind it. `wat/service.wat` + `wat-scripts/fanout/circuit.wat`.

Read `DESIGN-every-client-call-has-a-deadline.md` first — it carries the census and the measured
answer to *where the helper can live*.

## ⛔ THE TWO FACTS THAT SHAPE THE WORK

1. **A plain sibling `defn` is unreachable from a process-locus impl.** Proven by
   `wat-scripts/scratch-pad/probe-what-a-process-impl-can-call.wat` — the child dies with
   `UnknownCallee`. Run it; it fails by design and the failure is the result. The helper
   therefore goes in **`wat/service.wat`**, not `circuit.wat`.
2. **The locus is in the type.** A process handle is `(Handle :- [Wire])`, a thread handle is
   `(Handle :- [Shared])`; `if` cannot unify them. Do not try to write one code path over both.

## READ IN ORDER

| room | why you are there |
|---|---|
| `wat/service.wat:3102-3120` | `send-keep-serving?` — **the precedent**: a parametric stdlib defn (`:- [R O]`) placed after the macro. Copy its shape and its placement |
| `circuit.wat:424-470` | the existing deadlined `check` — raw `kernel::send`, the `(first (conj (Vector :- [<peer-ty>]) timer))` laundering, `select`, and the `idx`-discriminated arms. **This is the body to lift** |
| `circuit.wat:448-455` | the timer construction and its **inert** payload — required by the type, never read, because `idx` discriminates |
| `circuit.wat:515` | `Seen/mark` — bare, gains a deadline |
| `circuit.wat:526` | `Queue/ack` — bare, gains a deadline |
| `circuit.wat:402` | `Queue/receive` — `:wait` stays (it is a server bound and still correct); a client deadline is added around it |

## SKETCH

In `wat/service.wat`, **after** the `defservice` macro so nothing above `:896` moves:

```wat
(:wat::core::defn :wat::service::call-by-deadline :- [I O]
  [peer <- (:wat::kernel::Peer :- [:I :O])  op <- :I
   ms <- :wat::core::i64  inert <- :O]
  -> (:wat::core::Option :- [:O])
  ;; send, then select [peer timer]. idx 0 = a real reply -> Some. idx 1 = the deadline -> None.
  ;; `inert` is the timer's payload: the type demands a value, and it is never read.
  …)
```

At each call site: `Some reply` → proceed; `None` → the deadline fired; redial and retry exactly
as `check` does today.

## BLAST RADIUS

`wat/service.wat` (one added defn, at the end) and `circuit.wat` (four call sites). No `src/`,
no `sqs.wat`, no codemod, no nextest config.

⚠ **A stdlib edit means a rebuild before any probe or circuit run** — `wat/*.wat` is frozen into
the binary at build time.

## STOP TRIGGERS

- **STOP-1** — if the parametric signature cannot express the `select` laundering (the
  homogeneous-peer-vector trick at `circuit.wat:445-450`) with `:I`/`:O` type params, STOP and
  report the exact checker error. Do not fall back to a per-surface copy.
- **STOP-2** — if `check`'s numbers change after being refactored onto the helper, STOP. The
  refactor must be behaviour-identical; that is what makes it safe to trust at the other three
  sites.
- **STOP-3** — if giving `Queue/receive` a client deadline requires removing or shortening
  `:wait`, STOP and report. They are different bounds and both are correct; one must not be
  traded for the other.
- **STOP-4** — do not touch `claim deadline exhausted`, the redelivery fixture, or anything
  perf-shaped. All three are open and all three are other stones.

## PRIOR RESULT TO COPY FOR SHAPE

`SCORE-a-ledger-is-a-receipt.md` — the immediately prior strike, and its discipline: it reported
a red rather than re-running it, and it named what it could not show.
