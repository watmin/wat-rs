# Arc 130 — HolonLRU Substrate Reshape BRIEF (slice 2 substrate side)

**Drafted 2026-05-06.** Sweep 2a of arc 130's HolonLRU cleanup
trajectory. Slice 1 (wat-lru substrate reshape) shipped 2026-05-01,
its consumer sweep (Vector/length) shipped this morning, its test
file rebuild shipped this afternoon — all Mode A clean, workspace
at 0 failed. Now: HolonLRU substrate reshape per the same pattern.

User direction 2026-05-06: "lets get holon-lru cleaned up." Per
the four questions, sequential is more obvious/simple/honest than
bundled — this brief is sweep 2a (substrate-only); sweep 2b (test
rebuild + retire `:should-panic` annotations) follows immediately
after, with both committed atomically per the no-broken-commits
discipline.

## Goal

Mirror wat-lru's post-arc-130 substrate pattern in HolonLRU's
substrate file. Introduce `Handle`, `DriverPair`, unified `Reply`
enum + helper-verb signature changes; retire the old per-verb
channel families (`GetReply*`, `PutAck*`); reshape spawn factory
+ driver loop to pair-by-index discipline.

## Substrate evidence (verified pre-brief)

**Template** — `crates/wat-lru/wat/lru/CacheService.wat` (post-arc-130):
- `Reply<V>` enum unifying GetResult + PutAck (line 57)
- `Handle<K,V> = (ReqTx<K,V>, ReplyRx<V>)` (line 74)
- `DriverPair<K,V> = (ReqRx<K,V>, ReplyTx<V>)` (line 80)
- spawn factory pre-allocates N pairs; HandlePool<Handle<K,V>>
- driver loop selects at index → replies via DriverPair at same index

**Target** — `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat` (current OLD shape):
- Per-verb channel families:
  - `GetReplyTx`, `GetReplyRx`, `GetReplyPair` (lines 75-82)
  - `PutAckTx`, `PutAckRx`, `PutAckChannel` (lines 63-68)
- Helper verbs take 3 channel ends:
  - `HologramCacheService/get(req-tx, reply-tx, reply-rx, probes)` (lines 492-495)
  - `HologramCacheService/put(req-tx, ack-tx, ack-rx, entries)` (lines 516-519)
