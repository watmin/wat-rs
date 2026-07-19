;; wat/query.wat — Arc 278 stone S4: the :wat::query backend-agnostic storage CONTRACT, on the
;; SERVICES-AS-SURFACES operation model.
;;
;; Ratified in docs/arc/2026/06/278-rules-engine/DESIGN-store-contract.md (S0), migrated to the
;; operation model at S4 per arc 293 Path B (`823b20ac`): `Store` is now a `:nature :wat::kernel::Peer'`
;; surface — a DIALED PEER of a `:satisfies Store` service IS a Store, intrinsically (no wrapper
;; struct, no `extend-type`). The narrow waist is still DynamoDB's (pk, sk, data) + named-GSI
;; (ipk, isk) shape: all keys are EDN-form STRINGS the consumer serializes/hydrates; `data` is
;; opaque EDN the backend never inspects (§ data model). `wat.query` (the rete-as-datalog filter)
;; reasons over decoded records in working memory; the backend only ever hands it opaque pages.
;;
;; ─── the operation model (S4) ──────────────────────────────────────────────────────────────────
;; Every fallible/successful op returns a per-op OUTCOME ENUM named `Store::<Op>Response`
;; (`:Success` first, then that op's own error variants — never a bare success type, never a
;; generic `Result<T,Error>`). The error channel is an errors-as-record model on the RECOVERY axis
;; (the caller's forced branch: retry / surface / abort) — `Transient` / `Constraint` / `Fatal`,
;; each carrying a `reason <- Reason` (an OPEN surface — any pure record satisfies it; `Fault
;; [message <- String]` is the concrete default a backend with nothing more structured reaches for).
;;
;; Only outward refs: `:wat::core::*` (String/i64/keyword/nil/Vector/Option/HashMap/Struct) +
;; `:wat::enum::Pure` + `:wat::kernel::Peer'`. Loads after `wat/core.wat` (defrecord/defenum/
;; defsurface + those primitives) and `wat/service.wat` (the `Peer'` nature + `:satisfies`
;; machinery); placed near the rete sources — this is the query engine's vocabulary.

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

;; ─── the pages — results + the keyset resume cursor (vocabulary; the operation model's own
;; `Store::ScanResponse::Success`/`ScanIndexResponse::Success` carry rows+cursor directly rather
;; than nesting one of these — kept as the shared shape for consumers that want to box a page) ────
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

;; ─── the error vocabulary — recovery-axis records over an OPEN Reason surface ───────────────────
;; `Reason` has zero features: any pure record satisfies it ambiently (an OPEN Record surface)
;; — no `extend-type`/`derive` needed.
(:wat::core::defsurface :wat::query::Reason :nature :wat::core::Record :features [])

(:wat::core::defrecord :wat::query::Transient  [reason <- :wat::query::Reason]) ;; retry — momentarily unavailable
(:wat::core::defrecord :wat::query::Constraint [reason <- :wat::query::Reason]) ;; surface — schema/uniqueness violation
(:wat::core::defrecord :wat::query::Fatal      [reason <- :wat::query::Reason]) ;; abort — unrecoverable

;; a concrete default `Reason` satisfier for a backend with nothing more structured to say.
(:wat::core::defrecord :wat::query::Fault [message <- :wat::core::String])

;; ─── the contract — the Store surface, on the operation model ──────────────────────────────────
;; :nature :wat::kernel::Peer' — a satisfier is a `:satisfies Store` defservice; a dialed
;; `Peer'<Store::Op,Store::Reply>` IS a Store INTRINSICALLY (arc 293 Path B) — no wrapper struct,
;; no extend-type. `ReadStore` (the S0 read-only narrowing) is DELETED here: no live consumer, and
;; its only satisfiers were the wrapper structs this stone removes; reintroduce as a Store-peer
;; read-only narrowing when a real read-only consumer needs it.
;;
;; ─── the surface OWNS its protocol (arc 278 S4c) ──────────────────────────────────────────────
;; The per-op request/response records live in the surface's `:messages` block — convention-named
;; `Store::<Op>Request`/`Store::<Op>Response` (the `defservice :satisfies` macro synthesizes
;; req-ty/resp-ty from these exact names — wat/service.wat:1046-1051). Owning them here means a
;; `:satisfies Store` service ships the protocol across a process fork via the surface's
;; surface-forms carrier (else the forked child never receives them → StartupError). The SHARED
;; domain vocabulary they are built from (StoredRow/Row/IndexRow/IndexKey/Page/IndexPage/
;; TableSchema/IndexSchema) + the error records (Reason/Transient/Constraint/Fatal/Fault) stay
;; top-level: they cross via stdlib, are not per-op messages.
(:wat::core::defsurface :wat::query::Store :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :wat::query::Store::EnsureSchemaRequest
     [table   <- :wat::query::TableSchema
      indexes <- (:wat::core::Vector :wat::query::IndexSchema)])

   (:wat::core::defenum :wat::query::Store::EnsureSchemaResponse :wat::enum::Pure
     :Success    []
     :Constraint [err <- :wat::query::Constraint]
     :Fatal      [err <- :wat::query::Fatal])

   (:wat::core::defrecord :wat::query::Store::PutRequest
     [rows <- (:wat::core::Vector :wat::query::StoredRow)])

   (:wat::core::defenum :wat::query::Store::PutResponse :wat::enum::Pure
     :Success    []
     :Constraint [err <- :wat::query::Constraint]
     :Transient  [err <- :wat::query::Transient]
     :Fatal      [err <- :wat::query::Fatal])

   (:wat::core::defrecord :wat::query::Store::ScanRequest         ;; a base-table page request
     [pk     <- :wat::core::String
      sk-lo  <- :wat::core::String
      sk-hi  <- :wat::core::String
      limit  <- :wat::core::i64
      cursor <- (:wat::core::Option :wat::core::String)])        ;; None = first page; Some sk = resume after (keyset)

   (:wat::core::defenum :wat::query::Store::ScanResponse :wat::enum::Pure
     :Success   [rows   <- (:wat::core::Vector :wat::query::Row)
                 cursor <- (:wat::core::Option :wat::core::String)]
     :Transient [err <- :wat::query::Transient]
     :Fatal     [err <- :wat::query::Fatal])

   (:wat::core::defrecord :wat::query::Store::ScanIndexRequest    ;; a GSI page request
     [index  <- :wat::core::String
      ipk    <- :wat::core::String
      isk-lo <- :wat::core::String
      isk-hi <- :wat::core::String
      limit  <- :wat::core::i64
      cursor <- (:wat::core::Option :wat::core::String)])

   (:wat::core::defenum :wat::query::Store::ScanIndexResponse :wat::enum::Pure
     :Success   [rows   <- (:wat::core::Vector :wat::query::IndexRow)
                 cursor <- (:wat::core::Option :wat::core::String)]
     :Transient [err <- :wat::query::Transient]
     :Fatal     [err <- :wat::query::Fatal])]
  :features
  [;; idempotently establish the store for (pk,sk,data) + the declared GSIs. Called once at
   ;; consumer init.
   (ensure-schema [self <- :wat::query::Store  req <- :wat::query::Store::EnsureSchemaRequest]
     -> :wat::query::Store::EnsureSchemaResponse)

   ;; write a batch ATOMICALLY (one transaction). Each row carries its opaque data + the
   ;; (ipk,isk) it projects to for each declared GSI (supplied by the consumer's write path —
   ;; the backend cannot read `data`).
   (put [self <- :wat::query::Store  req <- :wat::query::Store::PutRequest]
     -> :wat::query::Store::PutResponse)

   ;; a PAGE on the base key: pk fixed, sk in a prefix/range, ordered ASC, after `cursor`.
   (scan [self <- :wat::query::Store  req <- :wat::query::Store::ScanRequest]
     -> :wat::query::Store::ScanResponse)

   ;; a PAGE on a named GSI: ipk fixed, isk in a prefix/range, ordered ASC, after `cursor`.
   (scan-index [self <- :wat::query::Store  req <- :wat::query::Store::ScanIndexRequest]
     -> :wat::query::Store::ScanIndexResponse)])
