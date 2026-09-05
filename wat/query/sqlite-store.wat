;; wat/query/sqlite-store.wat — Arc 278 stone S4: `:wat::query::sqlite-store'` — the sqlite
;; `:wat::query::Store` backend, migrated to the services-as-surfaces OPERATION MODEL
;; (`:satisfies :wat::query::Store` + `:impls`), built as SQL over S1's raw `:wat::sqlite'` verbs
;; (wat/sqlite.wat). Differential-tested against the S-mem mem-store' oracle
;; (tests/rete/probe_arc278_sqlite_store_differential.{wat,rs}).
;;
;; ─── S4: from a peer-wrapping satisfier to an intrinsic peer ────────────────────────────────────
;; T1a promoted the struct-over-Connection satisfier into a `sqlite-store'` defservice (an actor
;; owning the `Connection` on its OWN thread, durable = `path` + `index-names`, ephemeral = the
;; opened `conn`) plus a `SqliteStore` wrapper struct around the connected peer. S4 (arc 293 Path
;; B) DELETES that wrapper: `sqlite-store'` now `:satisfies :wat::query::Store` directly, so a
;; dialed peer IS the Store, intrinsically — no `extend-type`. The SQL + pure helpers are
;; UNCHANGED; the 4 op bodies now read fields off `Store::<Op>Request` and return the per-op
;; `Store::<Op>Response` OUTCOME ENUM (`:Success` + that op's own error variants) instead of a
;; generic `(Result :- [_ query::Error])`. S2's `SqliteStore/open` convenience constructor stays RETIRED
;; (the scope-trap in mem.wat's NOTE forbids it — construct inline).
;;
;; Ratified design (S2): DESIGN-STONE-S2-sqlite-satisfier.md. DDB-faithful — a GSI is a SEPARATE
;; COMPLETE TABLE `index_<name>(ipk,isk,pk,sk,data,PK(ipk,isk,pk,sk))`, not a native sqlite index
;; on the `main` table. `scan`/`scan-index` are ONE keyset primitive (parameterized by table +
;; partition-col + sort-col); `put` is clear-then-insert (upsert-safe). `put`'s clear step must
;; DELETE this row's old projection from EVERY declared GSI table, so the service carries the full
;; `index-names` set on its durable Record (an update that STOPS projecting into a GSI must still
;; clear its old `index_<name>` row).

;; ─── the error lift — :wat::sqlite'::Fault -> :wat::query::Fault (message only — the concrete
;; default `Reason` satisfier; a structured `:wat::sqlite'::Reason` is a later stone) — the
;; satisfier's mechanism, so it lives in :wat::query with the contract ──────────────────────────
(:wat::core::defn :wat::query::lift-fault [f <- :wat::sqlite::Fault] -> :wat::query::Fault
  (:wat::query::Fault :message (:wat::sqlite::Fault/message f)))

;; ─── per-op response builders — classify a raw sqlite Result into the op's own outcome enum.
;; Each `Store::<Op>Response` exposes only the error variants that op's surface declares; a sqlite
;; classification with no matching variant on THIS op folds into `:Fatal` (defensive — never hit
;; against this store's own schema/queries, documented per fold-site below). ─────────────────────
(:wat::core::defn :wat::query::ensure-schema-response
  [r <- (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])] -> :wat::query::Store::EnsureSchemaResponse
  (:wat::core::match r 
    ((:wat::core::Ok _) (:wat::query::Store::EnsureSchemaResponse::Success))
    ((:wat::core::Err e)
      (:wat::core::match e 
        ((:wat::sqlite::Error::Constraint f)
          (:wat::query::Store::EnsureSchemaResponse::Constraint (:wat::query::Constraint :reason (:wat::query::lift-fault f))))
        ;; EnsureSchemaResponse has no :Transient variant (schema DDL has no meaningful
        ;; "retry and it'll pass" semantics at this contract layer) — fold into :Fatal.
        ((:wat::sqlite::Error::Transient f)
          (:wat::query::Store::EnsureSchemaResponse::Fatal (:wat::query::Fatal :reason (:wat::query::lift-fault f))))
        ((:wat::sqlite::Error::Fatal f)
          (:wat::query::Store::EnsureSchemaResponse::Fatal (:wat::query::Fatal :reason (:wat::query::lift-fault f))))))))

(:wat::core::defn :wat::query::put-response
  [r <- (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])] -> :wat::query::Store::PutResponse
  (:wat::core::match r 
    ((:wat::core::Ok _) (:wat::query::Store::PutResponse::Success))
    ((:wat::core::Err e)
      (:wat::core::match e 
        ((:wat::sqlite::Error::Transient f)
          (:wat::query::Store::PutResponse::Transient (:wat::query::Transient :reason (:wat::query::lift-fault f))))
        ((:wat::sqlite::Error::Constraint f)
          (:wat::query::Store::PutResponse::Constraint (:wat::query::Constraint :reason (:wat::query::lift-fault f))))
        ((:wat::sqlite::Error::Fatal f)
          (:wat::query::Store::PutResponse::Fatal (:wat::query::Fatal :reason (:wat::query::lift-fault f))))))))

