# BRIEF — the owner faces an outcome

**Read in order.** `DESIGN.md` beside this file first — it contains one refuted assumption you must
not re-derive, and the contract decision.

## The work, in one paragraph

`<S>/stop` and `<S>/hibernate` are generated methods that return a bare value and therefore raise on
every outcome except a clean reply — including two whose own panic strings say the peer is alive.
Give them an outcome type and a wall-clock-bounded **re-recv**, so an owner faces a value instead of
dying. This is the stone that makes *"a momentary failure must not crash us"* true; stones 1a/1b only
made the fact representable.

## The rooms

| where | why you are going there |
|---|---|
| `wat/service.wat:2963` `stop-method-body` | the generated `stop` client. Four non-`Message` arms, all `assertion-failed!`. This is the primary strike. |
| `wat/service.wat:2993` `stop-method` | its `defn` wrapper — `-> ~resp-ty` becomes the new outcome type. |
| `wat/service.wat:3015` `hibernate-method-body` | the mirror. Same shape, same change. |
| `wat/service.wat:3042` `hibernate-method` | its `defn` wrapper — `-> ~record-ty-ann`. |
| `wat/service.wat:2287` | ⛔ READ THIS FIRST: *"Stop → send Final + terminate (no recur)"*. Why the loop re-RECVs and never re-sends. |
| `wat/service.wat:3781` `CallOutcome` | the shape to copy: a parametric outcome enum over `O`, and it gained a variant cleanly earlier today. |
| `src/types.rs:1878` `RecvOutcome` | the five arms your loop dispatches on, each with a comment saying what it is NOT. `Stopped`/`TimedOut` say ALIVE. |
| `wat-scripts/fanout/circuit.wat:3208` | the call site that dies today, in `tw-stop-rts`. Your green proof. |

## Implementation sketch

Mint beside `CallOutcome` (`wat/service.wat:3781`):

```
(:wat::core::defenum :wat::service::StopOutcome :- [T] :wat::enum::Pure
  :Stopped [state <- :T]
  :Gone    [cause <- :wat::kernel::LociDiedError]
  :GaveUp  [waited-ms <- :wat::core::i64  last <- :wat::core::String])
```

Then `stop-method-body` becomes: send `Admin::Stop` once (keep today's four-armed `SendOutcome`
tolerance verbatim), take `t0`, then a recursive helper bounded by wall clock:

```
recv:
  Message(Status::Stopped s)  -> Stopped s
  Message(other)              -> KEEP today's raise (a protocol violation, not momentary)
  TimedOut | Stopped | Malformed(_) -> budget left ? re-RECV : GaveUp{elapsed, "<variant>"}
  Closed                      -> Gone Disconnected
  Lost(c)                     -> Gone c
```

`hibernate` is the same with `Status::Hibernated` and its own outcome (`HibernateOutcome :- [T]`, or
reuse `StopOutcome` — your call, state it in the SCORE).

## Blast radius, measured across all three carriers of wat source

```
.wat      45 occurrences / 22 files   (/stop + /hibernate)
.rs       2                            tests/process/probe_arc272_rs2_{process,thread}_stop_returns_final_state.rs
.jsonl    0
```

⭑ **Count with `grep -o … | wc -l`, never `grep -c`** — `grep -c` counts LINES, and a one-line
`.jsonl` fixture reported "1" where there were 11 earlier today.

Every call site does `let _ = (.../stop h)` or binds the state. Each must now face the outcome. Where a
caller genuinely only needs the state and a failure is a real defect there (the harness's own
teardown folds), `Gone`/`GaveUp` may raise **at the call site** — that is a caller's choice and it is
honest; what must stop is the GENERATED method deciding it for everyone.

## STOP triggers

1. **STOP-1 — if `Admin::Stop` turns out to be re-sendable after all**, stop and say so: the DESIGN's
   whole mechanism rests on `service.wat:2287` saying it terminates. Do not quietly switch to a
   re-ask.
2. **STOP-2 — if `stop` has a Rust-side twin** (as `send-recv-form` does at `src/runtime.rs:6678`),
   stop and report it before changing either. Stone 1b's crash MOVED instead of clearing because only
   one of two copies was fixed. Grep `src/runtime.rs` for a generated `/stop` before you start.
3. **STOP-3 — if the call-site census disagrees with the table above**, report the real number. Stone
   1a went RED twice from missed carriers: `tests/**/*.wat` (138 files) and wat inside `.rs`/`.jsonl`.
4. **STOP-4 — if `GaveUp` cannot carry `last`** in the shape you choose, stop. A give-up that does not
   name which bound it hit is the defect this stone is partly drawn to remove, not an acceptable
   simplification.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run` — the build does NOT compile
  tests, and that gap reddened the floor once today.
- `./scripts/floor.sh`, and read the **Summary line**, never a piped exit code.
- The 15-second proof, which must stop dying:
  `./target/release/wat wat-scripts/fanout/circuit.wat 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500`
- The happy path must be unchanged: `… 2000 4 3 8192 true 1000` → `distinct=8000 dup=0`.

## Shape to copy

`docs/excursus/2026/08/001-sns-sqs/a-call-outcome-cannot-lie/` — the same move one tier down
(replacing a lying return with a parametric outcome enum), including how its SCORE reported the call-site
migration. And `.../the-fill-poller-gets-what-the-drain-got/SCORE.md` for the wall-clock-bounded
give-up that names its bound.
