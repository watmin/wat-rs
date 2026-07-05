# DESIGN — `TelemetryService'` + the query surface (rete-as-datalog, paginated)

> **STATUS: DESIGN RATIFIED + REFINED (2026-07-04), unbuilt.** The contractual surface below is settled through
> the session's rulings. The **query vocabulary is intueri-cast + ratified**; the **write-side names are provisional**
> (`wat.telemetry/*`, the enums, the producers, the verbs, the store abstraction) — they get a single intueri cast
> before code lands. This doc is the durable record so the next self resumes from it, not from re-derivation.
>
> Refined this session: the **two-layer split** (`wat.query` = the general engine, `wat.telemetry` = its consumer);
> the **surface architecture** (`Scope`/`Metric`/`Log` exact surfaces, `Record`/`LogMessage` open surfaces —
> mechanism grounded, below); the **service serves read AND write** (four ops, `defservice` serializes); the
> **enums grow-as-needed**; the **store is swappable**. All four prior open items are resolved (see § Resolved).

## Why — the interrogation, rebuilt correctly

*"I cannot diagnose a system I cannot interrogate — we are building the interrogation."* This is the interface the
builder wanted from **Elasticsearch, CloudWatch, Nagios, Grafana** — not to *be* them, but because he needs
high-fidelity metrics and logs so **the machines speak and we read them, and rapidly find the next attack** (the
DDoS/anomaly lineage; the chaos engine, R25 `MACHINA CHAOS DOMAT`). It is *the database is the debugger* made a
service.

The filter is **reasoning over data, via data — and that is the rete we built.** Pure rules cascading rules at the
line as server-side filter expressions (`PORTA PORTAM APERIT`): slurp a page of records into working memory,
alpha-filter, serve the `Deduction`s; want more, resume from the offset (`NextToken`). An assault on the query
surfaces of Mongo/DDB/the lot — because their filter is not *reasoning*; ours is. **Logs are data here** (pure
records, EDN — never strings), so the same rete reasons over logs and metrics alike.

It is also the **exemplar for `defservice`** (arc-170: "replace the hand-rolled services with defservice," done
right) **and the instrument the rete streaming service dogfoods to measure itself** — measure-first (R25/R26): the
rete-as-a-service emits telemetry to the telemetry service to measure its own behavior, and the telemetry service's
own query-back *is* a datalog which *is* rete. The loop closes; the measurer and the measured are the same substrate
turned on itself. This lands the assault toward the north star: **`wat-mcp`** — the wat REPL (arc 118 `DVO MVNDI VNA
LINGVA`) plugged into the machine, usable by any instance.

The legacy telemetry stack is **annihilated, not preserved** (the bridge that got us here — built in Rust once, in
an early wat once; the shape is what we kept, not the code). Records-are-EDN (arc 300) retired the old carriers
(`HolonAST`/`NoTag`/`Tagged`/`Event`): a record writes as `#wat.ns/Name {…}` (tagged, round-trip-safe) and decodes
straight back to the populated instance, so the data carrier is the record itself.

## The two layers — the general engine, and its consumer

- **`wat.query`** — the **general-purpose rete-as-datalog / rete-as-filter.** Domain-blind: it filters *anything*
  satisfying `wat.query/Record`. The query vocabulary (`Record`/`Lemma`/`Deduction`/`TableSchema`/`IndexSchema`/
  `IndexTarget`/`Query`/`Result`/`NextToken`) lives here and is ratified. Anyone can write records satisfying the
  query interface and use the rete-filter to read them back.
- **`wat.telemetry`** — a **consumer** of `wat.query`. Its `Metric` and `Log` satisfy `wat.query/Record`; the
  telemetry service writes them and queries them back through the shared engine.

## The data model — `(namespace, time, data)`, DynamoDB-shaped, storage swappable

A DynamoDB single-table design **per store**: primary key `(pk, sk)` = `(namespace, iso8601-nanos)`; `data` is the
stored record's **tagged EDN**. Rows are time-sorted within a namespace; a query selects a namespace over a
timeframe, filters server-side (rete), paginates.