(:wat::core::defn :wat::query::delete-response
  [r <- (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])] -> :wat::query::Store::DeleteResponse
  (:wat::core::match r
    ((:wat::core::Ok _) (:wat::query::Store::DeleteResponse::Success))
    ((:wat::core::Err e)
      (:wat::core::match e
        ((:wat::sqlite::Error::Transient f)
          (:wat::query::Store::DeleteResponse::Transient (:wat::query::Transient :reason (:wat::query::lift-fault f))))
        ((:wat::sqlite::Error::Constraint f)
          (:wat::query::Store::DeleteResponse::Constraint (:wat::query::Constraint :reason (:wat::query::lift-fault f))))
        ((:wat::sqlite::Error::Fatal f)
          (:wat::query::Store::DeleteResponse::Fatal (:wat::query::Fatal :reason (:wat::query::lift-fault f))))))))

(:wat::core::defn :wat::query::scan-response
  [r <- (:wat::core::Result :- [(:wat::core::Vector :- [:wat::query::Row]) :wat::sqlite::Error])
   limit <- :wat::core::i64]
  -> :wat::query::Store::ScanResponse
  (:wat::core::match r 
    ((:wat::core::Err e)
      (:wat::core::match e 
        ((:wat::sqlite::Error::Transient f)
          (:wat::query::Store::ScanResponse::Transient (:wat::query::Transient :reason (:wat::query::lift-fault f))))
        ;; ScanResponse has no :Constraint variant (a read cannot violate a write constraint) —
        ;; fold into :Fatal (defensive; never hit against this store's own schema).
        ((:wat::sqlite::Error::Constraint f)
          (:wat::query::Store::ScanResponse::Fatal (:wat::query::Fatal :reason (:wat::query::lift-fault f))))
        ((:wat::sqlite::Error::Fatal f)
          (:wat::query::Store::ScanResponse::Fatal (:wat::query::Fatal :reason (:wat::query::lift-fault f))))))
    ((:wat::core::Ok rows)
      (:wat::core::let
        [full?    (:wat::core::= (:wat::core::count rows) limit)
         next-cur (:wat::core::if full?
                    (:wat::core::Some
                      (:wat::query::Row/sk (:wat::core::Option/expect (:wat::core::last rows) "scan: rows non-empty when full")))
                    :wat::core::None)]
        (:wat::query::Store::ScanResponse::Success rows next-cur)))))

(:wat::core::defn :wat::query::scan-index-response
  [r <- (:wat::core::Result :- [(:wat::core::Vector :- [:wat::query::IndexRow]) :wat::sqlite::Error])
   limit <- :wat::core::i64]
  -> :wat::query::Store::ScanIndexResponse
  (:wat::core::match r 
    ((:wat::core::Err e)
      (:wat::core::match e 
        ((:wat::sqlite::Error::Transient f)
          (:wat::query::Store::ScanIndexResponse::Transient (:wat::query::Transient :reason (:wat::query::lift-fault f))))
        ((:wat::sqlite::Error::Constraint f)
          (:wat::query::Store::ScanIndexResponse::Fatal (:wat::query::Fatal :reason (:wat::query::lift-fault f))))
        ((:wat::sqlite::Error::Fatal f)
          (:wat::query::Store::ScanIndexResponse::Fatal (:wat::query::Fatal :reason (:wat::query::lift-fault f))))))
    ((:wat::core::Ok rows)
      (:wat::core::let
        [full?    (:wat::core::= (:wat::core::count rows) limit)
         next-cur (:wat::core::if full?
                    (:wat::core::Some
                      (:wat::query::IndexRow/isk (:wat::core::Option/expect (:wat::core::last rows) "scan-index: rows non-empty when full")))
                    :wat::core::None)]
        (:wat::query::Store::ScanIndexResponse::Success rows next-cur)))))

(:wat::core::defn :wat::query::count-index-response
  [r <- (:wat::core::Result :- [:wat::core::i64 :wat::sqlite::Error])]
  -> :wat::query::Store::CountIndexResponse
  (:wat::core::match r
    ((:wat::core::Err e)
      (:wat::core::match e
        ((:wat::sqlite::Error::Transient f)
          (:wat::query::Store::CountIndexResponse::Transient (:wat::query::Transient :reason (:wat::query::lift-fault f))))
        ((:wat::sqlite::Error::Constraint f)
          (:wat::query::Store::CountIndexResponse::Fatal (:wat::query::Fatal :reason (:wat::query::lift-fault f))))
        ((:wat::sqlite::Error::Fatal f)
          (:wat::query::Store::CountIndexResponse::Fatal (:wat::query::Fatal :reason (:wat::query::lift-fault f))))))
    ((:wat::core::Ok n)
      (:wat::query::Store::CountIndexResponse::Ok n))))

(:wat::core::defn :wat::query::count-from-cell-rows
  [r <- (:wat::core::Result :- [(:wat::core::Vector :- [(:wat::core::Vector :- [:wat::sqlite::Cell])]) :wat::sqlite::Error])]
  -> (:wat::core::Result :- [:wat::core::i64 :wat::sqlite::Error])
  (:wat::core::match r
    ((:wat::core::Err e) (:wat::core::Err e))
    ((:wat::core::Ok cell-rows)
      (:wat::core::if (:wat::core::empty? cell-rows)
        (:wat::core::Ok 0)
        (:wat::core::match (:wat::core::nth (:wat::core::nth cell-rows 0) 0)
          ((:wat::sqlite::Cell::I64 n) (:wat::core::Ok n))
          (_ (:wat::core::Ok 0)))))))

;; ─── Cell -> String unpacking (pk/sk/data/ipk/isk columns are always TEXT NOT NULL, so Str is the
;; live path; the other arms are exhaustiveness-only, never hit against this store's own schema) ──
(:wat::core::defn :wat::query::cell->string [c <- :wat::sqlite::Cell] -> :wat::core::String
  (:wat::core::match c 
    ((:wat::sqlite::Cell::Str s) s)
    ((:wat::sqlite::Cell::I64 n) (:wat::i64::to-string n))
    ((:wat::sqlite::Cell::F64 _) "")
    ((:wat::sqlite::Cell::Nil) "")))

(:wat::core::defn :wat::query::row-from-cells
  [cells <- (:wat::core::Vector :- [:wat::sqlite::Cell])] -> :wat::query::Row
  (:wat::query::Row
    :pk (:wat::query::cell->string (:wat::core::nth cells 0))    ;; pk
    :sk (:wat::query::cell->string (:wat::core::nth cells 1))    ;; sk
    :data (:wat::query::cell->string (:wat::core::nth cells 2))))  ;; data

(:wat::core::defn :wat::query::index-row-from-cells
  [cells <- (:wat::core::Vector :- [:wat::sqlite::Cell])] -> :wat::query::IndexRow
  ;; select order is (ipk,isk,pk,sk,data); IndexRow's own field order is (pk,sk,ipk,isk,data).
  (:wat::query::IndexRow
    :pk (:wat::query::cell->string (:wat::core::nth cells 2))    ;; pk
    :sk (:wat::query::cell->string (:wat::core::nth cells 3))    ;; sk
    :ipk (:wat::query::cell->string (:wat::core::nth cells 0))    ;; ipk
    :isk (:wat::query::cell->string (:wat::core::nth cells 1))    ;; isk
    :data (:wat::query::cell->string (:wat::core::nth cells 4))))  ;; data

;; ─── ensure-schema — main + one complete table per named GSI ────────────────────────────────────
(:wat::core::defn :wat::query::ensure-index-tables
  [conn <- :wat::sqlite::Connection indexes <- (:wat::core::Vector :- [:wat::query::IndexSchema])]
  -> (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])
  (:wat::core::if (:wat::core::empty? indexes)
    (:wat::core::Ok nil)
    (:wat::core::let
      [ix   (:wat::core::first indexes)
       tl   (:wat::core::rest indexes)
       name (:wat::query::IndexSchema/name ix)
       ddl  (:wat::core::format
              "CREATE TABLE IF NOT EXISTS [index_{name}] (ipk TEXT NOT NULL, isk TEXT NOT NULL, pk TEXT NOT NULL, sk TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(ipk, isk, pk, sk))"
              :name name)]
      (:wat::core::match (:wat::sqlite::execute-ddl conn ddl)
        
        ((:wat::core::Err e) (:wat::core::Err e))
        ((:wat::core::Ok _) (:wat::query::ensure-index-tables conn tl))))))

;; ─── put — clear-then-insert, one row at a time inside the caller's BEGIN/COMMIT ────────────────
(:wat::core::defn :wat::query::clear-index-projections
  [conn <- :wat::sqlite::Connection names <- (:wat::core::Vector :- [:wat::core::String])
   pk <- :wat::core::String sk <- :wat::core::String]
  -> (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])
  (:wat::core::if (:wat::core::empty? names)
    (:wat::core::Ok nil)
    (:wat::core::let
      [nm  (:wat::core::first names)
       tl  (:wat::core::rest names)
       sql (:wat::core::format "DELETE FROM [index_{name}] WHERE pk=? AND sk=?" :name nm)]
      (:wat::core::match
        (:wat::sqlite::execute conn sql
          (:wat::core::Vector :- [:wat::sqlite::Param] (:wat::sqlite::Param::Str pk) (:wat::sqlite::Param::Str sk)))
        
        ((:wat::core::Err e) (:wat::core::Err e))
        ((:wat::core::Ok _) (:wat::query::clear-index-projections conn tl pk sk))))))

