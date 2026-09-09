;; probe-which-index-the-gsi-queries-use.wat — arc 278, stone "the GSI delete gets its reverse mapping"
;;
;; ⚠ ROW 6 / STOP-3, ASKED OF SQLITE RATHER THAN INFERRED FROM A LATENCY.
;;
;; The stone adds `CREATE INDEX [index_<name>_by_key] ON [index_<name>] (pk, sk)` so that
;; `DELETE FROM [index_<name>] WHERE pk=? AND sk=?` is a SEEK instead of a full table scan. Row 6's
;; guard is that this must NOT perturb the three read queries, which all predicate on `ipk` with an
;; `isk` range and are served by `PRIMARY KEY(ipk, isk, pk, sk)`.
;;
;; A latency number cannot answer that question: the sweep's `scan-index` per-call time also moves
;; with worker count, page-cache pressure and the empty-vs-full page mix. `EXPLAIN QUERY PLAN` can —
;; it names the index SQLite chose, for the EXACT statement text the store issues.
;;
;; The four statements are quoted from wat/query/sqlite-store.wat verbatim:
;;   :418  scan          SELECT pk, sk, data FROM main WHERE pk=?1 AND sk>=?2 AND sk<=?3 ...
;;   :438  scan-index    SELECT ipk, isk, pk, sk, data FROM [index_{name}] WHERE ipk=?1 AND ...
;;   :458  count-index   SELECT COUNT(*) FROM (SELECT 1 FROM [index_{name}] WHERE ipk=?1 AND ...)
;;   :228  the DELETE    DELETE FROM [index_{name}] WHERE pk=? AND sk=?
;;
;; WHAT TO READ. `del=` must name `index_by-visible-at_by_key` AFTER the stone and must say
;; `SCAN` BEFORE it — that is the whole defect and the whole fix, stated by the planner. `six=`
;; and `cix=` must name the table's PRIMARY KEY (`sqlite_autoindex_index_by-visible-at_1`) BOTH
;; before and after; if either ever names `_by_key`, the planner HAS switched and row 6 is broken.

(:wat::core::defn :qp::open!
  [path <- :wat::core::String] -> :wat::sqlite::Connection
  (:wat::core::Result/expect (:wat::sqlite::open path) "qp: open failed"))

(:wat::core::defn :qp::ddl!
  [conn <- :wat::sqlite::Connection  sql <- :wat::core::String] -> :wat::core::nil
  (:wat::core::Result/expect (:wat::sqlite::execute-ddl conn sql) "qp: ddl failed"))

;; EXPLAIN QUERY PLAN yields (id, parent, notused, detail); `detail` is the human-readable plan and
;; is the only column worth printing. Every row is joined with "|" so a multi-loop plan is visible.
(:wat::core::defn :qp::plan
  [conn <- :wat::sqlite::Connection  sql <- :wat::core::String] -> :wat::core::String
  (:wat::core::match
    (:wat::sqlite::select conn
      (:wat::core::format "EXPLAIN QUERY PLAN {s}" :s sql)
      (:wat::core::Vector :- [:wat::sqlite::Param]))
    ((:wat::core::Err e)
      (:wat::core::format "PLAN-FAILED({m})" :m (:wat::query::sqlite-error-message e)))
    ((:wat::core::Ok rows)
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::String  r <- (:wat::core::Vector :- [:wat::sqlite::Cell])]
          -> :wat::core::String
          (:wat::core::let
            [d (:wat::query::cell->string (:wat::core::nth r 3))]
            (:wat::core::if (:wat::core::= acc "")
              d
              (:wat::core::format "{a}|{d}" :a acc :d d))))
        ""
        rows))))

;; which indexes SQLite holds on the GSI table, by name — the additive change, listed
(:wat::core::defn :qp::index-list
  [conn <- :wat::sqlite::Connection  table <- :wat::core::String] -> :wat::core::String
  (:wat::core::match
    (:wat::sqlite::select conn
      (:wat::core::format
        "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='{t}' ORDER BY name"
        :t table)
      (:wat::core::Vector :- [:wat::sqlite::Param]))
    ((:wat::core::Err e)
      (:wat::core::format "LIST-FAILED({m})" :m (:wat::query::sqlite-error-message e)))
    ((:wat::core::Ok rows)
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::String  r <- (:wat::core::Vector :- [:wat::sqlite::Cell])]
          -> :wat::core::String
          (:wat::core::let
            [d (:wat::query::cell->string (:wat::core::nth r 0))]
            (:wat::core::if (:wat::core::= acc "") d (:wat::core::format "{a}+{d}" :a acc :d d))))
        ""
        rows))))

