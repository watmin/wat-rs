# DESIGN — the wat telemetry facility (`Journal` sink + `Span` producers + the query surface)

> ⚠️ **TRUED 2026-07-11 — read this banner before the body below; it supersedes stale names + shapes.**
> 1. **NAMING RATIFIED — the sink is `Journal`, not `TelemetryService'`.** intueri cast twice (surface name):
>    the first cast retired `TelemetryService'` (the `Service` suffix = the "absent-domain" mumble) and picked
>    `Sink`; a second cast weighed the builder's `Journal` against `Sink` and `Journal` **won outright** — `Sink`
>    half-speaks (fails *Honest* on the QUERY half, since a "sink" reads write-only in common usage), while
>    `Journal` speaks both halves (written-to AND queried; the `journald`/`journalctl` precedent is this design's
>    exact write+query shape). **Locked:** surface `:wat::telemetry'::Journal` (`:nature :wat::kernel::Peer'`) /
>    service `:wat::telemetry'::journal'` (`:satisfies`) / verbs `write-metrics`·`query-metrics`·`write-logs`·
>    `query-logs` (kebab) / messages `Journal::WriteMetricsRequest/Response` … Everywhere below that reads
>    `TelemetryService'` = `Journal`/`journal'`.
> 2. **The `:ops` shape below is SUPERSEDED by arc 278 S4c.** A `defservice` may no longer use `:ops`; it must
>    declare a `defsurface :nature :wat::kernel::Peer'` (owning its per-op `Request`/`Response` in a `:messages`
>    block) and `:satisfies` it with `:impls` (bodies only). Exemplar: `wat/query.wat` (`Store`) + `wat/query/mem.wat`
>    (`mem-store'`). The current `Journal` surface spec lives in **`DESIGN-STONE-T1b1-journal-surface.md`**.
> 3. **The query ops are T2-GATED.** `query-metrics`/`query-logs` reference `:wat::query::Query`/`Result`, which do
>    **not exist yet** (they are the rete-as-datalog vocabulary — **T2**). So the `Journal` surface ships the WRITE
>    half now (`write-metrics`/`write-logs`); the query ops JOIN the surface at T2 with the `Query`/`Result` vocab +
>    the rete filter. T1b's mem↔sqlite differential reads back through the store's own `scan`.
>
> **STATUS: DESIGN ratified + names intueri-cast; TOOLING BUILT — 278 unblocked (2026-07-04).** This is **what wat's
> logging + metrics facility will be.** The door we thought open was closed — **surface-splice was designed-but-unbuilt**;
> it is **open now**: surface-splice + one-way aggregate construction shipped (arc-293, `4c98b2ef`), so the records are
> buildable. `Metric`/`Log` are **`defrecord`s** splicing the `Scope` surface (NOT surfaces themselves — a surface is a
> constraint, a record is the concrete data).
> **Home: CORE** — the facility diagnoses rete, and rete is core, so it lives in the **substrate adjacent to the rete
> tooling**, NOT a battery crate; the legacy `wat-telemetry` crate stays where it is as a bridge, untouched.
> **Namespace: `:wat::telemetry'`** (primed — staged to replace the `:wat::telemetry` bridge) for the write-side;
> `:wat::query` (net-new, unprimed) for the engine.
> The **whole vocabulary is intueri-cast + ratified**: the two service names, the ops, `open`/`timed`, the store nouns.
> This doc is the durable record so the next self resumes from it, not from re-derivation.
>
> **Settled this session (shape, pending the builder's assessment):** the facility is **two composing `defservice`s** —
> a long-lived **sink** (given logs/metrics, or queried) and a short-lived **unit-of-work** producer (one per unit of
> work, closing over a ref to the sink). **Nothing is mutable** — state threads through the actor (the serialization IS
> the mutex we don't write). `WorkUnit`/`WorkUnitLog`/the legacy `Service<E,G>` **do not survive** — these surfaces +
> the two defservices are their replacement, made *more correct* (pure records, not the legacy carriers).

## Why — the interrogation, rebuilt correctly

*"I cannot diagnose a system I cannot interrogate — we are building the interrogation."* This is the interface the
builder wanted from **Elasticsearch, CloudWatch, Nagios, Grafana** — not to *be* them, but because he needs
high-fidelity metrics and logs so **the machines speak and we read them, and rapidly find the next attack** (the
DDoS/anomaly lineage; the chaos engine, R25 `MACHINA CHAOS DOMAT`). It is *the database is the debugger* made a
service. **We are building what wat's logging + metrics facility will be.**

The filter is **reasoning over data, via data — and that is the rete we built.** Pure rules cascading rules at the line
as server-side filter expressions (`PORTA PORTAM APERIT`): slurp a page of records into working memory, alpha-filter,
serve the `Deduction`s; want more, resume from the offset (`NextToken`). An assault on the query surfaces of Mongo/DDB —
because their filter is not *reasoning*; ours is. **Logs are data here** (pure records, EDN — never strings), so the same
rete reasons over logs and metrics alike.

It is also the **exemplar for `defservice`** (arc-170: "replace the hand-rolled services with defservice," done right)
**and the instrument the rete streaming service dogfoods to measure itself** — measure-first (R25/R26). The loop closes;
the measurer and the measured are the same substrate turned on itself. This lands the assault toward the north star:
**`wat-mcp`** — the wat REPL (arc 118 `DVO MVNDI VNA LINGVA`) plugged into the machine, usable by any instance.

The legacy telemetry stack is **annihilated, not preserved.** `WorkUnit`/`WorkUnitLog`/`Service<E,G>` emit through the
retired carriers — `:wat::edn::NoTag`, `:wat::edn::Tagged`, `:wat::holon::HolonAST`, `:wat::WatAST`, `Event::{Metric,Log}`.
Records-are-EDN (arc 300) retired every one: a record writes as `#wat.ns/Name {…}` (tagged, round-trip-safe) and decodes
straight back to the populated instance, so the data carrier is the record itself. Making the facility *more correct* =
pure `EdnRepresentable` `Metric`/`Log`, no legacy carriers.

## The two layers — the general engine, and its consumer

- **`wat.query`** — the **general-purpose rete-as-datalog / rete-as-filter.** Domain-blind: it filters *anything*
  satisfying `wat.query/Record`. The query vocabulary is ratified.
- **`wat.telemetry`** — a **consumer** of `wat.query`. Its `Metric` and `Log` satisfy `wat.query/Record`; the sink writes
  them and queries them back through the shared engine.

## The two composing services — the sink, and the unit of work

The facility is **two `defservice`s**, not a service plus a mutable holder. **Nothing mutates.** In wat, `incr!` returns a
holder with `counter + 1`; appending to a vector returns a new vector — value-threading, never mutation. And when state
must persist across calls, **that IS a service** — the actor's one-op-at-a-time serialization is the mutex we never
write (zero-mutex / CSP). So both halves are services; state lives in the actor and threads forward (TCO gen_server),
never in a cell.

- **`TelemetryService'` — THE SINK** (long-lived). It is **given logs/metrics, or queried. It creates nothing.** It owns
  the store (`:ephemeral`, thread-owned, never crossing a thread) and serves the four verbs
  (`WriteMetrics`/`QueryMetrics`/`WriteLogs`/`QueryLogs`). One op at a time — the serialization is the mutex over the
  shared store handle.

- **`Span` — THE PRODUCER** (short-lived, one instance per unit of work). A `Span` in the observability sense: an
  open/close interval that carries events (logs) + attributes (tags) and **nests** (child spans) — which is exactly a
  unit of work. It is **its own service**, holding the accumulating state (counters + durations) plus the **`Scope`** it
  stamps on every record it emits (`Span` *carries* a `Scope`; they are distinct — the live producer vs. the correlation
  record). It threads its state forward each op, and **closes over a ref to the sink** as an **injected, explicit
  dependency** (`:calls`) — *where it writes is not the caller's concern.* The caller opens a `Span` from a sink ref,
  works through it (log / count / time), and closes it; on close the accumulated metrics emit to the sink. **Nesting:** a
  `Span` can open a **child** `Span` — its own instance, its own `uuid`, the same sink.

The dependency flow, made explicit: `sink-ref → provision → Span (closes over sink-ref) → caller logs/counts/times
→ on close, metrics emitted to the sink.` The caller sees only units of work and log/emit; the sink is injected.

## The data model — `(namespace, time, data)`, DynamoDB-shaped, storage swappable

A DynamoDB single-table design **per store**: primary key `(pk, sk)` = `(namespace, iso8601-nanos)`; `data` is the stored
record's **tagged EDN**. Rows are time-sorted within a namespace; a query selects a namespace over a timeframe, filters
server-side (rete), paginates.

**The store is swappable.** What we depend on is *"a thing that holds records indexed and sorted by `(pk, sk)`"* — that
abstraction is the swap point at scale. `sqlite` is **one driver** (the local default); DDB/Mongo slot in behind the same
shape. **Metrics and logs are different shapes → separate stores** (per kind). A batch is homogeneous — all metrics OR all
logs, never mixed — and holds ≥ 1.

## The correlation model — the unit of work; `Scope` the shared constraint

`Metric` and `Log` are not independent rows — they are the output of a **unit of work**, and the whole point is
**correlation**. A unit of work opens with a `namespace` + `tags`, mints a **`uuid`** (its identity), stamps its start,
counts (`Incr`), times sub-blocks (`Timed`), logs statements (`Log`), and on close emits its metrics — all sharing that
`uuid`. Metrics ↔ logs join on it — the trace/span observability model, served by a **GSI on `uuid`**.

The four common fields — `namespace`, `uuid`, `tags`, `time-ns` — are lifted into **`Scope`**, an **exact surface**. The
`Metric` and `Log` **`defrecord`s** each **splice `Scope` in** via `[~@wat.telemetry'/Scope own…]` in their field vector —
*not* re-listed. This makes "same unit of work" a **structural fact**, carries the correlation key by construction, and
keeps the common shape from a **single source** (*derive-is-the-wall* — composed in, never re-implemented and hoped). The
splice mechanism is built (arc-293, 2026-07-04): it lives in the **aggregate field vector** (`defrecord`/`defstruct`/
`defholon`), not in a surface's `:features`.

## The surface architecture (structural, all the way down)

`defsurface` with `:features [field :- Type …]` is an **exact surface** (required typed fields); `:features []` is an
**open surface** (any record with the right holder satisfies it). A concrete record **structurally satisfies** a surface
when it carries all the floor fields at those types (grounded: `wat/core.wat` `:wat::core::Error`/`Fault`, arc 296 S1/S2).
The whole contract is imposed structurally — a record that isn't the right shape *cannot be written* (constraint
engineering: no form for the wrong thing). **FQDN disambiguates always** — `:wat::telemetry::Unit` never collides with
anything; names are judged on one axis only: *does the name say what it is inside `:wat::telemetry::`.*

arc-293's model is sharp: **a `defsurface` is a CONSTRAINT** (satisfied, never constructed); **a `defrecord` is the
concrete data** that structurally satisfies it. So the correlation core and the payload are **surfaces**; the stored rows
are **records** that satisfy them:

```
SURFACES (constraints — satisfied, never constructed):
  wat.query/Record    open   — anything the query engine matches (a stored row → WM fact)   [wat.query: net-new, unprimed]
  Scope               exact  — the correlation core: namespace, uuid, tags, time-ns
  LogMessage          open   — any pure record the caller defines (a Log's payload)

RECORDS (concrete data — constructed + stored; each SPLICES Scope, thereby satisfying Scope + wat.query/Record):
  Metric = defrecord [~@Scope  start-time-ns  name  (value :- Numeric)  (unit :- Unit)]
  Log    = defrecord [~@Scope  caller  (level :- Level)  (message :- LogMessage)]
```

Surface-splice is now **built** (arc-293, 2026-07-04): `~@:Surface` in a `defrecord` field vector expands to the surface's
`Field` members, merged flat before the own fields; a name repeated at a conflicting type is a compile error. And
construction is **unified** — one `aggregate-new` path mints every holder's ctor from the registered (splice-expanded)
fields, so struct / core-record / holon-record construct identically. A `Metric`/`Log` carrying
`namespace/uuid/tags/time-ns` therefore satisfies the `Scope` surface structurally, and the open `wat.query/Record`.

## The contractual surface (the source of truth for callers)

```clojure
;; ═══ wat.query — the general rete-as-datalog (domain-blind, RATIFIED, unchanged) ══════════════

(wat.core/defsurface wat.query/Record   :holder wat.core/Record :features [])   ;; open: a stored row → WM fact
(wat.core/defrecord  wat.query/Lemma     [])                                     ;; intermediate stepping-stone fact
(wat.core/defrecord  wat.query/Deduction [record :- wat.query/Record])           ;; terminal; the only fact queried out
(wat.core/defrecord wat.query/TableSchema [pk :- :String  sk :- :String])
(wat.core/defrecord wat.query/IndexSchema [pk :- :String sk :- :String ipk :- :String isk :- :String])
(wat.core/defrecord wat.query/IndexTarget [name :- :String pk :- :String sk :- :String])
(wat.core/defrecord wat.query/NextToken   [resume-time :- wat.core/Instant])
(wat.core/defrecord wat.query/Query
  [namespace :- wat.core/String  start-time :- wat.core/Instant  end-time :- wat.core/Instant
   index :- (wat.core/Option wat.query/IndexTarget)  rules :- (wat.core/Vector wat.rete/Rule)
   next-token :- (wat.core/Option wat.query/NextToken)])
(wat.core/defrecord wat.query/Result
  [deductions :- (wat.core/Vector wat.query/Deduction)  next-token :- (wat.core/Option wat.query/NextToken)])

;; ═══ wat.telemetry — the records [names settled on clarity; FQDN → no collisions] ═════════════

(wat.core/typealias wat.telemetry'/Tags (wat.core/HashMap wat.core/Keyword wat.core/String))

;; a metric value's storage type, by variant NAME. GROWS — i64/f64 to launch (the rest of wat's number
;; tower, incl. the arc-300 BigInt/BigRational, added later as cases demand).
(wat.core/defenum wat.telemetry'/Numeric wat.enum/Pure
  i64 [val :- wat.core/i64]
  f64 [val :- wat.core/f64])

;; the metric's semantic unit — a closed enum (name holds value); GROWS AS IT MUST (Nanos added this session).
(wat.core/defenum wat.telemetry'/Unit wat.enum/Pure
  Count Nanos Millis Bytes Percent)

;; a log's level — a closed enum.
(wat.core/defenum wat.telemetry'/Level wat.enum/Pure
  Debug Info Warn Error)

;; the correlation core — the shared constraint both Metric and Log satisfy (exact surface).
(wat.core/defsurface wat.telemetry'/Scope :holder wat.core/Record
  :features [namespace :- wat.core/String    ;; pk
             uuid      :- wat.core/Uuid       ;; unit-of-work correlation id → GSI
             tags      :- wat.telemetry'/Tags  ;; dimensions
             time-ns   :- wat.core/i64])      ;; epoch nanos — the sk

;; a metric — a defRECORD (concrete data): SPLICE the Scope surface's attributes, then own fields. The splice inlines
;; Scope's namespace/uuid/tags/time-ns; a Metric carrying them satisfies the Scope surface + the open wat.query/Record.
(wat.core/defrecord wat.telemetry'/Metric
  [~@wat.telemetry'/Scope                          ;; namespace, uuid, tags, time-ns (= the span END, the sk)
   start-time-ns :- wat.core/i64                  ;; the span START (epoch nanos)
   name          :- wat.core/Keyword              ;; the counter/timer name → GSI candidate
   value         :- wat.telemetry'/Numeric         ;; count / len / sum — variant name holds the storage type
   unit          :- wat.telemetry'/Unit])          ;; :Count / :Nanos / … — orthogonal to Numeric's storage type

;; a log message — an OPEN surface (the constraint the payload satisfies): any pure record the caller defines.
(wat.core/defsurface wat.telemetry'/LogMessage :holder wat.core/Record :features [])

;; a log — a defRECORD: SPLICE Scope, then own fields (exact envelope, open payload).
(wat.core/defrecord wat.telemetry'/Log
  [~@wat.telemetry'/Scope                          ;; namespace, uuid, tags, time-ns (= the emit moment, the sk)
   caller  :- wat.core/Keyword                    ;; producer identity → GSI candidate
   level   :- wat.telemetry'/Level                 ;; a closed enum
   message :- wat.telemetry'/LogMessage])          ;; a PURE RECORD satisfying the open LogMessage surface

;; ═══ TelemetryService' — THE SINK. given logs/metrics, or queried. owns the store. creates nothing. ═══
;;   [sink name: PROVISIONAL — intueri]
(wat.service/defservice wat.telemetry'/TelemetryService'
  :ephemeral [store <- wat.telemetry'/Store]                             ;; sqlite = one driver behind (pk,sk) holder
  :ops [(WriteMetrics [batch <- (wat.core/Vector wat.telemetry'/Metric)] -> Ok)
        (QueryMetrics  [q     <- wat.query/Query]                       -> wat.query/Result)
        (WriteLogs     [batch <- (wat.core/Vector wat.telemetry'/Log)]   -> Ok)
        (QueryLogs     [q     <- wat.query/Query]                       -> wat.query/Result)])
;;  defservice SERIALIZES — one op at a time (the actor IS the mutex over the store handle).
;;  the kind rides the VERB (no `table` field on Query).

;; ═══ Span — THE PRODUCER. its own service; state threads via the actor. closes over a ref to the
;;     sink (:calls — the injected dep). logs write NOW; metrics fire on Close. nests via Sub. ═══
;;   [uow name + provisioning verb: PROVISIONAL — intueri]
(wat.service/defservice wat.telemetry'/Span
  :calls   [sink <- wat.telemetry'/TelemetryService']                    ;; injected dependency, made explicit
  :durable [scope     <- wat.telemetry'/Scope                            ;; namespace/uuid/tags/start-time-ns
            counters  <- (wat.core/HashMap wat.core/Keyword wat.core/i64)
            durations <- (wat.core/HashMap wat.core/Keyword (wat.core/Vector wat.core/i64))]  ;; nanos samples
  :ops [(Log   [caller <- wat.core/Keyword  level <- wat.telemetry'/Level  message <- wat.telemetry'/LogMessage] -> Ok)
        ;;   build (Log scope caller level message); (sink WriteLogs [it]) NOW; state unchanged
        (Incr  [name <- wat.core/Keyword]                    -> Ok)  ;; state' counters[name] + 1
        (Timed [name <- wat.core/Keyword  nanos <- wat.core/i64] -> Ok)
        ;;   PURE op: find-or-create durations[name], state' durations[name] ++ nanos. NOT a closure — the
        ;;   timing widget (below) already measured. does NOT touch counters (count = (len durations[name])).
        (Nest  [namespace <- wat.core/String  tags <- wat.telemetry'/Tags] -> wat.telemetry'/Span)
        ;;   a NESTED unit of work — its own instance/uuid, same injected sink; its own Close emits its own metrics
        (Close [] -> Done)])
;;   Close: counters + durations → Metric rows → (sink WriteMetrics batch). See § Emission.

;; provisioning `open` (intueri-ratified): from a sink ref + namespace + tags → a Span instance that closes
;; over the sink (mints uuid, stamps start-time-ns). e.g. (Span::open sink :market-eval {…}). Pairs with Close.

;; the TIMING WIDGET `timed` — a MACRO at the call site (the Clojure `time` idiom: wrap the body form, no thunk).
;; the impure edge: reads the clock, runs the (impure) body, feeds name+nanos to the pure Timed op, returns the ret.
;; closures never enter the actor → survives a remote sink boundary. (intueri-ratified.)
(wat.core/defmacro wat.telemetry'/timed [uow name & body]
  `(:let [start#   (wat.time/epoch-nanos (wat.time/now))
          ret#     (wat.core/do ~@body)
          elapsed# (wat.core/- (wat.time/epoch-nanos (wat.time/now)) start#)]
     (~uow/Timed ~name elapsed#)   ;; the pure op
     ret#))
```

## Recording work — the UX

```clojure
(:let [sink (… a ref to wat.telemetry'/TelemetryService' …)
       u    (wat.telemetry'/Span::open sink :market-eval {:asset :BTC})]  ;; provision: inject sink, mint uuid
  (u/Log :fetcher :info (:MyEvent …))               ;; a Log written NOW, correlated by u's uuid
  (u/Incr :requests)                                ;; a pure counter
  (wat.telemetry'/timed u :fetch (do-fetch))         ;; WIDGET: times (do-fetch), records nanos via u/Timed, returns its value
  (:let [inner (u/Nest :fetch-detail {:host :h1})]  ;; a nested unit — its own uuid, same sink
    (inner/Incr :retries)
    (inner/Close))                                  ;; inner's metrics emitted
  (u/Close))                                        ;; outer's metrics emitted; logs were already written
```

The caller sees only units of work and `Log`/`Incr`/`Timed`. Who `u` writes to (the sink) is an injected dependency, not
the caller's concern.

## Semantics

### Write
Two homogeneous batch paths (`WriteMetrics` / `WriteLogs`), each ≥ 1. The sink stamps `sk` (iso8601-nanos, from
`time-ns`). Each record is written as its **tagged EDN** into the `data` column of its store, plus the projected
key/index columns (below).

### `Timed` — the pure op + the timing widget (the Clojure `time` idiom)
Split along purity, per the builder: the **op** `Timed [name nanos]` is **pure** — find-or-create `durations[name]`,
append the nanos, thread state'; it never sees a closure or the clock. The **timing widget** is a **macro** at the call
site (Clojure's `time`: wrap the body form, no thunk) — the impure edge that reads the clock, runs the impure body, feeds
`name + elapsed-nanos` to the pure op, and returns the body's value untouched. Closures never enter the actor, so the
split survives a remote sink boundary. A pure state-transition dealing with impurity handled entirely at the widget.

### Emission (on `Close`) — counters and durations differ
```
each counter  name  →  1 Metric  { name              , value = counters[name] , unit :Count }
each duration name  →  2 Metrics { <name>/count       , value = (len samples)  , unit :Count }
                                  { <name>/duration    , value = (sum samples)  , unit :Nanos }
```
So three `(u/Timed :fetch …)` calls taking 100, 20, 50 ns accumulate `{:fetch [100 20 50]}` and emit exactly
`:fetch/count = 3` (`:Count`) and `:fetch/duration = 170` (`:Nanos`). *(This REPLACES the legacy per-sample fanout — the
old `WorkUnit` emitted one row per sample; the correct shape is the aggregate: count + sum.)*

### Query — the rete filter, per page
Server-side filtering is **rete-as-datalog** (`wat.query`), run **one page at a time**:
1. Range-scan a page of rows by `(pk, sk)` (or the GSI's projected columns if `index` is `Some`).
2. Assert each row as a **`Record`** fact into working memory.
3. Fire the caller's `rules`. Rules cascade forward (`PORTA PORTAM APERIT`): deduce a **`Lemma`** (stepping-stone), a
   downstream rule stands on it, until a terminal rule deduces a **`Deduction`** wrapping the matched `Record`.
4. **Query out only `Deduction`s.** Return them + the `sk` we stopped at as the `next-token`.

**The wall — rules may deduce ONLY `Lemma` or `Deduction`.** **No beta tree — pagination imposes it:** a join partner may
be on an unfetched/evicted page, so every rule is **per-record (alpha-only) by construction**, which is exactly what
makes the query **streamable**.

### Read AND write in one sink (`defservice` serializes)
The sink exposes all four ops; `defservice` serializes one at a time, so the four never race over the shared `store`
handle — the one-op-at-a-time *is* the mutex we don't write. One process serves whatever it's asked: write live, diagnose
after, or both.

### Pagination (DynamoDB-style)
`NextToken {resume-time}` is the `sk` the scan stopped at. Fetch a page, the rules deduce M matches, return the M
`Deduction`s + `NextToken{last-sk}`; the client re-calls with the token. A page that reaches the end with no deductions
returns empty `deductions` + `next-token None` (done).

### GSIs — the crux, and why the store needs projection
SQL cannot index into the opaque `data` EDN. A base-table query ranges on `(pk, sk)`; a GSI query needs `(pk, sk, ipk,
isk)`, and `ipk`/`isk` live *inside* the record's EDN. So GSIs require the **write path to project the index-key
attributes out of the record into real, indexed columns** at write time — one materialized secondary index per GSI (e.g.
the `uuid` correlation index). `TableSchema` (base) and `IndexSchema` (a GSI's projected columns) declare which columns
each mode ranges on; `IndexTarget {name pk sk}` selects a GSI at query time. Both modes flow through the identical page →
assert → fire → paginate loop; only the store's `WHERE`/index differs — a driver concern.

## Resolved (2026-07-04)

1. **The facility is TWO composing `defservice`s** — the long-lived **sink** (`TelemetryService'`: given logs/metrics or
   queried; owns the store; creates nothing) and the short-lived **unit-of-work** producer (`Span`: one per unit of
   work, closes over a ref to the sink as an injected dep, threads its accumulating state, emits metrics on close).
2. **Nothing is mutable.** `incr!`/append return new holders; state that persists across calls lives in the actor and
   threads forward (the serialization is the mutex). The legacy `ThreadOwnedCell` `WorkUnit` is gone.
3. **`WorkUnit`/`WorkUnitLog`/`Service<E,G>` do not survive** — replaced entirely by the two defservices + these surfaces,
   made *more correct*: pure `EdnRepresentable` records, no `NoTag`/`Tagged`/`HolonAST`/`WatAST`/`Event` carriers.
4. **`time-ns` is first-class.** `Scope` carries `time-ns :- :i64` (epoch nanos, the sk); `Metric` also carries
   `start-time-ns :- :i64` (the span start).
5. **`Timed` splits into a pure op + a timing-widget macro** (the Clojure `time` idiom). The op `Timed [name nanos]` is
   pure state-append; the widget macro measures the clock around the body, feeds the op, and returns the body's value —
   closures never enter the actor. Durations are `i64` nanos. **`Incr` is pure counters; `Timed` is durations-only**
   (no double-count).
6. **Emission is the aggregate**, not the fanout: a counter → 1 Metric; a duration name → 2 Metrics (`/count` = len,
   `/duration` = sum). `:Nanos` joins the `Unit` enum.
7. **Nesting** via `Nest` — a nested unit of work is its own instance/uuid, same injected sink.
8. **FQDN → no collisions.** Names are judged on clarity within `:wat::telemetry::` alone. (Retained from the query
   layer: kind rides the verb; enums grow-as-needed; the store is swappable, sqlite one driver.)
9. **The whole vocabulary is intueri-cast + ratified** (two casts this session): the producer surface resolved to
   `TelemetryService'` (sink) / `Span` (producer) / `open` (provision) / `Log`·`Incr`·`Timed`·`Nest`·`Close` (ops)
   / `timed` (widget macro) / `Write*`·`Query*` (sink verbs) / `Store`·`Table`·`Index` (store nouns). See § Naming.

## Resolved (2026-07-05) — the storage-backend model  ⚠️ items 11–14 SUPERSEDED — see "THE CIRCUIT" below

10. **The storage abstraction lives in the sink's `:ephemeral` as a surface-typed attribute** — `[store <- wat.query/Store]`
    holds *any* backend's satisfier, and the ops call `(:wat::query/Store::put store …)`, which **dispatches to the
    concrete backend at runtime** (existential dispatch, `src/runtime.rs:5339`; surface-typed field, `src/check.rs:13666`).
    **PROVEN** end-to-end: `probes/surface-field-dispatch.wat` → `142` (a `defstruct` satisfier `extend-type`d to a
    methods-bearing `Store` surface, held in a struct **attribute typed as the surface**, dispatched *through the field*).
    This is what makes the sink backend-blind.
11. **The durable/ephemeral split carries the backend, correctly.** `:durable` is EDN — it holds the backend **spec** (a
    path / dsn / url — data that crosses the wire) alongside hibernation state; `:ephemeral` holds the **live `Store`
    satisfier**, born **thread-local in `:init`** from the durable spec, never hibernated, never crossing. The resource is a
    *deferred computation of the spec* — on hibernate/resume you keep the spec (+ counters) and `:init` **reopens** the
    connection (R5 at the service layer: store the recipe, not the resource).
12. **IPC is edn-only → NO opener injection.** You pass **data** (the spec `String`), never a resource, never a closure (a
    closure isn't EDN, so it can't cross the actor boundary). The backend is therefore opened *inside* `:init` — the spec is
    an operating-input to a multi-param `:init` (`(Record, …operating-inputs) -> State`, `wat-tests/service-multiparam-init.wat`),
    not handed in pre-opened.
13. **ONE backend-blind service, NO macro.** A macro emitting N services with different `:ephemeral` is only forced if
    `:ephemeral` holds the *concrete* resource type (sqlite `Connection` ≠ mysql `Handle` ≠ mongo `Client`). The **`Store`
    abstraction makes the `:ephemeral` field shape identical across every backend** → one service. Backend *choice* is the
    spec + a per-backend `open` named in `:init` (or an `open-store : BackendConfig -> Store` factory); the `Store`-speaking
    ops are authored once. This is why the macro dissolved.
14. **293.W makes it correct-by-construction.** A `:holder :Struct` surface field is impure → it can live *only* in an
    impure struct / `:ephemeral`, never in a portable record / durable / on the wire. The compiler *forbids* a live
    connection from crossing the boundary — exactly the thread-local + edn-only rule, enforced structurally, for free.

## Resolved (2026-07-05, evening) — THE CIRCUIT: the store is DECOMPLECTED, the sink is GIVEN it (SUPERSEDES 11–14)

> Items 11–14 above describe the **FUSED** shape — the sink *opens* its own store in `:init` from a spec. That is a
> **sqlite-only** shape, and it is superseded. Grounded against the founding `wat-rs/docs/CIRCUIT.md`, the lineage
> `holon-lab-trading/docs/CIRCUIT.md`, and book ch097 (*Lingua Ignea* — the FPGA-on-CPU / homoiconic-CGRA facet):
> the facility is a **sub-circuit of the wat fabric**, and its wiring obeys the fabric's one law.

**The law (this is the fabric's physics, not a design choice).** *Resources are opened by the worker that uses them;
pipes cross threads, resources don't.* A live resource (a sqlite `Connection`, a `PersistentVector`) is a component's
**internal state** — it cannot ride a wire. Only **signals** cross: an `addr`, a `Page`, a batch of `Metric`s (all
EDN). This IS ZERO-MUTEX — each service is a `serve`-loop handling one signal at a time, so mutual exclusion is
structural (no lock written, none to forget), and *"a lie about state has no metal to live on."*

**Why 11–14 fail the requirement.** The requirement: the store works with **both** mem and sqlite, shown by a
differential. But "the sink opens the store in `:init`" can only ever open a **`Connection`** (a resource, born on the
sink's thread). `mem-store'` is a **`defservice`** (a peer), and mem.wat's own scope-NOTE proves a peer opened inside a
helper fn (`:init` *is* one) is dead on return. So the fused sink structurally **cannot** hold a mem backend. The
"both backends" requirement is the proof that the store must be **decomplected out of the sink**.

**The corrected circuit (R28 decomplection applied to the old fused Sqlite-driver):**

- **The store is its OWN service** (both backends, uniform): `mem-store'` (owns a `PersistentVector`) and
  **`sqlite-store'`** (owns a `Connection`) — each opens *its own* resource in *its own* `:init` on *its own* thread
  (the resource-user opening its resource — founding-legal, no scope trap). This **promotes S2's struct-`SqliteStore`
  into a `sqlite-store'` service**: S2's SQL hoisted into a `serve`-loop with the `Connection` in its own `:ephemeral`,
  plus a peer-wrapping `SqliteStore` satisfier — mirroring `mem-store'` almost line-for-line. (S2's struct was correct
  for the same-thread S2 differential; a `Connection` cannot be *given* across a wire, so for injection it must be a
  service.)
- **The sink is GIVEN the store, not opens it.** `:ephemeral [store <- :wat::query::Store]` — a **surface-typed** port
  (backend-blind, the `surface-field-dispatch → 142` shape); it holds a satisfier wrapping the store service's **peer**.
  Handed in at `start` (a live operating-input), never opened in `:init`. `:durable` carries the reconnect spec (the
  store's addr) for hibernate/resume — still R5 (store the recipe, not the resource).
- **The differential is a RE-WIRE.** Swap the store the sink is given (`mem-store'` ↔ `sqlite-store'`); the sink + Span
  are **byte-identical**; same ops → same persisted + read-back. That IS "the store works with both, shown" — swapping
  one component on the breadboard.
- **293.W holds, more cleanly.** Only the store's **peer/addr** crosses to the sink (portable EDN); the live
  `Connection`/`PersistentVector` never leaves its own service's thread. Correct by construction.

**`with-span` — the user's whole working surface** (the `with-open` idiom; NOT "bracket" — `wat/bracket.wat` is Ruby's
`Parallel`). A call-site macro like `timed` (no closure crosses the actor), acquire → use → guaranteed `Close`. The
binding is a proper `[name value]` pair, the value being the fresh span:

```clojure
(:wat::core::defmacro :wat::telemetry'::with-span [binding & body]
  ;; binding = [name (Span/open sink :ns tags)]  — name + its value, like let / with-open
  (:wat::core::let [name (:wat::core::first binding)  open-expr (:wat::core::second binding)]
    `(:wat::core::let [~name ~open-expr  result# (:wat::core::do ~@body)  _close# (~name/Close)] result#)))
```

**Honest seam (kept visible):** `with-span` fires `Close` on **normal completion**. Close-*on-error* (Ruby `ensure`,
Clojure `finally`) needs a wat unwind/finally primitive — errors here are values (`Result`), not stack-unwinds, so the
happy path always reaches `Close`; unwind-safety (or an actor-supervisor reaping an orphaned Span) is a **named
follow-on**, not silently assumed.

**The user-forms UX (ratified — the builder: "now *that's* a surface"):**

```clojure
;; ── THE EDGE — setup, once ──────────────────────────────────────────────
(:wat::core::defn :app::main [] -> :wat::core::nil
  (:wat::core::let
    [store (:wat::sqlite'::SqliteStore/start :locus (:wat::spawn::thread) :db "runs/market.db")   ;; ↔ MemStore = differential
     sink  (:wat::telemetry'::TelemetryService'/start :locus (:wat::spawn::thread) :store store)]
    (:app::evaluate-market sink :BTC)))                        ;; hand `sink` to the work — all it needs

;; ── THE WORK — the user's whole surface: a sink + a fresh span ──────────
(:wat::core::defn :app::evaluate-market
  [sink <- :wat::telemetry'::TelemetryService'  asset <- :wat::core::keyword] -> :wat::core::nil
  (:wat::telemetry'::with-span [span (:wat::telemetry'::Span/open sink :market-eval {:asset asset})]
    (span/Incr :candles-seen)
    (:wat::core::let [price (:wat::telemetry'::timed span :fetch-price (:market::fetch-price asset))]
      (span/Log :evaluator :info (:market::PriceObserved asset price))
      (:wat::core::if (:wat::core::> price 100000) (span/Incr :above-threshold) (span/Incr :below-threshold))
      (:wat::telemetry'::with-span [risk (span/Nest :risk-check {:asset asset})]   ;; nested UoW, own uuid
        (risk/Incr :checks)
        (:wat::telemetry'::timed risk :score (:risk::score price))))))
```

The user holds ONE thing — a `sink`. `with-span` does the rest; no `open`/`Close` by hand; the backend is invisible.
(The exact type of the `sink` handle a worker receives — the connected peer vs the service name — is pinned in the strike.)

**Deferred UX (a NAMED follow-on — AFTER the shapes are in place, per the builder):** restrict what callers can
*express* so they cannot footgun wrong metric/tag/counter names (constraint-engineering the caller surface — arc-299
spec-in-wat is the likely tool), and/or **auto-find** the deps (the sink) so they can be omitted. Not built in T1; the
shapes come first.

**Names to cast intueri before their strikes** (the standing rule — never narrate): **`with-span`** (the macro) and
**`sqlite-store'`** (the new store service). The rest of the vocabulary is already cast + ratified.

**The strike order (wiring order — each provable alone):**
1. **`sqlite-store'`** — promote S2's SQL into a service (mirrors `mem-store'`); so a sink can be handed either backend.
2. **the sink** (`TelemetryService'`) — given a store, **differential-tested mem ↔ sqlite** (same ops → same result).
3. **the span + `with-span` + `timed`** — the surface; emission-on-`Close`, nesting, the two call-site macros.

## Build implications (for the strike) — everything is CORE

- **Home: the CORE substrate** (`wat/`), adjacent to the rete tooling — NOT a battery crate. The facility diagnoses rete,
  and rete is core, so the facility is core. The legacy `wat-telemetry` / `wat-telemetry-sqlite` crates stay where they
  are as **bridges**, untouched.
- **sqlite becomes CORE too — and PRIMED (`:wat::sqlite'` / `:rust::sqlite'`).** Because the core facility depends on the
  store, the store — **sqlite included** — is core. But the `wat-sqlite` battery is *loaded* and defines a full
  `:wat::sqlite::*` namespace (`open`/`begin`/`commit`/`execute`/`Db`/`ReadHandle`) + `:rust::sqlite::*`; leaving that
  bridge untouched means a core sqlite would collide. So the core sqlite is **`:wat::sqlite'`** (wat surface) +
  **`:rust::sqlite'`** (Rust bindings) — **primed, staged to replace the `:wat::sqlite` battery bridge, coexisting with
  it** (exactly the `:wat::telemetry'` pattern). It carries the `(pk, sk, data, …projected-index-columns)` layout + GSI
  secondary indexes + write-path projection + range-scan/page read-path, all behind the **swappable store abstraction**
  (`Store`) — sqlite is **one driver**; the abstraction is the swap point. (arc-085 `auto-*` enum-derive is NOT the fit.)
- **`TelemetryService'` and `Span` are core `defservice`s** (in `wat/`, adjacent to rete), the sink holding the store
  handle in `:ephemeral`, `Span` calling the sink via `:calls`. Core substrate may use the `:rust::sqlite'::*` interop
  directly; ordinary consumers use `:wat::` verbs (arc-002 `NAMESPACE-PRINCIPLE`).
- **The query engine is a `wat.query` rete consumer** — alpha-only, native `fire-rules'`, `Record → Lemma* → Deduction`.
  The smallest slice of the engine (no beta), the on-ramp to the streaming rete service (R25), which **dogfoods this
  facility to measure itself.**

## Naming discipline

The **whole vocabulary is intueri-cast + ratified** (2026-07-04, two casts weighed against the disk):

- **query layer** — `Record` / `Lemma` / `Deduction` / `TableSchema` / `IndexSchema` / `IndexTarget` / `Query` /
  `Result` / `NextToken`.
- **records + enums** — `Scope` / `Metric` / `Log` / `LogMessage` / `Numeric` / `Unit` / `Level` / `Tags` (settled on
  clarity — FQDN means no collision axis).
- **producer surface** — the **sink** `TelemetryService'` (the prime replaces the legacy `Service<E,G>`; `Service` is the
  essential nature — remote/local/N-host, mutex-free), the **producer** `Span`, provisioning `open` (pairs with
  `Close`), ops `Log` / `Incr` / `Timed` / `Nest` / `Close`, the timing-widget macro `timed`, sink verbs `Write*` /
  `Query*`, and the store nouns `Store` / `Table` / `Index` (mirroring `wat.query/{TableSchema, IndexSchema}`).

Standing rule (builder): **every naming decision is resolved by CASTING intueri** — materialize the candidates as an
artifact, spawn the ward, weigh its verdict against the disk, ratify — never by narrating the ward.
(`feedback_cast_wards_never_narrate_naming_via_intueri`; 278 interstitial `INCANTO NON NARRO`.)
