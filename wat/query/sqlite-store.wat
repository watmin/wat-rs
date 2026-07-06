;; wat/query/sqlite-store.wat — Arc 278 stone S2: `:wat::sqlite'::SqliteStore` — the sqlite
;; `:wat::query::Store` / `ReadStore` satisfier, built as SQL over S1's raw `:wat::sqlite'`
;; verbs (wat/sqlite.wat). Differential-tested against the S-mem MemStore oracle
;; (tests/rete/probe_arc278_sqlite_store_differential.{wat,rs}).
;;
;; Ratified design: DESIGN-STONE-S2-sqlite-satisfier.md. DDB-faithful — a GSI is a SEPARATE
;; COMPLETE TABLE `index_<name>(ipk,isk,pk,sk,data,PK(ipk,isk,pk,sk))`, not a native sqlite index
;; on the `main` table. `scan`/`scan-index` are ONE keyset primitive (parameterized by table +
;; partition-col + sort-col); `put` is clear-then-insert (upsert-safe).
;;
;; ─── why the struct carries `index-names` (a deviation from the design sketch's single-field
;; illustration, NOT from the ratified model) ────────────────────────────────────────────────────
;; `put`'s clear step must DELETE this row's old projection from EVERY declared GSI table, not just
;; the ones the incoming row happens to project into this call (an update that STOPS projecting
;; into a GSI must still have its old `index_<name>` row cleared) — the design's own words: "the
;; index tables carry (pk,sk) so the clear needs no old-item read". `put`'s contract signature
;; (`Store/put [self rows]`) carries no index list, so the ONLY honest way for `put` to know the
;; full GSI-name set is for `self` to already carry it — exactly the design's "the wat-layer
;; SqliteStore already knows its declared GSIs" (the metadata-table alternative is explicitly
;; named OPTIONAL/out-of-scope for S2). So `SqliteStore` carries `conn` + `index-names`, populated
;; once at construction (mirrors `ensure-schema`'s own index list — the caller supplies both).
;;
;; ─── the satisfier struct + constructor ─────────────────────────────────────────────────────────
(:wat::core::defstruct :wat::sqlite'::SqliteStore
  [conn        <- :wat::sqlite'::Connection
   index-names <- (:wat::core::Vector :wat::core::String)])

;; convenience constructor: open `path` (":memory:" or a file path) + set the WAL/NORMAL pragmas
;; the design calls for, wrapping the opened Connection with the caller-declared GSI-name set.
(:wat::core::defn :wat::sqlite'::SqliteStore/open
  [path <- :wat::core::String index-names <- (:wat::core::Vector :wat::core::String)]
  -> :wat::core::Result<wat::sqlite'::SqliteStore,wat::sqlite'::Error>
  (:wat::core::match (:wat::sqlite'::open path)
    -> :wat::core::Result<wat::sqlite'::SqliteStore,wat::sqlite'::Error>
    ((:wat::core::Err e) (:wat::core::Err e))
    ((:wat::core::Ok conn)
      (:wat::core::match (:wat::sqlite'::pragma conn "journal_mode" "WAL")
        -> :wat::core::Result<wat::sqlite'::SqliteStore,wat::sqlite'::Error>
        ((:wat::core::Err e) (:wat::core::Err e))
        ((:wat::core::Ok _)
          (:wat::core::match (:wat::sqlite'::pragma conn "synchronous" "NORMAL")
            -> :wat::core::Result<wat::sqlite'::SqliteStore,wat::sqlite'::Error>
            ((:wat::core::Err e) (:wat::core::Err e))
            ((:wat::core::Ok _) (:wat::core::Ok (:wat::sqlite'::SqliteStore conn index-names)))))))))

;; ─── the error lift — :wat::sqlite'::Error -> :wat::query::Error (both recovery-axis, field-for-
;; field identical Fault shape: op/code/diagnostic/message) ─────────────────────────────────────
(:wat::core::defn :wat::sqlite'::lift-fault [f <- :wat::sqlite'::Fault] -> :wat::query::Fault
  (:wat::query::Fault
    (:wat::sqlite'::Fault/op f)
    (:wat::sqlite'::Fault/code f)
    (:wat::sqlite'::Fault/diagnostic f)
    (:wat::sqlite'::Fault/message f)))

(:wat::core::defn :wat::sqlite'::lift-error [e <- :wat::sqlite'::Error] -> :wat::query::Error
  (:wat::core::match e -> :wat::query::Error
    ((:wat::sqlite'::Error::Transient f)  (:wat::query::Error::Transient  (:wat::sqlite'::lift-fault f)))
    ((:wat::sqlite'::Error::Constraint f) (:wat::query::Error::Constraint (:wat::sqlite'::lift-fault f)))
    ((:wat::sqlite'::Error::Fatal f)      (:wat::query::Error::Fatal      (:wat::sqlite'::lift-fault f)))))

;; ─── Cell -> String unpacking (pk/sk/data/ipk/isk columns are always TEXT NOT NULL, so Str is the
;; live path; the other arms are exhaustiveness-only, never hit against this store's own schema) ──
(:wat::core::defn :wat::sqlite'::cell->string [c <- :wat::sqlite'::Cell] -> :wat::core::String
  (:wat::core::match c -> :wat::core::String
    ((:wat::sqlite'::Cell::Str s) s)
    ((:wat::sqlite'::Cell::I64 n) (:wat::core::i64::to-string n))
    ((:wat::sqlite'::Cell::F64 _) "")
    ((:wat::sqlite'::Cell::Nil) "")))

(:wat::core::defn :wat::sqlite'::row-from-cells
  [cells <- (:wat::core::Vector :wat::sqlite'::Cell)] -> :wat::query::Row
  (:wat::query::Row
    (:wat::sqlite'::cell->string (:wat::core::nth cells 0))    ;; pk
    (:wat::sqlite'::cell->string (:wat::core::nth cells 1))    ;; sk
    (:wat::sqlite'::cell->string (:wat::core::nth cells 2))))  ;; data

(:wat::core::defn :wat::sqlite'::index-row-from-cells
  [cells <- (:wat::core::Vector :wat::sqlite'::Cell)] -> :wat::query::IndexRow
  ;; select order is (ipk,isk,pk,sk,data); IndexRow's own field order is (pk,sk,ipk,isk,data).
  (:wat::query::IndexRow
    (:wat::sqlite'::cell->string (:wat::core::nth cells 2))    ;; pk
    (:wat::sqlite'::cell->string (:wat::core::nth cells 3))    ;; sk
    (:wat::sqlite'::cell->string (:wat::core::nth cells 0))    ;; ipk
    (:wat::sqlite'::cell->string (:wat::core::nth cells 1))    ;; isk
    (:wat::sqlite'::cell->string (:wat::core::nth cells 4))))  ;; data

;; ─── the keyset page-builders — next-cursor = last row's sort key IFF the page came back full ───
(:wat::core::defn :wat::sqlite'::build-page
  [rows <- (:wat::core::Vector :wat::query::Row) limit <- :wat::core::i64] -> :wat::query::Page
  (:wat::core::let
    [full?    (:wat::core::= (:wat::core::count rows) limit)
     next-cur (:wat::core::if full?
                (:wat::core::Some
                  (:wat::query::Row/sk (:wat::core::Option/expect (:wat::core::last rows) "scan: rows non-empty when full")))
                :wat::core::None)]
    (:wat::query::Page rows next-cur)))

(:wat::core::defn :wat::sqlite'::build-index-page
  [rows <- (:wat::core::Vector :wat::query::IndexRow) limit <- :wat::core::i64] -> :wat::query::IndexPage
  (:wat::core::let
    [full?    (:wat::core::= (:wat::core::count rows) limit)
     next-cur (:wat::core::if full?
                (:wat::core::Some
                  (:wat::query::IndexRow/isk (:wat::core::Option/expect (:wat::core::last rows) "scan-index: rows non-empty when full")))
                :wat::core::None)]
    (:wat::query::IndexPage rows next-cur)))

;; ─── ensure-schema — main + one complete table per named GSI ────────────────────────────────────
(:wat::core::defn :wat::sqlite'::ensure-index-tables
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
        ((:wat::core::Ok _) (:wat::sqlite'::ensure-index-tables conn tl))))))

;; ─── put — clear-then-insert, one row at a time inside the caller's BEGIN/COMMIT ────────────────
(:wat::core::defn :wat::sqlite'::clear-index-projections
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
        ((:wat::core::Ok _) (:wat::sqlite'::clear-index-projections conn tl pk sk))))))

(:wat::core::defn :wat::sqlite'::insert-index-projections
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
        (:wat::core::None (:wat::sqlite'::insert-index-projections conn tl pk sk data index-keys))
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
              ((:wat::core::Ok _) (:wat::sqlite'::insert-index-projections conn tl pk sk data index-keys)))))))))

(:wat::core::defn :wat::sqlite'::put-one-row
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
        (:wat::core::match (:wat::sqlite'::clear-index-projections conn index-names pk sk)
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
                (:wat::sqlite'::insert-index-projections conn index-names pk sk data index-keys)))))))))

(:wat::core::defn :wat::sqlite'::put-rows
  [conn <- :wat::sqlite'::Connection index-names <- (:wat::core::Vector :wat::core::String)
   rows <- (:wat::core::Vector :wat::query::StoredRow)]
  -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
  (:wat::core::if (:wat::core::empty? rows)
    (:wat::core::Ok nil)
    (:wat::core::match (:wat::sqlite'::put-one-row conn index-names (:wat::core::first rows))
      -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
      ((:wat::core::Err e) (:wat::core::Err e))
      ((:wat::core::Ok _) (:wat::sqlite'::put-rows conn index-names (:wat::core::rest rows))))))

;; ─── the satisfier — the 4 Store methods, SQL over S1's verbs ───────────────────────────────────
(:wat::core::extend-type :wat::sqlite'::SqliteStore :wat::query::Store
  (ensure-schema [self table indexes]
    (:wat::core::let
      [conn    (:wat::sqlite'::SqliteStore/conn self)
       chained
         (:wat::core::match
           (:wat::sqlite'::execute-ddl conn
             "CREATE TABLE IF NOT EXISTS main (pk TEXT NOT NULL, sk TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(pk,sk))")
           -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
           ((:wat::core::Err e) (:wat::core::Err e))
           ((:wat::core::Ok _) (:wat::sqlite'::ensure-index-tables conn indexes)))]
      (:wat::core::match chained -> :wat::core::Result<wat::core::nil,wat::query::Error>
        ((:wat::core::Ok _) (:wat::core::Ok nil))
        ((:wat::core::Err e) (:wat::core::Err (:wat::sqlite'::lift-error e))))))

  (put [self rows]
    (:wat::core::let
      [conn    (:wat::sqlite'::SqliteStore/conn self)
       names   (:wat::sqlite'::SqliteStore/index-names self)
       chained
         (:wat::core::match (:wat::sqlite'::begin conn)
           -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
           ((:wat::core::Err e) (:wat::core::Err e))
           ((:wat::core::Ok _)
             (:wat::core::match (:wat::sqlite'::put-rows conn names rows)
               -> :wat::core::Result<wat::core::nil,wat::sqlite'::Error>
               ((:wat::core::Err e) (:wat::core::Err e))
               ((:wat::core::Ok _) (:wat::sqlite'::commit conn)))))]
      (:wat::core::match chained -> :wat::core::Result<wat::core::nil,wat::query::Error>
        ((:wat::core::Ok _) (:wat::core::Ok nil))
        ((:wat::core::Err e) (:wat::core::Err (:wat::sqlite'::lift-error e))))))

  (scan [self q]
    (:wat::core::let
      [conn (:wat::sqlite'::SqliteStore/conn self)
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
      (:wat::core::match res -> :wat::core::Result<wat::query::Page,wat::query::Error>
        ((:wat::core::Err e) (:wat::core::Err (:wat::sqlite'::lift-error e)))
        ((:wat::core::Ok cell-rows)
          (:wat::core::Ok
            (:wat::sqlite'::build-page (:wat::core::mapv :wat::sqlite'::row-from-cells cell-rows) lim))))))

  (scan-index [self q]
    (:wat::core::let
      [conn (:wat::sqlite'::SqliteStore/conn self)
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
      (:wat::core::match res -> :wat::core::Result<wat::query::IndexPage,wat::query::Error>
        ((:wat::core::Err e) (:wat::core::Err (:wat::sqlite'::lift-error e)))
        ((:wat::core::Ok cell-rows)
          (:wat::core::Ok
            (:wat::sqlite'::build-index-page (:wat::core::mapv :wat::sqlite'::index-row-from-cells cell-rows) lim)))))))

;; the read-only edge — SqliteStore's scan/scan-index (from the Store impl above) also satisfy
;; ReadStore, exactly mirroring MemStore's `derive` (extend-type's edge-only half).
(:wat::core::derive :wat::sqlite'::SqliteStore :wat::query::ReadStore)
