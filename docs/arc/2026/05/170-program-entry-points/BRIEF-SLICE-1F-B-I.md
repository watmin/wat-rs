# Arc 170 slice 1f-β-i — BRIEF

**Substrate; opus + wat-author.** Mint the FIRST of three
wat-side service implementations: `StdInService`. This stone
mints the dynamic-membership service pattern; slices 1f-β-ii
(StdOutService) and 1f-β-iii (StdErrService) apply the pattern
mechanically.

Architecture lock-in: see REALIZATIONS-SLICE-1.md passes 15 +
16 + 17 and BUILD-PLAN.md § Slice 1f-β.

## Mission

Create:
- `wat/kernel/services/stdin.wat` — the StdInService wat program
- `wat-tests/kernel/services/stdin.wat` — hermetic deftests
  exercising the protocol
- Register both in `src/stdlib.rs` (path + include_str! entries)

The service:
1. Owns an `:wat::io::IOReader` for fd 0 (passed in at spawn)
2. Guards a `HashMap<ThreadId, (req-rx, reply-tx)>` routing table
3. Loops via TCO over `(values routing-table) + control-pipe`
   via `:wat::kernel::select`
4. On a thread's req-rx firing: read next line from fd 0, parse
   via `:wat::edn::read` to HolonAST, send via that thread's
   reply-tx
5. On control-pipe firing: match `Signal::add` /
   `Signal::remove`, mutate routing table via `assoc` / `dissoc`,
   recurse with new state
6. Exits via scope-drop: when all control-pipe senders drop →
   recv returns Disconnected → TCO bottoms out → service exits

This matches pass 16's locked protocol shape verbatim.

## Required wat-side declarations

### `:wat::kernel::ThreadId` (mint as typealias)

The substrate has no ThreadId type yet. Mint as:

```
(:wat::core::typealias :wat::kernel::ThreadId
  :wat::core::i64)
```

Reasoning: a monotonic counter assigned by the runtime (slice
1f-γ) per spawn-thread cycle. Identity is structural (the
runtime's ledger). Newtype-vs-typealias: typealias is simpler;
no constructor ceremony; reads cleanly at every call site.
Surface as honest delta if newtype reads better at the user
sites you're about to write.

Place this declaration in `wat/kernel/services/stdin.wat`
(first ServiceId-using slice; subsequent stones use it via
already-loaded typealias). Reasonable alternative: a new
`wat/kernel/types.wat` if multiple kernel modules need
ThreadId. Surface the choice.

### `:wat::kernel::services::StdInService::Signal` (the locked enum)

```
(:wat::core::enum :wat::kernel::services::StdInService::Signal
  (add    (thread-id :wat::kernel::ThreadId)
          (req-rx    :wat::kernel::Receiver<wat::core::nil>)
          (reply-tx  :wat::kernel::Sender<wat::holon::HolonAST>))
  (remove (thread-id :wat::kernel::ThreadId)))
```

Per pass 16 protocol settlement. Two variants only — `add` and
`remove`. No `replace`/`reload`/`sighup` — shutdown is via
scope-drop per SERVICE-PROGRAMS.md.

### Typealiases for the routing-table entry + control-pipe family

Following the channel-naming convention from `wat/console.wat`
(ReqTx / ReqRx / ReqChannel pattern), declare per-purpose
aliases. Example sketch:

```
;; The per-thread channel halves the service stores
(:wat::core::typealias :wat::kernel::services::StdInService::ThreadReq
  :(wat::kernel::Receiver<wat::core::nil>,
    wat::kernel::Sender<wat::holon::HolonAST>))

;; The routing table type
(:wat::core::typealias :wat::kernel::services::StdInService::Routing
  :wat::core::HashMap<wat::kernel::ThreadId,
                      wat::kernel::services::StdInService::ThreadReq>)

;; The control pipe shape
(:wat::core::typealias :wat::kernel::services::StdInService::ControlRx
  :wat::kernel::Receiver<wat::kernel::services::StdInService::Signal>)
(:wat::core::typealias :wat::kernel::services::StdInService::ControlTx
  :wat::kernel::Sender<wat::kernel::services::StdInService::Signal>)
```

Adjust names if a cleaner shape emerges; surface as honest
delta.

### `:wat::kernel::services::StdInService::spawn`

The spawn fn (called by the runtime in slice 1f-γ; called by
tests directly in this slice):

```
(:wat::core::define
  (:wat::kernel::services::StdInService::spawn
    (reader :wat::io::IOReader)
    -> :(wat::kernel::Thread<wat::core::nil, wat::core::nil>,
         wat::kernel::services::StdInService::ControlTx))
  ...)
```

Returns a tuple: (ProgramHandle, control-pipe-tx). Caller (the
runtime OR the test harness) holds the ControlTx in inner
scope; the Thread handle goes in outer scope per the
SERVICE-PROGRAMS.md lockstep.

Internally, spawn creates the control-pipe channel and invokes
`:wat::kernel::spawn-thread` with a closure that captures the
reader + control-rx and runs the driver loop.

## The driver loop — canonical TCO pattern

Mirror `wat-tests/service-template.wat`'s structure (per-variant
dispatch + select-over-Vec) BUT replace the static Vec<ReqRx>
with the HashMap routing table and ADD the control-pipe handler.

