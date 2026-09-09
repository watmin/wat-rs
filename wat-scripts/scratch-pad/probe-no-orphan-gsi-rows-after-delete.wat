;; probe-no-orphan-gsi-rows-after-delete.wat — arc 278, stone "the GSI delete gets its reverse mapping"
;;
;; ★★ ROWS 1 AND 2 OF THE STONE, CHECKED DIRECTLY AND NOT INFERRED.
;;
;; `sqlite-store` models a GSI as a SEPARATE COMPLETE TABLE
;; `index_<name>(ipk,isk,pk,sk,data,PRIMARY KEY(ipk,isk,pk,sk))`, and both `delete` and `put`'s
;; clear step remove a row's projection with `DELETE FROM [index_<name>] WHERE pk=? AND sk=?`.
;; The stone adds `CREATE INDEX IF NOT EXISTS [index_<name>_by_key] ON [index_<name>] (pk, sk)`
;; to `ensure-schema` so that predicate is a seek rather than a scan.
;;
;; An ORPHAN GSI row — an `index_*` row whose (pk, sk) no longer exists in `main` — is INVISIBLE to
;; the circuit's `distinct`/`dup` counters (those count delivered messages, not index rows), and it
;; is the failure that matters most here: `count-index` derives `depth`, `depth` drives
;; `room = cap - depth`, and that drives ADMISSION. An orphan makes the queue refuse publishers for
;; capacity it actually has. So this probe asks SQLite itself, over a FILE-backed store (a second
;; connection can then read the very tables the service wrote), rather than inferring from any
;; higher-level count.
;;
;; What it checks, per GSI table, after put/delete/re-put cycles:
;;   orphans        SELECT COUNT(*) FROM index_X i LEFT JOIN main m USING(pk,sk) WHERE m.pk IS NULL
;;                  -> must be 0. THE row-1 gate.
;;   missing        the same join the other way -> must be 0. A live row must still be projected;
;;                  a delete that removed too much is as wrong as one that removed too little.
;;   dup-projections  (pk,sk) appearing more than once in one GSI table -> must be 0. This is the
;;                  orphan the LEFT JOIN CANNOT see: a STALE projection under a different (ipk,isk)
;;                  but the SAME base key. `put`'s clear step is what must remove it.
;;   rows           index_X row count -> must equal `main`'s row count.
;;   count-index    the Store-surface depth -> must equal the live row count. THE row-2 gate.
;;
;; ⚠ NEGATIVE CONTROL. A gate that cannot fail proves nothing (R59). Phase 3 INJECTS a ghost row
;; straight into `index_by-visible-at` and requires the orphan query to report exactly 1, then
;; removes it and requires 0 again. If `neg-control=1/0` does not print, the orphan query is not
;; measuring what it claims.
;;
;; Two GSIs are declared on purpose: `index_<name>_by_key` must not COLLIDE between GSI tables
;; (with `IF NOT EXISTS` a collision would silently leave the second table unindexed, not error).
;; `ix-idx1`/`ix-idx2` read `sqlite_master` for the two index names — 0/0 before the stone lands,
;; 1/1 after. Everything else in this probe must hold BOTH ways: the change is additive.

;; ─── dialling and driving the store ────────────────────────────────────────────────────────────
(:wat::core::defn :orph::dial-store
  [a <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "orph: dial-store failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :orph::ensure
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::core::nil
  (:wat::core::match
    (:wat::query::Store/ensure-schema st
      (:wat::query::Store::EnsureSchemaRequest
        :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
        :indexes (:wat::core::Vector :- [:wat::query::IndexSchema]
                   (:wat::query::IndexSchema
                     :name "by-visible-at" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk")
                   (:wat::query::IndexSchema
                     :name "by-owner" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::EnsureSchemaResponse::Success) nil)
        (_ (:wat::kernel::assertion-failed! "orph: ensure-schema not Success"
             :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "orph: ensure-schema recv failed"
         :wat::core::None :wat::core::None))))

(:wat::core::defn :orph::sk-of [i <- :wat::core::i64] -> :wat::core::String
  (:wat::core::format "{n}" :n (:wat::i64::+ 10000 i)))

;; one row, projecting into BOTH declared GSIs. `ipk1` lets a re-put move a row to a DIFFERENT
;; partition of `by-visible-at` — that is what makes a stale projection visible.
(:wat::core::defn :orph::row-at
  [i <- :wat::core::i64  ipk1 <- :wat::core::String]
  -> :wat::query::StoredRow
  (:wat::core::let [k (:orph::sk-of i)]
    (:wat::query::StoredRow
      :pk "t" :sk k :data k
      :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                    "by-visible-at" (:wat::query::IndexKey :ipk ipk1 :isk k)
                    "by-owner"      (:wat::query::IndexKey :ipk "own" :isk k)))))

(:wat::core::defn :orph::put-batch
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   rows <- (:wat::core::Vector :- [:wat::query::StoredRow])]
  -> :wat::core::nil
  (:wat::core::match (:wat::query::Store/put st (:wat::query::Store::PutRequest :rows rows))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::PutResponse::Success) nil)
        (_ (:wat::kernel::assertion-failed! "orph: put not Success"
             :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "orph: put recv failed"
         :wat::core::None :wat::core::None))))

