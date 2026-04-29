# Arc 087 — `:wat::std::telemetry::ConsoleLogger`

**Status:** SHIPPED 2026-04-29.

**Predecessors:**
- Arc 081 — `:wat::std::telemetry::Console/dispatcher` (queue-fronted
  rendering factory). The asynchronous variant.
- Arc 086 — EDN round-trip + natural formats. Provides the 5 render
  modes ConsoleLogger dispatches over (`:Edn`, `:NoTagEdn`, `:Json`,
  `:NoTagJson`, `:Pretty`).
- Arc 056 — `:wat::time::Instant` + clock primitives. The time
  source closures pass to the logger.

**Surfaced by:** the user's UX iteration (2026-04-29):

> "shipping something like a logger who emits time + data is what i
> want... actually... no... it should be... time + caller + data"
>
> "we need the ux to not suck... how can we have a closure over a
> caller identifier so that the caller doesn't need to self identify
> on every emission?"
>
> "ConsoleLogger? its built onto the console?... a CloudWatchLogger
> or DataDogLogger would be something else"
>
> "we should have debug, info -> stdout and warn, error -> stderr"

The arc names the substrate's first concrete Logger — destination-
bound (Console), structured (4-field LogLine struct), level-routed
(stdout vs stderr), caller-closured. The pattern (`<Destination>Logger`)
is a template future siblings follow.

---

## What this arc is, and is not

**Is:**
- `:wat::std::telemetry::LogLine<E>` — 4-field substrate struct
  (`time` / `level` / `caller` / `data`). Rendered as the canonical
  log-line shape via the EDN renderers.
- `:wat::std::telemetry::ConsoleLogger` — closure-over-state struct
  carrying `(con-tx, caller, now-fn, format)`. Built once per
  producer; passed by reference into hot paths.
- `Logger/log` (universal) + `Logger/{debug,info,warn,error}`
  (convenience methods) — sugar over the universal form with the
  level baked in.
- Level routing — `:debug` and `:info` emit to stdout via
  `Console/out`; `:warn` and `:error` emit to stderr via
  `Console/err`.
- 5 format choices via the existing `Console::Format` enum
  (extended in arc 086 with `:NoTagEdn` and `:NoTagJson`).

**Is not:**
- A queue-fronted service. ConsoleLogger renders + writes
  synchronously in the producer's thread. For high-volume
  decoupled-from-Console-driver-latency producers, the explicit
  `Console/dispatcher` factory (arc 081) sits behind a Service
  shell.
- `SqliteLogger` / `DataDogLogger` / `CloudWatchLogger`. Future
  siblings. Trader's first sqlite use case is variant-keyed via
  `Sqlite/auto-spawn` (arc 085) — the variants encode their own
  identity, no Logger needed.
- Generic over destination. The "ConsoleLogger" name reflects
  what it's bound to. Each destination gets its own Logger struct
  with destination-specific state.

---

## Surface

```scheme
;; The line shape — 4-field struct, rendered as a map per the
;; user's directive [time level caller data].
(:wat::core::struct :wat::std::telemetry::LogLine<E>
  (time :wat::time::Instant)
  (level :wat::core::keyword)
  (caller :wat::core::keyword)
  (data :E))


;; Producer-side recorder. Holds destination + identity + clock +
;; format. Built once per producer; the producer's hot path calls
;; the convenience methods.
(:wat::core::struct :wat::std::telemetry::ConsoleLogger
  (con-tx :wat::std::service::Console::Tx)
  (caller :wat::core::keyword)
  (now-fn :fn(())->wat::time::Instant)
  (format :wat::std::telemetry::Console::Format))


;; Universal — pass level explicitly for dynamic-level emission.
(:wat::std::telemetry::ConsoleLogger/log<E>
  (logger :ConsoleLogger)
  (level :wat::core::keyword)
  (entry :E)
  -> :())

;; Convenience — sugar with level baked in. Routes per the rule:
;;   :debug + :info → Console/out (stdout)
;;   :warn  + :error → Console/err (stderr)
;;   custom keywords (e.g. :trace) → stdout fallback
(:wat::std::telemetry::ConsoleLogger/debug<E> ...)
(:wat::std::telemetry::ConsoleLogger/info<E>  ...)
(:wat::std::telemetry::ConsoleLogger/warn<E>  ...)
(:wat::std::telemetry::ConsoleLogger/error<E> ...)
```

### Producer UX

```scheme
;; Once in :user::main:
((market-log :ConsoleLogger)
 (:wat::std::telemetry::ConsoleLogger/new
   con-tx
   :market.observer
   (:wat::core::lambda ((_u :()) -> :wat::time::Instant) (:wat::time::now))
   :wat::std::telemetry::Console::Format::Edn))

;; Producer thread — caller never self-identifies, time auto-stamped:
(:wat::std::telemetry::ConsoleLogger/info  market-log (:Event::Buy 100.5 7))
(:wat::std::telemetry::ConsoleLogger/warn  market-log (:Event::CircuitBreak "spike"))
(:wat::std::telemetry::ConsoleLogger/error market-log (:Event::CircuitBreak "down"))
```

