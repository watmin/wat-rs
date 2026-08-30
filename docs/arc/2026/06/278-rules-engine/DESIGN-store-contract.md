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

## The data model — DynamoDB as the narrow waist; `(pk, sk, data)` all EDN

A record is **`(pk, sk, data)`** — **all three are EDN forms**, and the backend **parses none of them**:
- **`pk`** — the partition key: *just enough EDN to name a partition* (a namespace, an entity kind, …).
- **`sk`** — the sort key: *just enough EDN to sort within the partition.* An **orderable** string; NOT assumed to be
  time (time is one consumer's choice).
- **`data`** — the record's full tagged EDN (the most robust form). Opaque to the backend; only the consumer + rete
  decode it.

The store keys and orders on `pk`/`sk` and returns `data`; it stores serialized strings and **never inspects their
structure.**

**Keys are strings that are EDN/s-expr data forms** — this is the load-bearing convention. A key like
`#wat.telemetry'/sk {:kind :metric :name "requests" :time #inst "…"}` serializes to the sort string, and the *same
string* **hydrates back into a domain object** (`read-string` → a record) in the consumer's hand — copy a key out of a
log, parse it into existence, hand it to a func. The **consumer owns key structure and order-preservation** (ISO-8601
time sorts lexicographically; the consumer builds sensible keys); the store **never policies them** — a nonsense key
just yields a nonsense query result. One partition holds **unboundedly many sort-key shapes** (`{:kind :metric …}`,
`{:kind :log …}`, `{:uow …}`); you slice with a **prefix/range query on `sk`** — the exact single-table-design power,
but over **typed, hydratable EDN forms**, not Rick-Houlihan `TYPE#id#TYPE#id` term-octothorpe strings (no delimiter
grammar to escape or collide with).

A **named GSI** is an independent access path defined by two **projected keys `(ipk, isk)`** (also EDN-form strings).
Because `data` is opaque, **the backend cannot derive the index keys from it** — the **consumer supplies `(ipk, isk)`
per index at write time.** A GSI query selects by `ipk`, prefix/range-scans by `isk`, returns the same opaque `data`.

This is DynamoDB's model used deliberately as the **narrow waist** ([[project_wat_is_linux_best_of_breed]]): the minimal
key-value+index shape every real store expresses, so a consumer written to it ports across backends unchanged. **The
store hosts data; the consumer sets the rules** — how records map to `(pk,sk)`, what the sort-key shapes mean, which GSIs
to project, and (for telemetry) that a metrics `sk` is time so the prefix/range *is* the time slice. The store is dumb
and general; the intelligence lives in the consumer.

## The contract — the `Store` surface

`Store` is a **methods-bearing surface** (arc-293 subsumes `defprotocol`: a methods-only surface *is* a protocol). A
backend **satisfies** it by supplying the method impls; the consumer holds a `Store` and never names the concrete
backend. The handle holds a live resource (a connection), so a satisfier is a **`Struct`** (impure holder) — hence the
surface's `:holder` bound is `:wat::core::Struct` (widest: accepts struct/record/holon).

```clojure
;; the store handle a backend hands back (a live, thread-owned resource → a Struct). Read-write.
;; open / open-readonly are per-BACKEND free functions (a constructor, not a Store method) — each backend
;; provides its own, returning a value that satisfies Store / ReadStore.

;; ═══ the contract — names intueri-cast + ratified (2026-07-05) ═════════════════════════════════
(wat.core/defsurface wat.query/Store :holder wat.core/Struct
  :features
  [;; idempotently establish the store for (pk,sk,data) + the declared GSIs. Called once at consumer init.
   (ensure-schema [self <- :Store  table <- wat.query/TableSchema  indexes <- (wat.core/Vector wat.query/IndexSchema)] -> wat.query/Ok)

   ;; write a batch ATOMICALLY (one transaction). Each row carries its opaque data + the (ipk,isk) it projects
   ;; to for each declared GSI (supplied by the consumer's write path — the backend cannot read `data`).
   (put [self <- :Store  rows <- (wat.core/Vector wat.query/StoredRow)] -> wat.query/Ok)

   ;; a PAGE on the base key: pk fixed, sk in a prefix/range, ordered ASC, after `cursor`.  → Page of Row.
   (scan [self <- :Store  q <- wat.query/ScanRequest] -> wat.query/Page)

   ;; a PAGE on a named GSI: ipk fixed, isk in a prefix/range, ordered ASC, after `cursor`.  → IndexPage of IndexRow.
   (scan-index [self <- :Store  q <- wat.query/IndexScanRequest] -> wat.query/IndexPage)])

;; a read-only satisfier — the capability-honest half (the type is the proof a reader cannot write).
(wat.core/defsurface wat.query/ReadStore :holder wat.core/Struct
  :features [(scan       [self <- :ReadStore  q <- wat.query/ScanRequest]      -> wat.query/Page)
             (scan-index [self <- :ReadStore  q <- wat.query/IndexScanRequest] -> wat.query/IndexPage)])

;; ═══ the value shapes the contract speaks — keys are EDN-form STRINGS (see § data model) ═══════
;; already ratified: TableSchema [pk sk] · IndexSchema [pk sk ipk isk] · NextToken [resume-time]

;; — the WRITE input —
(wat.core/defrecord wat.query/StoredRow                       ;; one record to put
  [pk         :- wat.core/String                              ;; EDN-form key string; the consumer serializes ↔ hydrates
   sk         :- wat.core/String
   data       :- wat.core/String                              ;; the record's tagged EDN, opaque to the backend
   index-keys :- (wat.core/HashMap wat.core/String wat.query/IndexKey)])  ;; index-name → (ipk, isk)
(wat.core/defrecord wat.query/IndexKey  [ipk :- wat.core/String  isk :- wat.core/String])

;; — the READ results (what scan / scan-index hand back; the consumer HYDRATES `data` + works the keys) —
(wat.core/defrecord wat.query/Row      [pk :- wat.core/String  sk :- wat.core/String  data :- wat.core/String])
(wat.core/defrecord wat.query/IndexRow                        ;; the 4-keyed index row
  [pk :- wat.core/String  sk :- wat.core/String               ;; the base keys
   ipk :- wat.core/String isk :- wat.core/String              ;; the GSI's OWN keys
   data :- wat.core/String])

;; — the PAGE requests (a prefix/range on the sort key; range subsumes prefix: begins_with p = [p, p+sentinel]) —
(wat.core/defrecord wat.query/ScanRequest                     ;; a base-table page request
  [pk     :- wat.core/String
   sk-lo  :- wat.core/String  sk-hi :- wat.core/String        ;; the sort-key prefix/range slice (the consumer's time slice, etc.)
   limit  :- wat.core/i64
   cursor :- (wat.core/Option wat.core/String)])              ;; None = first page; Some sk = resume after (keyset)
(wat.core/defrecord wat.query/IndexScanRequest                ;; a GSI page request
  [index  :- wat.core/String
   ipk    :- wat.core/String
   isk-lo :- wat.core/String  isk-hi :- wat.core/String
   limit  :- wat.core/i64
   cursor :- (wat.core/Option wat.core/String)])

;; — the PAGES (the results + the keyset resume cursor) —
(wat.core/defrecord wat.query/Page       [rows :- (wat.core/Vector wat.query/Row)       next-cursor :- (wat.core/Option wat.core/String)])
(wat.core/defrecord wat.query/IndexPage  [rows :- (wat.core/Vector wat.query/IndexRow)  next-cursor :- (wat.core/Option wat.core/String)])
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
  The error channel is **RESOLVED (2026-07-05)**: an errors-as-record **defenum on the recovery axis** — the satisfier's
  `…::Error` with variants the caller is forced to branch on (sqlite's: `Transient` / `Constraint` / `Fatal`), each
  carrying a fault record (op / code / sql / message). See `DESIGN-sqlite-core.md § Errors`.

## The abstraction measure — sqlite, mysql, mongo (the proof the contract is right)

The contract is *correct* iff independent backends satisfy it while sharing nothing internally. They do:

| contract op | **sqlite** (`:wat::sqlite'`) | **mysql** (hypothesis) | **mongo** (hypothesis) |
|---|---|---|---|
| model | `main(pk,sk,data, +ipk-cols)` `PK(pk,sk)` | InnoDB `main(pk,sk,data, +ipk-cols)` `PK(pk,sk)` | collection `{pk,sk,data,uuid,…}`, `_id=(pk,sk)` |
| GSI | native `CREATE INDEX(ipk_col, sk)` | native secondary `INDEX(ipk_col, sk)` | compound index `{uuid:1, sk:1}` |
| `put` | replace-by-`(pk,sk)`: `DELETE`+`INSERT` in `BEGIN`/`COMMIT` (PutItem; a duplicate key is unrepresentable) | `REPLACE` / `INSERT … ON DUPLICATE KEY UPDATE` | `replaceOne({pk,sk}, doc, {upsert:true})` |
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

## Naming (intueri-cast + ratified, 2026-07-05)

`Store` / `ReadStore` · `ensure-schema` / `put` / `scan` / `scan-index` · `StoredRow` / `IndexKey` · `Row` / `IndexRow` ·
`ScanRequest` / `IndexScanRequest` · `Page` / `IndexPage`. (`ScanRequest`/`IndexScanRequest` over `ScanQuery`/`IndexScan`
— the `…Query`/`Scan` suffix sits too close to the taken rete `wat.query/Query`/`Result`; `Row`/`IndexRow` are the
backend-agnostic result records the sqlite driver *produces*.)

## Resolved (2026-07-05)

- **The error channel shape — RESOLVED:** an errors-as-record defenum on the recovery axis (`Transient`/`Constraint`/
  `Fatal`, each carrying a fault record). See `DESIGN-sqlite-core.md § Errors`.
- **A satisfier held behind the surface type WORKS — proven** (`probes/surface-field-dispatch.wat` → `142`): a
  methods-bearing `defsurface` (`Store`), a `defstruct` satisfier `extend-type`d to it, held in a struct **attribute
  typed as the surface** (`[store <- :wat::query/Store]`), dispatches its methods (`put`/`scan`) to the concrete
  backend at runtime (`src/runtime.rs:5339` existential dispatch; `src/check.rs:13666` surface-typed field). This is
  what lets a consumer (the telemetry service) hold a `Store` attribute and never name a backend. The 293.W containment
  rule makes it correct-by-construction: a `:holder :Struct` surface field is impure → it can only live in a struct /
  `:ephemeral`, never in a portable record / durable / on the wire (a live connection *cannot* cross the boundary).

## Open (for the strike)

- **Home/namespace** — the contract lives in `:wat::query` (the general engine's vocabulary, net-new, unprimed), core,
  adjacent to rete; a satisfier lives in its backend's namespace (`:wat::sqlite'`).
