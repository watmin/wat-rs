# DESIGN — STONE T1b.1: the `Journal` surface (the telemetry sink's S4c contract, write half)

> **STONE T1b.1 — the first of the T1b sink build.** Post-S4c, a `defservice` must declare a `defsurface` and wear it
> (`:satisfies` + `:impls`); `:ops` is retired. So the telemetry sink (`journal'`) needs a surface — `Journal` — the way
> `mem-store'`/`sqlite-store'` wear `Store`. This stone lands **just the surface** (a pure `defsurface`, no impl), so the
> next stone (`journal'`, the service) builds on a settled contract. **Write half only** — the query ops are T2-gated
> (see § Scope).
>
> **Names — intueri-cast + ratified (2026-07-11):** surface `:wat::telemetry'::Journal` / service `:wat::telemetry'::journal'`
> / verbs `write-metrics`·`query-metrics`·`write-logs`·`query-logs`. `Journal` over `Sink` (`Sink` half-speaks — fails
> *Honest* on the query half; `Journal` is written-to AND read-back, the `journald`/`journalctl` shape). See the parent
> `DESIGN-telemetry-service-and-query-surface.md` banner.

## Why — the surface is the contract the sink wears

`journal'` is the long-lived actor that is *given metrics/logs, or queried; it creates nothing.* Under S4c it cannot
carry inline `:ops`; it must `:satisfies` a surface that owns the protocol. `Journal` is that surface — a
`:nature :wat::kernel::Peer'` contract, exactly the shape `Store` has (`wat/query.wat:101`). A dialed
`Peer'<Journal::Op,Journal::Reply>` **is** a `Journal` intrinsically (arc 293 Path B) — no wrapper struct.

## Scope — WRITE half now; query ops at T2 (forced by the disk, not preference)

The `query-metrics`/`query-logs` ops reference `:wat::query::Query` and `:wat::query::Result`, which **do not exist on
disk** — they are the rete-as-datalog vocabulary (`Query`/`Result`/`Lemma`/`Deduction`/`NextToken`/`IndexTarget`), built
in **T2**. S4c requires a satisfier to implement *every* surface method, so declaring the query ops now would force
`journal'` to implement a query it cannot honestly perform. Therefore:

- **This stone declares `write-metrics` + `write-logs`** (+ their `Journal::Write*Request/Response` messages).
- **`query-metrics`/`query-logs` JOIN the surface at T2** — a `defsurface`-extend alongside the `Query`/`Result` vocab
  and the rete filter. (`journal'` is the only satisfier, so it simply gains two `:impls` at T2 — trivial growth.)
- **The T1b mem↔sqlite differential (STONE T1b.3) reads back through the store's own `scan`** — proving the *write* path
  lands byte-identical rows over both backends. The *query* differential is T2's.

This is a clean stepping-stone (write path → query path), and it is the design's own T1b→T2 order.

## The one contract decision — errors are the store's errors, surfaced (not a parallel vocabulary)

`write-metrics` delegates to `store/put`, which returns `Store::PutResponse::{Success | Constraint err | Transient err |
Fatal err}` where `err : :wat::query::{Constraint|Transient|Fatal}` (over the shared `Reason`/`Fault` records). The
`Journal::WriteMetricsResponse` **reuses those same `:wat::query::` error records** as its payload — a pass-through, not a
re-wrap. Rationale: the journal's write failures *are* its store's put failures; inventing a parallel
`:wat::telemetry'::Constraint`/… would duplicate a vocabulary (derive-is-the-wall). This makes `Journal` depend on
`:wat::query::`, which fixes load order (§ Home).

## The surface (mirrors `wat/query.wat`'s `Store`, exactly)

```clojure
(:wat::core::defsurface :wat::telemetry'::Journal :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :wat::telemetry'::Journal::WriteMetricsRequest
     [batch <- (:wat::core::Vector :wat::telemetry'::Metric)])
   (:wat::core::defenum :wat::telemetry'::Journal::WriteMetricsResponse :wat::enum::Pure
     :Success    []
     :Constraint [err <- :wat::query::Constraint]
     :Transient  [err <- :wat::query::Transient]
     :Fatal      [err <- :wat::query::Fatal])

   (:wat::core::defrecord :wat::telemetry'::Journal::WriteLogsRequest
     [batch <- (:wat::core::Vector :wat::telemetry'::Log)])
   (:wat::core::defenum :wat::telemetry'::Journal::WriteLogsResponse :wat::enum::Pure
     :Success    []
     :Constraint [err <- :wat::query::Constraint]
     :Transient  [err <- :wat::query::Transient]
     :Fatal      [err <- :wat::query::Fatal])]
  :features
  [;; write a metrics batch (≥1, homogeneous) ATOMICALLY through the owned store.
   (write-metrics [self <- :wat::telemetry'::Journal  req <- :wat::telemetry'::Journal::WriteMetricsRequest]
     -> :wat::telemetry'::Journal::WriteMetricsResponse)

   ;; write a logs batch (≥1, homogeneous) ATOMICALLY through the owned store.
   (write-logs [self <- :wat::telemetry'::Journal  req <- :wat::telemetry'::Journal::WriteLogsRequest]
     -> :wat::telemetry'::Journal::WriteLogsResponse)])
```