### Output shape (`:Edn` format)

```
[stdout]
#wat.std.telemetry/LogLine {:time #inst "2024-..." :level :info :caller :market.observer :data #demo.Event/Buy [100.5 7]}

[stderr]
#wat.std.telemetry/LogLine {:time #inst "2024-..." :level :warn :caller :market.observer :data #demo.Event/CircuitBreak ["spike"]}
```

Same shape across all 5 formats (per arc 086), differing only in
tag/sentinel policy and rendering style.

---

## Why direct (no Service queue)

The earlier `Console/dispatcher` factory (arc 081) sits inside an
arc-080 `Service<E,G>` shell — queue-fronted; producer enqueues a
batch, worker thread renders + sends. For high-volume telemetry
or producer-decoupled-from-render-latency cases, that's the right
shape.

ConsoleLogger is direct: render + send happens in the producer's
thread. Justification:
- Dev-time logging volume is low. Queue overhead doesn't pay off.
- `Console/out` and `Console/err` are themselves queue-fronted
  (the `Console` driver fans tagged-stdio writes from N producer
  handles). The double-queue (logger queue + console queue) is
  one queue too many for this volume.
- Per `feedback_verbose_is_honest` — direct wins when nothing
  asynchrony eliminates.

The two paths coexist:
- `ConsoleLogger` — direct; one of-many producer-side instance.
- `Console/dispatcher` (arc 081) — queue-fronted; for batch-
  accumulating high-volume scenarios.

---

## Slice plan

Single session. Built iteratively as the user surfaced
requirements:

1. ConsoleLogger struct + universal `/log` form.
2. Convenience methods `/debug` `/info` `/warn` `/error`.
3. Level routing (stdout vs stderr).
4. LogLine struct (replacing the original 4-tuple) — for
   named-field render via arc 086's `value_to_edn_with`.
5. FQDN discriminator on inner enum (`_type :demo.Event/Buy`,
   not bare `:Buy`).
6. `:NoTagEdn` and `:NoTagJson` format variants for human-readable
   and ingestion-tooling consumption (arc 086).

`examples/console-demo/` walkable demo — `cargo run -p console-demo`
shows all 5 formats with both stdout and stderr lines.

---

## Open follow-ups

- **`SqliteLogger`** — the sibling for sqlite destinations. Trader's
  current shape uses `Sqlite/auto-spawn` directly with variant-keyed
  persistence; SqliteLogger lands when a consumer wants
  time+caller automatically added to every row across variants
  (vs. baking those fields into the variants themselves).
- **`CloudWatchLogger` / `DataDogLogger`** — future external-API
  destination siblings. Same shape: closure over `(api-handle,
  caller, clock, format-policy)`.
- **Per-emission level filtering at the Logger** — current
  ConsoleLogger emits every level. A `min-level` field on the
  struct could drop entries below a threshold without producer-
  side `if level >=` ceremony. Future arc when a consumer wants
  it.
- **Sampling / rate-limiting at the Logger** — high-frequency
  producers might want every-Nth-emission shaping. Future arc.
- **Multi-destination tee** — `LoggerSet` that fans one emission
  to multiple destination Loggers. Composes: log to console for
  dev visibility AND sqlite for archival. Future arc when a
  consumer hits the same need twice.

---

## Test strategy

- `examples/console-demo/wat/main.wat` — runnable demonstration.
  All 4 producer-side levels exercised across all 5 formats.
  `cargo run -p console-demo` produces the canonical output
  visible in the INSCRIPTION.
- Workspace tests (728 substrate Rust + every wat-suite) green;
  the demo compiles + runs end-to-end.

The pattern is exercise-by-demo rather than `wat-tests/` deftests
because the assertion shape (stdout/stderr split + format-correct
rendering across 5 variants) reads cleanest as a runnable example.
A future deftest could capture stdout via `RunHermeticAst` and
assert exact strings; current verification is visual + runtime.

---

## Dependencies

**Upstream:**
- Arc 081 — `Console::Format` enum + the original
  `Console/dispatcher` shape this arc's `ConsoleLogger` dispatches
  over.
- Arc 086 — `value_to_edn_with(types)` for named-field render;
  `:NoTagEdn` + `:NoTagJson` formats for tagless human-readable +
  ingestion-tooling use.
- Arc 056 — `:wat::time::Instant` + `now` for the clock.

**Downstream this arc unblocks:**
- Trader's `:user::main` wiring (lab proposal 059-001 milestone 3)
  — every producer thread builds a ConsoleLogger for dev-time
  observability of trader internals while sqlite handles archival.
- Future `<Destination>Logger` siblings follow this arc's
  template: closure-over-state struct + universal `/log` +
  level-convenience methods.

PERSEVERARE.
