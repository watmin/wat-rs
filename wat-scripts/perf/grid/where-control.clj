;; wat-scripts/perf/grid/where-control.clj — THE `where`-CLAUSE EXPRESSIVITY CORPUS, Clara side.
;;
;; The twin of where-control.wat (read its header FIRST — it carries the whole STOP-1 story: `cond`
;; is rejected inside a `where` because it is a MACRO that never survives the quasiquote-then-later-
;; eval pipeline, and `Option/expect`/`Result/expect`/raw `Some`/`None`/`Ok`/`Err` CONSTRUCTION are
;; all rejected by the purity fence — none of that has a Clara side to mirror, because Clara's
;; `:test` is arbitrary eval'd Clojure with no purity fence at all. This file only carries the 9
;; rows that DID compile on the wat side).
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-control.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-control.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs
;;
;; ── THE Option REPRESENTATION NOTE (rule 4 — mirror, don't idiomatise) ──────────────────────────
;;
;; wat's `o` field is `(Option :- [i64])` (`Some(i)` / `None`), constructed OUTSIDE any `where` at seed
;; time (row 7 only ever MATCHES a bound `?o`, never constructs one — constructing an Option INSIDE
;; a where-reachable path is itself a STOP-1, see the .wat header). Clojure has no built-in tagged
;; Option, so this side represents it the plain idiomatic way — `nil` for `None`, the bare `i64` for
;; `Some(i)` — and mirrors wat's `(match ?o ((Some v) body) (None false))` as
;; `(if (some? ?o) body false)`. That is a representation choice forced by the language gap, not an
;; idiom swap of the CONSTRAINT: both sides ask exactly "is o present, and if so does its value pass
;; the test," nothing more.
;;
;; ── THE match-ON-i64 NOTE (row 6) ────────────────────────────────────────────────────────────────
;;
;; wat's `(match ?a (0 …) (1 …) (2 …) (3 …))` dispatches on literal equality — Clojure's `case` is
;; the direct analogue (a literal-value dispatch, not a structural/guard match), so it mirrors the
;; OPERATION rather than swapping in an idiom.

(ns where-control
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(def items 180)                                    ;; the stream size, both sides

(defrecord Req [k a b n o])
(defrecord Hit [k])

;; row 5's pure fn, mirroring :wsc::bump EXACTLY.
(defn bump [x] (+ x 1))

;; THE SHARED LEADING CONDITION — every row binds all five fields off one [Req ...] pattern.

;; ROW 1 — `if` returning a bool, as the WHOLE predicate. 90/180 (see .wat header for the derivation).
(defrule if-whole
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?n n) (= ?o o)]
  [:test (if ?n (> ?a 1) (< ?a 2))]
  => (insert! (->Hit ?k)))

;; ROW 2 — `if` NESTED inside a comparison: the `if` returns an i64, then compared. 70/180.
(defrule if-nested-cmp
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?n n) (= ?o o)]
  [:test (> (if ?n ?a ?b) 4)]
  => (insert! (->Hit ?k)))

;; ROW 3 — CHAINED `if` as the WHOLE predicate (the `cond`-shaped logic, respelled — see STOP-1 in
;; the .wat header). 90/180.
(defrule if-chain-whole
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?n n) (= ?o o)]
  [:test (if (= ?a 0) true (if (= ?a 1) false (if (= ?a 2) true false)))]
  => (insert! (->Hit ?k)))

;; ROW 4 — `let` binding a LOCAL, used TWICE. 110/180.
(defrule let-twice
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?n n) (= ?o o)]
  [:test (let [s (+ ?a ?b)] (and (> s 4) (< s 12)))]
  => (insert! (->Hit ?k)))

;; ROW 5 — a `let` whose bound value is a CALL to a pure fn, used in two places
;; (common-subexpression shape). 135/180.
(defrule let-call-cse
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?n n) (= ?o o)]
  [:test (let [c (bump ?a)] (and (> c 1) (< c 5)))]
  => (insert! (->Hit ?k)))

;; ROW 6 — literal-dispatch match (`case` mirrors wat's `match` on i64 literals — see header note).
;; 90/180.
(defrule match-i64
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?n n) (= ?o o)]
  [:test (case ?a 0 false 1 true 2 false 3 true)]
  => (insert! (->Hit ?k)))

;; ROW 7 — Option handling: `(if (some? ?o) …)` mirrors wat's `(match ?o ((Some v) …) (None false))`
;; — see the Option-representation note above. 60/180.
(defrule option-match
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?n n) (= ?o o)]
  [:test (if (some? ?o) (> ?o 90) false)]
  => (insert! (->Hit ?k)))

;; ROW 8 — a NESTED `if` inside a `let` inside a boolean composition — the DEEP control shape.
;; 15/180.
(defrule deep-nest
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?n n) (= ?o o)]
  [:test (let [s (+ ?a ?b)] (and ?n (if (> s 6) true (< s 3))))]
  => (insert! (->Hit ?k)))

;; ROW 9 — the `cond`-shaped branch-with-a-`let`-arm, respelled with `if` (STOP-1, see header).
;; 50/180.
(defrule if-let-arm
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?n n) (= ?o o)]
  [:test (if ?n (let [s (+ ?a ?b)] (> s 8)) (< ?b 3))]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [Hit (= ?k k)])

;; THE ROW TABLE — mirrors where-control.wat's `build-rules` cond.
(def rows
  [[1 "if-whole"       if-whole]
   [2 "if-nested-cmp"  if-nested-cmp]
   [3 "if-chain-whole" if-chain-whole]
   [4 "let-twice"      let-twice]
   [5 "let-call-cse"   let-call-cse]
   [6 "match-i64"      match-i64]
   [7 "option-match"   option-match]
   [8 "deep-nest"      deep-nest]
   [9 "if-let-arm"     if-let-arm]])

;; seed-req i — the SAME formulas as :wsc::seed, computed independently:
;;   a(i) = i mod 4
;;   b(i) = i mod 9
;;   n(i) = i mod 6 == 0
;;   o(i) = nil if i mod 3 == 0 else i               (None / Some i)
(defn seed-req [i]
  (let [a        (- i (* (quot i 4) 4))
        b        (- i (* (quot i 9) 9))
        n        (= 0 (- i (* (quot i 6) 6)))
        is-mult3 (= 0 (- i (* (quot i 3) 3)))
        o        (if is-mult3 nil i)]
    (->Req i a b n o)))

(def seeds (mapv seed-req (range items)))

(defn run-row [[row nm rule]]
  (let [session (apply insert (mk-session [rule hit-q] :cache false) seeds)
        codes   (sort (map :?k (query (fire-rules session) hit-q)))]
    (str "row " row " " nm " n=" (count codes) " ->"
         (apply str (map #(str " " %) codes)))))

;; `prn`, not `println` — see where-shapes.clj's identical note: the wat side EDN-encodes its
;; String through :wat::kernel::println, so `prn` is what makes the two outputs byte-identical.
(defn -main [& _] (doseq [r rows] (prn (run-row r))))

(-main)