(:wat::core::defn :wat::query::insert-index-projections
  [conn <- :wat::sqlite::Connection names <- (:wat::core::Vector :- [:wat::core::String])
   pk <- :wat::core::String sk <- :wat::core::String data <- :wat::core::String
   index-keys <- (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey])]
  -> (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])
  (:wat::core::if (:wat::core::empty? names)
    (:wat::core::Ok nil)
    (:wat::core::let
      [nm (:wat::core::first names)
       tl (:wat::core::rest names)]
      (:wat::core::match (:wat::hashmap::get index-keys nm)
        
        (:wat::core::None (:wat::query::insert-index-projections conn tl pk sk data index-keys))
        ((:wat::core::Some ik)
          (:wat::core::let
            [sql (:wat::core::format "INSERT INTO [index_{name}] (ipk,isk,pk,sk,data) VALUES (?,?,?,?,?)" :name nm)
             params (:wat::core::Vector :- [:wat::sqlite::Param]
                      (:wat::sqlite::Param::Str (:wat::query::IndexKey/ipk ik))
                      (:wat::sqlite::Param::Str (:wat::query::IndexKey/isk ik))
                      (:wat::sqlite::Param::Str pk) (:wat::sqlite::Param::Str sk) (:wat::sqlite::Param::Str data))]
            (:wat::core::match (:wat::sqlite::execute conn sql params)
              
              ((:wat::core::Err e) (:wat::core::Err e))
              ((:wat::core::Ok _) (:wat::query::insert-index-projections conn tl pk sk data index-keys)))))))))

