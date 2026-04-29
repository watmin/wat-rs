# Arc 081 — `:wat::std::telemetry::Console/dispatcher` — INSCRIPTION

**Status:** shipped 2026-04-29.

The substrate now ships a Console destination for arc 080's
Service<E,G>. **Not** a queue-fronted service of its own — a
dispatcher FACTORY: given a Console::Tx + a format choice, returns a
closure that renders any entry as a single line and sends it via the
existing tagged-stdout `Console/out` primitive.

The user's load-bearing directive shaped the shape:

> "no free form log lines... no rando (println! ...) bullshit.... the
> users must operate on data at all times"

Every line out of stdout is now structured EDN (or JSON), one entry
per line, written through a single uniform path. Existing `Console/err`
T1/T2/T3 stderr-checkpoint use cases are unaffected — they're at a
lower layer for raw diagnostic markers.

**Design:** [`DESIGN.md`](./DESIGN.md).

---

## What shipped from DESIGN's plan

DESIGN proposed Console as a queue-fronted service paralleling
Sqlite. **As-shipped is simpler:** Console is a dispatcher factory,
not its own service. Justification:

- A queue-fronted Console would have its own driver, its own select
  loop, its own HandlePool. All it would do is render-then-print —
  there's no stateful per-entry work.
- Composing instead means: arc 080's `Service<E,G>` provides the
  queue + driver + cadence; Console contributes the per-entry
  dispatcher closure. One factory; no parallel service plumbing.
- Per memory `feedback_verbose_is_honest`: the queue-fronted shape
  would have eliminated nothing while adding a parallel driver.
  Reject.

The result: Console is one factory function. Consumer-facing surface
is small.

---

## What shipped

### File: `wat/std/telemetry/Console.wat` (~70 lines)

```scheme
;; Format knob — picked once at factory-call time.
(:wat::core::enum :wat::std::telemetry::Console::Format
  :Edn       ;; render via :wat::edn::write (compact single-line)
  :Json      ;; render via :wat::edn::write-json (compact JSON)
  :Pretty)   ;; render via :wat::edn::write-pretty (multi-line)

;; The factory. Captures con-tx + format; returns the per-entry
;; dispatcher closure.
(:wat::std::telemetry::Console/dispatcher<E>
  (con-tx :wat::std::service::Console::Tx)
  (format :wat::std::telemetry::Console::Format)
  -> :fn(E)->())
```

The returned closure, on each entry:
1. Renders via the format-selected wat-edn primitive.
2. Appends a newline (one entry, one line).
3. Sends through `:wat::std::service::Console/out` (tagged-stdout
   write — fire-and-forget; symmetric error posture with the
   existing Console primitive).

EDN's deterministic write makes each line independently parseable.

### Tests: `wat-tests/std/telemetry/Console.wat`

2 deftests:
- `test-dispatcher-three-edn-entries` — build factory; dispatch
  three entries; assert stdout captures three EDN lines (using a
  helper `dispatch-three-edn` that takes Console::Tx as a parameter
  per the function-decomposition discipline).
- `test-dispatcher-format-knob` — same shape with `:Json` format;
  assert stdout captures three JSON lines.

Both green.

---

## Discoveries during implementation

- **Enum variant constructor syntax requires leading colons.** Wrote
  `(:Edn)` instead of `:Edn` initially; type checker rejected. Fixed
  to bare keyword form for unit variants.
- **Closure captures are Send-safe.** The factory builds the
  dispatcher closure capturing `con-tx` (a `Sender` Arc) and `format`
  (an enum value). Both cross thread boundaries cleanly when arc
  080's `Service/spawn` carries the dispatcher to its worker.
- **Function-decomposition discipline (memory `feedback_simple_forms_per_func`)**
  surfaced the test-deadlock I hit and recovered from — moved the
  three-entry-dispatch logic into a helper taking Console::Tx as a
  parameter, leaving each function's outer let* simple.

## Differences from DESIGN

- DESIGN proposed `Console::Format` with two variants (Edn / Json);
  shipped with three (Edn / Json / Pretty). Pretty was added when
  arc 079 shipped `write-pretty` as a sibling primitive — including
  it here is free.
- DESIGN proposed a queue-fronted `Console<E,G>` service; shipped as
  a pure factory (see "What shipped from DESIGN's plan" above).

## What's still uncovered

- **Per-line filtering / sampling.** The dispatcher renders every
  entry. Consumer that wants to drop / throttle entries does it
  upstream of the Console dispatcher (in their own pipeline stage).
  Future arc could add a sampling factory if a consumer asks; today
  every entry is one line.
- **Multiplexing across multiple format outputs.** A consumer that
  wants both EDN and JSON simultaneously can build two dispatchers
  + tee. Substrate doesn't ship a tee factory; explicit composition
  reads honestly today.

## Consumer impact

Unblocks any consumer that wants structured stdout via the substrate
Service shell. The first real consumer is the trader — proof
sessions can render LogEntry as EDN-per-line for live debugging
during a long run, vs the previous "tail the SQLite db with sqlite3
CLI" pattern.

PERSEVERARE.
