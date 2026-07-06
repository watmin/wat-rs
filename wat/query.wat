;; wat/query.wat — Arc 278 stone S0: the :wat::query backend-agnostic storage CONTRACT.
;;
;; Ratified in docs/arc/2026/06/278-rules-engine/DESIGN-store-contract.md. PURE DECLARATIONS
;; ONLY — no backend, no logic. `Store`/`ReadStore` are methods-bearing surfaces (arc-293's
;; defprotocol-subsuming form); a satisfier (`:wat::sqlite'::*` et al., stone S2) supplies the
;; impls. The narrow waist is DynamoDB's (pk, sk, data) + named-GSI (ipk, isk) shape: all keys
;; are EDN-form STRINGS the consumer serializes/hydrates; `data` is opaque EDN the backend never
;; inspects (§ data model). `wat.query` (the rete-as-datalog filter) reasons over decoded
;; records in working memory; the backend only ever hands it opaque pages.
;;
;; Two pinnings applied over the design doc's prose (`-> Ok` / `-> Page`):
;;   - errors-are-values: every fallible method returns
;;     `:wat::core::Result<T,wat::query::Error>` — never a bare success type.
;;   - the error channel is an errors-as-record `defenum` on the RECOVERY axis (the caller's
;;     forced branch: retry / surface / abort) — `:Transient` / `:Constraint` / `:Fatal`, each
;;     carrying a `Fault` (op / code / diagnostic / message).
;;
;; Only outward refs: `:wat::core::*` (String/i64/keyword/nil/Vector/Option/HashMap/Struct/
;; Result) + `:wat::enum::Pure`. Loads after `wat/core.wat` (defrecord/defenum/defsurface + those
;; primitives); placed near the rete sources — this is the query engine's vocabulary.

;; ─── the error channel — recovery-axis enum, each variant carrying a Fault ──────────────────
(:wat::core::defrecord :wat::query::Fault
  [op      <- :wat::core::keyword                          ;; which contract method faulted
   code    <- :wat::core::i64                               ;; backend-native error code (0 if n/a)
   diagnostic <- :wat::core::String                         ;; opaque backend-native diagnostic text (SQL text,
                                                            ;; command trace, key — driver-supplied; NOT SQL-specific)
   message <- :wat::core::String])                          ;; human-readable detail

(:wat::core::defenum :wat::query::Error :wat::enum::Pure
  :Transient  [fault <- :wat::query::Fault]                 ;; retry — the backend is momentarily unavailable
  :Constraint [fault <- :wat::query::Fault]                 ;; surface — a schema/uniqueness violation
  :Fatal      [fault <- :wat::query::Fault])                ;; abort — an unrecoverable backend condition

;; ─── the write input ─────────────────────────────────────────────────────────────────────────
(:wat::core::defrecord :wat::query::IndexKey                 ;; a named GSI's own projected keys
  [ipk <- :wat::core::String
   isk <- :wat::core::String])

(:wat::core::defrecord :wat::query::StoredRow                ;; one record to `put`
  [pk         <- :wat::core::String                          ;; EDN-form key string; consumer serializes <-> hydrates
   sk         <- :wat::core::String
   data       <- :wat::core::String                          ;; the record's tagged EDN, opaque to the backend
   index-keys <- (:wat::core::HashMap :wat::core::String :wat::query::IndexKey)]) ;; index-name -> (ipk,isk)

;; ─── the read results — what scan / scan-index hand back ───────────────────────────────────
(:wat::core::defrecord :wat::query::Row
  [pk   <- :wat::core::String
   sk   <- :wat::core::String
   data <- :wat::core::String])

(:wat::core::defrecord :wat::query::IndexRow                 ;; the 4-keyed index row
  [pk   <- :wat::core::String                                ;; the base keys
   sk   <- :wat::core::String
   ipk  <- :wat::core::String                                ;; the GSI's own keys
   isk  <- :wat::core::String
   data <- :wat::core::String])

