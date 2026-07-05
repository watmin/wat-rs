# DESIGN — the `Store` contract: the backend requirement spec (backend-agnostic)

> **STATUS: DESIGN (2026-07-05), unbuilt.** The store the telemetry service writes to and queries is **swappable** —
> sqlite today, mysql/mongo/our-own later — and the consumer must never know which. This doc is that swap point made
> explicit: **the contract every backend must satisfy.** `:wat::sqlite'` (the first satisfier) is designed separately
> (`DESIGN-sqlite-core.md`); this doc is the contract it — and every future backend — is held to.

## Why a contract, not a driver

Two facts force this:
- **The telemetry service is backend-agnostic.** `TelemetryService'` writes `Metric`/`Log` records and queries them back;
  *where* they live is an injected dependency it never inspects. It holds a `[store <- :Store]` and calls the contract.
- **`wat.query` is backend-agnostic.** The rete-as-datalog filter reasons over **decoded records** in working memory; the
  backend only ever hands it **opaque pages**. The engine never touches SQL, Mongo, or any driver.

So the design is not "the sqlite driver." It is **the contract** — and the measure of whether the contract is right is
that sqlite, mysql, and mongo can each satisfy it while sharing *nothing* internally (see § The abstraction measure).
Exploiting a backend's features (native indexes, WAL, keyset pagination) is a **private driver detail the contract hides.**

## The data model — DynamoDB as the narrow waist

A record is **`(pk, sk, data)`**:
- **`pk`** — partition key (a string): *which namespace* the record belongs to.
- **`sk`** — sort key (an **orderable** string): time-ordering within the partition (iso8601-nanos for telemetry).
- **`data`** — an **OPAQUE payload the backend NEVER parses.** It is the record's tagged EDN; only the consumer + rete
  decode it. The backend stores bytes and returns bytes.