Sketch:

```
(:wat::core::define
  (:wat::kernel::services::StdInService::loop
    (reader  :wat::io::IOReader)
    (control-rx :wat::kernel::services::StdInService::ControlRx)
    (routing :wat::kernel::services::StdInService::Routing)
    -> :wat::core::nil)
  (let [select-set (build-select-set routing control-rx)]
    (match (:wat::kernel::select select-set)
      ((<routing-entry fired>)
        ;; read next line; parse; route via reply-tx
        ;; recurse with same routing
        ...)
      ((<control-rx fired>)
        ;; recv Signal; assoc/dissoc; recurse with new routing
        ...))))
```

Use `:wat::kernel::select` directly. Per pass 16: the routing
table is `HashMap<ThreadId, ThreadReq>`; each iteration of the
loop rebuilds the select set from current values + control-rx.
TCO recurses with the mutated routing table.

If the substrate's `select` doesn't accept a heterogeneous
selector set (HashMap values + control-rx) cleanly, surface the
shape friction as honest delta — DON'T expand scope unilaterally
to add new substrate primitives.

## Stdin-specific behavior

Per pass 16: the StdInService is REQUEST-RESPONSE.
1. Service blocks in select; one of the routing entries' req-rx
   fires (a thread called readln + sent `()` request)
2. Service reads next line from fd 0 via
   `:wat::io::IOReader/read-line`
3. Service parses via `:wat::edn::read` to HolonAST (or an
   appropriate read primitive — verify in substrate)
4. Service sends parsed HolonAST via that thread's reply-tx
5. Recurses with same routing

If multiple threads have outstanding readln requests at once,
select picks one; first-served. Per slice 1f-α's discipline,
the loop is sequential — no concurrent reads of fd 0.

## Test approach — `wat-tests/kernel/services/stdin.wat`

Use `:wat::test::deftest-hermetic` for full isolation (each
test runs in its own forked process — kernel-isolated; see
pass 17 + arc 124).

### Required test rows