;; put every i in [lo, hi), 50 rows per request
(:wat::core::defn :orph::put-range
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   lo <- :wat::core::i64  hi <- :wat::core::i64  ipk1 <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::let
    [batch 50
     n      (:wat::i64::- hi lo)
     nbatch (:wat::i64::/ (:wat::i64::+ n (:wat::i64::- batch 1)) batch)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::nil  b <- :wat::core::i64] -> :wat::core::nil
        (:wat::core::let
          [start (:wat::i64::+ lo (:wat::i64::* b batch))
           stop  (:wat::core::if (:wat::i64::>= (:wat::i64::+ start batch) hi)
                   hi
                   (:wat::i64::+ start batch))
           rows (:wat::core::foldl
                  (:wat::core::fn [rs <- (:wat::core::Vector :- [:wat::query::StoredRow])
                                   i  <- :wat::core::i64]
                    -> (:wat::core::Vector :- [:wat::query::StoredRow])
                    (:wat::core::conj rs (:orph::row-at i ipk1)))
                  (:wat::core::Vector :- [:wat::query::StoredRow])
                  (:wat::core::range start stop))]
          (:orph::put-batch st rows)))
      nil
      (:wat::core::range 0 nbatch))))

(:wat::core::defn :orph::delete-batch
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   keys <- (:wat::core::Vector :- [:wat::query::Key])]
  -> :wat::core::nil
  (:wat::core::match (:wat::query::Store/delete st (:wat::query::Store::DeleteRequest :keys keys))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::DeleteResponse::Success) nil)
        (_ (:wat::kernel::assertion-failed! "orph: delete not Success"
             :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "orph: delete recv failed"
         :wat::core::None :wat::core::None))))

;; delete every i in [lo, hi) BY BASE KEY — the DDB contract the stone is completing
(:wat::core::defn :orph::delete-range
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   lo <- :wat::core::i64  hi <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::let
    [batch 50
     n      (:wat::i64::- hi lo)
     nbatch (:wat::i64::/ (:wat::i64::+ n (:wat::i64::- batch 1)) batch)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::nil  b <- :wat::core::i64] -> :wat::core::nil
        (:wat::core::let
          [start (:wat::i64::+ lo (:wat::i64::* b batch))
           stop  (:wat::core::if (:wat::i64::>= (:wat::i64::+ start batch) hi)
                   hi
                   (:wat::i64::+ start batch))
           keys (:wat::core::foldl
                  (:wat::core::fn [ks <- (:wat::core::Vector :- [:wat::query::Key])
                                   i  <- :wat::core::i64]
                    -> (:wat::core::Vector :- [:wat::query::Key])
                    (:wat::core::conj ks (:wat::query::Key :pk "t" :sk (:orph::sk-of i))))
                  (:wat::core::Vector :- [:wat::query::Key])
                  (:wat::core::range start stop))]
          (:orph::delete-batch st keys)))
      nil
      (:wat::core::range 0 nbatch))))

;; the Store-surface depth — row 2's instrument, the SAME call the queue's cap gate makes
(:wat::core::defn :orph::count-index
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   index <- :wat::core::String  ipk <- :wat::core::String]
  -> :wat::core::i64
  (:wat::core::match
    (:wat::query::Store/count-index st
      (:wat::query::Store::CountIndexRequest
        :index index :ipk ipk :isk-lo "0" :isk-hi "z" :limit 1000000))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::CountIndexResponse::Ok n) n)
        (_ -1)))
    (_ -2)))

;; ─── the second connection — SQLite answers about its own tables ───────────────────────────────
(:wat::core::defn :orph::open!
  [path <- :wat::core::String] -> :wat::sqlite::Connection
  (:wat::core::Result/expect (:wat::sqlite::open path) "orph: inspect open failed"))

(:wat::core::defn :orph::scalar
  [conn <- :wat::sqlite::Connection  sql <- :wat::core::String] -> :wat::core::i64
  (:wat::core::match
    (:wat::sqlite::select conn sql (:wat::core::Vector :- [:wat::sqlite::Param]))
    ((:wat::core::Ok rows)
      (:wat::core::if (:wat::core::empty? rows)
        (:wat::kernel::assertion-failed!
          (:wat::core::format "orph: scalar returned no row: {s}" :s sql)
          :wat::core::None :wat::core::None)
        (:wat::core::match (:wat::core::nth (:wat::core::nth rows 0) 0)
          ((:wat::sqlite::Cell::I64 n) n)
          (_ (:wat::kernel::assertion-failed!
               (:wat::core::format "orph: scalar not I64: {s}" :s sql)
               :wat::core::None :wat::core::None)))))
    ((:wat::core::Err e)
      (:wat::kernel::assertion-failed!
        (:wat::core::format "orph: select failed ({m}): {s}"
          :m (:wat::query::sqlite-error-message e) :s sql)
        :wat::core::None :wat::core::None))))

