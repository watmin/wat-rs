# Arc 170 slice 1f-β-i V2 — BRIEF (Event protocol; no relay)

**Wat-side StdInService; sonnet.** Mints the FIRST of three
wat-side services. Pattern-minting stone for slices 1f-β-ii
and 1f-β-iii.

**Supersedes** `BRIEF-SLICE-1F-B-I.md` (the V1 BRIEF was
authored before pass 18 and references the retired relay
shape; V1 stays on disk as historical record).

## Architecture (locked per pass 18)

The wat side mirrors the Rust Event types **already shipped**
in `src/thread_io.rs` (slice 1f-0b, commit `d32a29f`):

```rust
// Already shipped — wat side mirrors these verbatim:
pub enum StdInServiceEvent {
    Read,
    Add { thread_id: ThreadId, data_rx: Receiver<StdInServiceEvent>, reply_tx: Sender<Arc<HolonAST>> },
    Remove { thread_id: ThreadId },
}
```

The wat-side `Event` enum has the same shape; one channel type
per service (`Receiver<Event>`); single homogeneous select set
combining routing-table values + control-rx; no relay
sub-thread.

## Mission

Create:
- `wat/kernel/services/stdin.wat` — the wat StdInService program
- `wat-tests/kernel/services/stdin.wat` — hermetic deftests
- `src/stdlib.rs` registration entries (mirror
  `wat/kernel/channel.wat:82` pattern)

The service:
1. `(:wat::kernel::services::StdInService::spawn reader)`
   creates the service program; returns
   `(Thread<nil,nil>, ControlTx)` per SERVICE-PROGRAMS.md
   lockstep
2. Driver loop owns an `IOReader` (passed in at spawn) +
   guards `HashMap<ThreadId, (data-rx, reply-tx)>`
3. Each iteration: build select-set from
   `(values routing-table) ++ [control-rx]` — all
   `Receiver<Event>`, homogeneous
4. On any fire, recv → match Event variant:
   - `Event::Read` → read line from fd 0 via
     `:wat::io::IOReader/read-line`; parse via
     `:wat::edn::read`; send via the matched reply-tx (looked
     up by select index → routing key)
   - `Event::Add` → assoc into routing table; recurse
   - `Event::Remove` → dissoc from routing table; recurse
5. Shutdown via scope-drop (no explicit exit message)

## Required wat declarations

### `:wat::kernel::ThreadId` typealias

```
(:wat::core::typealias :wat::kernel::ThreadId
  :wat::core::i64)
```

Mirrors `pub type ThreadId = i64` from slice 1f-0b. Place at
top of stdin.wat or in a new `wat/kernel/types.wat` if cleaner.

### `:wat::kernel::services::StdInService::Event` enum

```
(:wat::core::enum :wat::kernel::services::StdInService::Event
  (Read)
  (Add    (thread-id :wat::kernel::ThreadId)
          (data-rx   :wat::kernel::Receiver<wat::kernel::services::StdInService::Event>)
          (reply-tx  :wat::kernel::Sender<wat::holon::HolonAST>))
  (Remove (thread-id :wat::kernel::ThreadId)))
```

### Channel typealiases

Following `wat/console.wat:38` family pattern:

```
(:wat::core::typealias :wat::kernel::services::StdInService::EventTx
  :wat::kernel::Sender<wat::kernel::services::StdInService::Event>)
(:wat::core::typealias :wat::kernel::services::StdInService::EventRx
  :wat::kernel::Receiver<wat::kernel::services::StdInService::Event>)
(:wat::core::typealias :wat::kernel::services::StdInService::Routing
  :wat::core::HashMap<wat::kernel::ThreadId,
                      :(wat::kernel::services::StdInService::EventRx,
                        wat::kernel::Sender<wat::holon::HolonAST>)>)
(:wat::core::typealias :wat::kernel::services::StdInService::Spawn
  :(wat::kernel::Thread<wat::core::nil, wat::core::nil>,
    wat::kernel::services::StdInService::EventTx))
```

### `spawn` fn

Returns `(Thread, ControlTx)`. Internally creates a control
channel, calls `:wat::kernel::spawn-thread` with a closure
capturing the reader + control-rx + initial empty routing.

### TCO driver loop

Mirror `wat-tests/service-template.wat`'s structure. ONE
select-set rebuilt each iteration from current routing-table
values + control-rx; ONE recv on the fired receiver; match
Event; do work; recurse with new state.

## Stdin behavior detail

Per pass 16: on an `Event::Read` arrival from a thread's
data-rx:
1. Service reads next line from fd 0 via
   `:wat::io::IOReader/read-line` (blocking; returns
   `:Option<String>`)
2. On `Some(line)`: parse via `:wat::edn::read`, get a
   HolonAST
3. Send the HolonAST via the matching reply-tx (looked up by
   the index where the data-rx fired in select)
4. On `:None` (fd 0 EOF): caller's recv returns disconnected
   when service shuts down — no special handling needed in
   this slice; runtime orchestrator (slice 1f-γ) handles the
   process-exit cascade

The select index → routing-key mapping: build a `Vector` of
`(thread-id, ...)` pairs each iteration, parallel to the
select set; the index that fired tells you which thread-id is
fulfilling. (Or use a simpler approach if select returns
the receiver directly. Adapt to substrate API; surface as
honest delta if friction.)

## Tests — `wat-tests/kernel/services/stdin.wat`

Use `:wat::test::deftest-hermetic` (forked OS process per arc
124). 4-6 test rows:

