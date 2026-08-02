;; wat-scripts/perf/grid/where-shapes.clj — THE `where`-CLAUSE EXPRESSIVITY CORPUS, Clara side.
;;
;; The twin of where-shapes.wat. Same fact stream, same predicates, same output format — so the
;; whole verdict is:
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-shapes.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-shapes.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs
;;
;; An empty diff means every row derives the same set in both engines. A hunk NAMES the row that
;; diverged, because each row is its own line carrying its own count.
;;
;; ── WHY EVERY ROW GETS ITS OWN SESSION ────────────────────────────────────────────────────────
;;
;; `mk-session` is called with an EXPLICIT PRODUCTION LIST — `(mk-session [rule hit-q] …)` — never
;; with the namespace symbol. That is load-bearing, not style: `(mk-session 'where-shapes)` collects
;; EVERY defrule in the namespace, so all six rules would fire into one session, the derived sets
;; would UNION, and a divergence could not name the shape that caused it. Per-row sessions keep each
;; shape isolated, exactly as the wat side's `run-row` does.
;;
;; ── FAITHFULNESS, NOT IDIOM ───────────────────────────────────────────────────────────────────
;;
;; Every predicate MIRRORS the wat operation rather than idiomatising it, so a row measures the
;; constraint and not a translation choice. Concretely: `quot`, never `/`. Clojure's `/` on two
;; integers yields a RATIO, so `(/ ?k 10)` would silently change the semantics where wat's `i64::/`
;; truncates. For k >= 0, `(- k (* (quot k 10) 10))` is exactly k mod 10 — the same arithmetic the
;; wat side spells out, written out here rather than collapsed to `mod`.
;;
;; ── GROWING THE CORPUS ────────────────────────────────────────────────────────────────────────
;;
;;   1. a `defrule` below (copy its neighbour; only the `:test` differs)
;;   2. one entry in `rows`
;;   3. the mirrored arm in where-shapes.wat + bump its `row-count`

(ns where-shapes
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clojure.string :as str]))

(def items 200)                                    ;; the stream size, both sides

(defrecord Client [rep])
(defrecord Req    [k client name tags limit])
(defrecord Hit    [k])

;; row 5's user-defined pure fn, mirroring wsh::big? EXACTLY: k mod 7 > 3 via quot.
(defn big? [k] (> (- k (* (quot k 7) 7)) 3))

;; THE SHARED LEADING CONDITION — every row binds all five fields off one [Req ...] pattern, so a
;; shape can only ever perturb its own trailing :test, never the token stream every row shares.
;; (Written out per rule because Clara's defrule takes the pattern literally.)

;; ROW 1 — arithmetic. k mod 10 == 3 => 20 of 200.
(defrule arith
  [Req (= ?k k) (= ?c client) (= ?n name) (= ?t tags) (= ?l limit)]
  [:test (= 3 (- ?k (* (quot ?k 10) 10)))]
  => (insert! (->Hit ?k)))

;; ROW 2 — record accessor. rep(k) = (k mod 5) - 2; rep > 0 => 80 of 200.
(defrule accessor
  [Req (= ?k k) (= ?c client) (= ?n name) (= ?t tags) (= ?l limit)]
  [:test (> (:rep ?c) 0)]
  => (insert! (->Hit ?k)))

;; ROW 3 — String verb. name(k) = "ad"+k when k mod 3 == 0 else "zz"+k => 67 of 200.
(defrule string-shape
  [Req (= ?k k) (= ?c client) (= ?n name) (= ?t tags) (= ?l limit)]
  [:test (str/starts-with? ?n "ad")]
  => (insert! (->Hit ?k)))

;; ROW 4 — collection verb. tags(k) has length (k mod 4); count > 1 => 100 of 200.
(defrule collection
  [Req (= ?k k) (= ?c client) (= ?n name) (= ?t tags) (= ?l limit)]
  [:test (> (count ?t) 1)]
  => (insert! (->Hit ?k)))

;; ROW 5 — a user-defined pure fn, not an inline expression => 84 of 200.
(defrule userfn
  [Req (= ?k k) (= ?c client) (= ?n name) (= ?t tags) (= ?l limit)]
  [:test (big? ?k)]
  => (insert! (->Hit ?k)))

;; ROW 6 — CROSS-VARIABLE comparison: two BOUND VARS, not a var against a constant.
;; limit(i) = (i mod 7) * 20, so the threshold varies per fact. i > 20*(i mod 7) => 139 of 200.
(defrule cross-var
  [Req (= ?k k) (= ?c client) (= ?n name) (= ?t tags) (= ?l limit)]
  [:test (> ?k ?l)]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [Hit (= ?k k)])

;; THE ROW TABLE — mirrors where-shapes.wat's `build-rules` cond. The display name lives here rather
;; than being read off the production, so the two sides print the same token for the same shape.
(def rows
  [[1 "arith"      arith]
   [2 "accessor"   accessor]
   [3 "string"     string-shape]
   [4 "collection" collection]
   [5 "userfn"     userfn]
   [6 "cross-var"  cross-var]])

;; seed-req i — the SAME formulas as wsh::seed, computed independently rather than kept as a
;; hand-synced table:
;;   rep(i)   = (i mod 5) - 2
;;   name(i)  = "ad"+i if i mod 3 == 0 else "zz"+i
;;   tags(i)  = a vector of length (i mod 4), contents [0, len)
;;   limit(i) = (i mod 7) * 20
(defn seed-req [i]
  (let [rep      (- (- i (* (quot i 5) 5)) 2)
        is-ad    (= 0 (- i (* (quot i 3) 3)))
        nm       (str (if is-ad "ad" "zz") i)
        tags-len (- i (* (quot i 4) 4))
        lim      (* (- i (* (quot i 7) 7)) 20)]
    (->Req i (->Client rep) nm (vec (range tags-len)) lim)))

(def seeds (mapv seed-req (range items)))

(defn run-row [[row nm rule]]
  (let [session (apply insert (mk-session [rule hit-q] :cache false) seeds)
        codes   (sort (map :?k (query (fire-rules session) hit-q)))]
    ;; The rendering mirrors the wat side's `render-ints` fold EXACTLY — one leading space per
    ;; element, so the empty case is "-> " with nothing after it on BOTH sides. `str/join` would
    ;; differ there, and a format that only agrees on non-empty input is a format that hides the
    ;; one case most likely to be a bug.
    (str "row " row " " nm " n=" (count codes) " ->"
         (apply str (map #(str " " %) codes)))))

;; `prn`, not `println`: the wat side reaches stdout through :wat::kernel::println, which EDN-encodes
;; — so its String arrives quoted. `prn` quotes too, which is what makes the two outputs
;; byte-identical and `diff` the entire verdict.
(defn -main [& _] (doseq [r rows] (prn (run-row r))))

(-main)