## Home + load order

- **File:** the `Journal` surface lives with the telemetry vocabulary. It references `:wat::telemetry'::Metric`/`Log`
  (`wat/telemetry.wat`, T0) **and** `:wat::query::{Constraint,Transient,Fatal}` (`wat/query.wat`) — so it must load
  **after both**. Two placements, brief-author's call:
  - **(preferred)** append the surface to `wat/telemetry.wat` and move its stdlib manifest slot to **after `wat/query.wat`**
    (telemetry.wat currently only deps core.wat; adding the surface adds a query.wat dep). Verify nothing loads
    telemetry records *before* query.wat (grep — the sink/span consumers are all later stones).
  - **(fallback)** a new sibling `wat/telemetry/journal.wat` holding only the surface, slotted after `wat/query.wat` +
    `wat/telemetry.wat`. Mirrors `wat/query/mem.wat` being a sibling of `wat/query.wat`. Choose this if moving
    telemetry.wat's slot disturbs load order.
- **Manifest:** `src/stdlib.rs` — add/move the include so `Journal` freezes after `Store`'s vocabulary.

## Out of scope (affirmatively cut — not deferred)

- **`journal'` the service** — STONE T1b.2 (the satisfier: holds a `Store` peer via multiparam `:init`, `:impls`
  serialize `Metric`/`Log` → `StoredRow` → `store/put`).
- **The mem↔sqlite differential** — STONE T1b.3.
- **`query-metrics`/`query-logs`** — T2 (need `:wat::query::Query`/`Result` + the rete filter). Tracked, not deferred.

## The RED disconfirming probe (the acceptance gate)

`tests/services/probe_arc278_journal_surface.{wat,rs}` — a **throwaway toy satisfier** proving the surface compiles + is
satisfiable + replies through the wire (mirroring `mem-store'`'s satisfaction of `Store`; the toy is NOT `journal'`):

```clojure
;; probe_arc278_journal_surface.wat — a toy Journal satisfier, no store, replies Success.
(:wat::service::defservice :probe::toy-journal'
  :satisfies :wat::telemetry'::Journal
  :durable   []
  :ephemeral []
  :impls
  [(write-metrics [s req] (:wat::service::Outcome::Reply s (:wat::telemetry'::Journal::WriteMetricsResponse::Success)))
   (write-logs    [s req] (:wat::service::Outcome::Reply s (:wat::telemetry'::Journal::WriteLogsResponse::Success)))])
;; :probe::run — start it on a thread, call write-metrics with a 1-Metric batch, return the response tag.
```

- **RED now** (surface absent): freezing the fixture fails with *unknown surface `:wat::telemetry'::Journal`* — the exact
  gap, nothing else around it.
- **GREEN after the strike:** the surface freezes; the toy `:satisfies` it; `write-metrics` returns
  `Journal::WriteMetricsResponse::Success`.

## Expectations (fixed before the strike)

| what | command | expected |
|---|---|---|
| build | `cargo build --release` | clean |
| the surface freezes + synthesizes messages | `cargo nextest run --release -E 'test(/probe_arc278_journal_surface/)' --test-threads=1` | PASS — toy replies `WriteMetricsResponse::Success` |
| a swapped/wrong response type is a located error (negative) | `.wat.bad` variant replying a `Store::PutResponse` from `write-metrics` | TypeMismatch at the reply site |
| full floor | `cargo nextest run --release` (FOREGROUND) | prior floor + this; 0-new (modulo the known `no_inlined_wat` lint) |

**Runtime prediction:** 20–35 min (a templated `defsurface` mirroring `Store` + the manifest slot + the toy probe).
**Trap-doors:** the load-order move (STOP if telemetry records are consumed before query.wat — report, don't reorder blind);
the `:wat::enum::Pure` variant syntax for the Response enums (copy `Store::PutResponse` exactly).