A **named GSI** (global secondary index) is an independent access path defined by two **projected keys `(ipk, isk)`**.
Because `data` is opaque, **the backend cannot derive the index keys from it** — so the **consumer supplies `(ipk, isk)`
per index at write time** (the write path knows the record's shape; the backend does not). A GSI query selects by `ipk`
and ranges/sorts by `isk`, returning the same opaque `data`.

This is DynamoDB's model used deliberately as the **narrow waist** ([[project_wat_is_linux_best_of_breed]]): the minimal
key-value+index shape every real store can express, so a consumer written to it ports across backends unchanged.

## The contract — the `Store` surface

`Store` is a **methods-bearing surface** (arc-293 subsumes `defprotocol`: a methods-only surface *is* a protocol). A
backend **satisfies** it by supplying the method impls; the consumer holds a `Store` and never names the concrete
backend. The handle holds a live resource (a connection), so a satisfier is a **`Struct`** (impure holder) — hence the
surface's `:holder` bound is `:wat::core::Struct` (widest: accepts struct/record/holon).

```clojure
;; the store handle a backend hands back (a live, thread-owned resource → a Struct). Read-write.
;; open / open-readonly are per-BACKEND free functions (a constructor, not a Store method) — each backend
;; provides its own, returning a value that satisfies Store / ReadStore.

;; ═══ the contract — names PROVISIONAL (intueri) ══════════════════════════════════════════════
(wat.core/defsurface wat.query/Store :holder wat.core/Struct
  :features
  [;; idempotently establish the store for (pk,sk,data) + the declared GSIs. Called once at consumer init.
   (ensure-schema [self <- :Store  table <- wat.query/TableSchema  indexes <- (wat.core/Vector wat.query/IndexSchema)] -> wat.query/Ok)

   ;; write a batch ATOMICALLY (one transaction). Each row carries its opaque data + the (ipk,isk) it projects
   ;; to for each declared GSI (supplied by the consumer's write path — the backend cannot read `data`).
   (put [self <- :Store  rows <- (wat.core/Vector wat.query/StoredRow)] -> wat.query/Ok)

   ;; range-scan a PAGE on the base key: pk fixed, sk in [lo,hi], ordered by sk ASC, after `cursor`.
   ;; returns up to `limit` rows (sk, data) + a next-cursor (the last sk) iff more remain.
   (scan [self <- :Store  q <- wat.query/ScanQuery] -> wat.query/Page)

   ;; range-scan a PAGE on a named GSI: ipk fixed, isk in [lo,hi], ordered by isk ASC, after `cursor`.
   (scan-index [self <- :Store  q <- wat.query/IndexScan] -> wat.query/Page)])

;; a read-only satisfier — the capability-honest half (the type is the proof a reader cannot write).
;; carries only the read half of Store.
(wat.core/defsurface wat.query/ReadStore :holder wat.core/Struct
  :features [(scan       [self <- :ReadStore  q <- wat.query/ScanQuery]  -> wat.query/Page)
             (scan-index [self <- :ReadStore  q <- wat.query/IndexScan]  -> wat.query/Page)])

;; ═══ the value shapes the contract speaks (records; some already ratified in wat.query) ═══════
;; already ratified: TableSchema [pk sk] · IndexSchema [pk sk ipk isk] · NextToken [resume-time]
(wat.core/defrecord wat.query/StoredRow                       ;; one record to put
  [pk       :- wat.core/String
   sk       :- wat.core/String
   data     :- wat.core/String                                ;; the tagged EDN, opaque to the backend
   index-keys :- (wat.core/HashMap wat.core/String wat.query/IndexKey)])  ;; index-name → (ipk, isk)
(wat.core/defrecord wat.query/IndexKey  [ipk :- wat.core/String  isk :- wat.core/String])

(wat.core/defrecord wat.query/ScanQuery                       ;; a base-table page request
  [pk     :- wat.core/String
   sk-lo  :- wat.core/String  sk-hi :- wat.core/String
   limit  :- wat.core/i64
   cursor :- (wat.core/Option wat.core/String)])              ;; None = first page; Some sk = resume after
(wat.core/defrecord wat.query/IndexScan                       ;; a GSI page request
  [index  :- wat.core/String
   ipk    :- wat.core/String
   isk-lo :- wat.core/String  isk-hi :- wat.core/String
   limit  :- wat.core/i64
   cursor :- (wat.core/Option wat.core/String)])
(wat.core/defrecord wat.query/Page                            ;; a page of opaque rows + the resume cursor
  [rows        :- (wat.core/Vector wat.query/StoredRow)       ;; (pk, sk, data) — data still opaque
   next-cursor :- (wat.core/Option wat.core/String)])         ;; Some sk = call again; None = done
```

## Semantics (the contract's promises)

- **`data` is opaque, always.** No backend parses it; no operation filters on its contents. Filtering is the consumer's
  job (`wat.query` rete, over decoded records). This is the load-bearing boundary — it is what makes the backend
  swappable.
- **`put` is atomic.** A batch either fully lands or fully rolls back — one transaction. A telemetry batch is homogeneous
  (all `Metric` or all `Log`) and ≥ 1.
- **`scan`/`scan-index` are paginated by KEYSET, not offset.** `cursor` is the last `sk`/`isk` returned; the next page is
  everything strictly after it. `next-cursor = None` means the page reached the end. No `OFFSET` (which re-scans);
  the cursor rides the key order. `NextToken` (the telemetry-facing token) *is* this cursor.
- **Ordering is by the sort key**, ascending, within a fixed partition/index key. The contract guarantees the order;
  the backend picks the mechanism (a B-tree, an LSM, whatever).
- **The consumer supplies projected index keys** at `put` time (`StoredRow.index-keys`). Adding a GSI = declaring an
  `IndexSchema` at `ensure-schema` + supplying its `(ipk,isk)` in every subsequent `put`. Backfilling an existing store
  with a new GSI is out of scope for v1 (declare indexes up front).
- **Errors are VALUES, not panics** (per `DESIGN-sqlite-core.md § errors`): a recoverable condition (store unreachable,
  constraint violation) returns a typed error; only an invariant violation the caller already guaranteed may panic.
  The contract's return types carry the error channel (the exact shape is settled with the sqlite design).

## The abstraction measure — sqlite, mysql, mongo (the proof the contract is right)

The contract is *correct* iff independent backends satisfy it while sharing nothing internally. They do:

| contract op | **sqlite** (`:wat::sqlite'`) | **mysql** (hypothesis) | **mongo** (hypothesis) |
|---|---|---|---|
| model | `main(pk,sk,data, +ipk-cols)` `PK(pk,sk)` | InnoDB `main(pk,sk,data, +ipk-cols)` `PK(pk,sk)` | collection `{pk,sk,data,uuid,…}`, `_id=(pk,sk)` |
| GSI | native `CREATE INDEX(ipk_col, sk)` | native secondary `INDEX(ipk_col, sk)` | compound index `{uuid:1, sk:1}` |
| `put` | prepared `INSERT` in `BEGIN`/`COMMIT` | `INSERT` in a txn | `insertMany(ordered:true)` |
| `scan` | `WHERE pk=? AND sk>?cur ORDER BY sk LIMIT n` | identical SQL | `find({pk, sk:{$gt:cur}}).sort({sk:1}).limit(n)` |
| `scan-index` | `WHERE ipk_col=? AND sk … (idx)` | identical | `find({uuid, sk:{$gt}}).sort({sk:1}).limit(n)` |

They **diverge on every mechanic and converge on every promise.** That divergence-with-conformance is the measure: the
contract names *what* (records by `(pk,sk)`, indexed by named `(ipk,isk)`, atomic batch writes, keyset-paginated ordered
scans), never *how*. (SlugDB's separate-index-table trick was a *DynamoDB* necessity — DDB has no native secondary
indexes, so a GSI there is a materialized copy; sqlite/mysql get native B-trees, mongo gets compound indexes. Each
backend's private business.)

## What this delivers

- **The consumer contract is frozen while backends improve.** `TelemetryService'` and `wat.query` are written to `Store`;
  a better backend (mysql at scale, mongo, our-own when sqlite lacks) drops in and **no consumer changes a line.**
- **sqlite is the first satisfier — a bridge to its own demise.** We have it now and make progress with it; the day
  sqlite is the bottleneck, the contract is already the seam that lets us replace it.

## Open (for the strike)

- **Names** (intueri): `Store` / `ReadStore` / `ensure-schema` / `put` / `scan` / `scan-index` / `StoredRow` /
  `IndexKey` / `ScanQuery` / `IndexScan` / `Page`. Cast once against the real forms.
- **The error channel shape** — settled jointly with `DESIGN-sqlite-core.md § errors` (typed error vs errors-as-record).
- **Home/namespace** — the contract lives in `:wat::query` (the general engine's vocabulary, net-new, unprimed), core,
  adjacent to rete; a satisfier lives in its backend's namespace (`:wat::sqlite'`).