(:wat::core::defn :wat::query::put-one-row
  [conn <- :wat::sqlite::Connection index-names <- (:wat::core::Vector :- [:wat::core::String])
   row <- :wat::query::StoredRow]
  -> (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])
  (:wat::core::let
    [pk         (:wat::query::StoredRow/pk row)
     sk         (:wat::query::StoredRow/sk row)
     data       (:wat::query::StoredRow/data row)
     index-keys (:wat::query::StoredRow/index-keys row)
     key-params (:wat::core::Vector :- [:wat::sqlite::Param] (:wat::sqlite::Param::Str pk) (:wat::sqlite::Param::Str sk))]
    (:wat::core::match (:wat::sqlite::execute conn "DELETE FROM main WHERE pk=? AND sk=?" key-params)
      
      ((:wat::core::Err e) (:wat::core::Err e))
      ((:wat::core::Ok _)
        (:wat::core::match (:wat::query::clear-index-projections conn index-names pk sk)
          
          ((:wat::core::Err e) (:wat::core::Err e))
          ((:wat::core::Ok _)
            (:wat::core::match
              (:wat::sqlite::execute conn "INSERT INTO main (pk,sk,data) VALUES (?,?,?)"
                (:wat::core::Vector :- [:wat::sqlite::Param]
                  (:wat::sqlite::Param::Str pk) (:wat::sqlite::Param::Str sk) (:wat::sqlite::Param::Str data)))
              
              ((:wat::core::Err e) (:wat::core::Err e))
              ((:wat::core::Ok _)
                (:wat::query::insert-index-projections conn index-names pk sk data index-keys)))))))))

