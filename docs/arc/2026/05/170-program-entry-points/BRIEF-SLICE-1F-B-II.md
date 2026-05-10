# Arc 170 slice 1f-β-ii — BRIEF (wat-side StdOutService)

**Sonnet pattern-apply.** Mirror of slice 1f-β-i (committed at
`e898c7a`). Mints the SECOND of three wat-side substrate stdio
services. Pattern is now well-trodden; sonnet should ship in
30-45 min.

## Architecture (locked per pass 18)

Mirror the Rust `StdOutServiceEvent` enum **already shipped** in
`src/thread_io.rs` (slice 1f-0b, commit `d32a29f`):

```rust
// Already shipped — wat side mirrors verbatim:
pub enum StdOutServiceEvent {
    Write { line: String },
    Add {
        thread_id: ThreadId,
        data_rx: Receiver<StdOutServiceEvent>,
        ack_tx: Sender<()>,
    },
    Remove { thread_id: ThreadId },
}
```

Same structural shape as StdInService. Key differences from
slice 1f-β-i:

| Aspect | StdInService (1f-β-i) | StdOutService (1f-β-ii) |
|---|---|---|
| Event variant 1 | `Read` (no payload) | `Write { line }` (carries `:wat::core::String`) |
| Reply-channel value | `Sender<wat::holon::HolonAST>` (parsed AST) | `Sender<wat::core::nil>` (ack only) |
| Routing entry field | `reply-tx` | `ack-tx` |
| Service owns | `IOReader` (fd 0) | `IOWriter` (fd 1) |
| IO primitive | `IOReader/read-line` | `IOWriter/writeln` |

Everything else identical: single homogeneous `Receiver<Event>`
select set; `Vector<RoutingEntry>` routing (substrate-select
requires stable index order; HashMap semantics documented in the
conceptual typealias per slice 1f-β-i precedent); TCO loop; no
relay sub-thread; scope-drop shutdown.

## Mission