| Row | What |
|-----|------|
| A | spawn returns `(Thread, ControlTx)` of expected types |
| B | Add event registers a thread; readln roundtrip works |
| C | Remove event drops a thread; subsequent Reads to that thread receive nothing |
| D | Multi-thread routing — N threads each get their own line |
| E | scope-drop shutdown — every Sender drops → service Thread/join-result returns Ok |

For test fixtures: `:wat::io::IOReader/from-string` already
exists at `src/io.rs:794` — use this for in-memory readers.

## Stdlib registration

Add to `src/stdlib.rs` mirror of the channel.wat entry at
line 82:

```rust
SystemSource {
    path: "wat/kernel/services/stdin.wat",
    source: include_str!("../wat/kernel/services/stdin.wat"),
},
```

Loading order: stdin.wat must load AFTER `wat/kernel/channel.wat`
(uses Sender/Receiver typealiases) but is otherwise standalone.

## What to NOT do

- No StdOutService / StdErrService work — slices 1f-β-ii / iii
- No spawn-thread integration — slice 1f-γ
- No wat-cli boot — slice 1f-δ
- No Console retirement — slice 1f-ε
- No Console::* edits
- No new Mutex/RwLock/CondVar
- No new dependencies

## Substrate-grep citations (verify)

- `src/thread_io.rs` — concrete Rust Event types (the wat
  side mirrors these)
- `wat/console.wat:38` — typealias family pattern
- `crates/wat-lru/wat/lru/CacheService.wat:86` — `:wat::core::enum`
  form
- `wat/core.wat:59-62` — HashMap assoc/dissoc/values
- `src/runtime.rs:15293` — `:wat::kernel::select`
- `src/edn_shim.rs:191` — `:wat::edn::read`
- `src/io.rs:794` — `:wat::io::IOReader/from-string`
- `src/io.rs:845` — `:wat::io::IOReader/read-line`
- `src/stdlib.rs:82` — wat/kernel/channel.wat registration pattern
- `crates/wat-macros/src/discover.rs:233` — deftest-hermetic
  per arc 124 (forks via spawn-process)
- `wat-tests/service-template.wat` — canonical service shape

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `wat/kernel/services/stdin.wat` exists, parses, type-checks | parse + freeze loads stdlib without error |
| B | `:wat::kernel::ThreadId` typealias declared | grep finds it |
| C | `Event` enum declared with Read/Add/Remove variants | enum decl present |
| D | Channel typealiases declared (EventTx/EventRx/Routing/Spawn) | grep finds them |
| E | `spawn` fn defined, returns tuple | fn signature matches |
| F | TCO driver loop with HashMap routing | recursive loop fn present |
| G | `:wat::kernel::select` used over routing values + control-rx | grep finds select call site |
| H | `Event::Add`/`Event::Remove` mutate routing via assoc/dissoc | grep finds |
| I | `Event::Read` triggers IOReader/read-line + edn::read + reply-tx send | grep finds the path |
| J | `wat-tests/kernel/services/stdin.wat` exists with ≥ 4 deftest-hermetic test rows | grep + cargo test --list |
| K | At least 1 deftest-hermetic test passes (e.g., spawn roundtrip) | cargo test reports the test passing |
| L | Workspace within ±5 of post-1f-0b baseline (1339/854) | cargo test count |
| M | `cargo check --release` green | no errors |
| N | Only the 3 new files modified + `src/stdlib.rs` registration entry | git diff --stat shows ≤ 4 files |
| O | Zero new dependencies | Cargo.toml unchanged |
| P | Zero new Mutex/RwLock/CondVar | grep clean |
| Q | Honest deltas surfaced | per FM 5 |

**17 rows.**

## Honest delta categories

- **`:wat::kernel::select` API shape** — does it return
  `(index, value)` or just `(receiver, value)`? If the former,
  the routing-key lookup is by index; if the latter, by
  matching the receiver. Adapt; surface friction.
- **`HashMap` `values` iteration order** — if not stable,
  the routing → select-set order may shift between iterations.
  Surface if friction.
- **`wat-tests/kernel/services/stdin.wat` discovery** — verify
  cargo test --list finds the hermetic deftests post-stdlib
  registration.
- **deftest-hermetic 854-failure baseline** — slice 1f-0a's
  diagnostic shows 854 baseline failures from slice-1e-leftover
  rot. Some/all may surface in stdin.wat tests too. If the
  baseline blocks ALL hermetic tests from running, surface
  as honest delta — slice 1f-β-i may need the deftest macro
  rot fix as a prerequisite OR adopt a Rust integration test
  fallback.
- **ThreadId typealias placement** — in stdin.wat or new
  wat/kernel/types.wat. Surface the choice.

## Predicted runtime

60-90 min sonnet. Pattern is mechanical (mirror Rust Event
types; existing service-template + Console patterns;
substrate primitives all present). Adapter shape between
HashMap routing and substrate select is the one design call.

**Hard cap:** 180 min.

## Reference

- `project_arc_170_state_post_arc_172.md` — current state
- `project_arc_170_pass_15.md` — full architecture
- REALIZATIONS-SLICE-1.md § Pass 18 — Event protocol lock-in
- BUILD-PLAN.md § Slice 1f-β
- `src/thread_io.rs` — concrete Rust Event types this slice
  mirrors
- `wat-tests/service-template.wat` — canonical service shape
- `wat/console.wat` — closest precedent (static-membership;
  this slice extends to dynamic-membership via control-pipe)