(:wat::core::defn :wat::query::put-rows
  [conn <- :wat::sqlite::Connection index-names <- (:wat::core::Vector :- [:wat::core::String])
   rows <- (:wat::core::Vector :- [:wat::query::StoredRow])]
  -> (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])
  (:wat::core::if (:wat::core::empty? rows)
    (:wat::core::Ok nil)
    (:wat::core::match (:wat::query::put-one-row conn index-names (:wat::core::first rows))
      
      ((:wat::core::Err e) (:wat::core::Err e))
      ((:wat::core::Ok _) (:wat::query::put-rows conn index-names (:wat::core::rest rows))))))

;; delete — same recursive batch as put-rows. GSI clear is `clear-index-projections`,
;; which DELETEs each `index_<name>` by (pk, sk) — the columns those tables already
;; carry. No read of the row, no index-keys on Key. A missing key is DELETE of 0
;; rows then the same clear (also 0) — Success, same as put's clear-then-insert
;; of a brand-new key.
(:wat::core::defn :wat::query::delete-one-key
  [conn <- :wat::sqlite::Connection index-names <- (:wat::core::Vector :- [:wat::core::String])
   key <- :wat::query::Key]
  -> (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])
  (:wat::core::let
    [pk (:wat::query::Key/pk key)
     sk (:wat::query::Key/sk key)
     key-params (:wat::core::Vector :- [:wat::sqlite::Param] (:wat::sqlite::Param::Str pk) (:wat::sqlite::Param::Str sk))]
    (:wat::core::match (:wat::sqlite::execute conn "DELETE FROM main WHERE pk=? AND sk=?" key-params)
      ((:wat::core::Err e) (:wat::core::Err e))
      ((:wat::core::Ok _) (:wat::query::clear-index-projections conn index-names pk sk)))))

(:wat::core::defn :wat::query::delete-rows
  [conn <- :wat::sqlite::Connection index-names <- (:wat::core::Vector :- [:wat::core::String])
   keys <- (:wat::core::Vector :- [:wat::query::Key])]
  -> (:wat::core::Result :- [:wat::core::nil :wat::sqlite::Error])
  (:wat::core::if (:wat::core::empty? keys)
    (:wat::core::Ok nil)
    (:wat::core::match (:wat::query::delete-one-key conn index-names (:wat::core::first keys))
      ((:wat::core::Err e) (:wat::core::Err e))
      ((:wat::core::Ok _) (:wat::query::delete-rows conn index-names (:wat::core::rest keys))))))