(:wat::core::defn :orph::exec!
  [conn <- :wat::sqlite::Connection  sql <- :wat::core::String] -> :wat::core::i64
  (:wat::core::match
    (:wat::sqlite::execute conn sql (:wat::core::Vector :- [:wat::sqlite::Param]))
    ((:wat::core::Ok n) n)
    ((:wat::core::Err e)
      (:wat::kernel::assertion-failed!
        (:wat::core::format "orph: execute failed ({m}): {s}"
          :m (:wat::query::sqlite-error-message e) :s sql)
        :wat::core::None :wat::core::None))))

(:wat::core::defn :orph::main-rows [conn <- :wat::sqlite::Connection] -> :wat::core::i64
  (:orph::scalar conn "SELECT COUNT(*) FROM main"))

(:wat::core::defn :orph::ix-rows
  [conn <- :wat::sqlite::Connection  name <- :wat::core::String] -> :wat::core::i64
  (:orph::scalar conn (:wat::core::format "SELECT COUNT(*) FROM [index_{n}]" :n name)))

;; ★★ THE ROW-1 QUERY. index rows whose base key is GONE from `main`.
(:wat::core::defn :orph::orphans
  [conn <- :wat::sqlite::Connection  name <- :wat::core::String] -> :wat::core::i64
  (:orph::scalar conn
    (:wat::core::format
      "SELECT COUNT(*) FROM [index_{n}] i LEFT JOIN main m ON i.pk = m.pk AND i.sk = m.sk WHERE m.pk IS NULL"
      :n name)))

;; the same join reversed — a live row that lost its projection (a delete that removed too much)
(:wat::core::defn :orph::missing
  [conn <- :wat::sqlite::Connection  name <- :wat::core::String] -> :wat::core::i64
  (:orph::scalar conn
    (:wat::core::format
      "SELECT COUNT(*) FROM main m LEFT JOIN [index_{n}] i ON i.pk = m.pk AND i.sk = m.sk WHERE i.pk IS NULL"
      :n name)))

;; the orphan the LEFT JOIN cannot see — one base key projected twice (a stale (ipk,isk))
(:wat::core::defn :orph::dup-projections
  [conn <- :wat::sqlite::Connection  name <- :wat::core::String] -> :wat::core::i64
  (:orph::scalar conn
    (:wat::core::format
      "SELECT COUNT(*) FROM (SELECT pk, sk, COUNT(*) AS c FROM [index_{n}] GROUP BY pk, sk HAVING c > 1)"
      :n name)))

(:wat::core::defn :orph::has-index?
  [conn <- :wat::sqlite::Connection  name <- :wat::core::String] -> :wat::core::i64
  (:orph::scalar conn
    (:wat::core::format
      "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'index_{n}_by_key'"
      :n name)))

;; one line of health for one GSI table, at one moment
(:wat::core::defn :orph::health
  [conn <- :wat::sqlite::Connection  name <- :wat::core::String] -> :wat::core::String
  (:wat::core::format "rows={r},orphans={o},missing={m},dups={d}"
    :r (:orph::ix-rows conn name)
    :o (:orph::orphans conn name)
    :m (:orph::missing conn name)
    :d (:orph::dup-projections conn name)))

