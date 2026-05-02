# Arc 119 — Sonnet Brief: Consumer Sweep (step 7)

**Status:** durable record of the brief sent to sonnet for arc
119's consumer sweep (step 7 of the execution checklist). Same
brief stays in this file as the reference for re-attempts and
post-mortems.

## Provenance

After steps 2-5 (LRU + HolonLRU substrate reshapes) shipped via
`BRIEF-LRU-RESHAPE.md` and `BRIEF-HOLON-LRU-RESHAPE.md`, step 6
captured the workspace test baseline:

- 1475 passed / 5 failed / 0 ignored
- 5 failing tests are wat-tests using the OLD single-item API:
  - 1 in `crates/wat-lru/wat-tests/lru/CacheService.wat`
  - 4 in `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`

The substrate-as-teacher diagnostic stream reports for each
failing test — wat-tests are calling
`(Request::Put k v)` (single-item) where the substrate now
expects `(Request::Put entries ack-tx)` (batch + ack-tx).

Step 7's job is to update the wat-tests to use the new batch
shape. After step 7 ships green, step 8 (closure paperwork)
follows.

## Goal: 1480 / 0 / 0

Make `cargo test --release --workspace` from `/home/watmin/work/holon/wat-rs/` report:

- 1480 passed
- 0 failed
- 0 ignored

That's `1475 + 5` (the 5 currently-failing wat-tests, fixed).

## Scope (workspace boundary discipline)

**In scope:**
- `crates/wat-lru/wat-tests/lru/CacheService.wat` — 1 failing test
- `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` — 4 failing tests

**Explicitly OUT OF SCOPE:**
- `holon-lab-trading/wat/cache/L2-spawn.wat` (or anything in
  `holon-lab-trading/`) — separate workspace, separate session,
  separate downstream arc.
- Any other crate or file the failing tests do NOT touch.

If you find yourself editing a file outside the two paths
above, STOP and report. The lab gets fixed in a future session
once it picks up against the new substrate.

## Anchor docs (read in order)