- spawn factory does per-call allocation in consumers (current driver of arc 126's check)

**Critical naming difference vs wat-lru:**

HolonLRU is **concrete** — `K = V = :wat::holon::HolonAST` throughout
(per substrate header comment line 20: *"K = V =
:wat::holon::HolonAST throughout (concrete, not parametric)"*).
This means the new typealiases do NOT carry type parameters:

- wat-lru: `:wat::lru::Handle<K,V>`
- HolonLRU: `:wat::holon::lru::HologramCacheService::Handle` (no `<K,V>`)

Substrate naming pattern remains the same (Handle / DriverPair /
Reply / ReplyTx / ReplyRx / ReplyChannel); just no parametric heads.

## What to do

### Pre-flight crawl (mandatory before editing)

1. **Read wat-lru's CacheService.wat in full** — your template.
   Pay attention to:
   - Typealias section structure (lines 50-110)
   - Reply<V> enum shape (line 57)
   - Handle<K,V>, DriverPair<K,V> typealiases
   - spawn factory (whole body)
   - driver loop / loop-step (the select + reply path)
   - Helper verbs `:wat::lru::get`, `:wat::lru::put` (lines around 480-560 in CacheService.wat — they take Handle)
2. **Read HolonLRU's HologramCacheService.wat in full** — your target.
   Identify:
   - All `GetReply*` typealiases to retire
   - All `PutAck*` typealiases to retire
   - spawn factory's old shape
   - driver loop's old shape
   - Helper verbs' old 3-channel signatures
3. **Read wat-lru's INSCRIPTION.md** if it exists, plus
   `docs/arc/2026/05/130-cache-services-pair-by-index/SCORE-SUBSTRATE-CONSUMER-SWEEP.md`
   + `SCORE-TEST-FILE-REBUILD.md` for cascade context.

### The reshape (substrate-only edits)

In `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`:

#### Section A — Typealiases

Retire (DELETE):
- `PutAckTx`, `PutAckRx`, `PutAckChannel` (lines ~63-68)
- `GetReplyTx`, `GetReplyRx`, `GetReplyPair` (lines ~75-82)

Introduce (ADD), mirroring wat-lru's pattern but CONCRETE (no `<V>`):
- `Reply` enum with two variants: `GetResult` carrying `:wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>`; `PutAck` carrying nothing (unit variant)
- `ReplyTx` typealias = `:wat::kernel::Sender<wat::holon::lru::HologramCacheService::Reply>`
- `ReplyRx` typealias = `:wat::kernel::Receiver<...::Reply>`
- `ReplyChannel` typealias = `(ReplyTx, ReplyRx)` (the pair as a tuple)
- `Handle` typealias = `(ReqTx, ReplyRx)` — the client view
- `DriverPair` typealias = `(ReqRx, ReplyTx)` — the driver view

The `ReqTx` / `ReqRx` / `ReqChannel` typealiases (if they exist; check) keep their shape — only the reply side changes.

#### Section B — Spawn factory

The current spawn factory needs to:
- Pre-allocate `count` ReplyChannels at startup
- Build a HandlePool<Handle> from the `count` (ReqTx, ReplyRx) pairs
- Build the driver's view as `Vector<DriverPair>` from the `count` (ReqRx, ReplyTx) pairs
- Driver thread takes its DriverPair vector + the LocalCache + reporter + cadence + the request channels' ReqRx side via the DriverPair list

Mirror wat-lru's spawn factory body verbatim where structure permits;
adapt for HolonLRU's concrete typing.

Spawn returns the same shape as wat-lru: `(HandlePool<Handle>,
Thread<unit,unit>)`.

#### Section C — Driver loop

The driver loop's select fires at index `i`. The DriverPair vector
provides the matching `ReplyTx` at index `i`. After processing the
Request, send the appropriate Reply variant on that ReplyTx:
- `Request::Get(probes)` → process → send `Reply::GetResult(results)`
- `Request::Put(entries)` → process → send `Reply::PutAck`

Mirror wat-lru's loop-step body verbatim where structure permits.

#### Section D — Helper verbs

Replace the 3-channel-end helper signatures with Handle-taking
signatures. Helpers do send-AND-recv internally per arc 110's contract:

```scheme
;; OLD (retire):
(:wat::core::define
  (:wat::holon::lru::HologramCacheService/get
    (req-tx :ReqTx) (reply-tx :GetReplyTx) (reply-rx :GetReplyRx)
    (probes :Vector<HolonAST>) -> :Vector<Option<HolonAST>>) ...)

;; NEW (introduce):
(:wat::core::define
  (:wat::holon::lru::HologramCacheService/get
    (handle :Handle) (probes :Vector<HolonAST>)
    -> :Vector<Option<HolonAST>>)
  (:wat::core::let*
    (((req-tx ...) (:wat::core::first handle))
     ((reply-rx ...) (:wat::core::second handle))
     ;; send Request::Get; recv Reply::GetResult; extract results)
    ...))
```

Same shape for `put`: takes Handle + entries; sends Request::Put;
recvs Reply::PutAck. Both helpers do the panic-loud Result/expect
+ Option/expect dance per arc 110 (mirror wat-lru).

### Verification

This sweep's substrate change will BREAK consumer tests by design —
the OLD helper verb signatures don't match the NEW signatures.
The 9 `:should-panic("channel-pair-deadlock")` tests will start
failing with TYPE-ERROR-shaped panics (substring won't match
"channel-pair-deadlock" anymore) instead of passing.

**This is expected.** Sweep 2b (next brief, draft after this one
returns) wipes + rebuilds the test file using the new substrate
shape, and retires the `:should-panic` annotations.

For THIS sweep, verify:
1. **Substrate file parses cleanly** — `cargo test --release -p wat-holon-lru --test test 2>&1 | grep -E "parse|syntax"` should be empty (substrate doesn't have syntactic errors)
2. **Substrate file type-checks AGAINST ITSELF** — the substrate's internal helper verbs unify with the new typealiases; no type errors INSIDE the substrate file
3. **Consumer test failures are TYPE ERRORS** (or "unknown function" errors), NOT something else — verifies the reshape is structurally valid; the consumers just haven't caught up yet

Run `cargo test --release -p wat-holon-lru --test test` and report
the failure shape. If failures are TYPE-MISMATCH style (expected),
substrate sweep is clean. If failures are PARSE / unexpected
substrate errors, STOP and report — substrate edits introduced a
new bug.

## Constraints

- **Substrate-only edits.** ONLY 1 file modified:
  `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`.
  NO test-file edits. NO Rust source edits. NO other crate.
- **Do NOT modify consumer tests.** The 9 `:should-panic` tests
  + the proof at arc-119 step-B will all fail post-reshape.
  Sweep 2b fixes them. Leave them alone.
- **Do NOT commit, do NOT push.** Working tree stays modified;
  orchestrator commits sweep 2a + sweep 2b atomically AFTER
  sweep 2b's tests verify workspace clean.
- **STOP at first red, but distinguish red types.** A red in
  CONSUMER tests is expected (workspace dirty between 2a and 2b).
  A red in SUBSTRATE compilation (parse / check errors inside
  HologramCacheService.wat itself) is unexpected — STOP + report.
- **No grinding.** Mirror the wat-lru template; the architecture
  is settled. If you find a HolonLRU-specific edge case the
  template doesn't cover, surface it; don't invent novel substrate.

## Out of scope

- Test-file rebuild (sweep 2b)
- `:should-panic` retirement (sweep 2b)
- arc-119 proof retirement (sweep 2b)
- Slice 1 INSCRIPTION (later)
- Arc 130 INSCRIPTION (later)
- Any consumer code in lab repos

## Reporting

Target ~250 words:

1. **Pre-flight crawl confirmation:** wat-lru template read; HolonLRU
   target read; cascade SCOREs read.

2. **Section-by-section edit summary:**
   - Section A: typealiases — N retired, M added; brief diff stat
   - Section B: spawn factory — body rewritten; what changed
   - Section C: driver loop — body rewritten; what changed
   - Section D: helper verbs — get + put signatures changed
3. **File LOC delta:** before / after.

4. **Verification:**
   - Substrate parses cleanly (yes/no)
   - Substrate self-types cleanly (yes/no)
   - Consumer test failure shape (type-mismatch ✓ / unexpected ✗)

5. **Path:** Mode A clean (substrate reshape ships; consumer
   failures expected) / Mode B substrate-internal-bug (substrate
   itself has issues — STOP) / Mode C unexpected-failure-shape
   (consumer fails for a reason that's NOT type mismatch).

6. **Honest deltas:** any HolonLRU-specific divergence from
   wat-lru's template + reasoning.

## What success looks like

**Mode A clean ship**: Substrate reshape lands; substrate file
parses + self-types; consumer tests fail with type-mismatch
errors (expected); ready for sweep 2b to rebuild the consumer
tests.

**Mode B**: Substrate has internal bug; STOP + report; orchestrator
adjusts brief.

**Mode C**: Consumer fails for unexpected reason; surface it.

## Why this brief matters for the cooperation

User direction 2026-05-06 set the trajectory: cleanup HolonLRU.
Per the four questions, sequential is more obvious/simple/honest
than bundled — this brief is the substrate-side sweep; sweep 2b
follows.

The mutual-agreement chain:
- User → Orchestrator: "lets get holon-lru cleaned up"
- Orchestrator → Sonnet (this brief): substrate reshape mirroring
  wat-lru's pattern; substrate-only edits
- Sonnet → Reality: substrate file ships in new shape; consumer
  tests fail by design; sweep 2b queued

Mode A clean = the substrate reshape pattern propagates from
wat-lru to HolonLRU as expected; sweep 2b can proceed.