(:wat::core::defn :orph::compute [] -> :wat::core::String
  (:wat::core::let
    [;; a fresh file per run — no stale schema, and a second connection can read it (WAL)
     path (:wat::core::format "/tmp/wat-orph-probe-{t}.db"
            :t (:wat::time::epoch-nanos (:wat::time::now)))
     ins  (:orph::open! path)
     sh   (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
            :record (:wat::query::sqlite-store::Record
                      :path path
                      :index-names (:wat::core::Vector :- [:wat::core::String]
                                     "by-visible-at" "by-owner")))
     st   (:orph::dial-store (:wat::query::sqlite-store::Handle/addr sh))
     _e1  (:orph::ensure st)
     ;; TRAP-DOOR: ensure-schema runs on EVERY store start and must stay idempotent
     _e2  (:orph::ensure st)
     ix1  (:orph::has-index? ins "by-visible-at")
     ix2  (:orph::has-index? ins "by-owner")

     ;; ── PHASE 1: 200 rows, both GSIs, partition "idx" ─────────────────────────────────────
     _p1   (:orph::put-range st 0 200 "idx")
     p1m   (:orph::main-rows ins)
     p1a   (:orph::health ins "by-visible-at")
     p1b   (:orph::health ins "by-owner")
     p1c   (:orph::count-index st "by-visible-at" "idx")
     p1d   (:orph::count-index st "by-owner" "own")

     ;; ── PHASE 2: delete 80 of them BY BASE KEY ────────────────────────────────────────────
     _p2   (:orph::delete-range st 40 120)
     p2m   (:orph::main-rows ins)
     p2a   (:orph::health ins "by-visible-at")
     p2b   (:orph::health ins "by-owner")
     p2c   (:orph::count-index st "by-visible-at" "idx")
     p2d   (:orph::count-index st "by-owner" "own")

     ;; ── PHASE 3: NEGATIVE CONTROL — the orphan query must be able to FAIL ─────────────────
     _g1   (:orph::exec! ins
             "INSERT INTO [index_by-visible-at] (ipk,isk,pk,sk,data) VALUES ('idx','99999','t','99999','ghost')")
     gpos  (:orph::orphans ins "by-visible-at")
     _g2   (:orph::exec! ins "DELETE FROM [index_by-visible-at] WHERE data = 'ghost'")
     gneg  (:orph::orphans ins "by-visible-at")

     ;; ── PHASE 4: re-put 40 rows into a DIFFERENT GSI partition. `put`'s clear step must
     ;;    remove the old projection, or `by-visible-at` ends up with 160 rows and
     ;;    count-index("idx") stays 120 — a stale projection no LEFT JOIN would catch. ───────
     _p4   (:orph::put-range st 0 40 "idx2")
     p4m   (:orph::main-rows ins)
     p4a   (:orph::health ins "by-visible-at")
     p4b   (:orph::health ins "by-owner")
     p4c   (:orph::count-index st "by-visible-at" "idx")
     p4c2  (:orph::count-index st "by-visible-at" "idx2")
     p4d   (:orph::count-index st "by-owner" "own")

     ;; ── PHASE 5: delete EVERYTHING, including keys that were already deleted ─────────────
     _p5   (:orph::delete-range st 0 200)
     p5m   (:orph::main-rows ins)
     p5a   (:orph::health ins "by-visible-at")
     p5b   (:orph::health ins "by-owner")
     p5c   (:orph::count-index st "by-visible-at" "idx")
     p5c2  (:orph::count-index st "by-visible-at" "idx2")
     p5d   (:orph::count-index st "by-owner" "own")

     ;; the whole point, as one word
     ok (:wat::core::and
          (:wat::core::and
            (:wat::core::and (:wat::core::= p1m 200) (:wat::core::= p2m 120))
            (:wat::core::and (:wat::core::= p4m 120) (:wat::core::= p5m 0)))
          (:wat::core::and
            (:wat::core::and
              (:wat::core::and (:wat::core::= p1c 200) (:wat::core::= p1d 200))
              (:wat::core::and (:wat::core::= p2c 120) (:wat::core::= p2d 120)))
            (:wat::core::and
              (:wat::core::and
                (:wat::core::and (:wat::core::= p4c 80) (:wat::core::= p4c2 40))
                (:wat::core::and (:wat::core::= p4d 120) (:wat::core::= p5c 0)))
              (:wat::core::and
                (:wat::core::and (:wat::core::= p5c2 0) (:wat::core::= p5d 0))
                (:wat::core::and (:wat::core::= gpos 1) (:wat::core::= gneg 0))))))]
    (:wat::core::format
      "ix-by-key=[{ix1}/{ix2}];p1=main={p1m},vis[{p1a}],own[{p1b}],depth={p1c}/{p1d};p2=main={p2m},vis[{p2a}],own[{p2b}],depth={p2c}/{p2d};neg-control={gpos}/{gneg};p4=main={p4m},vis[{p4a}],own[{p4b}],depth-idx={p4c},depth-idx2={p4c2},depth-own={p4d};p5=main={p5m},vis[{p5a}],own[{p5b}],depth={p5c}/{p5c2}/{p5d};VERDICT={ok}"
      :ix1 ix1 :ix2 ix2
      :p1m p1m :p1a p1a :p1b p1b :p1c p1c :p1d p1d
      :p2m p2m :p2a p2a :p2b p2b :p2c p2c :p2d p2d
      :gpos gpos :gneg gneg
      :p4m p4m :p4a p4a :p4b p4b :p4c p4c :p4c2 p4c2 :p4d p4d
      :p5m p5m :p5a p5a :p5b p5b :p5c p5c :p5c2 p5c2 :p5d p5d
      :ok (:wat::core::if ok "NO-ORPHANS" "ORPHANS-OR-WRONG-DEPTH"))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:orph::compute)))
