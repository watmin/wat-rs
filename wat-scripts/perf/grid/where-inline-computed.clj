;; where-inline-computed.clj — the Clara half of THE POSITION AXIS.
;;
;; ── WHY THIS TWIN MATTERS MORE THAN A NORMAL ONE ─────────────────────────────────────────────
;;
;; The wat half writes each predicate TWICE — once as an inline constraint, once in a `where`
;; fence — because fix-list entry F was a defect that lived in one position and was invisible in
;; the other. But two of OUR rows agreeing proves nothing on its own: both are our compiler, and
;; entry F is precisely a case where our two positions disagreed for the life of the engine while
;; every one of our own gates was satisfied.
;;
;; Clara is the arbiter, and it can arbitrate this because it HAS both positions:
;;   · a predicate inside the bracket — `[Req (> (+ k 2) 100)]` — is Clara's inline constraint
;;   · `[:test …]` is Clara's fence
;;
;; So all four rows below carry ONE semantic answer written four ways, and the byte-diff catches a
;; disagreement no matter which of the four spellings is the odd one out.
;;
;; ⚠ Measured 2026-08-28: the grid's Clara half uses `[:test …]` in 128 rows and an in-pattern
;; predicate in 7, while the wat half contained ZERO inline constraints. The corpus was one-sided
;; on both engines at once, which is exactly how a position-specific defect survives 36 axes.
;;
;; `mk-session` takes an EXPLICIT production list per row — never the namespace symbol — so the
;; four rules do not collapse into one session and union their derived sets.

(ns where-inline-computed
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(def items 210)

(defrecord Req [k])
(defrecord Hit [k])

;; ROW 1 — INLINE, computed operand. `(+ k 2)` inside the pattern: Clara's inline constraint.
(defrule inline-gt
  [Req (= ?k k) (> (+ k 2) 100)]
  => (insert! (->Hit ?k)))

;; ROW 2 — FENCE, the identical predicate via :test.
(defrule fence-gt
  [Req (= ?k k)]
  [:test (> (+ ?k 2) 100)]
  => (insert! (->Hit ?k)))

;; ROW 3 — INLINE, exact equality. n=1 brackets the answer from both sides.
(defrule inline-eq
  [Req (= ?k k) (= (+ k 2) 100)]
  => (insert! (->Hit ?k)))

;; ROW 4 — FENCE, exact equality.
(defrule fence-eq
  [Req (= ?k k)]
  [:test (= (+ ?k 2) 100)]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [?fact <- Hit])

(def rows
  [[1 "inline-gt" inline-gt]
   [2 "fence-gt"  fence-gt]
   [3 "inline-eq" inline-eq]
   [4 "fence-eq"  fence-eq]])

(def seeds (mapv (fn [i] (->Req i)) (range items)))

(defn run-row [[row nm rule]]
  (let [session (apply insert (mk-session [rule hit-q] :cache false) seeds)
        codes   (sort (map #(:k (:?fact %)) (query (fire-rules session) hit-q)))]
    ;; Mirrors the wat side's `render-ints` fold EXACTLY — one leading space per element.
    (str "row " row " " nm " n=" (count codes) " ->"
         (apply str (map #(str " " %) codes)))))

;; `prn`, not `println`: matches wat's :wat::kernel::println EDN-quoting of Strings.
(defn -main [& _] (doseq [r rows] (prn (run-row r))))

(-main)