;; ─── the sqlite-store' SERVICE — an actor owning a thread-local Connection ──────────────────────
;; durable = `path` + the declared `index-names` (the clear step needs the full GSI set — see the
;; header). ephemeral = the live `conn`, opened in :init from `path` + the WAL/NORMAL pragmas S2 set.
;; `:satisfies :wat::query::Store` puts this on the operation model: each impl is `(<op> [s req]
;; body)` — `req` is the `Store::<Op>Request` record; the SQL logic is UNCHANGED from T1a, only
;; rewrapped to read request fields and to classify its raw sqlite `Result` into the op's own
;; `Store::<Op>Response` outcome enum via the response builders above. `s` is UNCHANGED on Reply —
;; the mutation is inside sqlite via the conn, not in State.
(:wat::service::defservice :wat::query::sqlite-store
  :satisfies :wat::query::Store
  :durable   [path        <- :wat::core::String
              index-names  <- (:wat::core::Vector :- [:wat::core::String])]
  :ephemeral [conn <- :wat::sqlite::Connection]
  :init (:wat::core::fn [record <- :wat::query::sqlite-store::Record]
          -> :wat::query::sqlite-store::State
          (:wat::query::sqlite-store::State
            :durable record
            :conn
            (:wat::core::let
              [path (:wat::query::sqlite-store::Record/path record)
               conn (:wat::core::Result/expect (:wat::sqlite::open path) "sqlite-store: open failed")
               _wal (:wat::core::Result/expect (:wat::sqlite::pragma conn "journal_mode" "WAL")
                      "sqlite-store: journal_mode=WAL pragma failed")
               _syn (:wat::core::Result/expect (:wat::sqlite::pragma conn "synchronous" "NORMAL")
                      "sqlite-store: synchronous=NORMAL pragma failed")]
              conn)))
  :impls
  [(ensure-schema [s ctx req]
     (:wat::core::let
       [table   (:wat::query::Store::EnsureSchemaRequest/table req)
        indexes (:wat::query::Store::EnsureSchemaRequest/indexes req)
        conn (:wat::query::sqlite-store::State/conn s)
        chained
          (:wat::core::match
            (:wat::sqlite::execute-ddl conn
              "CREATE TABLE IF NOT EXISTS main (pk TEXT NOT NULL, sk TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(pk,sk))")
            
            ((:wat::core::Err e) (:wat::core::Err e))
            ((:wat::core::Ok _) (:wat::query::ensure-index-tables conn indexes)))]
       (:wat::service::Outcome::Continue s (:wat::core::Some (:wat::query::Store::Reply::EnsureSchema (:wat::query::ensure-schema-response chained))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat::query::sqlite-store::Op])]))))

   (put [s ctx req]
     (:wat::core::let
       [new-rows (:wat::query::Store::PutRequest/rows req)
        conn  (:wat::query::sqlite-store::State/conn s)
        names (:wat::query::sqlite-store::Record/index-names (:wat::query::sqlite-store::State/durable s))
        chained
          (:wat::core::match (:wat::sqlite::begin conn)
            
            ((:wat::core::Err e) (:wat::core::Err e))
            ((:wat::core::Ok _)
              (:wat::core::match (:wat::query::put-rows conn names new-rows)
                
                ((:wat::core::Err e) (:wat::core::Err e))
                ((:wat::core::Ok _) (:wat::sqlite::commit conn)))))]
       (:wat::service::Outcome::Continue s (:wat::core::Some (:wat::query::Store::Reply::Put (:wat::query::put-response chained))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat::query::sqlite-store::Op])]))))

   (delete [s ctx req]
     (:wat::core::let
       [keys  (:wat::query::Store::DeleteRequest/keys req)
        conn  (:wat::query::sqlite-store::State/conn s)
        names (:wat::query::sqlite-store::Record/index-names (:wat::query::sqlite-store::State/durable s))
        chained
          (:wat::core::match (:wat::sqlite::begin conn)
            ((:wat::core::Err e) (:wat::core::Err e))
            ((:wat::core::Ok _)
              (:wat::core::match (:wat::query::delete-rows conn names keys)
                ((:wat::core::Err e) (:wat::core::Err e))
                ((:wat::core::Ok _) (:wat::sqlite::commit conn)))))]
       (:wat::service::Outcome::Continue s (:wat::core::Some (:wat::query::Store::Reply::Delete (:wat::query::delete-response chained))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat::query::sqlite-store::Op])]))))

   (scan [s ctx req]
     (:wat::core::let
       [conn (:wat::query::sqlite-store::State/conn s)
        pk   (:wat::query::Store::ScanRequest/pk req)
        lo   (:wat::query::Store::ScanRequest/sk-lo req)
        hi   (:wat::query::Store::ScanRequest/sk-hi req)
        lim  (:wat::query::Store::ScanRequest/limit req)
        cur  (:wat::query::Store::ScanRequest/cursor req)
        cur-param (:wat::core::match cur 
                    (:wat::core::None (:wat::sqlite::Param::Nil))
                    ((:wat::core::Some c) (:wat::sqlite::Param::Str c)))
        params (:wat::core::Vector :- [:wat::sqlite::Param]
                 (:wat::sqlite::Param::Str pk) (:wat::sqlite::Param::Str lo) (:wat::sqlite::Param::Str hi)
                 cur-param (:wat::sqlite::Param::I64 lim))
        res (:wat::sqlite::select conn
              "SELECT pk, sk, data FROM main WHERE pk=?1 AND sk>=?2 AND sk<=?3 AND (?4 IS NULL OR sk>?4) ORDER BY sk ASC LIMIT ?5"
              params)
        rows-res (:wat::core::match res 
                   ((:wat::core::Err e) (:wat::core::Err e))
                   ((:wat::core::Ok cell-rows) (:wat::core::Ok (:wat::core::mapv :wat::query::row-from-cells cell-rows))))]
       (:wat::service::Outcome::Continue s (:wat::core::Some (:wat::query::Store::Reply::Scan (:wat::query::scan-response rows-res lim))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat::query::sqlite-store::Op])]))))

   (scan-index [s ctx req]
     (:wat::core::let
       [conn (:wat::query::sqlite-store::State/conn s)
        name (:wat::query::Store::ScanIndexRequest/index req)
        ipk  (:wat::query::Store::ScanIndexRequest/ipk req)
        lo   (:wat::query::Store::ScanIndexRequest/isk-lo req)
        hi   (:wat::query::Store::ScanIndexRequest/isk-hi req)
        lim  (:wat::query::Store::ScanIndexRequest/limit req)
        cur  (:wat::query::Store::ScanIndexRequest/cursor req)
        cur-param (:wat::core::match cur 
                    (:wat::core::None (:wat::sqlite::Param::Nil))
                    ((:wat::core::Some c) (:wat::sqlite::Param::Str c)))
        sql (:wat::core::format
              "SELECT ipk, isk, pk, sk, data FROM [index_{name}] WHERE ipk=?1 AND isk>=?2 AND isk<=?3 AND (?4 IS NULL OR isk>?4) ORDER BY isk ASC LIMIT ?5"
              :name name)
        params (:wat::core::Vector :- [:wat::sqlite::Param]
                 (:wat::sqlite::Param::Str ipk) (:wat::sqlite::Param::Str lo) (:wat::sqlite::Param::Str hi)
                 cur-param (:wat::sqlite::Param::I64 lim))
        res (:wat::sqlite::select conn sql params)
        rows-res (:wat::core::match res 
                   ((:wat::core::Err e) (:wat::core::Err e))
                   ((:wat::core::Ok cell-rows) (:wat::core::Ok (:wat::core::mapv :wat::query::index-row-from-cells cell-rows))))]
       (:wat::service::Outcome::Continue s (:wat::core::Some (:wat::query::Store::Reply::ScanIndex (:wat::query::scan-index-response rows-res lim))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat::query::sqlite-store::Op])]))))

   (count-index [s ctx req]
     (:wat::core::let
       [conn (:wat::query::sqlite-store::State/conn s)
        name (:wat::query::Store::CountIndexRequest/index req)
        ipk  (:wat::query::Store::CountIndexRequest/ipk req)
        lo   (:wat::query::Store::CountIndexRequest/isk-lo req)
        hi   (:wat::query::Store::CountIndexRequest/isk-hi req)
        sql (:wat::core::format
              "SELECT COUNT(*) FROM [index_{name}] WHERE ipk=?1 AND isk>=?2 AND isk<=?3"
              :name name)
        params (:wat::core::Vector :- [:wat::sqlite::Param]
                 (:wat::sqlite::Param::Str ipk) (:wat::sqlite::Param::Str lo) (:wat::sqlite::Param::Str hi))
        res (:wat::sqlite::select conn sql params)
        n-res (:wat::query::count-from-cell-rows res)]
       (:wat::service::Outcome::Continue s (:wat::core::Some (:wat::query::Store::Reply::CountIndex (:wat::query::count-index-response n-res))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat::query::Store::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat::query::sqlite-store::Op])]))))])
