;; wat/query/sqlite-store.wat — Arc 278 stone T1a: `:wat::query::sqlite-store'` (a defservice) +
;; `:wat::query::SqliteStore` (the satisfier wrapping its peer) — the sqlite `:wat::query::Store` /
;; `ReadStore` backend, built as SQL over S1's raw `:wat::sqlite'` verbs (wat/sqlite.wat).
;; Differential-tested against the S-mem MemStore oracle
;; (tests/rete/probe_arc278_sqlite_store_differential.{wat,rs}).
;;
;; ─── T1a: promote the struct-over-Connection satisfier into a SERVICE ───────────────────────────
;; S2 held a live `:wat::sqlite'::Connection` in a bare `defstruct` — which can never cross the wire,
;; so a telemetry sink (T1b) could never be GIVEN the store. T1a promotes it into a `sqlite-store'`
;; defservice (an actor owning the `Connection` on its OWN thread, durable = `path` + `index-names`,
;; ephemeral = the opened `conn`) plus a `SqliteStore` satisfier that wraps only the service PEER
;; (a wire-crossable value) — mirroring `mem-store'`/`MemStore` (wat/query/mem.wat) almost
;; line-for-line. The SQL + pure helpers are UNCHANGED from S2; the 4 Store-method bodies became the
;; service op bodies, and the satisfier's 4 methods are now client RPCs. Per intueri (verdict A), the
;; satisfier + its pure helpers moved namespace `:wat::sqlite'` → `:wat::query` (a Store satisfier
;; lives with the CONTRACT, like MemStore). The RAW DRIVER stays `:wat::sqlite'`
;; (Connection/open/execute/select/Param/Cell/Error/Fault). S2's `SqliteStore/open` convenience
;; constructor is RETIRED (the scope-trap in mem.wat's NOTE forbids it — construct inline).
;;
;; Ratified design (S2): DESIGN-STONE-S2-sqlite-satisfier.md. DDB-faithful — a GSI is a SEPARATE
;; COMPLETE TABLE `index_<name>(ipk,isk,pk,sk,data,PK(ipk,isk,pk,sk))`, not a native sqlite index
;; on the `main` table. `scan`/`scan-index` are ONE keyset primitive (parameterized by table +
;; partition-col + sort-col); `put` is clear-then-insert (upsert-safe). `put`'s clear step must
;; DELETE this row's old projection from EVERY declared GSI table, so the service carries the full
;; `index-names` set on its durable Record (an update that STOPS projecting into a GSI must still
;; clear its old `index_<name>` row).

;; ─── the error lift — :wat::sqlite'::Error -> :wat::query::Error (both recovery-axis, field-for-
;; field identical Fault shape: op/code/diagnostic/message) — the satisfier's mechanism, so it lives
;; in :wat::query with the contract ────────────────────────────────────────────────────────────
(:wat::core::defn :wat::query::lift-fault [f <- :wat::sqlite'::Fault] -> :wat::query::Fault
  (:wat::query::Fault
    (:wat::sqlite'::Fault/op f)
    (:wat::sqlite'::Fault/code f)
    (:wat::sqlite'::Fault/diagnostic f)
    (:wat::sqlite'::Fault/message f)))

(:wat::core::defn :wat::query::lift-error [e <- :wat::sqlite'::Error] -> :wat::query::Error
  (:wat::core::match e -> :wat::query::Error
    ((:wat::sqlite'::Error::Transient f)  (:wat::query::Error::Transient  (:wat::query::lift-fault f)))
    ((:wat::sqlite'::Error::Constraint f) (:wat::query::Error::Constraint (:wat::query::lift-fault f)))
    ((:wat::sqlite'::Error::Fatal f)      (:wat::query::Error::Fatal      (:wat::query::lift-fault f)))))

;; ─── Cell -> String unpacking (pk/sk/data/ipk/isk columns are always TEXT NOT NULL, so Str is the
;; live path; the other arms are exhaustiveness-only, never hit against this store's own schema) ──
(:wat::core::defn :wat::query::cell->string [c <- :wat::sqlite'::Cell] -> :wat::core::String
  (:wat::core::match c -> :wat::core::String
    ((:wat::sqlite'::Cell::Str s) s)
    ((:wat::sqlite'::Cell::I64 n) (:wat::core::i64::to-string n))
    ((:wat::sqlite'::Cell::F64 _) "")
    ((:wat::sqlite'::Cell::Nil) "")))

(:wat::core::defn :wat::query::row-from-cells
  [cells <- (:wat::core::Vector :wat::sqlite'::Cell)] -> :wat::query::Row
  (:wat::query::Row
    (:wat::query::cell->string (:wat::core::nth cells 0))    ;; pk
    (:wat::query::cell->string (:wat::core::nth cells 1))    ;; sk
    (:wat::query::cell->string (:wat::core::nth cells 2))))  ;; data

(:wat::core::defn :wat::query::index-row-from-cells
  [cells <- (:wat::core::Vector :wat::sqlite'::Cell)] -> :wat::query::IndexRow
  ;; select order is (ipk,isk,pk,sk,data); IndexRow's own field order is (pk,sk,ipk,isk,data).
  (:wat::query::IndexRow
    (:wat::query::cell->string (:wat::core::nth cells 2))    ;; pk
    (:wat::query::cell->string (:wat::core::nth cells 3))    ;; sk
    (:wat::query::cell->string (:wat::core::nth cells 0))    ;; ipk
    (:wat::query::cell->string (:wat::core::nth cells 1))    ;; isk
    (:wat::query::cell->string (:wat::core::nth cells 4))))  ;; data

;; ─── the keyset page-builders — next-cursor = last row's sort key IFF the page came back full ───
(:wat::core::defn :wat::query::build-page
  [rows <- (:wat::core::Vector :wat::query::Row) limit <- :wat::core::i64] -> :wat::query::Page
  (:wat::core::let
    [full?    (:wat::core::= (:wat::core::count rows) limit)
     next-cur (:wat::core::if full?
                (:wat::core::Some
                  (:wat::query::Row/sk (:wat::core::Option/expect (:wat::core::last rows) "scan: rows non-empty when full")))
                :wat::core::None)]
    (:wat::query::Page rows next-cur)))

(:wat::core::defn :wat::query::build-index-page
  [rows <- (:wat::core::Vector :wat::query::IndexRow) limit <- :wat::core::i64] -> :wat::query::IndexPage
  (:wat::core::let
    [full?    (:wat::core::= (:wat::core::count rows) limit)
     next-cur (:wat::core::if full?
                (:wat::core::Some
                  (:wat::query::IndexRow/isk (:wat::core::Option/expect (:wat::core::last rows) "scan-index: rows non-empty when full")))
                :wat::core::None)]
    (:wat::query::IndexPage rows next-cur)))

;; ─── ensure-schema — main + one complete table per named GSI ────────────────────────────────────
(:wat::core::defn :wat::query::ensure-index-tables
  [conn <- :wat::sqlite'::Connection indexes <- (:wat::core::Vector :wat::query::IndexSchema)]
  -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
  (:wat::core::if (:wat::core::empty? indexes)
    (:wat::core::Ok nil)
    (:wat::core::let
      [ix   (:wat::core::first indexes)
       tl   (:wat::core::rest indexes)
       name (:wat::query::IndexSchema/name ix)
       ddl  (:wat::core::format
              "CREATE TABLE IF NOT EXISTS [index_{name}] (ipk TEXT NOT NULL, isk TEXT NOT NULL, pk TEXT NOT NULL, sk TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(ipk, isk, pk, sk))"
              :name name)]
      (:wat::core::match (:wat::sqlite'::execute-ddl conn ddl)
        -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
        ((:wat::core::Err e) (:wat::core::Err e))
        ((:wat::core::Ok _) (:wat::query::ensure-index-tables conn tl))))))

;; ─── put — clear-then-insert, one row at a time inside the caller's BEGIN/COMMIT ────────────────
(:wat::core::defn :wat::query::clear-index-projections
  [conn <- :wat::sqlite'::Connection names <- (:wat::core::Vector :wat::core::String)
   pk <- :wat::core::String sk <- :wat::core::String]
  -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
  (:wat::core::if (:wat::core::empty? names)
    (:wat::core::Ok nil)
    (:wat::core::let
      [nm  (:wat::core::first names)
       tl  (:wat::core::rest names)
       sql (:wat::core::format "DELETE FROM [index_{name}] WHERE pk=? AND sk=?" :name nm)]
      (:wat::core::match
        (:wat::sqlite'::execute conn sql
          (:wat::core::Vector :wat::sqlite'::Param (:wat::sqlite'::Param::Str pk) (:wat::sqlite'::Param::Str sk)))
        -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
        ((:wat::core::Err e) (:wat::core::Err e))
        ((:wat::core::Ok _) (:wat::query::clear-index-projections conn tl pk sk))))))

(:wat::core::defn :wat::query::insert-index-projections
  [conn <- :wat::sqlite'::Connection names <- (:wat::core::Vector :wat::core::String)
   pk <- :wat::core::String sk <- :wat::core::String data <- :wat::core::String
   index-keys <- (:wat::core::HashMap :wat::core::String :wat::query::IndexKey)]
  -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
  (:wat::core::if (:wat::core::empty? names)
    (:wat::core::Ok nil)
    (:wat::core::let
      [nm (:wat::core::first names)
       tl (:wat::core::rest names)]
      (:wat::core::match (:wat::core::HashMap/get index-keys nm)
        -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
        (:wat::core::None (:wat::query::insert-index-projections conn tl pk sk data index-keys))
        ((:wat::core::Some ik)
          (:wat::core::let
            [sql (:wat::core::format "INSERT INTO [index_{name}] (ipk,isk,pk,sk,data) VALUES (?,?,?,?,?)" :name nm)
             params (:wat::core::Vector :wat::sqlite'::Param
                      (:wat::sqlite'::Param::Str (:wat::query::IndexKey/ipk ik))
                      (:wat::sqlite'::Param::Str (:wat::query::IndexKey/isk ik))
                      (:wat::sqlite'::Param::Str pk) (:wat::sqlite'::Param::Str sk) (:wat::sqlite'::Param::Str data))]
            (:wat::core::match (:wat::sqlite'::execute conn sql params)
              -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
              ((:wat::core::Err e) (:wat::core::Err e))
              ((:wat::core::Ok _) (:wat::query::insert-index-projections conn tl pk sk data index-keys)))))))))

(:wat::core::defn :wat::query::put-one-row
  [conn <- :wat::sqlite'::Connection index-names <- (:wat::core::Vector :wat::core::String)
   row <- :wat::query::StoredRow]
  -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
  (:wat::core::let
    [pk         (:wat::query::StoredRow/pk row)
     sk         (:wat::query::StoredRow/sk row)
     data       (:wat::query::StoredRow/data row)
     index-keys (:wat::query::StoredRow/index-keys row)
     key-params (:wat::core::Vector :wat::sqlite'::Param (:wat::sqlite'::Param::Str pk) (:wat::sqlite'::Param::Str sk))]
    (:wat::core::match (:wat::sqlite'::execute conn "DELETE FROM main WHERE pk=? AND sk=?" key-params)
      -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
      ((:wat::core::Err e) (:wat::core::Err e))
      ((:wat::core::Ok _)
        (:wat::core::match (:wat::query::clear-index-projections conn index-names pk sk)
          -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
          ((:wat::core::Err e) (:wat::core::Err e))
          ((:wat::core::Ok _)
            (:wat::core::match
              (:wat::sqlite'::execute conn "INSERT INTO main (pk,sk,data) VALUES (?,?,?)"
                (:wat::core::Vector :wat::sqlite'::Param
                  (:wat::sqlite'::Param::Str pk) (:wat::sqlite'::Param::Str sk) (:wat::sqlite'::Param::Str data)))
              -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
              ((:wat::core::Err e) (:wat::core::Err e))
              ((:wat::core::Ok _)
                (:wat::query::insert-index-projections conn index-names pk sk data index-keys)))))))))

(:wat::core::defn :wat::query::put-rows
  [conn <- :wat::sqlite'::Connection index-names <- (:wat::core::Vector :wat::core::String)
   rows <- (:wat::core::Vector :wat::query::StoredRow)]
  -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
  (:wat::core::if (:wat::core::empty? rows)
    (:wat::core::Ok nil)
    (:wat::core::match (:wat::query::put-one-row conn index-names (:wat::core::first rows))
      -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
      ((:wat::core::Err e) (:wat::core::Err e))
      ((:wat::core::Ok _) (:wat::query::put-rows conn index-names (:wat::core::rest rows))))))

;; ─── the sqlite-store' SERVICE — an actor owning a thread-local Connection ──────────────────────
;; durable = `path` + the declared `index-names` (the clear step needs the full GSI set — see the
;; header). ephemeral = the live `conn`, opened in :init from `path` + the WAL/NORMAL pragmas S2 set.
;; The ops are S2's Store-method bodies, calling the raw `:wat::sqlite'` verbs on
;; `(sqlite-store'::State/conn s)`; each carries its fallible `Result<_,query::Error>` IN its
;; response (the SQL is fallible; the lift to `query::Error` happens here so the satisfier just
;; forwards). `s` is UNCHANGED on Reply — the mutation is inside sqlite via the conn, not in State.
(:wat::service::defservice :wat::query::sqlite-store'
  :durable   [path        <- :wat::core::String
              index-names  <- (:wat::core::Vector :wat::core::String)]
  :ephemeral [conn <- :wat::sqlite'::Connection]
  :init (:wat::core::fn [record <- :wat::query::sqlite-store'::Record]
          -> :wat::query::sqlite-store'::State
          (:wat::query::sqlite-store'::State record
            (:wat::core::let
              [path (:wat::query::sqlite-store'::Record/path record)
               conn (:wat::core::Result/expect (:wat::sqlite'::open path) "sqlite-store': open failed")
               _wal (:wat::core::Result/expect (:wat::sqlite'::pragma conn "journal_mode" "WAL")
                      "sqlite-store': journal_mode=WAL pragma failed")
               _syn (:wat::core::Result/expect (:wat::sqlite'::pragma conn "synchronous" "NORMAL")
                      "sqlite-store': synchronous=NORMAL pragma failed")]
              conn)))
  :ops
  [(:EnsureSchema [s <- :State table <- :wat::query::TableSchema
                   indexes <- (:wat::core::Vector :wat::query::IndexSchema)]
     -> [result <- :wat::core::Result<wat::core::nil,wat::query::Error>]
     (:wat::core::let
       [conn (:wat::query::sqlite-store'::State/conn s)
        chained
          (:wat::core::match
            (:wat::sqlite'::execute-ddl conn
              "CREATE TABLE IF NOT EXISTS main (pk TEXT NOT NULL, sk TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(pk,sk))")
            -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
            ((:wat::core::Err e) (:wat::core::Err e))
            ((:wat::core::Ok _) (:wat::query::ensure-index-tables conn indexes)))]
       (:wat::service::Outcome::Reply s
         (:wat::query::sqlite-store'::EnsureSchemaResponse
           (:wat::core::match chained -> :wat::core::Result<wat::core::nil,wat::query::Error>
             ((:wat::core::Ok _) (:wat::core::Ok nil))
             ((:wat::core::Err e) (:wat::core::Err (:wat::query::lift-error e))))))))

   (:Put [s <- :State new-rows <- (:wat::core::Vector :wat::query::StoredRow)]
     -> [result <- :wat::core::Result<wat::core::nil,wat::query::Error>]
     (:wat::core::let
       [conn  (:wat::query::sqlite-store'::State/conn s)
        names (:wat::query::sqlite-store'::Record/index-names (:wat::query::sqlite-store'::State/durable s))
        chained
          (:wat::core::match (:wat::sqlite'::begin conn)
            -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
            ((:wat::core::Err e) (:wat::core::Err e))
            ((:wat::core::Ok _)
              (:wat::core::match (:wat::query::put-rows conn names new-rows)
                -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
                ((:wat::core::Err e) (:wat::core::Err e))
                ((:wat::core::Ok _) (:wat::sqlite'::commit conn)))))]
       (:wat::service::Outcome::Reply s
         (:wat::query::sqlite-store'::PutResponse
           (:wat::core::match chained -> :wat::core::Result<wat::core::nil,wat::query::Error>
             ((:wat::core::Ok _) (:wat::core::Ok nil))
             ((:wat::core::Err e) (:wat::core::Err (:wat::query::lift-error e))))))))

   (:Scan [s <- :State q <- :wat::query::ScanRequest]
     -> [result <- :wat::core::Result<wat::query::Page,wat::query::Error>]
     (:wat::core::let
       [conn (:wat::query::sqlite-store'::State/conn s)
        pk   (:wat::query::ScanRequest/pk q)
        lo   (:wat::query::ScanRequest/sk-lo q)
        hi   (:wat::query::ScanRequest/sk-hi q)
        lim  (:wat::query::ScanRequest/limit q)
        cur  (:wat::query::ScanRequest/cursor q)
        cur-param (:wat::core::match cur -> :wat::sqlite'::Param
                    (:wat::core::None (:wat::sqlite'::Param::Nil))
                    ((:wat::core::Some c) (:wat::sqlite'::Param::Str c)))
        params (:wat::core::Vector :wat::sqlite'::Param
                 (:wat::sqlite'::Param::Str pk) (:wat::sqlite'::Param::Str lo) (:wat::sqlite'::Param::Str hi)
                 cur-param (:wat::sqlite'::Param::I64 lim))
        res (:wat::sqlite'::select conn
              "SELECT pk, sk, data FROM main WHERE pk=?1 AND sk>=?2 AND sk<=?3 AND (?4 IS NULL OR sk>?4) ORDER BY sk ASC LIMIT ?5"
              params)]
       (:wat::service::Outcome::Reply s
         (:wat::query::sqlite-store'::ScanResponse
           (:wat::core::match res -> :wat::core::Result<wat::query::Page,wat::query::Error>
             ((:wat::core::Err e) (:wat::core::Err (:wat::query::lift-error e)))
             ((:wat::core::Ok cell-rows)
               (:wat::core::Ok
                 (:wat::query::build-page (:wat::core::mapv :wat::query::row-from-cells cell-rows) lim))))))))

   (:ScanIndex [s <- :State q <- :wat::query::IndexScanRequest]
     -> [result <- :wat::core::Result<wat::query::IndexPage,wat::query::Error>]
     (:wat::core::let
       [conn (:wat::query::sqlite-store'::State/conn s)
        name (:wat::query::IndexScanRequest/index q)
        ipk  (:wat::query::IndexScanRequest/ipk q)
        lo   (:wat::query::IndexScanRequest/isk-lo q)
        hi   (:wat::query::IndexScanRequest/isk-hi q)
        lim  (:wat::query::IndexScanRequest/limit q)
        cur  (:wat::query::IndexScanRequest/cursor q)
        cur-param (:wat::core::match cur -> :wat::sqlite'::Param
                    (:wat::core::None (:wat::sqlite'::Param::Nil))
                    ((:wat::core::Some c) (:wat::sqlite'::Param::Str c)))
        sql (:wat::core::format
              "SELECT ipk, isk, pk, sk, data FROM [index_{name}] WHERE ipk=?1 AND isk>=?2 AND isk<=?3 AND (?4 IS NULL OR isk>?4) ORDER BY isk ASC LIMIT ?5"
              :name name)
        params (:wat::core::Vector :wat::sqlite'::Param
                 (:wat::sqlite'::Param::Str ipk) (:wat::sqlite'::Param::Str lo) (:wat::sqlite'::Param::Str hi)
                 cur-param (:wat::sqlite'::Param::I64 lim))
        res (:wat::sqlite'::select conn sql params)]
       (:wat::service::Outcome::Reply s
         (:wat::query::sqlite-store'::ScanIndexResponse
           (:wat::core::match res -> :wat::core::Result<wat::query::IndexPage,wat::query::Error>
             ((:wat::core::Err e) (:wat::core::Err (:wat::query::lift-error e)))
             ((:wat::core::Ok cell-rows)
               (:wat::core::Ok
                 (:wat::query::build-index-page (:wat::core::mapv :wat::query::index-row-from-cells cell-rows) lim))))))))])

;; ─── the wrapper — extend-types the connected client peer to Store / ReadStore ──────────────
;; Mirrors MemStore exactly: `self` carries only the connected `Peer'` (constructed once, INLINE per
;; mem.wat's scope NOTE — no convenience constructor). Each method is a client RPC that forwards the
;; response's already-lifted `Result<_,query::Error>` straight through as the Store method's result.
(:wat::core::defstruct :wat::query::SqliteStore
  [peer <- :wat::kernel::Peer'<wat::query::sqlite-store'::Op,wat::query::sqlite-store'::Reply>])

(:wat::core::extend-type :wat::query::SqliteStore :wat::query::Store
  (ensure-schema [self table indexes]
    (:wat::core::let
      [r (:wat::query::sqlite-store'/ensure-schema (:wat::query::SqliteStore/peer self)
           (:wat::query::sqlite-store'/ensure-schema-request table indexes))]
      (:wat::query::sqlite-store'::EnsureSchemaResponse/result r)))
  (put [self rows]
    (:wat::core::let
      [r (:wat::query::sqlite-store'/put (:wat::query::SqliteStore/peer self)
           (:wat::query::sqlite-store'/put-request rows))]
      (:wat::query::sqlite-store'::PutResponse/result r)))
  (scan [self q]
    (:wat::core::let
      [r (:wat::query::sqlite-store'/scan (:wat::query::SqliteStore/peer self)
           (:wat::query::sqlite-store'/scan-request q))]
      (:wat::query::sqlite-store'::ScanResponse/result r)))
  (scan-index [self q]
    (:wat::core::let
      [r (:wat::query::sqlite-store'/scan-index (:wat::query::SqliteStore/peer self)
           (:wat::query::sqlite-store'/scan-index-request q))]
      (:wat::query::sqlite-store'::ScanIndexResponse/result r))))

;; the read-only edge — SqliteStore's scan/scan-index (from the Store impl above) also satisfy
;; ReadStore, exactly mirroring MemStore's `derive` (extend-type's edge-only half).
(:wat::core::derive :wat::query::SqliteStore :wat::query::ReadStore)
