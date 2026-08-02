;; wat-scripts/perf/grid/where-boolean.clj — THE `where`-CLAUSE EXPRESSIVITY CORPUS,
;; BOOLEAN-COMPOSITION family, Clara side.
;;
;; The twin of where-boolean.wat. Same fact stream, same predicates, same output format:
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-boolean.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-boolean.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs
;;
;; ── FAITHFULNESS, NOT IDIOM ───────────────────────────────────────────────────────────────────
;;
;; `and` / `or` / `not` are Clojure's own special forms here — genuinely the SAME vocabulary as
;; wat's, and both are documented short-circuiting, left-to-right. That is exactly what row
;; `shortcircuit-and` puts to the test: `(and (not= ?l 0) (> (quot 100 ?l) 20))` would throw
;; `ArithmeticException: Divide by zero` on any fact with `?l = 0` if Clojure's `and` did not
;; short-circuit before evaluating its second operand. A clean `n=120` here is the same kind of
;; proof as on the wat side — a crash-free run through the very facts that could not survive an
;; eager `and`.
;;
;; Divison mirrors wat's truncating `i64::/` via `quot`, exactly as where-shapes.clj does.
;;
;; ── WHY EVERY ROW GETS ITS OWN SESSION ────────────────────────────────────────────────────────
;;
;; `mk-session` is called with an EXPLICIT PRODUCTION LIST — never the namespace symbol — so all
;; fifteen rules do not collapse into one session and union their derived sets. See
;; where-shapes.clj's header for the full rationale; identical here.

(ns where-boolean
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(def items 210)                                    ;; 2*3*5*7 — CRT-clean, both sides

(defrecord Req [k a b c d l])
(defrecord Hit [k])

;; row 14's user-defined pure fn, mirroring wsb::edge? EXACTLY: k < 30 or k >= 180.
(defn edge? [k] (or (< k 30) (>= k 180)))

;; THE SHARED LEADING CONDITION — every row binds all six fields off one [Req ...] pattern.
;; (Written out per rule because Clara's defrule takes the pattern literally.)

;; ROW 1 — and/2. k mod 2==0 and k mod 3==0 => k mod 6==0 => 35/210.
(defrule and2
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (and ?a ?b)]
  => (insert! (->Hit ?k)))

;; ROW 2 — or/2. |a|+|b|-|a&b| = 105+70-35 => 140/210.
(defrule or2
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (or ?a ?b)]
  => (insert! (->Hit ?k)))

;; ROW 3 — not/1. 210 - 42 => 168/210.
(defrule not1
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (not ?c)]
  => (insert! (->Hit ?k)))

;; ROW 4 — and/3. k mod 30==0 => 7/210.
(defrule and3
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (and ?a ?b ?c)]
  => (insert! (->Hit ?k)))

;; ROW 5 — or/3. inclusion-exclusion => 154/210.
(defrule or3
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (or ?a ?b ?c)]
  => (insert! (->Hit ?k)))

;; ROW 6 — and/4, the full conjunction. k mod 210==0 => only k=0 => 1/210.
(defrule and4
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (and ?a ?b ?c ?d)]
  => (insert! (->Hit ?k)))

;; ROW 7 — NESTED, two levels: (and (or a b) (not c)) => 112/210.
(defrule nest-and-or-not
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (and (or ?a ?b) (not ?c))]
  => (insert! (->Hit ?k)))

;; ROW 8 — NESTED, two levels: (or (and a b) (and c d)) => 40/210.
(defrule nest-or-and-and
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (or (and ?a ?b) (and ?c ?d))]
  => (insert! (->Hit ?k)))

;; ROW 9 — THREE LEVELS DEEP: (and (or (and a b) c) (not (and c d))) => 64/210.
(defrule nest3
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (and (or (and ?a ?b) ?c) (not (and ?c ?d)))]
  => (insert! (->Hit ?k)))

;; ROW 10 — DE MORGAN PAIR #1a: ¬(a∧b) => 175/210.
(defrule demorgan-nand-a
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (not (and ?a ?b))]
  => (insert! (->Hit ?k)))

;; ROW 11 — DE MORGAN PAIR #1b: (¬a)∨(¬b) — MUST derive the same set as row 10.
(defrule demorgan-nand-b
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (or (not ?a) (not ?b))]
  => (insert! (->Hit ?k)))

;; ROW 12 — DE MORGAN PAIR #2a: ¬(a∨b) => 70/210.
(defrule demorgan-nor-a
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (not (or ?a ?b))]
  => (insert! (->Hit ?k)))

;; ROW 13 — DE MORGAN PAIR #2b: (¬a)∧(¬b) — MUST derive the same set as row 12.
(defrule demorgan-nor-b
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (and (not ?a) (not ?b))]
  => (insert! (->Hit ?k)))

;; ROW 14 — a boolean-valued user fn composed with inline boolean operators at the call site.
;; edge?(k) and not c => 48/210.
(defrule userfn
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (and (edge? ?k) (not ?c))]
  => (insert! (->Hit ?k)))

;; ROW 15 — SHORT-CIRCUIT-SENSITIVE. l != 0 and (100/l) > 20 => 120/210, and must NOT throw on the
;; 30 facts with l = 0 (see header note).
(defrule shortcircuit-and
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?l l)]
  [:test (and (not= ?l 0) (> (quot 100 ?l) 20))]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [Hit (= ?k k)])

;; THE ROW TABLE — mirrors where-boolean.wat's `build-rules` cond.
(def rows
  [[1  "and2"             and2]
   [2  "or2"              or2]
   [3  "not1"             not1]
   [4  "and3"             and3]
   [5  "or3"              or3]
   [6  "and4"             and4]
   [7  "nest-and-or-not"  nest-and-or-not]
   [8  "nest-or-and-and"  nest-or-and-and]
   [9  "nest3"            nest3]
   [10 "demorgan-nand-a"  demorgan-nand-a]
   [11 "demorgan-nand-b"  demorgan-nand-b]
   [12 "demorgan-nor-a"   demorgan-nor-a]
   [13 "demorgan-nor-b"   demorgan-nor-b]
   [14 "userfn"           userfn]
   [15 "shortcircuit-and" shortcircuit-and]])

;; seed-req i — the SAME formulas as wsb::seed, computed independently rather than kept as a
;; hand-synced table:
;;   a(i) = i mod 2 == 0
;;   b(i) = i mod 3 == 0
;;   c(i) = i mod 5 == 0
;;   d(i) = i mod 7 == 0
;;   l(i) = i mod 7
(defn seed-req [i]
  (let [m7 (- i (* (quot i 7) 7))
        a  (= 0 (- i (* (quot i 2) 2)))
        b  (= 0 (- i (* (quot i 3) 3)))
        c  (= 0 (- i (* (quot i 5) 5)))
        d  (= 0 m7)]
    (->Req i a b c d m7)))

(def seeds (mapv seed-req (range items)))

(defn run-row [[row nm rule]]
  (let [session (apply insert (mk-session [rule hit-q] :cache false) seeds)
        codes   (sort (map :?k (query (fire-rules session) hit-q)))]
    ;; Mirrors the wat side's `render-ints` fold EXACTLY — one leading space per element.
    (str "row " row " " nm " n=" (count codes) " ->"
         (apply str (map #(str " " %) codes)))))

;; `prn`, not `println`: matches wat's :wat::kernel::println EDN-quoting of Strings.
(defn -main [& _] (doseq [r rows] (prn (run-row r))))

(-main)
