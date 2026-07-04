# DESIGN — `TelemetryService'` + the query surface (rete-as-datalog, paginated)

> **STATUS: DESIGN RATIFIED (2026-07-04), unbuilt.** The contractual surface below is settled through the
> session's casts + rulings (recorded in `278/REALIZATIONS.md` R26 `EXPERGISCIMVR` + the `INCANTO NON NARRO`
> interstitial). The **query vocabulary names are intueri-cast + ratified**; the **write-side type/op names are
> provisional** (they get their own intueri cast before code lands). This doc is the durable record so the next
> self resumes from it, not from re-derivation.

## Why — the tool, rebuilt correctly

We are rebuilding the telemetry service as a **`defservice`** — an arc-170 item ("make IPC sane; replace hand-rolled
services with `defservice`") done *right*. It is also the **exemplar for the rete streaming service** (the chaos
engine, R25 `MACHINA CHAOS DOMAT`): a `defservice` that persists records and serves a **rete-filtered, paginated
query** is the shape the streaming datalog inherits.

The legacy telemetry stack (`crates/wat-telemetry*`, `Event`/`WorkUnit`/`WorkUnitLog`, arc 085/091) is **annihilated,
not preserved** (builder: *"we are annihilating what exists — we will not keep any contract that exists"*). Its
carriers — `HolonAST`, the `:wat::edn::NoTag`/`Tagged` write-strategy newtypes, the `Event` sum type — were the
**pre-`EdnRepresentable`** way to make structured data round-trip. Arc **300** ("wat source IS EDN") retired the need:
**records ARE EDN by construction** — a record writes as `#wat.ns/Name {…}` (tagged, round-trip-safe) and decodes
straight back to the populated record instance. So the data carrier is now the **record itself**, and `HolonAST` /
`NoTag` / `Tagged` / `Event` do not appear.

## The data model — `(namespace, time, data)`, DynamoDB-shaped

A DynamoDB single-table design **per store**: primary key `(pk, sk)` = `(namespace, iso8601-nanos)`; `data` is the
stored record's **tagged EDN**. Rows are time-sorted within a namespace. The query is "select from a namespace over a
timeframe, filter server-side, paginate." Read `data` back → the record instance → rete matches on it.

**Metrics and logs are different shapes → separate tables** (builder ruling). A batch is homogeneous — all metrics OR
all logs, never mixed — and holds ≥ 1.

## The correlation model — the unit of work (from the `WorkUnit` / `WorkUnitLog` review)