(:wat::core::defn :qp::compute [] -> :wat::core::String
  (:wat::core::let
    [path (:wat::core::format "/tmp/wat-qplan-probe-{t}.db"
            :t (:wat::time::epoch-nanos (:wat::time::now)))
     conn (:qp::open! path)
     ;; the schema `ensure-schema` builds, quoted. The store service itself is not needed here —
     ;; this probe is about the PLANNER's choice given the schema, so it builds the schema directly
     ;; and asks. `index-list` below is what proves which world it is in.
     _t1 (:qp::ddl! conn
           "CREATE TABLE IF NOT EXISTS main (pk TEXT NOT NULL, sk TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(pk,sk))")
     _t2 (:qp::ddl! conn
           "CREATE TABLE IF NOT EXISTS [index_by-visible-at] (ipk TEXT NOT NULL, isk TEXT NOT NULL, pk TEXT NOT NULL, sk TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(ipk, isk, pk, sk))")
     ;; ⚠ THE CONTROL: a second GSI table, schema-identical, that does NOT get the stone's index.
     ;; `del0=` below is therefore the BEFORE plan and `del=` the AFTER plan, printed side by side
     ;; in ONE run — no rebuild, no stash, no trusting a remembered number.
     _t0 (:qp::ddl! conn
           "CREATE TABLE IF NOT EXISTS [index_control] (ipk TEXT NOT NULL, isk TEXT NOT NULL, pk TEXT NOT NULL, sk TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(ipk, isk, pk, sk))")
     ;; ⚠ this one line is the stone.
     _t3 (:qp::ddl! conn
           "CREATE INDEX IF NOT EXISTS [index_by-visible-at_by_key] ON [index_by-visible-at] (pk, sk)")
     ixs (:qp::index-list conn "index_by-visible-at")
     ix0 (:qp::index-list conn "index_control")
     ;; the SAME delete, on the table the stone did not touch
     del0 (:qp::plan conn "DELETE FROM [index_control] WHERE pk='a' AND sk='b'")
     ;; and the same scan-index, on the untouched table — so `six` can be compared to a control too
     six0 (:qp::plan conn
            "SELECT ipk, isk, pk, sk, data FROM [index_control] WHERE ipk='p' AND isk>='0' AND isk<='z' AND (NULL IS NULL OR isk>NULL) ORDER BY isk ASC LIMIT 64")
     ;; the DELETE the stone is about (:228)
     del (:qp::plan conn "DELETE FROM [index_by-visible-at] WHERE pk='a' AND sk='b'")
     ;; scan-index (:438)
     six (:qp::plan conn
           "SELECT ipk, isk, pk, sk, data FROM [index_by-visible-at] WHERE ipk='p' AND isk>='0' AND isk<='z' AND (NULL IS NULL OR isk>NULL) ORDER BY isk ASC LIMIT 64")
     ;; count-index (:458)
     cix (:qp::plan conn
           "SELECT COUNT(*) FROM (SELECT 1 FROM [index_by-visible-at] WHERE ipk='p' AND isk>='0' AND isk<='z' LIMIT 8192)")
     ;; scan on the base table (:418) — must be untouched; there is no new index on `main` at all
     scn (:qp::plan conn
           "SELECT pk, sk, data FROM main WHERE pk='p' AND sk>='0' AND sk<='z' AND (NULL IS NULL OR sk>NULL) ORDER BY sk ASC LIMIT 64")
     ;; put's own GSI clear is the SAME predicate as the delete (clear-index-projections), and put's
     ;; base-table clear predicates on main's full primary key
     mdl (:qp::plan conn "DELETE FROM main WHERE pk='a' AND sk='b'")]
    (:wat::core::format
      "gsi-indexes=[{ixs}]\ncontrol-indexes=[{ix0}]\nBEFORE del0={del0}\nAFTER  del={del}\nBEFORE six0={six0}\nAFTER  six={six}\ncix={cix}\nscn={scn}\nmdl={mdl}"
      :ixs ixs :ix0 ix0 :del0 del0 :del del :six0 six0 :six six
      :cix cix :scn scn :mdl mdl)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:qp::compute)))
