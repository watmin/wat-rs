;; wat-scripts/perf/grid/where-join-order.wat — THE JOIN / :where INTERLEAVING FAMILY, wat side.
;;
;; Twin of where-join-order.clj. Same fact stream, same predicates, same output format:
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-join-order.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-join-order.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs        # empty  ⇒  wat == Clara on every interleaving
;;
;; `check-where-shapes.sh where-join-order` is that, wrapped. Clara is the good ref; this file
;; comes to parity when the engine left-activates a HashJoin whose parent is a TestNode.
;;
;; ── WHY THIS FAMILY EXISTS ────────────────────────────────────────────────────────────────────
;;
;; The rest of the `where-*` corpus is one fact + a trailing `where` (see where-shapes.wat rule 1).
;; That never builds TestNode → HashJoin, so it cannot catch A1: a `:where` between two positive
;; joins compiles, fires green, and derives nothing (both impls). Clara 0.24.0 derives the same
;; bag either order. Rows 1 and 3 are the hole; rows 2 and 4 are the order wat already honors —
;; same predicate, same stream, so a correct engine prints identical `n=` / sets on 1↔2 and 3↔4.
;;
;; ── THE FOUR RULES (same as where-shapes.wat; restated) ────────────────────────────────────────
;; 1. Each row is its own session (build-rules picks one Rule).
;; 2. EVERY ROW MUST DISCRIMINATE A PROPER SUBSET — 0 < n < 40 — written in the comment.
;; 3. SEED FROM A FORMULA OVER `i`: Left(k=i, n=i) + Right(k=i), i in [0, 40).
;; 4. MIRROR THE OPERATION: `i64::>` against 10 / 25 on both sides; no idiom swap.

(:wat::core::defn :wjo::items [] -> :wat::core::i64 40)

(:wat::core::defn :wjo::row-count [] -> :wat::core::i64 4)

(:wat::core::defrecord :wjo::Left  [k <- :wat::core::i64  n <- :wat::core::i64])
(:wat::core::defrecord :wjo::Right [k <- :wat::core::i64])
(:wat::core::defrecord :wjo::Hit   [k <- :wat::core::i64])

;; ROW 1 — filter BETWEEN the two joins. n > 10 → k in 11..39 => 29/40.
(:wat::rete::defrule :wjo::where-between
  :when
  [(:wjo::Left (?k <- :k) (?n <- :n))
   (:wat::rete::where (:wat::rete::core::i64::> ?n 10))
   (:wjo::Right (?k <- :k))]
  :then
  [(:wjo::Hit ?k)])

;; ROW 2 — joins first, then the same filter. Same set as row 1.
(:wat::rete::defrule :wjo::join-then-where
  :when
  [(:wjo::Left (?k <- :k) (?n <- :n))
   (:wjo::Right (?k <- :k))
   (:wat::rete::where (:wat::rete::core::i64::> ?n 10))]
  :then
  [(:wjo::Hit ?k)])

;; ROW 3 — tighter mid-chain filter. n > 25 → k in 26..39 => 14/40.
(:wat::rete::defrule :wjo::where-between-hi
  :when
  [(:wjo::Left (?k <- :k) (?n <- :n))
   (:wat::rete::where (:wat::rete::core::i64::> ?n 25))
   (:wjo::Right (?k <- :k))]
  :then
  [(:wjo::Hit ?k)])

;; ROW 4 — same tight filter, joins first. Same set as row 3.
(:wat::rete::defrule :wjo::join-then-where-hi
  :when
  [(:wjo::Left (?k <- :k) (?n <- :n))
   (:wjo::Right (?k <- :k))
   (:wat::rete::where (:wat::rete::core::i64::> ?n 25))]
  :then
  [(:wjo::Hit ?k)])

(:wat::core::defn :wjo::build-rules [row <- :wat::core::i64] -> :wat::core::PersistentVector<wat::rete::Rule>
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1) (:wjo::where-between))
      ((:wat::core::= row 2) (:wjo::join-then-where))
      ((:wat::core::= row 3) (:wjo::where-between-hi))
      ((:wat::core::= row 4) (:wjo::join-then-where-hi))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::core::String/concat "where-join-order: unknown row " (:wat::core::i64::to-string row))
          :wat::core::None :wat::core::None)))))

(:wat::core::defn :wjo::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::PersistentVector<wat::core::Record>  i <- :wat::core::i64]
                      -> :wat::core::PersistentVector<wat::core::Record>
        (:wat::core::PersistentVector/conj
          (:wat::core::PersistentVector/conj acc (:wjo::Left :k i :n i))
          (:wjo::Right :k i)))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

(:wat::core::defn :wjo::derived-ints
  [fired <- :wat::rete::Session] -> :wat::core::Vector<wat::core::i64>
  (:wat::core::sort
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [f <- :wjo::Hit] -> :wat::core::i64 (:wjo::Hit/k f))
        (:wat::rete::query-by-type-string fired "wjo::Hit")))))

(:wat::core::defn :wjo::render-ints [v <- :wat::core::Vector<wat::core::i64>] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  x <- :wat::core::i64] -> :wat::core::String
      (:wat::core::String/concat acc
        (:wat::core::String/concat " " (:wat::core::i64::to-string x))))
    ""
    v))

(:wat::core::defn :wjo::rule-display-name
  [full <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  seg <- :wat::core::String] -> :wat::core::String seg)
    full
    (:wat::core::string::split full "::")))

(:wat::core::defn :wjo::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let [rules   (:wjo::build-rules row)
                    rule    (:wat::core::first rules)
                    staged  (:wjo::seed (:wat::rete::compile rules) (:wjo::items))
                    fired   (:wat::rete::fire-rules staged)
                    derived (:wjo::derived-ints fired)
                    n       (:wat::core::Vector/length derived)]
    (:wat::core::String/concat
      (:wat::core::String/concat
        (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
        (:wat::core::String/concat " " (:wjo::rule-display-name (:wat::rete::Rule/name rule))))
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))
        (:wat::core::String/concat " ->" (:wjo::render-ints derived))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wjo::run-row row)))
    nil
    (:wat::core::range 1 (:wat::core::i64::+ (:wjo::row-count) 1))))