`Metric` and `Log` are not independent rows — they are the output of a **unit of work** (a measurement scope), and
the whole point is **correlation**. The legacy `WorkUnit` / `WorkUnitLog` had this shape *right* (builder: *"they had
the shape i wanted, but they were built wrong"*); the rebuild keeps the shape, fixes the carriers.

- **A unit of work** = a scope opened with a `namespace` + `tags`; it mints a **`uuid`** (its identity). Inside it,
  the body bumps counters (`incr!`) and times sub-blocks (`timed`). At scope-close it ships **metrics**: each counter
  → ONE `Metric` (final count, `Numeric/i64`, `Unit/Count` — CloudWatch model); each duration → ONE `Metric` PER
  SAMPLE (`Numeric/f64`, `Unit/Seconds`). Every metric carries the scope's `namespace`, `uuid`, `tags`, and span.
- **A logger** bound to the scope emits **logs** that pull `namespace` + `uuid` + `tags` from the same scope, plus a
  `caller` (producer identity), a `level`, and a structured `message` (a pure record).
- **`uuid` is the correlation key.** A unit-of-work's `Metric`s and `Log`s share it — "everything in this unit of
  work" is one `uuid`, and metrics ↔ logs join on it. The trace/span observability model — exactly what a **GSI on
  `uuid`** serves.

**Producer helpers (rebuilt): `WorkUnit'` / `WorkUnitLog'`** — the scope + logger that mint the `uuid`, collect
counters/durations into `Metric` records and level'd `Log` records, and ship them through `write-metrics` /
`write-logs`. Same behavior `WorkUnit`/`WorkUnitLog` always had; clean carriers (records not `HolonAST`/`WatAST`;
`HashMap<Keyword,String>` tags; a `defservice` sink). `[names: PROVISIONAL]`

**The closed-set rule (why `value`/`unit`/`level` are enums).** A field over a **closed set** is an enum whose
variant name holds the value (`Numeric` i64/f64 · `Unit` Count/Seconds/… · `Level` Debug/Info/Warn/Error) — only valid
values are representable, and they round-trip through EDN (`wat-tests/edn/roundtrip.wat`). A field that is an **open
identifier** stays `Keyword`/`String` (`namespace`, `uuid`, `caller`, `name`, tag values). Constraint engineering at
the field layer.

## The contractual surface (the source of truth for callers)

```clojure
;; ═══ TelemetryService' — the contractual surface ═══════════════════════════════════════

;; ─────────────────────────── WRITE ───────────────────────────
;; You pass a BATCH (size ≥ 1). A batch is all-metrics OR all-logs — never mixed.
;; The service stamps sk (time) on receipt.

;; Metric and Log are the CORRELATED unit-of-work records (see "The correlation model").
;; Both satisfy wat.query/Record. Their index-key fields (namespace, the time, uuid, caller,
;; name) get projected to columns; everything else rides in the record's tagged EDN.

(wat.core/typealias wat.query/Tags               ;; imposed on both paths
  (wat.core/HashMap wat.core/Keyword wat.core/String))

;; value's TYPE is held by its own variant NAME — a count is #Numeric/i64 [7], a duration #Numeric/f64 [0.42].
;; enums round-trip through EDN (wat-tests/edn/roundtrip.wat: #ns/Variant [body] reconstructed from the type registry).
(wat.core/defenum wat.query/Numeric wat.enum/Pure
  i64 [val :- wat.core/i64]
  f64 [val :- wat.core/f64])

;; the metric's SEMANTIC unit — a CLOSED set, so an enum (name holds value); PROVISIONAL variant set
(wat.core/defenum wat.query/Unit wat.enum/Pure
  Count Seconds Millis Bytes Percent)

;; a log's level — a CLOSED set, so an enum (same principle as Unit / Numeric)
(wat.core/defenum wat.query/Level wat.enum/Pure
  Debug Info Warn Error)

;; a metric — a unit-of-work's counter/duration data point (was Event::Metric)   [names: PROVISIONAL]
(wat.core/defrecord wat.query/Metric
  [namespace  :- wat.core/String        ;; pk
   uuid       :- wat.core/Uuid          ;; unit-of-work correlation id       → GSI
   start-time :- wat.core/Instant       ;; scope span (start)
   end-time   :- wat.core/Instant       ;; scope span (end) — sk
   name       :- wat.core/Keyword       ;; the counter/timer name            → GSI candidate
   value      :- wat.query/Numeric      ;; the count/duration — variant name (i64/f64) holds the storage type
   unit       :- wat.query/Unit         ;; the semantic unit — a closed enum, orthogonal to Numeric's storage type
   tags       :- wat.query/Tags])

;; a log message — a SURFACE: the caller passes ANY pure record they define
(wat.core/defsurface wat.query/LogMessage :holder wat.core/Record :features [])

;; a log — a unit-of-work's structured log line (was Event::Log)                 [names: PROVISIONAL]
(wat.core/defrecord wat.query/Log
  [namespace :- wat.core/String         ;; pk
   uuid      :- wat.core/Uuid           ;; correlation id                    → GSI
   time      :- wat.core/Instant        ;; emit moment — sk
   caller    :- wat.core/Keyword        ;; producer identity                 → GSI candidate
   level     :- wat.query/Level         ;; a closed enum (name holds value)
   tags      :- wat.query/Tags
   message   :- wat.query/LogMessage])  ;; a PURE RECORD (was Tagged<HolonAST> over a quoted WatAST)

;; the write verbs (defservice ops):
;;   (TelemetryService'/write-metrics conn (…batch of Metric…))  -> #…/WriteResponse {:ok true}
;;   (TelemetryService'/write-logs    conn (…batch of Log…))     -> #…/WriteResponse {:ok true}

;; ─────────────────────────── QUERY ───────────────────────────
;; You pass a Query, you get a paginated Result. Server-side filter is rete:
;;   each row → Record fact → user rules fire → deduce Lemma* → Deduction* → returned.

;; stored key-layouts (declared at creation) — "Schema", the definition:
(wat.core/defrecord wat.query/TableSchema [pk :- :String  sk :- :String])              ;; base: pk=namespace sk=iso8601
(wat.core/defrecord wat.query/IndexSchema [pk :- :String  sk :- :String
                                           ipk :- :String isk :- :String])             ;; a GSI's projected key columns

;; the runtime selectors + envelopes:
(wat.core/defrecord wat.query/IndexTarget [name :- :String  pk :- :String  sk :- :String]) ;; which GSI + its key-values
(wat.core/defrecord wat.query/NextToken   [resume-time :- wat.core/Instant])               ;; the sk we stopped at

(wat.core/defrecord wat.query/Query
  [table      :- wat.core/Keyword                        ;; :metrics | :logs  ← OPEN: field vs two verbs (see below)
   namespace  :- wat.core/String                         ;; the pk
   start-time :- wat.core/Instant                        ;; sk range lo
   end-time   :- wat.core/Instant                        ;; sk range hi
   index      :- (wat.core/Option wat.query/IndexTarget) ;; None = base table; Some = a GSI
   rules      :- (wat.core/Vector wat.rete/Rule)         ;; caller's filter rules — may deduce ONLY Lemma / Deduction
   next-token :- (wat.core/Option wat.query/NextToken)]) ;; resume cursor

;; the fact ladder the rules work over (server-side, per page):
;;   Record  →  Lemma*  →  Deduction   (Deduction wraps the matched Record; the ONLY fact queried out)

(wat.core/defsurface wat.query/Record   :holder wat.core/Record :features [])   ;; a stored row asserted into WM
(wat.core/defrecord  wat.query/Lemma     [])                                     ;; intermediate stepping-stone fact
(wat.core/defrecord  wat.query/Deduction [record :- wat.query/Record])           ;; terminal; the only fact queried out
(wat.core/defrecord  wat.query/Result
  [deductions :- (wat.core/Vector wat.query/Deduction)   ;; the matches this page
   next-token :- (wat.core/Option wat.query/NextToken)]) ;; Some = call again to continue; None = done

;; the query verb:
;;   (TelemetryService'/query conn a-Query)  ->  a-Result
```

## Semantics

### Write
- Two homogeneous batch paths (`write-metrics` / `write-logs`), each ≥ 1. The service stamps `sk` (iso8601-nanos) on
  receipt. Each record is written as its **tagged EDN** into the `data` column of its table, plus the projected
  key/index columns (below).

### Query — the rete filter, per page
Server-side filtering is **rete-as-datalog**, run **one page at a time**:
1. Range-scan a page of rows by `(pk, sk)` (or the GSI's projected columns if `index` is `Some`).
2. Assert each row as a **`Record`** fact into working memory.
3. Fire the caller's `rules`. Rules cascade forward (`PORTA PORTAM APERIT`): a rule deduces a **`Lemma`** (a
   stepping-stone), a downstream rule stands on it, until a terminal rule deduces a **`Deduction`** wrapping the
   matched `Record`.
4. **Query out only `Deduction`s.** Return them + the `sk` we stopped at as the `next-token`.

**The wall — rules may deduce ONLY `Lemma` or `Deduction`.** A closed set; a rule that deduces anything else has no
form. So "the results" is unambiguously "every `Deduction` after the fire."

**There is NO beta tree — pagination imposes it, precisely.** A beta (join) node correlates facts co-resident in
working memory, but a paginated query never holds the whole dataset — a join partner may be on a page not yet fetched
or already evicted. So a cross-record rule *cannot fire*: the join has no form. Every rule is **per-record (alpha-only)
by construction**, which is exactly what makes the query **streamable** — window the store, alpha-filter each page in
isolation, emit deductions + resume token, never hold the full dataset.

Rete-term note (for the record, since it recurs): **alpha** = "does THIS one fact match THIS pattern?" (all constraints
on one record — `form::matches?`). **beta** = "do these DIFFERENT facts join?" (a `?var` shared across conditions,
across records). **production/deduction** = "all conditions satisfied → fire → insert a fact." The query path needs
only alpha matching + production firing.

### Pagination (DynamoDB-style)
`NextToken {resume-time}` is the `sk` the scan stopped at. Fetch a full page (say 10 rows), the rules deduce M matches
(M ≤ page), return the M `Deduction`s + `NextToken{last-sk}`. The client re-calls with the token to continue. A page
that reaches the end with no deductions returns empty `deductions` + `next-token None` (done).

### GSIs — the crux, and why sqlite needs updates
**SQL cannot index into the opaque `data` EDN.** A base-table query ranges on `(pk, sk)` — columns we have. A GSI
query needs `(pk, sk, ipk, isk)`, and `ipk`/`isk` live *inside* the record's EDN, invisible to SQL. So supporting GSIs
requires the **write path to project the index-key attributes out of the record into real, indexed columns** at write
time — one materialized secondary index per GSI. `TableSchema` (base `pk`/`sk`) and `IndexSchema` (a GSI's projected
`pk`/`sk`/`ipk`/`isk`) are distinct types precisely because they declare which materialized columns each query mode
ranges on. `IndexTarget {name pk sk}` selects a GSI at query time. Both modes then flow through the identical
**page → assert → fire → paginate** loop; only the SQL `WHERE`/index differs.

## Open items (flagged, not decided)
1. **Table selection.** `Query.table :- :metrics | :logs` (one `query` verb) **vs** two verbs
   `query-metrics` / `query-logs` (mirroring the two write verbs). Ratify + (if the field wins) intueri-cast the name.
2. **Names are provisional** — `Metric` / `Log` / `LogMessage` / `Numeric` / `Unit` / `Level` / `WorkUnit'` /
   `WorkUnitLog'` / `write-metrics` / `write-logs`, the `Unit`/`Level` variant names, and the namespace (`wat.query`
   vs `wat.telemetry`). Cast intueri before code lands (naming discipline below).
3. **Enum variant SETS** — the provisional `Unit` set (`Count Seconds Millis Bytes Percent`), and whether `Numeric`
   needs more than `i64`/`f64` (e.g. bigint). Finalize against the metric domain.
4. **The shared correlation core.** `namespace` + `uuid` + `tags` (+ the time) are common to `Metric` and `Log`.
   Splice a `wat.query/Scope` surface into both (arc-293 `[~@:Scope own…]`) so they DRY-share it, or keep them flat?
   A real structural choice.

**RESOLVED this session:** `value` is `wat.query/Numeric` (the name-holds-the-type enum, `i64`/`f64`); `unit` is
`wat.query/Unit`; `level` is `wat.query/Level` — the closed-set rule (a closed enumeration is an enum, name holds
value; an open identifier stays `Keyword`/`String`).

## Build implications (for the strike, later)
- **`crates/wat-telemetry-sqlite` needs updates**: the `(pk, sk, data, …projected-index-columns)` table layout + GSI
  secondary indexes + the write-path projection of index-key attributes out of the record; and a range-scan/page
  read-path keyed on `(pk, sk)` / the GSI columns. (The current arc-085 `auto-*` enum-derive is NOT the fit — this is a
  fixed `(pk, sk, data, …)` layout, not a per-variant derive.)
- **`TelemetryService'` is a `defservice`** (baked source in `crates/wat-telemetry-sqlite/wat/telemetry/`), holding the
  sqlite handle in `:ephemeral`, with the write + query ops. It internally uses the `:rust::sqlite::*` interop (a
  baked source may; consumers use the `:wat::` verbs — arc-002 `NAMESPACE-PRINCIPLE`).
- **The query engine is a rete consumer** — alpha-only, `fire-rules'` (native), Record→Lemma→Deduction. It exercises
  the smallest slice of the engine (no beta), and is the on-ramp to the streaming rete service (R25).

## Naming discipline
The query vocabulary (`Record` / `Lemma` / `Deduction` / `TableSchema` / `IndexSchema` / `IndexTarget` / `Query` /
`Result` / `NextToken`) is **intueri-cast + ratified** this session. Standing rule (builder): **every naming decision
is resolved by CASTING intueri** — materialize the candidates as an artifact, spawn the ward, weigh its verdict,
ratify — never by narrating the ward. (`feedback_cast_wards_never_narrate_naming_via_intueri`; 278 interstitial
`INCANTO NON NARRO`.)