| Row | Verifies |
|-----|----------|
| A — spawn returns valid handle + control-tx | call spawn with a mock IOReader; assert types |
| B — Signal::add registers a thread (single client) | spawn service; send Signal::add with mock channels; verify by sending a readln request through the registered thread's pair |
| C — full readln roundtrip (single client) | service reads from a test-controlled IOReader containing `"42\n"`; the readln request returns the parsed HolonAST for 42 |
| D — Signal::remove drops a thread | register 2 threads; send Signal::remove for one; verify the removed thread's req-tx → service produces no reply (Disconnected on reply-rx OR no message arrives within timeout) |
| E — scope-drop shutdown | drop all control-pipe Senders + thread Senders; verify service Thread/join-result returns Ok cleanly |
| F — multi-client routing (2+ threads) | register N threads; each sends a readln request; service reads N lines from IOReader; each thread gets its own line (per first-served ordering OR however the substrate's select arbitrates) |

Skip first-panic semantics for stderr until 1f-β-iii.

### Test harness pattern

Each hermetic deftest:
1. Set up a mock IOReader (probably via `:wat::io::TempFile` or
   an in-memory reader pattern — find the existing precedent)
2. Spawn the service via `StdInService::spawn`
3. Allocate per-test channel pairs (req-tx, req-rx + reply-tx,
   reply-rx)
4. Send Signal::add to control-pipe to register the test's
   thread-id
5. Send `()` on req-tx (the readln request)
6. recv on reply-rx — assert returned HolonAST matches
7. Drop test scope → all Senders drop → service exits

If `:wat::io::TempFile` + IOReader isn't the right tooling,
surface as honest delta + find the canonical in-memory IOReader
pattern.

## `src/stdlib.rs` registration

Two new entries, modeled on the existing `wat/kernel/channel.wat`
entry at `src/stdlib.rs:82`:

```rust
SystemSource {
    path: "wat/kernel/services/stdin.wat",
    source: include_str!("../wat/kernel/services/stdin.wat"),
},
```

Make sure the registration ORDER is correct: the file must load
AFTER any deps it references (e.g., `:wat::kernel::ThreadId`
typealias must register before StdInService uses it). If
ThreadId lives in stdin.wat itself, no ordering concern.

## What to NOT do

- **No slice 1f-β-ii / 1f-β-iii work.** This is the StdInService
  ONLY. StdOut/StdErr ship in subsequent stones using the
  pattern minted here.
- **No slice 1f-γ work.** Don't modify spawn-thread or any
  runtime orchestrator code. The service is callable; its
  population by runtime is 1f-γ's concern.
- **No first-panic / libc::exit semantics.** Those are stderr
  + slice 1i.
- **No Console retirement / migration.** Console stays for now;
  retires in 1f-ε.
- **No wat-cli edits.** wat-cli is the OS boundary per pass 17;
  service spawning belongs in the runtime (1f-γ).
- **No new substrate primitives** unless verified necessary (and
  even then, surface as honest delta + STOP — don't expand
  scope). The expected substrate primitives are present:
  select, send, recv, IOReader/read-line, edn::read, HashMap
  ops, spawn-thread.

## Substrate-grep citations (verify before committing)

- `:wat::core::enum` form — see `crates/wat-lru/wat/lru/CacheService.wat:86` (Request<K,V>)
- `:wat::core::typealias` form — see `wat/console.wat:38` (Message)
- `:wat::core::newtype` form (if used instead of typealias) — see `wat/edn.wat:30-31`
- `:wat::core::HashMap` + `assoc`/`dissoc`/`values` — `wat/core.wat:59-62`
- `:wat::kernel::select` — `src/runtime.rs:15293`, registered at `src/check.rs:12656`
- `:wat::kernel::spawn-thread` — verify in `src/check.rs` + `src/runtime.rs`
- `:wat::io::IOReader/read-line` — see `examples/interrogate/wat/main.wat:131`
- `:wat::edn::read` — `src/edn_shim.rs:191`, registered at `src/check.rs:12795`,
  dispatched at `src/runtime.rs:3503`
- Canonical service-template — `wat-tests/service-template.wat`
- stdlib registration pattern — `src/stdlib.rs:82` (wat/kernel/channel.wat)
- `:wat::test::deftest-hermetic` — see `crates/wat-macros/src/discover.rs:233`
  + `arc 124 INSCRIPTION`; routes through `spawn-process` (tier-2 fork)

## Honest delta categories — surface, don't work-around

- **ThreadId representation** — typealias vs newtype. Surface
  the call.
- **ThreadId placement** — in stdin.wat itself OR a new
  `wat/kernel/types.wat`. Surface the call.
- **Service-spawn signature shape** — does it take just
  `IOReader`, or include something else? Mirror Console::spawn
  shape; adapt for dynamic membership.
- **Routing-table type alias name** — `Routing` vs `Handles` vs
  something else. The four questions on naming.
- **`:wat::kernel::select` over heterogeneous selector set** —
  if select doesn't accept a mixed set of routing-table values
  + control-rx cleanly, surface. Don't expand substrate.
- **In-memory IOReader for tests** — find canonical pattern;
  surface if not obvious.
- **edn::read return type** — confirm it returns
  `wat::holon::HolonAST` directly (not wrapped in Tagged/NoTag);
  if it's wrapped, the service needs to unwrap before sending.
- **`:wat::kernel::ThreadId` exact spelling** — if your
  declared typealias name slightly diverges from
  `:wat::kernel::ThreadId`, surface so subsequent slices
  reference it correctly.

## Ship criteria

- `wat/kernel/services/stdin.wat` parses + type-checks
- `wat-tests/kernel/services/stdin.wat` cargo test green (≥ 4
  passing test rows; ideal all 6)
- Workspace cargo test fail count within ±5 of baseline (1327
  passed / 855 failed post-slice-1f-α)
- `cargo check --release` green
- Zero new dependencies; zero new Mutex / RwLock / CondVar
- `src/stdlib.rs` registration in place
- Slice 1f-α's substrate primitives untouched
- All 6+ test rows visible in `cargo test --list`

**Predicted runtime:** 60-90 min opus + wat-author. Pattern
minting + 1 service implementation + 1 test file +
stdlib.rs registration.

**Hard cap:** 180 min (3 hours). Wakeup scheduled.

## Reference

- DESIGN.md (passes 1-17)
- REALIZATIONS-SLICE-1.md § Pass 15, § Pass 16, § Pass 17
- BUILD-PLAN.md § Slice 1f-β
- ZERO-MUTEX.md § Tier 3 + § Mini-TCP + § Pair-by-index vs
  embedded reply-tx (note: dynamic-membership uses HashMap-by-id
  routing, an extension of these patterns)
- SERVICE-PROGRAMS.md § The lockstep
- `wat-tests/service-template.wat` — canonical service shape
- `wat/console.wat` — closest existing precedent (static-membership;
  this slice extends to dynamic-membership)
- `crates/wat-lru/wat/lru/CacheService.wat` — enum-based Request
  pattern + Reply<V> with mini-TCP discipline
- `src/thread_io.rs` — the substrate-side counterpart that
  defines what types the channel ends carry (slice 1f-α
  shipped at fcaf600)
