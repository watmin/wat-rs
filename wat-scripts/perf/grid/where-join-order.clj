;; wat-scripts/perf/grid/where-join-order.clj — THE JOIN / :test INTERLEAVING FAMILY, Clara side.
;;
;; Twin of where-join-order.wat. Same fact stream, same predicates, same output format:
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-join-order.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-join-order.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs
;;
;; `check-where-shapes.sh where-join-order` is that, wrapped.
;;
;; ── WHY THIS FAMILY EXISTS ────────────────────────────────────────────────────────────────────
;;
;; Every other `where-*.clj` row is ONE fact pattern + a *trailing* `:test`. where-shapes.clj
;; says so out loud: "a shape can only ever perturb its own trailing :test, never the token
;; stream." That corpus cannot see a TestNode parenting a HashJoin, so it could not have caught
;; wat A1 (tmp/NOTE-where-between-joins-still-false-green.md): `:where` between two positive
;; joins compiles, fires green, and derives nothing. Clara 0.24.0 does not exhibit that — a
;; TestNode is left-only and `send-tokens` to its children, including a HashJoinNode
;; (`clara/rules/engine.cljc`). This pair is the Clara-good ref for that shape. Wat comes to
;; parity when the diff is empty.
;;
;; ── FAITHFULNESS, NOT IDIOM ───────────────────────────────────────────────────────────────────
;;
;; `>` on two bound ints is the same verb on both sides. No `quot`/RATIO trap. Per-row sessions
;; (`mk-session [rule hit-q]`) so the two orders cannot union into one bag.

(ns where-join-order
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(def items 40)

(defrecord Left  [k n])
(defrecord Right [k])
(defrecord Hit   [k])

;; ROW 1 — filter BETWEEN the two joins. Left, :test (> ?n 10), Right.
;; i in [0,40), n=i → k in 11..39 => 29 of 40.
(defrule where-between
  [Left (= ?k k) (= ?n n)]
  [:test (> ?n 10)]
  [Right (= ?k k)]
  => (insert! (->Hit ?k)))

;; ROW 2 — the order wat already honors. Left, Right, :test (> ?n 10). Same set as row 1.
(defrule join-then-where
  [Left (= ?k k) (= ?n n)]
  [Right (= ?k k)]
  [:test (> ?n 10)]
  => (insert! (->Hit ?k)))

;; ROW 3 — tighter mid-chain filter, still Join → Test → Join. n > 25 → k in 26..39 => 14 of 40.
(defrule where-between-hi
  [Left (= ?k k) (= ?n n)]
  [:test (> ?n 25)]
  [Right (= ?k k)]
  => (insert! (->Hit ?k)))

;; ROW 4 — same tight filter, joins first. Same set as row 3.
(defrule join-then-where-hi
  [Left (= ?k k) (= ?n n)]
  [Right (= ?k k)]
  [:test (> ?n 25)]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [Hit (= ?k k)])

(def rows
  [[1 "where-between"      where-between]
   [2 "join-then-where"    join-then-where]
   [3 "where-between-hi"   where-between-hi]
   [4 "join-then-where-hi" join-then-where-hi]])

(defn seed-pair [i] [(->Left i i) (->Right i)])

(def seeds (mapcat seed-pair (range items)))

(defn run-row [[row nm rule]]
  (let [session (apply insert (mk-session [rule hit-q] :cache false) seeds)
        codes   (sort (map :?k (query (fire-rules session) hit-q)))]
    (str "row " row " " nm " n=" (count codes) " ->"
         (apply str (map #(str " " %) codes)))))

(defn -main [& _] (doseq [r rows] (prn (run-row r))))

(-main)
