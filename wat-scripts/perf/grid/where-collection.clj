;; wat-scripts/perf/grid/where-collection.clj — THE `where`-CLAUSE EXPRESSIVITY CORPUS,
;; COLLECTIONS family, Clara side.
;;
;; The twin of where-collection.wat (read its header first — it grounds the surface, records the
;; two STOP-1 rejections, the two absent verbs, and the two "compiles but raises" fence gaps found
;; before a single row here was written). Same fact stream, same predicates, same output format:
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-collection.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-collection.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs
;;
;; ── FAITHFULNESS, NOT IDIOM ───────────────────────────────────────────────────────────────────
;;
;; `mod` is used directly on both sides — unlike where-shapes.wat's `/`, wat's `i64::mod` and
;; Clojure's `mod` agree exactly for the non-negative operands every formula here produces, so no
;; subtraction-expansion is needed to dodge a semantic gap (there isn't one here).
;;
;; `PersistentVector/get` (wat) returns `(Option :- [T])` (`None` on out-of-range); the direct Clojure
;; mirror is `(get v idx)`, which returns `nil` out-of-range — every element here is a non-nil i64,
;; so "present" and "absent" map onto `some?`/`nil?` exactly the way `Some`/`None` do on the wat
;; side. `contains?` (element membership, NOT Clojure's key-membership `contains?`) mirrors to
;; `(some #(= % needle) v)`, wrapped in `boolean` so a `:test` sees a genuine `true`/`false` rather
;; than a truthy element value or `nil`.
;;
;; ── GROWING THE CORPUS ────────────────────────────────────────────────────────────────────────
;;   1. a `defrule` below (copy its neighbour; only the `:test` differs)
;;   2. one entry in `rows`
;;   3. the mirrored arm in where-collection.wat + bump its `row-count`

(ns where-collection
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(def items 200)                                    ;; the stream size, both sides

(defrecord Item [k tags bound grid])
(defrecord Hit  [k])

;; row 7's user-defined pure fn over a WHOLE bound collection, mirroring wc::heavy? EXACTLY:
;; length(v) > 2 AND v contains 7.
(defn heavy? [v] (and (> (count v) 2) (boolean (some #(= % 7) v))))

;; THE SHARED LEADING CONDITION — every row binds all four fields off one [Item ...] pattern, so a
;; shape can only ever perturb its own trailing :test, never the token stream every row shares.

;; ROW 1 — LENGTH vs a BOUND i64 VAR. length(tags) > bound => 48/200.
(defrule length-bound
  [Item (= ?k k) (= ?t tags) (= ?b bound) (= ?g grid)]
  [:test (> (count ?t) ?b)]
  => (insert! (->Hit ?k)))

;; ROW 2 — ELEMENT ACCESS at a CONSTANT index 2, TOTAL on a short/empty vector via `get` -> nil.
;; get(tags,2) present and >5; absent -> false. => 54/200.
(defrule get-const
  [Item (= ?k k) (= ?t tags) (= ?b bound) (= ?g grid)]
  [:test (let [x (get ?t 2)] (if (nil? x) false (> x 5)))]
  => (insert! (->Hit ?k)))

;; ROW 3 — MEMBERSHIP. tags contains 6 => 38/200.
(defrule contains
  [Item (= ?k k) (= ?t tags) (= ?b bound) (= ?g grid)]
  [:test (boolean (some #(= % 6) ?t))]
  => (insert! (->Hit ?k)))

;; ROW 4 — NESTED COLLECTION, two levels in. First inner vector's length > 1; absent (grid empty)
;; -> false => 66/200.
(defrule nested
  [Item (= ?k k) (= ?t tags) (= ?b bound) (= ?g grid)]
  [:test (let [inner (get ?g 0)] (if (nil? inner) false (> (count inner) 1)))]
  => (insert! (->Hit ?k)))

;; ROW 5 — HIGHER-ORDER + CROSS-VAR. sum(tags) > bound, via `reduce` closing over `+`. => 150/200.
(defrule fold-sum-bound
  [Item (= ?k k) (= ?t tags) (= ?b bound) (= ?g grid)]
  [:test (> (reduce + 0 ?t) ?b)]
  => (insert! (->Hit ?k)))

;; ROW 6 — ELEMENT ACCESS at a DYNAMIC (bound-var) index `?b`. get(tags,bound) present and >3;
;; absent -> false => 34/200.
(defrule get-dynamic
  [Item (= ?k k) (= ?t tags) (= ?b bound) (= ?g grid)]
  [:test (let [x (get ?t ?b)] (if (nil? x) false (> x 3)))]
  => (insert! (->Hit ?k)))

;; ROW 7 — a PURE FN over the WHOLE bound collection (`heavy?` above) => 30/200.
(defrule userfn
  [Item (= ?k k) (= ?t tags) (= ?b bound) (= ?g grid)]
  [:test (heavy? ?t)]
  => (insert! (->Hit ?k)))

;; ROW 8 — HIGHER-ORDER, `every?` EMULATED via `reduce`/`and`, seed `true` (vacuous truth on the
;; empty-tags facts, matching wat's foldl). => 57/200.
(defrule fold-every-even
  [Item (= ?k k) (= ?t tags) (= ?b bound) (= ?g grid)]
  [:test (reduce (fn [acc x] (and acc (= 0 (mod x 2)))) true ?t)]
  => (insert! (->Hit ?k)))

;; ROW 9 — HIGHER-ORDER, `some?` EMULATED via `reduce`/`or`, seed `false` (vacuous falsity on the
;; empty-tags facts). => 38/200.
(defrule fold-some-zero
  [Item (= ?k k) (= ?t tags) (= ?b bound) (= ?g grid)]
  [:test (reduce (fn [acc x] (or acc (= x 0))) false ?t)]
  => (insert! (->Hit ?k)))

;; ROW 10 — NESTED + HIGHER-ORDER + CROSS-VAR composed: first inner vector (Option-safe), fold its
;; elements, compare to bound. => 76/200.
(defrule nested-fold-bound
  [Item (= ?k k) (= ?t tags) (= ?b bound) (= ?g grid)]
  [:test (let [inner (get ?g 0)] (if (nil? inner) false (> (reduce + 0 inner) ?b)))]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [Hit (= ?k k)])

;; THE ROW TABLE — mirrors where-collection.wat's `build-rules` cond.
(def rows
  [[1  "length-bound"      length-bound]
   [2  "get-const"         get-const]
   [3  "contains"          contains]
   [4  "nested"            nested]
   [5  "fold-sum-bound"    fold-sum-bound]
   [6  "get-dynamic"       get-dynamic]
   [7  "userfn"            userfn]
   [8  "fold-every-even"   fold-every-even]
   [9  "fold-some-zero"    fold-some-zero]
   [10 "nested-fold-bound" nested-fold-bound]])

;; seed-item i — the SAME formulas as wc::build-tags / wc::build-grid, computed independently
;; rather than kept as a hand-synced table:
;;   tags(i)  = a vector of length (i mod 6), element j = (i + 3j) mod 13
;;   bound(i) = i mod 8
;;   grid(i)  = a vector of (i mod 3) inner vectors; inner a has length (i+a) mod 4,
;;              element b = (i+a+b) mod 9
(defn build-tags [i]
  (let [len (mod i 6)]
    (mapv (fn [j] (mod (+ i (* j 3)) 13)) (range len))))

(defn build-inner [i a]
  (let [base (+ i a)
        len  (mod base 4)]
    (mapv (fn [b] (mod (+ base b) 9)) (range len))))

(defn build-grid [i]
  (let [outer-len (mod i 3)]
    (mapv (fn [a] (build-inner i a)) (range outer-len))))

(defn seed-item [i]
  (->Item i (build-tags i) (mod i 8) (build-grid i)))

(def seeds (mapv seed-item (range items)))

(defn run-row [[row nm rule]]
  (let [session (apply insert (mk-session [rule hit-q] :cache false) seeds)
        codes   (sort (map :?k (query (fire-rules session) hit-q)))]
    ;; mirrors the wat side's `render-ints` fold EXACTLY — one leading space per element, so the
    ;; empty case is "-> " with nothing after it on BOTH sides (see where-shapes.clj's identical note).
    (str "row " row " " nm " n=" (count codes) " ->"
         (apply str (map #(str " " %) codes)))))

;; `prn`, not `println`: see where-shapes.clj's identical note — the wat side EDN-encodes its
;; String output, so `prn` (which also quotes) is what makes the two outputs byte-identical.
(defn -main [& _] (doseq [r rows] (prn (run-row r))))

(-main)
