# DESIGN — the telemetry key-schema (the consumer's single-table design over `Store`)

> **STATUS: DESIGN (2026-07-05), model settled.** How `Metric`/`Log` map their fields into `(pk, sk)` EDN forms and which
> GSIs they project. This is the **consumer's** single-table design *on top of* the general `Store` contract
> (`DESIGN-store-contract.md`) — the store hosts, this doc sets the rules. It **supersedes** the earlier
> "different shapes → separate stores" line: the store holds `data` opaquely, so shape never justified splitting the
> correlation apart — metrics and logs are *related* (joined by `uuid`), so they share **one table.**

## Single-table, done responsibly

Single-table design means **co-locate what's accessed together** — not "one global table for the universe" (the
cargo-culted anti-pattern). Metrics and logs of a unit of work are *related* (the trace/span correlation, `P4` below), so
they live in **one table**, and the `uuid` GSI joins across the kind-partitions in a single query. A genuinely unrelated
future domain gets its **own** table — a `Store` instance *is* one table, and the consumer opens as many as its
access-pattern clusters need. No dogma either way.

## Access patterns

| # | pattern | how it's served |
|---|---|---|
| **P1** | write a batch (Span close / log emit) | `put` to the `(kind, ns)` partition, `sk = time`, + the `uow` GSI projection |
| **P2** | metrics in a namespace over a time range, time-ordered | `scan` the `(:metrics, ns)` partition, `sk` range on time |
| **P3** | logs in a namespace over a time range | `scan` the `(:logs, ns)` partition, `sk` range on time |
| **P4** | **correlate** — everything (metrics + logs) for a unit-of-work `uuid` | `scan-index` the `uow` GSI on `(:uuid, id)` — one query, across partitions |
| **P5?** | (future) by metric name / by caller / by level / all partitions' metadata | additional GSIs — incl. **SKI** (inverted) below |

## The keys — EDN/s-expr forms (kind in the PARTITION key, not the sort key)

The **kind is a partition dimension**, not a sort prefix. Metrics and logs get their own PKs; the sort key is *just*
time; the `uuid` GSI resolves correlation across the partitions.

```clojure
;; ── base table (one, single-table) ──
pk = "(:metrics, :market-eval)"      ;; metrics of a namespace → one partition   (an s-expr/EDN key string)
pk = "(:logs,    :market-eval)"      ;; logs of a namespace    → its own partition
sk = "#inst \"2026-07-05T…\""        ;; the sort is JUST time — no {:kind …} in the sk

;; ── the "uow" GSI — cross-partition correlation (the trace/span join) ──
ipk = "(:uuid, :01J…)"               ;; the unit-of-work id
isk = "#inst \"2026-07-05T…\""       ;; time within the unit of work
;;   P4: (scan-index store "uow" ipk=(:uuid, id) …) → every Metric AND Log of that unit of work, across partitions.
```

- **Why kind-in-pk beats kind-in-sk:** each kind's time-series is a clean partition (no mixed hot partition, no
  sort-key-prefix gymnastics); `P2`/`P3` are plain range-scans on their partition; `P4` is the GSI joining across them.
- **The keys are EDN/s-expr forms** — `(:metrics, :market-eval)` serializes to the `pk` string *and* hydrates back
  (`read-string` → the tuple) in the consumer's hand. `sk`/`isk` are `#inst` (ISO-8601 → lexicographic == chronological,
  order-safe). No Rick-Houlihan `TYPE#id#TYPE#id` term-octothorpe strings — typed, hydratable, no delimiter grammar.
- **The store never parses any of it.** These forms are the consumer's rules; `Store` just keys, orders, and returns
  opaque `data`.

## The SKI — inverted GSI (a general capability the contract already has)

A **sort-key index (SKI)** is a GSI keyed *inverted* — `ipk = <a shared sk value>`, `isk = <the base pk>` — so you can
query **every partition** that carries that sk value in one shot. The classic use: every partition writes a
`"metadata#"` item; a SKI on it lists all partitions' metadata at once.

```clojure
;; a partition-crossing metadata sweep, if an access pattern wants it:
;;   every partition also writes an sk = "(:metadata)" row; declare a GSI "ski":
ipk = "(:metadata)"                  ;; the shared sk value, now the GSI's partition key
isk = "(:metrics, :market-eval)"     ;; the base pk, now the GSI's sort key
;;   (scan-index store "ski" ipk=(:metadata) …) → one row per partition, all metadata in one query.
```

The store needs **no special case** — a SKI is just a GSI whose `(ipk, isk)` the consumer chose to invert. That the
inverted-index pattern falls out of the plain `(ipk, isk)` projection is another proof the `Store` abstraction is right.

## What this fixes / decides

- **Supersedes** `DESIGN-telemetry-service-and-query-surface.md`'s "metrics and logs → separate stores" — it is **one
  table**, kind in the composite PK.
- The `Scope`/`Metric`/`Log` **records are unchanged** (they carry `namespace`/`uuid`/`tags`/`time-ns` etc.); the
  key-schema is the write path's mapping of those fields into `StoredRow.{pk, sk, index-keys}` — the sink's write path
  builds the `(:metrics|:logs, ns)` pk, the `#inst` sk, and the `uow` GSI's `(ipk, isk)` from each record.

## Open (for the strike)

- The exact **EDN spelling** of the key forms (`"(:metrics, :ns)"` s-expr vs a tagged record `#…/pk {…}`) — a small
  serialization choice; both hydrate. Settle at the sink write-path strike.
- **Which GSIs ship v1** — the `uow` (`uuid`) correlation GSI is required (`P4`); `name`/`caller`/SKI GSIs are additive
  (declare when the access pattern surfaces; forward-ref backfill is out of scope).
