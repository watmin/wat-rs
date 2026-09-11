# DESIGN — the owner faces an outcome

Builder: *"you must not be able to crash us with momentary failues..."* and, on being shown that
`raise` is what makes it unavoidable: *"i'm probably 2-3 months out from killing raise entirely....
raises are placeholders until we know better."*

**Drawn 2026-09-11. NOT STRUCK.** Stone 2 of `a-momentary-failure-is-not-fatal`.

## Why

Stones 1a/1b gave the transport failure a typed path; it now reaches a named arm carrying its cause
(`Reply::Failed → ServiceEvent::Malformed → CallOutcome::Malformed → RecvOutcome::Malformed`). **And
the run still dies**, reproducibly in 15 s:

```
./target/release/wat wat-scripts/fanout/circuit.wat 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500
→ exit 2, raising the UNMIGRATED PLACEHOLDER inside the generated :demo::topic-worker/stop
```

It dies there because **`<S>/stop` returns a bare `T`**. A bare `T` has no room for *"asked, not
answered yet"*, so `assertion-failed!` is its only exit — and the generated body raises on **all
four** non-`Message` outcomes. Two of those arms' own panic strings say the peer is alive:

```
RecvOutcome::Stopped   → "…the service was ALIVE…"
RecvOutcome::TimedOut  → "recv: timed out — the peer is alive and silent"
```

`src/types.rs` agrees — *"DIED and NOTHING CLOSED: the peer is ALIVE and the channel is OPEN"*.
**The code states that nothing died and then kills the process.** Arc 278 minted these variants
precisely to stop conflating death with silence; the owner handling folds them back together.

## ⛔ THE ASSUMPTION THAT WAS WRONG, READ BEFORE BRIEFING

The earlier sketch said *"a bounded RE-ASK: send `Admin::Stop`, on TimedOut retry while budget
remains."* **That is refuted by `wat/service.wat:2287`:**

> *Stop → send Final(projected-state) up + terminate (returns nil, no recur)*

**`Admin::Stop` is not re-askable.** The first one terminates the service, so a second send reaches a
dead peer and no second reply will ever come — a re-ask would convert a slow stop into a guaranteed
`Gone`. The loop must be a bounded **RE-RECV**: send once, then keep *listening* within a wall-clock
budget. ★ This is the load-bearing assumption named and killed by a read, before the brief — which is
the one discipline that separates the stones that land here from the ones that die.

## What it delivers

`<S>/stop` and `<S>/hibernate` return an outcome instead of a bare value:

```
(:wat::core::defenum :wat::service::StopOutcome :- [T] :wat::enum::Pure
  :Stopped [state <- :T]                                        ;; clean
  :Gone    [cause <- :wat::kernel::LociDiedError]               ;; died before replying
  :GaveUp  [waited-ms <- :wat::core::i64                        ;; alive, never answered
            last <- :wat::core::String])                        ;; WHICH outcome ended it
```

Body — send once, then a **bounded re-recv**:

```
send Admin::Stop            ;; already outcome-tolerant today; keep that shape
deadline = now + budget-ms
loop:
  recv:
    Message(Status::Stopped s) -> Stopped s
    Message(other)             -> a protocol violation: keep today's raise (NOT momentary)
    TimedOut                   -> RE-RECV while budget remains, else GaveUp{waited,"TimedOut"}
    Stopped                    -> RE-RECV while budget remains, else GaveUp{waited,"Stopped"}
                                  (the peer is ALIVE — types.rs says so)
    Closed                     -> Gone Disconnected
    Lost(c)                    -> Gone c
    Malformed(_)               -> RE-RECV while budget remains, else GaveUp{waited,"Malformed"}
                                  (a garbled frame on the lineage peer is not the state we asked
                                   for, and the state may still arrive)
```

Two properties make it correct rather than merely raise-free:

- **Bounded by wall clock, never by an attempt count, and it names which bound it hit.** Verbatim the
  gateable property already owed as ruling #1 for `sweep-drained?` / `poll-until-visible-zero*` /
  `join-publishers*`. `stop` is the **fourth** member of that class.
- **`GaveUp` carries `last`.** Losing *which* outcome ended it is how today's crash cost an hour;
  `PatternMatchFailed` could not name its own scrutinee until this session fixed it.

## The one contract decision

**`Gone` is reserved for a peer that is actually gone — `Closed` or `Lost` — and nothing else.**
`TimedOut` / `Stopped` / `Malformed` are alive-signals and must never produce `Gone`, because a caller
distinguishing "the service died without replying" from "the service never answered in time" is the
entire point of the type. Collapsing them is what the four-armed raise did.

## Out of scope = REJECTED

- **`grant`.** It returns `nil` and matches `peer-process`, a different shape. 45 call sites and no
  evidence it belongs. Measure before moving it; do not bundle it on symmetry.
- **Removing `raise` anywhere else.** The builder's own 2–3 month arc. This stone must not ADD a
  raise; it does not undertake to remove the rest. The `Message(other)` protocol-violation raise
  STAYS — it is a genuine bug signal, not a momentary failure.
- **A retry inside `hibernate` that resumes.** Out of scope: hibernate gets the same outcome shape
  and the same bounded re-recv, nothing more.
- **The 549 placeholder arms from stone 1a.** Only the ones this stone's own paths touch.

## Blast radius, measured

```
/stop call sites        31      (git grep -oE '\(:[a-z0-9:_-]+/stop ' on *.wat)
/hibernate call sites    4
generated bodies         2      wat/service.wat stop-method-body :2953, hibernate-method-body :3005
```

⚠ The `/stop` count is a grep of occurrences (not lines) across `*.wat`; it has NOT been checked for
`.rs`-embedded or `.jsonl`-embedded call sites, and **stone 1a was reddened twice by exactly those
two carriers.** Enumerate them before claiming completeness (§STOP-3).

## Trap-doors named up front

1. **`Admin::Stop` terminates the service** — re-send is wrong; re-recv is the shape. Above.
2. **Three carriers of wat source, not one.** `.wat`, Rust string literals (`src/kernel/spawn.rs`),
   and `.jsonl` fixtures (`tests/cli/wat_mcp__*.jsonl`). Stone 1a's two floor reds were both this.
3. **`grep -c` counts LINES.** The `.jsonl` fixtures are one line each and reported "1 occurrence"
   when there were 11. Count with `grep -o … | wc -l`.
4. **Two implementations of the client template.** `wat/service.wat`'s `send-recv-form` AND
   `src/runtime.rs:6678`. If `stop` has a Rust-side twin, both need the change — stone 1b's crash
   MOVED rather than cleared because only one was fixed.
5. **The floor is the only verdict.** `cargo build --release` does NOT compile tests; it passed
   clean while a test construction site was broken. Use `cargo nextest run --release --no-run`.
