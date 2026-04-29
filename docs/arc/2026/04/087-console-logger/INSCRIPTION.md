# Arc 087 — `:wat::std::telemetry::ConsoleLogger` — INSCRIPTION

**Status:** shipped 2026-04-29.

The substrate's first concrete Logger. Closure over (con-tx, caller,
clock, format); per-emission convenience methods (`/debug` `/info`
`/warn` `/error`); level routing to stdout or stderr. The pattern
(`<Destination>Logger`) is what future siblings (`SqliteLogger`,
`CloudWatchLogger`, `DataDogLogger`) follow.

The user's framing was load-bearing: *"we need the ux to not suck...
how can we have a closure over a caller identifier so that the caller
doesn't need to self identify on every emission?"*

ConsoleLogger answers it. Producer says `(/info logger entry)` —
caller, time, format, channel-routing all closed over inside the
struct. The hot path emits one method call per log entry.

**Design:** [`DESIGN.md`](./DESIGN.md).

---

## What shipped

### `wat/std/telemetry/ConsoleLogger.wat` (~120 lines)

- `LogLine<E>` struct — 4 fields (`time` / `level` / `caller` /
  `data`). Rendered as a named-field map per arc 086's
  `value_to_edn_with`.
- `ConsoleLogger` struct — closure-over-(con-tx, caller, now-fn,
  format).
- `Logger/log` (universal; level as keyword arg).
- `Logger/{debug,info,warn,error}` convenience methods.
- Level routing (`route-by-level` helper) — `:debug`+`:info`
  → `Console/out` (stdout); `:warn`+`:error` → `Console/err`
  (stderr); custom levels → stdout fallback.
- `render-line` helper dispatches on `Console::Format` for all
  5 variants (Edn / NoTagEdn / Json / NoTagJson / Pretty).

### Producer UX (`examples/console-demo/wat/main.wat`)

```scheme
;; Once in :user::main:
((market-log :ConsoleLogger)
 (:wat::std::telemetry::ConsoleLogger/new
   con-tx :market.observer
   (:wat::core::lambda ((_u :()) -> :Instant) (:wat::time::now))
   :wat::std::telemetry::Console::Format::Edn))

;; Producer thread:
(:wat::std::telemetry::ConsoleLogger/info  market-log (:Event::Buy 100.5 7))
(:wat::std::telemetry::ConsoleLogger/warn  market-log (:Event::CircuitBreak "spike"))
(:wat::std::telemetry::ConsoleLogger/error market-log (:Event::CircuitBreak "down"))
```

Caller never self-identifies. Time auto-stamped. One method call
per emission.

### Output (verified end-to-end via `cargo run -p console-demo`)

```
stdout (:debug + :info):
[#inst "..." :info  :market.observer ...] — same shape across 5 formats
stderr (:warn + :error):
[#inst "..." :warn  :market.observer ...]
```

All 5 formats render correctly (sample of one entry per format
captured in arc 086's INSCRIPTION).

---

## What's still uncovered

- **`SqliteLogger`** — sibling for sqlite destinations. Trader's
  first sqlite path uses `Sqlite/auto-spawn` directly (variant-keyed
  persistence per arc 085); SqliteLogger ships when a consumer
  wants time+caller automatically added to every row across
  variants vs. baked into the variants themselves.
- **`CloudWatchLogger` / `DataDogLogger`** — external-API
  destinations. Future arcs.
- **Per-emission level filtering** — Logger emits every level
  today. A `min-level` knob is a future arc.
- **Sampling / rate-limiting** — every-Nth-emission shaping.
  Future arc.
- **Multi-destination tee** — `LoggerSet` fanning one emission
  across multiple destinations. Future arc.

## Consumer impact

Unblocks:
- **Trader's `:user::main` wiring** (lab proposal 059-001
  milestone 3) — every producer thread builds a ConsoleLogger for
  dev-time observability while sqlite handles archival.
- **Future `<Destination>Logger` siblings** — the pattern for
  closure-over-(destination-handle, caller, clock, format-policy)
  is documented; siblings follow.

PERSEVERARE.