**The store is swappable.** What we depend on is *"a thing that holds records indexed and sorted by `(pk, sk)`"* —
that abstraction is the swap point at scale. `sqlite` is **one driver** (the local default); DDB, Mongo, whatever,
slot in behind the same shape. `sqlite` is *not* the requirement, only the current holder-of-records-by-`(pk,sk)`.

**Metrics and logs are different shapes → separate stores** (per kind). A batch is homogeneous — all metrics OR all
logs, never mixed — and holds ≥ 1.

## The correlation model — the unit of work; `Scope` the shared constraint

`Metric` and `Log` are not independent rows — they are the output of a **unit of work** (a measurement scope), and
the whole point is **correlation**. A unit of work opens with a `namespace` + `tags`, mints a **`uuid`** (its
identity), bumps counters (`incr!`), times sub-blocks (`timed`), and emits its metrics and logs sharing that `uuid`.
Metrics ↔ logs join on it — the trace/span observability model, served by a **GSI on `uuid`**.

The four fields common to both — `namespace`, `uuid`, `tags`, `time` — are lifted into **`Scope`**, an **exact
surface** that `Metric` and `Log` each **splice in** via the arc-293 **surface-splice** `[~@wat.telemetry/Scope
own…]` — *not* re-listed. This makes "same unit of work" a **structural fact** (a shared constraint), carries the
correlation key by construction, and keeps the common shape from a **single source** (the *derive-is-the-wall*
doctrine — composed in, never re-implemented and hoped). `[name: PROVISIONAL — intueri]`

## The surface architecture (structural, all the way down)

`defsurface` with `:features [field :- Type …]` is an **exact surface** (required typed fields); `:features []` is an
**open surface** (any record with the right holder satisfies it). A concrete record **structurally satisfies** a
surface when it carries all the floor fields at those types (grounded: `wat/core.wat` `:wat::core::Error`/`Fault`,
arc 296 S1/S2). So the whole contract is imposed structurally — a record that isn't the right shape *cannot be
written* (constraint engineering: no form for the wrong thing).

```
wat.query/Record        open surface   — anything the query engine matches (a stored row → WM fact)
  └─ Scope              exact surface  — the correlation core (namespace, uuid, tags, time); the shared constraint
       ├─ Metric        exact surface  — Scope ⊕ start-time ⊕ name ⊕ (value :- Numeric) ⊕ (unit :- Unit)
       └─ Log           exact surface  — Scope ⊕ caller ⊕ (level :- Level) ⊕ (message :- LogMessage)
            └─ LogMessage   open surface — any pure record the caller defines
```

`Metric` is exact all the way down (closed `Numeric` payload); `Log` is exact-around-an-open-message. Both **splice
`Scope` in** — the arc-293 **surface-splice** `[~@wat.telemetry/Scope own-fields…]` (flat; NOT inheritance; the
293.4c surface-extend path lets a surface's `:features` splice another surface's members). So `Scope` is the
**single source** of the correlation core and its fields **cannot drift**: re-listing them in each would be
hand-authored duplication that rots — the *derive-is-the-wall* doctrine at the field layer, the shared constraint
**composed in, never re-implemented and hoped.** Both therefore satisfy `Scope` by construction, and both satisfy
`wat.query/Record`.

## The contractual surface (the source of truth for callers)