Create:
- `wat/kernel/services/stdout.wat` — the wat StdOutService program
- `wat-tests/kernel/services/stdout.wat` — hermetic deftests
  (architecturally-blocked per slice 1f-β-i's § Row K — but
  authored now so sonnet doesn't re-author them later)
- `src/stdlib.rs` registration entry mirroring slice 1f-β-i's
  pattern at `:85-93`

The service:
1. `(:wat::kernel::services::StdOutService::spawn writer)`
   creates the service program; returns
   `(Thread<nil,nil>, ControlTx)`
2. Driver loop owns an `IOWriter` (passed in at spawn) + guards
   `Vector<(ThreadId, EventRx, Sender<nil>)>`
3. Each iteration: build select-set from
   `(routing-rxs ++ [control-rx])`; all `Receiver<Event>`
4. On any fire, recv → match Event variant:
   - `Event::Write { line }` → `(:wat::io::IOWriter/writeln
     writer line)`; send `()` via the matched ack-tx (looked up
     by select index → routing key)
   - `Event::Add` → conj routing entry; recurse
   - `Event::Remove` → filter routing entry by thread-id; recurse
5. Shutdown via scope-drop (no explicit exit message)

## Required wat declarations

### `:wat::kernel::services::StdOutService::Event` enum

```
(:wat::core::enum :wat::kernel::services::StdOutService::Event
  (Write (line :wat::core::String))
  (Add
    (thread-id :wat::kernel::ThreadId)
    (data-rx :wat::kernel::Receiver<wat::kernel::services::StdOutService::Event>)
    (ack-tx :wat::kernel::Sender<wat::core::nil>))
  (Remove
    (thread-id :wat::kernel::ThreadId)))
```

### Channel typealiases (mirror slice 1f-β-i's pattern)

```
(:wat::core::typealias :wat::kernel::services::StdOutService::EventTx
  :wat::kernel::Sender<wat::kernel::services::StdOutService::Event>)
(:wat::core::typealias :wat::kernel::services::StdOutService::EventRx
  :wat::kernel::Receiver<wat::kernel::services::StdOutService::Event>)
(:wat::core::typealias :wat::kernel::services::StdOutService::Routing
  :wat::core::HashMap<wat::kernel::ThreadId,
                      :(wat::kernel::services::StdOutService::EventRx,
                        wat::kernel::Sender<wat::core::nil>)>)
(:wat::core::typealias :wat::kernel::services::StdOutService::Spawn
  :(wat::kernel::Thread<wat::core::nil, wat::core::nil>,
    wat::kernel::services::StdOutService::EventTx))
```

The conceptual `Routing` typealias documents HashMap intent per
the BRIEF; the driver state is `Vector<RoutingEntry>` for
stable select-index ordering (same honest-delta as slice
1f-β-i; document inline at the top of stdout.wat).

### `spawn` fn

Returns `(Thread, ControlTx)`. Internally creates a control
channel, calls `:wat::kernel::spawn-thread` with a closure
capturing the writer + control-rx + initial empty routing.

### TCO driver loop

Mirror slice 1f-β-i's shape exactly. The substantive change is
the Write-arm body: instead of `read-line` + `edn::read` +
`HolonAST`-send, it's `IOWriter/writeln` + `()`-send.

## Tests — `wat-tests/kernel/services/stdout.wat`

Mirror slice 1f-β-i's test shape. Author 4-5 hermetic test rows:

| Row | What |
|-----|------|
| A | spawn returns `(Thread, ControlTx)` of expected types |
| B | Add event registers a thread; Write roundtrip produces expected bytes via `IOWriter/new` + `IOWriter/snapshot` |
| C | Remove event drops a thread; subsequent Writes to that thread receive no ack |
| D | Multi-thread routing — N threads each get their Write echoed to the writer's buffer |
| E | scope-drop shutdown — every Sender drops → service `Thread/join-result` returns `Ok` |

**Note:** Per slice 1f-β-i's § Row K, hermetic tests can't
currently exercise the service because `deftest-hermetic`
expands to the phase-B-pending `run-sandboxed-hermetic-ast`.
The tests are authored now (so the pattern is captured); they
will run green once slices 1f-β-iii/γ/δ ship and
`deftest-hermetic` migrates to `spawn-process`. Document this
status inline in the test file's header comment.

## Stdlib registration

Add to `src/stdlib.rs` immediately after the stdin.wat entry
(currently at `:85-93`):

```rust
// Arc 170 slice 1f-β-ii — `:wat::kernel::services::StdOutService::*`
// (wat-side StdOutService program; mirrors StdOutServiceEvent
// from src/thread_io.rs).
WatSource {
    path: "wat/kernel/services/stdout.wat",
    source: include_str!("../wat/kernel/services/stdout.wat"),
},
```

Loading order: after `wat/kernel/channel.wat`; before any
consumer.

## What to NOT do

- No StdErrService work — slice 1f-β-iii
- No spawn-thread integration — slice 1f-γ
- No wat-cli boot — slice 1f-δ
- No Console retirement — slice 1f-ε
- No Console.wat edits
- No `deftest-hermetic` migration — architecturally downstream
  of slices β-iii/γ/δ
- No new Mutex / RwLock / CondVar
- No new dependencies

## Substrate-grep citations (verified pre-flight)

- `src/thread_io.rs:43-56` — concrete `StdOutServiceEvent`
- `src/io.rs:1078` — `:wat::io::IOWriter/writeln`
- `src/io.rs:877` — `:wat::io::IOWriter/new` (for test fixtures)
- `src/io.rs:413` — `IOWriter::snapshot` (for test assertions)
- `wat/console.wat:109+132` — IOWriter routing precedent
- `wat/kernel/services/stdin.wat` — pattern source (committed
  at `e898c7a`)
- `src/stdlib.rs:85-93` — stdin.wat registration; mirror for
  stdout.wat
- `crates/wat-lru/wat/lru/CacheService.wat:86` — `:wat::core::enum`
  form precedent

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `wat/kernel/services/stdout.wat` exists, parses, type-checks | `cargo check --release` green |
| B | `Event` enum declared with Write/Add/Remove variants | grep finds |
| C | Channel typealiases declared (EventTx/EventRx/Routing/Spawn) | grep finds |
| D | `spawn` fn defined, returns tuple | fn signature matches |
| E | TCO driver loop with Vector routing | recursive loop fn present |
| F | `:wat::kernel::select` used over routing values + control-rx | grep finds select call site |
| G | `Event::Add` / `Event::Remove` mutate routing via conj/filter | grep finds |
| H | `Event::Write` triggers `IOWriter/writeln` + ack-tx send | grep finds the path |
| I | `wat-tests/kernel/services/stdout.wat` exists with ≥ 4 deftest-hermetic rows | grep + cargo test --list |
| J | Workspace within ±5 of post-1f-β-i baseline (1339/859) — new failures match new test count | cargo test count |
| K | `cargo check --release` green | no errors |
| L | Only 3 files modified: 2 new wat files + `src/stdlib.rs` | git diff --stat ≤ 3 |
| M | Zero new dependencies | Cargo.toml unchanged |
| N | Zero new Mutex / RwLock / CondVar | grep clean |
| O | Honest deltas surfaced | per FM 5 |

**15 rows.** Row K (deftest-hermetic tests passing) is NOT
required — same § Row K out-of-scope rationale as slice 1f-β-i.

## Honest delta categories (anticipated)

- **HashMap → Vector routing-state**: same as slice 1f-β-i;
  document inline; reuse rationale.
- **ack-tx zero-payload send**: confirm `(:wat::kernel::send
  ack-tx ())` is the correct shape for `Sender<wat::core::nil>`.
  Adapt if substrate prefers a different unit-send idiom.
- **IOWriter/writeln vs write-string + newline**: `writeln`
  appends the newline; preferred. If a test fixture needs
  newline-free output, `write-string` is the alternate; surface
  if friction arises.

## Predicted runtime

30-45 min sonnet. Pattern is well-trodden post-1f-β-i:
- Event enum shape: mechanical mirror
- Channel typealiases: search-and-replace from stdin.wat
- spawn / driver-loop: structural copy with one verb swap
- Tests: structural copy with one assertion swap

**Hard cap:** 90 min (2× upper bound).

## Reference

- BRIEF V2 source pattern:
  `BRIEF-SLICE-1F-B-I-V2.md` (the predecessor's BRIEF — sonnet
  should consult for typealias-family shape)
- Pattern source on disk:
  `wat/kernel/services/stdin.wat` (committed at `e898c7a`)
- Score on the pattern:
  `SCORE-SLICE-1F-B-I-V2.md` (15/17; honest-delta categories
  documented)
- Architecture: REALIZATIONS pass 18 (unified Event protocol);
  TIERS.md § OS-boundary handling

## Path forward post-slice-1f-β-ii

1. Orchestrator scores; commits atomic-pair (deliverable + SCORE)
2. Slice 1f-β-iii BRIEF — StdErrService (third and final
   service mint; pattern-apply mirror)
3. Slice 1f-γ — runtime orchestrator (substrate Rust work)
4. Slice 1f-δ — boot integration + shutdown cascade
5. Adjacent to 1f-δ INSCRIPTION: `deftest-hermetic` migrates to
   `spawn-process`; all 859 baseline failures resolve
