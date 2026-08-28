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
;; fourteen rules do not collapse into one session and union their derived sets.
;;
;; ── ROWS 5-8: THE BINDER VECTOR, AND WHY CLARA IS LOAD-BEARING HERE ──────────────────────────
;;
;; Rows 1-4 nest the field reference in a CALL. Rows 5-8 nest it in a `let` BINDER — and on
;; 2026-08-28 that was a second live silent never-match, still open on the day entry F was
;; declared closed, because our rewriter walked `Keyword` and `List` and swept `Vector` into an
;; `other => clone` catch-all.
;;
;; ── ROWS 9-14: THREE FORMS WE HAD DENIED OURSELVES ───────────────────────────────────────────
;;
;; `cond`, `let` and `match` were REFUSED as the head of an inline constraint until 2026-08-28, on
;; the stated ground that they are "polymorphic in their body's type and the inline position has no
;; type check that could demand bool of them". Polymorphic-in-the-body means the type is a FUNCTION
;; of the body, and the body is in the AST — and `cond` was not failing a type test at all: the
;; macro expander descended into `where` bodies ONLY, so an inline `cond` never expanded.
;;
;; Clojure has all three, which is what makes rows 9-14 a comparison rather than a claim: `cond`
;; and `let` directly, and `case` for a match on a boolean (literal dispatch values, so `case` is
;; the honest mirror of wat's `(match <bool> (true …) (false …))`).
;;
;; ⛔ BOTH OUR ENGINES WERE WRONG, AGAIN. `$native` and `$oracle` share that rewriter's shape, so
;; they returned the same wrong answer and every differential gate we own was satisfied. That is
;; the repeat failure of this arc — the two engines do not merely agree, they inherit one mistake.
;; Clara is the only party here that did not, which is the entire reason these rows are written
;; against it rather than as a Rust probe alone. A probe we own cannot arbitrate a defect we own.

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

;; ROW 5 — INLINE, the field reference inside a `let` binder. Our side read n=0 here in BOTH
;; engines; Clara is what makes that a diff instead of a consensus.
(defrule inline-let-gt
  [Req (= ?k k) (> (let [x k] x) 100)]
  => (insert! (->Hit ?k)))

;; ROW 6 — FENCE, the identical predicate via :test. This position always worked on our side.
(defrule fence-let-gt
  [Req (= ?k k)]
  [:test (> (let [x ?k] x) 100)]
  => (insert! (->Hit ?k)))

;; ROW 7 — INLINE, exact equality. n=1 brackets never-match and always-match from both sides.
(defrule inline-let-eq
  [Req (= ?k k) (= (let [x k] x) 100)]
  => (insert! (->Hit ?k)))

;; ROW 8 — FENCE, exact equality.
(defrule fence-let-eq
  [Req (= ?k k)]
  [:test (= (let [x ?k] x) 100)]
  => (insert! (->Hit ?k)))

;; ROW 9 — `cond` as the inline HEAD. Our side refused this outright until 2026-08-28.
(defrule inline-cond
  [Req (= ?k k) (cond (> k 100) true :else false)]
  => (insert! (->Hit ?k)))

;; ROW 10 — FENCE via :test. This position always worked on our side, which is how the inline
;; refusal stayed invisible.
(defrule fence-cond
  [Req (= ?k k)]
  [:test (cond (> ?k 100) true :else false)]
  => (insert! (->Hit ?k)))

;; ROW 11 — `let` as the inline HEAD.
(defrule inline-let-head
  [Req (= ?k k) (let [x k] (> x 100))]
  => (insert! (->Hit ?k)))

;; ROW 12 — FENCE.
(defrule fence-let-head
  [Req (= ?k k)]
  [:test (let [x ?k] (> x 100))]
  => (insert! (->Hit ?k)))

;; ROW 13 — `match` as the inline HEAD. `case` over literal true/false is the honest mirror of
;; wat's `(match <bool> (true …) (false …))`. `= 100` so n=1 brackets it from both sides.
(defrule inline-match
  [Req (= ?k k) (case (= k 100) true true false false)]
  => (insert! (->Hit ?k)))

;; ROW 14 — FENCE.
(defrule fence-match
  [Req (= ?k k)]
  [:test (case (= ?k 100) true true false false)]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [?fact <- Hit])

(def rows
  [[1 "inline-gt"     inline-gt]
   [2 "fence-gt"      fence-gt]
   [3 "inline-eq"     inline-eq]
   [4 "fence-eq"      fence-eq]
   [5 "inline-let-gt" inline-let-gt]
   [6 "fence-let-gt"  fence-let-gt]
   [7 "inline-let-eq" inline-let-eq]
   [8 "fence-let-eq"  fence-let-eq]
   [9  "inline-cond"     inline-cond]
   [10 "fence-cond"      fence-cond]
   [11 "inline-let-head" inline-let-head]
   [12 "fence-let-head"  fence-let-head]
   [13 "inline-match"    inline-match]
   [14 "fence-match"     fence-match]])

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
