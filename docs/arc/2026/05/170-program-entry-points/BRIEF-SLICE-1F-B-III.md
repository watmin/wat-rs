# Arc 170 slice 1f-β-iii — BRIEF (wat-side StdErrService)

**Sonnet pattern-apply.** Final wat-side service mint of the trio.
Structurally identical to slice 1f-β-ii (StdOutService) — same
Write/ack-tx shape, same data-channel-Event dispatch. The only
semantic difference is fd 2 (stderr) vs fd 1 (stdout); per
TIERS.md doctrine, fd 2 carries only panic-cascade EDN.

## Architecture (locked per pass 18)

Mirror Rust `StdErrServiceEvent` enum (already shipped at
`src/thread_io.rs:60-68`, slice 1f-0b commit `d32a29f`):

```rust
pub enum StdErrServiceEvent {
    Write { line: String },
    Add {
        thread_id: ThreadId,
        data_rx: Receiver<StdErrServiceEvent>,
        ack_tx: Sender<()>,
    },
    Remove { thread_id: ThreadId },
}
```

**Identical shape to `StdOutServiceEvent`.** This slice is a
near-clone of slice 1f-β-ii (committed `fe9b9e9`) with the
namespace swap `StdOutService → StdErrService` everywhere.

## Mission

Create:
- `wat/kernel/services/stderr.wat` — wat-side StdErrService program
- `wat-tests/kernel/services/stderr.wat` — 4-5 hermetic deftests
  (will hit the § Row K architectural blocker like β-i and β-ii;
  author them anyway)
- `src/stdlib.rs` registration entry after stdout.wat entry

The service:
1. `(:wat::kernel::services::StdErrService::spawn writer)`
   creates the service program; returns `(Thread<nil,nil>, ControlTx)`
2. Driver loop owns an `IOWriter` on fd 2 + `Vector<RoutingEntry>`
3. Each iteration: build select-set from
   `(routing-rxs ++ [control-rx])`
4. On any fire, recv → match Event:
   - `Event::Write { line }` → `(:wat::io::IOWriter/writeln writer line)`; send `()` ack
   - `Event::Add` → conj entry; recurse
   - `Event::Remove` → filter entry by thread-id; recurse
5. Scope-drop shutdown

## Data-channel dispatch shape (explicit, per slice 1f-β-ii lesson)

The data-channel receives the full `Event` enum. Dispatch on
data-rx fire must `match event` and handle three arms:
- `Write { line }` → the productive case
- `Add` → no-op (Add should arrive only on control-rx; defensive arm)
- `Remove` → no-op (same reason; defensive arm)

This was the one structural delta sonnet surfaced in slice
1f-β-ii — calling it out here so it doesn't surface again as a
delta.

## Required wat declarations

### Enum

```
(:wat::core::enum :wat::kernel::services::StdErrService::Event
  (Write (line :wat::core::String))
  (Add
    (thread-id :wat::kernel::ThreadId)
    (data-rx :wat::kernel::Receiver<wat::kernel::services::StdErrService::Event>)
    (ack-tx :wat::kernel::Sender<wat::core::nil>))
  (Remove
    (thread-id :wat::kernel::ThreadId)))
```

### Channel typealiases (mirror β-ii's family)

```
(:wat::core::typealias :wat::kernel::services::StdErrService::EventTx ...)
(:wat::core::typealias :wat::kernel::services::StdErrService::EventRx ...)
(:wat::core::typealias :wat::kernel::services::StdErrService::Routing ...)
(:wat::core::typealias :wat::kernel::services::StdErrService::Spawn ...)
```

Search-and-replace `StdOutService → StdErrService` from stdout.wat.

### `spawn` fn + TCO driver loop

Mirror β-ii's structure exactly. No structural changes.

## Tests — `wat-tests/kernel/services/stderr.wat`

Mirror β-ii's test file. 4-5 hermetic rows:

| Row | What |
|-----|------|
| A | spawn returns `(Thread, ControlTx)` of expected types |
| B | Add → Write roundtrip produces expected bytes via IOWriter/new + IOWriter/snapshot |
| C | Remove drops thread; subsequent Writes receive no ack |
| D | Multi-thread routing — N threads' Writes all echo to writer's buffer |
| E | scope-drop shutdown — every Sender drops → Thread/join-result returns Ok |

Same § Row K caveat as β-i/β-ii — document inline in test file
header.

## Stdlib registration

Add to `src/stdlib.rs` immediately after the stdout.wat entry
(arc 170 slice 1f-β-ii ships it):

```rust
// Arc 170 slice 1f-β-iii — `:wat::kernel::services::StdErrService::*`.
WatSource {
    path: "wat/kernel/services/stderr.wat",
    source: include_str!("../wat/kernel/services/stderr.wat"),
},
```

## What to NOT do

- No spawn-thread integration (slice 1f-γ)
- No wat-cli boot (slice 1f-δ)
- No Console retirement (slice 1f-ε)
- No `deftest-hermetic` migration (architecturally downstream)
- No new Mutex / RwLock / CondVar
- No new dependencies

## Substrate-grep citations (verified pre-flight)

- `src/thread_io.rs:60-68` — concrete `StdErrServiceEvent`
- `src/io.rs:1078` — `:wat::io::IOWriter/writeln`
- `wat/kernel/services/stdout.wat` (committed `fe9b9e9`) — pattern source
- `wat-tests/kernel/services/stdout.wat` — test pattern source
- `src/stdlib.rs` — stdout.wat registration; mirror for stderr.wat

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `stderr.wat` parses, type-checks | `cargo check --release` green |
| B | `Event` enum declared (Write/Add/Remove) | grep |
| C | Channel typealiases (EventTx/EventRx/Routing/Spawn) | grep |
| D | `spawn` fn returns tuple `::Spawn` | grep |
| E | TCO driver loop with Vector routing | grep |
| F | `:wat::kernel::select` over routing + control-rx | grep |
| G | Add/Remove mutate routing via conj/filter | grep |
| H | Write triggers `IOWriter/writeln` + ack-tx send `()` | grep |
| I | Data-channel match has Write/Add/Remove arms (defensive completeness) | grep |
| J | `wat-tests/kernel/services/stderr.wat` exists with ≥ 4 deftest-hermetic rows | grep + cargo test --list |
| K | Workspace 1339/869 (1339/864 baseline + 5 new at § Row K) | cargo test count |
| L | `cargo check --release` green | no errors |
| M | Exactly 3 files modified (2 new wat + src/stdlib.rs) | git status |
| N | Zero new deps; zero Mutex/RwLock/CondVar | grep + Cargo.toml |
| O | Honest deltas surfaced | per FM 5 |

**15 rows.** Row K's deftest-hermetic blocker continues; not
required to pass.

## Predicted runtime

**10-15 min sonnet.** Pattern is fully trodden; this is namespace
swap + one fn signature update. Per slice 1f-β-ii's lesson, this
family of pattern-applies ships in sub-15-min.

**Hard cap:** 30 min.

## Reference

- Pattern source: `wat/kernel/services/stdout.wat` (committed `fe9b9e9`)
- Predecessor SCORE: `SCORE-SLICE-1F-B-II.md` (15/15; 8-min runtime)
- Predecessor BRIEF: `BRIEF-SLICE-1F-B-II.md`
- Architecture: REALIZATIONS pass 18; TIERS.md § OS-boundary handling
- TIERS.md doctrine: "Inside wat-land, fd 2 ONLY ever carries
  panic-cascade EDN. wat-cli has zero direct stderr writes."

## Path forward post-slice-1f-β-iii

1. Orchestrator scores; commits atomic-pair
2. **Trio complete** — all three wat-side services minted
3. Next: slice 1f-γ — runtime orchestrator (substrate Rust work
   that calls `spawn-thread` to boot each service program and
   threads ThreadIO through the per-thread cell)
4. Then slice 1f-δ — boot integration; `deftest-hermetic`
   migration closes § Row K; 854→0 baseline failures