```clojure
;; ═══ wat.query — the general rete-as-datalog (domain-blind, ratified) ═════════

(wat.core/defsurface wat.query/Record   :holder wat.core/Record :features [])   ;; open: a stored row asserted into WM
(wat.core/defrecord  wat.query/Lemma     [])                                     ;; intermediate stepping-stone fact
(wat.core/defrecord  wat.query/Deduction [record :- wat.query/Record])           ;; terminal; the only fact queried out

;; stored key-layouts (declared at creation):
(wat.core/defrecord wat.query/TableSchema [pk :- :String  sk :- :String])              ;; base: pk=namespace sk=iso8601
(wat.core/defrecord wat.query/IndexSchema [pk :- :String  sk :- :String
                                           ipk :- :String isk :- :String])             ;; a GSI's projected key columns
;; runtime selectors + envelopes:
(wat.core/defrecord wat.query/IndexTarget [name :- :String  pk :- :String  sk :- :String]) ;; which GSI + its key-values
(wat.core/defrecord wat.query/NextToken   [resume-time :- wat.core/Instant])               ;; the sk we stopped at

(wat.core/defrecord wat.query/Query
  [namespace  :- wat.core/String                        ;; the pk
   start-time :- wat.core/Instant                        ;; sk range lo
   end-time   :- wat.core/Instant                        ;; sk range hi
   index      :- (wat.core/Option wat.query/IndexTarget) ;; None = base table; Some = a GSI
   rules      :- (wat.core/Vector wat.rete/Rule)         ;; caller's filter rules — may deduce ONLY Lemma / Deduction
   next-token :- (wat.core/Option wat.query/NextToken)]) ;; resume cursor
;;  NB: NO `table` field — the kind rides the VERB (QueryMetrics / QueryLogs), not a field.

(wat.core/defrecord wat.query/Result
  [deductions :- (wat.core/Vector wat.query/Deduction)   ;; the matches this page
   next-token :- (wat.core/Option wat.query/NextToken)]) ;; Some = call again; None = done

;; ═══ wat.telemetry — a consumer of wat.query [names: PROVISIONAL, intueri] ════

(wat.core/typealias wat.telemetry/Tags (wat.core/HashMap wat.core/Keyword wat.core/String))

;; value's storage-type held by its variant NAME. GROWS AS WE CHOOSE — i64/f64 to launch,
;; the rest of Rust's/wat's numbers (incl. the arc-300 BigInt/BigRational tower) added later, as cases demand.
(wat.core/defenum wat.telemetry/Numeric wat.enum/Pure
  i64 [val :- wat.core/i64]
  f64 [val :- wat.core/f64])

;; the metric's semantic unit — a closed enum (name holds value); GROWS AS IT MUST.
(wat.core/defenum wat.telemetry/Unit wat.enum/Pure
  Count Seconds Millis Bytes Percent)

;; a log's level — a closed enum (same principle).
(wat.core/defenum wat.telemetry/Level wat.enum/Pure
  Debug Info Warn Error)

;; the correlation core — the shared constraint both Metric and Log satisfy (exact surface).
(wat.core/defsurface wat.telemetry/Scope :holder wat.core/Record
  :features [namespace :- wat.core/String       ;; pk
             uuid      :- wat.core/Uuid          ;; unit-of-work correlation id  → GSI
             tags      :- wat.telemetry/Tags     ;; dimensions
             time      :- wat.core/Instant])     ;; the sk

;; a metric — EXACT surface: Scope ⊕ span-start ⊕ name ⊕ value ⊕ unit.
;; SPLICE Scope in (surface-splice) — do NOT re-list its fields; Scope is the single source (derive-is-the-wall).
(wat.core/defsurface wat.telemetry/Metric :holder wat.core/Record
  :features [~@wat.telemetry/Scope                ;; splice the correlation core: namespace, uuid, tags, time(=end-time,the sk)
             start-time :- wat.core/Instant       ;; the scope span (start)
             name       :- wat.core/Keyword       ;; the counter/timer name          → GSI candidate
             value      :- wat.telemetry/Numeric  ;; the count/duration — variant name holds the storage type
             unit       :- wat.telemetry/Unit])   ;; the semantic unit — orthogonal to Numeric's storage type

;; a log message — OPEN surface: the caller passes ANY pure record they define (no field/method requirements).
(wat.core/defsurface wat.telemetry/LogMessage :holder wat.core/Record :features [])

;; a log — EXACT surface (exact envelope, open payload): Scope ⊕ caller ⊕ level ⊕ message.
;; SPLICE Scope in (surface-splice) — the same single source as Metric; the correlation core cannot drift.
(wat.core/defsurface wat.telemetry/Log :holder wat.core/Record
  :features [~@wat.telemetry/Scope                ;; splice the correlation core: namespace, uuid, tags, time(=emit moment,the sk)
             caller    :- wat.core/Keyword        ;; producer identity                → GSI candidate
             level     :- wat.telemetry/Level     ;; a closed enum
             message   :- wat.telemetry/LogMessage]) ;; a PURE RECORD (open surface)

;; the producers — the unit-of-work SCOPE (mints uuid, incr!/timed → Metric-shaped rows at close)
;; and the LOGGER (pulls namespace+uuid+tags from the scope, + caller + level + message → Log-shaped rows).
;; They are named in the Metric/Log FAMILIES (WorkUnit'/WorkUnitLog' were bridge placeholders — the shape kept,
;; the name retired). One intueri cast per family names the record AND its producer. [names: PROVISIONAL]

;; ═══ TelemetryService' — the defservice: read AND write, one op at a time ═════
;;   (TelemetryService'/WriteMetrics conn (…batch of Metric…))  -> #…/WriteResponse {:ok true}
;;   (TelemetryService'/QueryMetrics conn a-Query)              -> a-Result
;;   (TelemetryService'/WriteLogs    conn (…batch of Log…))     -> #…/WriteResponse {:ok true}
;;   (TelemetryService'/QueryLogs    conn a-Query)              -> a-Result
;; defservice SERIALIZES — one op at a time (the actor IS the synchronization; the sqlite handle in :ephemeral,
;; never crossing a thread). So ONE process serves both: write during a run, then diagnose after the fact on the
;; same db — or serve both live, the actor serializing. [service name + verb names: PROVISIONAL, intueri]
```