;; ─── the page requests — a prefix/range on the sort key (range subsumes prefix) ─────────────
(:wat::core::defrecord :wat::query::ScanRequest               ;; a base-table page request
  [pk     <- :wat::core::String
   sk-lo  <- :wat::core::String
   sk-hi  <- :wat::core::String
   limit  <- :wat::core::i64
   cursor <- (:wat::core::Option :wat::core::String)])       ;; None = first page; Some sk = resume after (keyset)

(:wat::core::defrecord :wat::query::IndexScanRequest           ;; a GSI page request
  [index  <- :wat::core::String
   ipk    <- :wat::core::String
   isk-lo <- :wat::core::String
   isk-hi <- :wat::core::String
   limit  <- :wat::core::i64
   cursor <- (:wat::core::Option :wat::core::String)])

;; ─── the pages — results + the keyset resume cursor ─────────────────────────────────────────
(:wat::core::defrecord :wat::query::Page
  [rows        <- (:wat::core::Vector :wat::query::Row)
   next-cursor <- (:wat::core::Option :wat::core::String)])

(:wat::core::defrecord :wat::query::IndexPage
  [rows        <- (:wat::core::Vector :wat::query::IndexRow)
   next-cursor <- (:wat::core::Option :wat::core::String)])

;; ─── schema declarations (ensure-schema input) ──────────────────────────────────────────────
(:wat::core::defrecord :wat::query::TableSchema
  [pk <- :wat::core::String
   sk <- :wat::core::String])

(:wat::core::defrecord :wat::query::IndexSchema
  [name <- :wat::core::String                                ;; the GSI's name — S2's secondary-complete-tables
                                                              ;; model makes this the table name (`index_<name>`)
   pk  <- :wat::core::String
   sk  <- :wat::core::String
   ipk <- :wat::core::String
   isk <- :wat::core::String])

;; ─── the contract — Store / ReadStore surfaces ──────────────────────────────────────────────
;; :nature :wat::core::Struct — a satisfier holds a live connection (impure); the 293.W
;; containment rule forbids a :Struct-nature surface field from crossing into a pure record /
;; durable / wire form (a live connection cannot cross that boundary) — correct by construction.

(:wat::core::defsurface :wat::query::Store :nature :wat::core::Struct
  :features
  [;; idempotently establish the store for (pk,sk,data) + the declared GSIs. Called once at
   ;; consumer init.
   (ensure-schema [self <- :wat::query::Store  table <- :wat::query::TableSchema
                   indexes <- (:wat::core::Vector :wat::query::IndexSchema)]
     -> :wat::core::Result<wat::core::nil,wat::query::Error>)

   ;; write a batch ATOMICALLY (one transaction). Each row carries its opaque data + the
   ;; (ipk,isk) it projects to for each declared GSI (supplied by the consumer's write path —
   ;; the backend cannot read `data`).
   (put [self <- :wat::query::Store  rows <- (:wat::core::Vector :wat::query::StoredRow)]
     -> :wat::core::Result<wat::core::nil,wat::query::Error>)

   ;; a PAGE on the base key: pk fixed, sk in a prefix/range, ordered ASC, after `cursor`.
   (scan [self <- :wat::query::Store  q <- :wat::query::ScanRequest]
     -> :wat::core::Result<wat::query::Page,wat::query::Error>)

   ;; a PAGE on a named GSI: ipk fixed, isk in a prefix/range, ordered ASC, after `cursor`.
   (scan-index [self <- :wat::query::Store  q <- :wat::query::IndexScanRequest]
     -> :wat::core::Result<wat::query::IndexPage,wat::query::Error>)])

;; a read-only satisfier — the capability-honest half (the type is the proof a reader cannot write).
(:wat::core::defsurface :wat::query::ReadStore :nature :wat::core::Struct
  :features
  [(scan [self <- :wat::query::ReadStore  q <- :wat::query::ScanRequest]
     -> :wat::core::Result<wat::query::Page,wat::query::Error>)
   (scan-index [self <- :wat::query::ReadStore  q <- :wat::query::IndexScanRequest]
     -> :wat::core::Result<wat::query::IndexPage,wat::query::Error>)])