1. `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` — the
   locked target shape (especially "The fix — symmetric batch
   protocol" + "Substrate work scope").
2. `docs/arc/2026/04/119-holon-lru-put-ack/BRIEF-LRU-RESHAPE.md`
   — the disciplines (send pattern, recv pattern, inner-colon
   antipattern). Apply them in the consumer-side too.
3. `crates/wat-lru/wat/lru/CacheService.wat` (post-reshape; at
   HEAD) — the canonical reference for the new API. Read its
   `:wat::lru::get` and `:wat::lru::put` verb signatures and the
   typealiases section.
4. `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
   (post-reshape; at HEAD) — the HolonLRU equivalent.

## Substrate-as-teacher: read the diagnostic first

Before editing, run:

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release -p wat-lru 2>&1 | grep -E "FAILED|failure:" | head -30
cargo test --release -p wat-holon-lru 2>&1 | grep -E "FAILED|failure:" | head -50
```

The substrate's type-check + assert-eq output IS the brief for
each test. Each failure tells you exactly what shape the test
is calling and what shape it should call. Use those messages to
guide your edits — don't pre-decide what to write before
seeing what the substrate is teaching.

## Required disciplines

Identical to BRIEF-LRU-RESHAPE.md §§ "Required disciplines".
Read them. The three call patterns to use:

### 1. Send pattern — one Result/expect, no Option layer

```scheme
((_send :wat::core::unit)
 (:wat::core::Result/expect -> :wat::core::unit
   (:wat::kernel::send tx val)
   "X: tx disconnected — peer died?"))
```

### 2. Recv pattern — Option/expect WRAPPING Result/expect

```scheme
(:wat::core::Option/expect -> :T
  (:wat::core::Result/expect -> :wat::core::Option<T>
    (:wat::kernel::recv rx)
    "X: rx peer died — protocol violation")
  "X: rx channel closed — peer dropped tx?")
```

For batch get's reply, `T = wat::core::Vector<wat::core::Option<V>>`.

### 3. Inner-colon antipattern (arc 115)

`Vector<Option<(K,V)>>` correct; `Vector<Option<:(K,V)>>` illegal.

## What the call-site shape changes look like

For a wat-test that previously called `(get req-tx reply-tx
reply-rx k)` to look up a single key, the new shape is:

```scheme
;; OLD (single-item):
(:wat::lru::get req-tx reply-tx reply-rx some-key)
;; → returns :Option<V>

;; NEW (batch-of-one):
(:wat::lru::get req-tx reply-tx reply-rx (:wat::core::vec some-key))
;; → returns :Vector<Option<V>>
;; pull the single result out: (:wat::core::first results) gives Option<V>
```

For Put, the changes are bigger because Put now takes ack-tx /
ack-rx (PutAck family) instead of reply-tx / reply-rx (Reply
family):

```scheme
;; OLD (single-item, reply-tx for unit reply):
(:wat::lru::put req-tx reply-tx reply-rx some-key some-val)
;; → returns :unit

;; NEW (batch-of-one, ack-tx family):
;; 1. Allocate the PutAckChannel near where ReplyChannel was allocated:
(:wat::core::let*
  (((ack-pair :wat::lru::PutAckChannel)
    (:wat::kernel::make-bounded-channel :wat::core::unit 1))
   ((ack-tx :wat::lru::PutAckTx) (:wat::core::first ack-pair))
   ((ack-rx :wat::lru::PutAckRx) (:wat::core::second ack-pair))
   ;; 2. Build the entries vec:
   ((entries :wat::core::Vector<wat::lru::Entry<K,V>>)
    (:wat::core::vec (:wat::core::Tuple some-key some-val))))
   ;; 3. Call put with the new signature:
   ((_unit :wat::core::unit)
    (:wat::lru::put req-tx ack-tx ack-rx entries)))
  ...)
```

For HolonLRU the call shape is the same with the
`HologramCacheService/` prefix on each verb and `HolonAST` as
both K and V.

Work from the diagnostic stream to know exactly which sites to
update; the patterns above are templates, not blanket rewrites.

## Validation gate

After editing, run:

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace 2>&1 | grep -E "^test result" \
  | grep -oE "[0-9]+ (passed|failed|ignored)" | sort | uniq -c
```

Target: `1480 passed`, `0 failed`, `0 ignored`.

If any test still fails, the substrate-as-teacher diagnostic
will tell you exactly which one and why. Iterate until green.

## Constraints

- Edit ONLY `crates/wat-lru/wat-tests/lru/CacheService.wat` and
  `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`.
- Do NOT touch substrate files
  (`crates/*/wat/**/*.wat`) — those are step 2-5's territory and
  shipped.
- Do NOT touch `holon-lab-trading/` — separate workspace,
  separate arc.
- Do NOT touch `src/*.rs`, `Cargo.toml`, or any docs.
- Do NOT add new tests; the existing tests should be updated to
  use the new API.

## Reporting back

When done (or blocked):
1. `git status --short` — file list (should show ONLY the two
   wat-tests files modified)
2. `git diff --stat` — line counts
3. `cargo test --release --workspace` final outcome — show the
   final summary line(s) confirming 1480 / 0 / 0
4. Any judgment calls beyond this brief

## Working directory

`/home/watmin/work/holon/wat-rs/`. All cargo commands work from
there directly.

## Cross-references

- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md` — locked
  target shape; execution checklist.
- `docs/arc/2026/04/119-holon-lru-put-ack/BRIEF-LRU-RESHAPE.md`
  — full disciplines + send/recv patterns.
- `docs/arc/2026/04/119-holon-lru-put-ack/BRIEF-HOLON-LRU-RESHAPE.md`
  — HolonLRU-specific differences.
- `crates/wat-lru/wat/lru/CacheService.wat` — substrate
  reference for the new API (HEAD).
- `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
  — HolonLRU substrate reference (HEAD).