## Semantics

### Write
- Two homogeneous batch paths (`WriteMetrics` / `WriteLogs`), each ≥ 1. The service stamps `sk` (iso8601-nanos) on
  receipt. Each record is written as its **tagged EDN** into the `data` column of its store, plus the projected
  key/index columns (below).

### Query — the rete filter, per page
Server-side filtering is **rete-as-datalog** (`wat.query`), run **one page at a time**:
1. Range-scan a page of rows by `(pk, sk)` (or the GSI's projected columns if `index` is `Some`).
2. Assert each row as a **`Record`** fact into working memory.
3. Fire the caller's `rules`. Rules cascade forward (`PORTA PORTAM APERIT`): deduce a **`Lemma`** (stepping-stone),
   a downstream rule stands on it, until a terminal rule deduces a **`Deduction`** wrapping the matched `Record`.
4. **Query out only `Deduction`s.** Return them + the `sk` we stopped at as the `next-token`.

**The wall — rules may deduce ONLY `Lemma` or `Deduction`.** A closed set; anything else has no form. **No beta tree
— pagination imposes it:** a join partner may be on an unfetched or evicted page, so every rule is **per-record
(alpha-only) by construction**, which is exactly what makes the query **streamable**. (Rete terms: *alpha* = one
fact matches one pattern; *beta* = different facts join; *production* = all conditions satisfied → fire → insert a
fact. The query path needs only alpha + production.)

### Read AND write in one service (`defservice` serializes)
The service exposes all four ops. `defservice` is an actor — `handle(msg, state) → (reply, state')`, **one op at a
time** — so `WriteMetrics`/`QueryMetrics`/`WriteLogs`/`QueryLogs` never race over the shared `sqlite` handle; the
one-op-at-a-time *is* the mutex we don't write (zero-mutex / CSP). Pop up a `TelemetryService'` in a fresh process
with a db handle and it serves whatever it's asked — write live, diagnose after, or both.

### Pagination (DynamoDB-style)
`NextToken {resume-time}` is the `sk` the scan stopped at. Fetch a page, the rules deduce M matches, return the M
`Deduction`s + `NextToken{last-sk}`; the client re-calls with the token to continue. A page that reaches the end
with no deductions returns empty `deductions` + `next-token None` (done).

### GSIs — the crux, and why the store needs projection
SQL cannot index into the opaque `data` EDN. A base-table query ranges on `(pk, sk)`; a GSI query needs
`(pk, sk, ipk, isk)`, and `ipk`/`isk` live *inside* the record's EDN. So supporting GSIs requires the **write path
to project the index-key attributes out of the record into real, indexed columns** at write time — one materialized
secondary index per GSI (e.g. the `uuid` correlation index). `TableSchema` (base) and `IndexSchema` (a GSI's
projected columns) declare which columns each mode ranges on; `IndexTarget {name pk sk}` selects a GSI at query
time. Both modes flow through the identical page → assert → fire → paginate loop; only the store's `WHERE`/index
differs — and the store is the swap point, so this is a driver concern.

## Resolved (this session — the four open items closed)

1. **Table selection** → **the kind rides the verb.** Four ops (`WriteMetrics`/`QueryMetrics`/`WriteLogs`/
   `QueryLogs`); `Query` **drops its `table` field** — the verb already knows the store.
2. **Namespace** → **two layers.** `wat.query` = the general rete-as-datalog engine (domain-blind); `wat.telemetry`
   = the consumer holding `Metric`/`Log`/`Scope`/the enums/the service.
3. **Enums grow-as-needed.** `Numeric` = `i64` + `f64` to launch (the rest of Rust's/wat's numbers, incl. BigInt/
   BigRational, added later as chosen); `Unit` = the current set, grown as it must. The closed-set→enum rule holds;
   the *set* is open to growth, not frozen.
4. **The correlation core is a shared surface, spliced in.** `Scope` (exact surface: `namespace`/`uuid`/`tags`/
   `time`) — a shared constraint both `Metric` and `Log` **splice** via arc-293 surface-splice `[~@Scope own…]`
   (the single source; NOT re-listed — derive-is-the-wall at the field layer).

Plus, ruled this session:
- **The service serves read AND write** (four ops; `defservice` serializes one-op-at-a-time; one process serves both).
- **`Metric`/`Log`/`Scope` are exact surfaces; `LogMessage`/`Record` are open surfaces** (mechanism grounded:
  `defsurface :features [typed fields]` + structural satisfaction).
- **The store is swappable** (`sqlite` is one driver behind the holds-records-by-`(pk,sk)` abstraction).
- **The producers fold into the `Metric`/`Log` families** (`WorkUnit'`/`WorkUnitLog'` retired as names; shapes kept).

## Build implications (for the strike, later)

- **The store layer** (currently `crates/wat-telemetry-sqlite`) needs the `(pk, sk, data, …projected-index-columns)`
  table layout + GSI secondary indexes + the write-path projection of index-key attributes out of the record; and a
  range-scan/page read-path keyed on `(pk, sk)` / the GSI columns — all behind the **swappable store abstraction**
  (name: intueri territory). (The arc-085 `auto-*` enum-derive is NOT the fit — this is a fixed
  `(pk, sk, data, …)` layout, not a per-variant derive.)
- **`TelemetryService'` is a `defservice`** (baked source in `crates/wat-telemetry-sqlite/wat/telemetry/`), holding
  the store handle in `:ephemeral`, with the four ops. It uses `:rust::sqlite::*` interop (a baked source may;
  consumers use the `:wat::` verbs — arc-002 `NAMESPACE-PRINCIPLE`).
- **The query engine is a `wat.query` rete consumer** — alpha-only, `fire-rules'` (native), `Record → Lemma* →
  Deduction`. The smallest slice of the engine (no beta), and the on-ramp to the streaming rete service (R25), which
  will **dogfood this telemetry service to measure itself.**
- **Names**: draw the whole `wat.telemetry` + store vocabulary as a candidate artifact and **cast intueri once**
  before code lands (naming discipline below).

## Naming discipline

The query vocabulary (`Record` / `Lemma` / `Deduction` / `TableSchema` / `IndexSchema` / `IndexTarget` / `Query` /
`Result` / `NextToken`) is **intueri-cast + ratified**. The **write-side vocabulary is provisional** — the
`wat.telemetry` namespace, `Scope` / `Metric` / `Log` / `LogMessage`, `Numeric` / `Unit` / `Level`, the producers
(the metric-scope + logger), the four verbs, and the **store abstraction** + runtime `table`/`index` nouns. Standing
rule (builder): **every naming decision is resolved by CASTING intueri** — materialize the candidates as an
artifact, spawn the ward, weigh its verdict, ratify — never by narrating the ward.
(`feedback_cast_wards_never_narrate_naming_via_intueri`; 278 interstitial `INCANTO NON NARRO`.)
