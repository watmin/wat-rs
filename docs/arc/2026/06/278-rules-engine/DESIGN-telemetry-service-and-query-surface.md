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

## The contractual surface (the source of truth for callers)

```clojure
;; ═══ TelemetryService' — the contractual surface ═══════════════════════════════════════

;; ─────────────────────────── WRITE ───────────────────────────
;; You pass a BATCH (size ≥ 1). A batch is all-metrics OR all-logs — never mixed.
;; The service stamps sk (time) on receipt.

(wat.core/typealias wat.telemetry/Tags               ;; imposed on both paths
  (wat.core/HashMap wat.core/Keyword wat.core/String))

;; a metric — a typed, aggregatable shape (own table)                     [names: PROVISIONAL]
(wat.core/defrecord wat.telemetry/Metric
  [namespace :- wat.core/String   name  :- wat.core/Keyword
   value     :- wat.core/f64      unit  :- wat.core/Keyword
   tags      :- wat.telemetry/Tags])

;; a log message — a SURFACE: the caller passes ANY pure record they define
(wat.core/defsurface wat.telemetry/LogMessage :holder wat.core/Record :features [])

;; a log entry — the envelope around the caller's message record (own table)  [names: PROVISIONAL]
(wat.core/defrecord wat.telemetry/LogEntry
  [namespace :- wat.core/String   level :- wat.core/Keyword
   tags      :- wat.telemetry/Tags  message :- wat.telemetry/LogMessage])

;; the write verbs (defservice ops):
;;   (TelemetryService'/write-metrics conn (…batch of Metric…))    -> #…/WriteResponse {:ok true}
;;   (TelemetryService'/write-logs    conn (…batch of LogEntry…))  -> #…/WriteResponse {:ok true}

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
2. **Write-side names are provisional** — `Metric` / `LogEntry` / `LogMessage` / `write-metrics` / `write-logs`. Cast
   intueri before code lands (naming discipline below).
3. **`Metric.value :- f64`** assumes numeric metrics (counts widen to f64). Decide integer/float split vs one numeric
   type — grounded against how metrics aggregate.

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
