;; wat-scripts/perf/grid/where-multivar.clj — MULTI-VARIABLE PREDICATES, Clara side.
;;
;; The twin of where-multivar.wat. Same fact stream, same predicates, same output format:
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-multivar.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-multivar.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs
;;
;; See where-multivar.wat's header for the family's rationale (three/four/five-var predicates,
;; transitive chains, arithmetic across vars, a var reused several times, multi-arg pure fns, a
;; type-crossing comparison, and an unused-binding row) and the fact-stream field formulas.
;;
;; ── FAITHFULNESS, NOT IDIOM ───────────────────────────────────────────────────────────────────
;;
;; Every predicate MIRRORS the wat operation: `and` <-> `:wat::core::and`, `mod` <-> `i64::mod`
;; (identical for ALL operands, not just the non-negative ones here — wat's i64::mod is FLOORED,
;; sign of the DIVISOR, deliberately clj-faithful and validated 16/16 vs clojure 1.12.4), and
;; the same left-to-right arithmetic grouping — nothing collapsed to a Clojure-only idiom.

(ns where-multivar
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(def items 200)

(defrecord Req [k a b c d e s])
(defrecord Hit [k])

;; combo?(a,b,c,d) := (a*c) > (b*d), mirroring wmv::combo? exactly.
(defn combo? [a b c d] (> (* a c) (* b d)))

;; pent?(a,b,c,d,e) := (a+b+c) mod (d+e+1) == 0, mirroring wmv::pent? exactly.
(defn pent? [a b c d e] (= (mod (+ a (+ b c)) (+ d (+ e 1))) 0))

;; THE SHARED LEADING CONDITION — every row binds all seven fields off one [Req ...] pattern.

;; ROW 1 — THREE bound vars. a+b > c+10 => 64 of 200.
(defrule three-var
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?e e) (= ?s s)]
  [:test (> (+ ?a ?b) (+ ?c 10))]
  => (insert! (->Hit ?k)))

;; ROW 2 — FOUR bound vars. (a+b) mod (c+1) == d => 36 of 200.
(defrule four-var
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?e e) (= ?s s)]
  [:test (= (mod (+ ?a ?b) (+ ?c 1)) ?d)]
  => (insert! (->Hit ?k)))

;; ROW 3 — FIVE bound vars, combined with `and`. (a mod 2==0) and (b>c) and (d>e) => 40 of 200.
(defrule five-var
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?e e) (= ?s s)]
  [:test (and (= 0 (mod ?a 2)) (> ?b ?c) (> ?d ?e))]
  => (insert! (->Hit ?k)))

;; ROW 4 — TRANSITIVE CHAIN. d < c AND c < b => 66 of 200.
(defrule chain
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?e e) (= ?s s)]
  [:test (and (< ?d ?c) (< ?c ?b))]
  => (insert! (->Hit ?k)))

;; ROW 5 — arithmetic ACROSS vars: (?a + ?b) > ?c => 183 of 200.
(defrule sum-vars
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?e e) (= ?s s)]
  [:test (> (+ ?a ?b) ?c)]
  => (insert! (->Hit ?k)))

;; ROW 6 — arithmetic ACROSS vars: (?a * ?b) > (?c + ?d) => 150 of 200.
(defrule prod-vars
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?e e) (= ?s s)]
  [:test (> (* ?a ?b) (+ ?c ?d))]
  => (insert! (->Hit ?k)))

;; ROW 7 — the SAME var used SEVERAL TIMES: (?a * ?a) - ?a > 20 => 90 of 200.
(defrule repeat-var
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?e e) (= ?s s)]
  [:test (> (- (* ?a ?a) ?a) 20)]
  => (insert! (->Hit ?k)))

;; ROW 8 — a pure fn taking FOUR bound vars at once: (combo? ?a ?b ?c ?d) => 95 of 200.
(defrule combo-fn
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?e e) (= ?s s)]
  [:test (combo? ?a ?b ?c ?d)]
  => (insert! (->Hit ?k)))

;; ROW 9 — vars of DIFFERENT TYPES compared through a fn: (count ?s) > ?c => 71 of 200.
;; `count` on a String mirrors wat's `string::length` (both give the character count of a String).
(defrule mixed-type
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?e e) (= ?s s)]
  [:test (> (count ?s) ?c)]
  => (insert! (->Hit ?k)))

;; ROW 10 — binds MANY vars, READS only one: ?a mod 3 == 0 => 73 of 200.
(defrule unused-binds
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?e e) (= ?s s)]
  [:test (= 0 (mod ?a 3))]
  => (insert! (->Hit ?k)))

;; ROW 11 — a pure fn taking FIVE bound vars at once: (pent? ?a ?b ?c ?d ?e) => 73 of 200.
(defrule five-fn
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?e e) (= ?s s)]
  [:test (pent? ?a ?b ?c ?d ?e)]
  => (insert! (->Hit ?k)))

;; ROW 12 — chain AND arithmetic combined: (?b < ?c) AND ((?c + ?d) > (?e * 3)) => 37 of 200.
(defrule chain-arith
  [Req (= ?k k) (= ?a a) (= ?b b) (= ?c c) (= ?d d) (= ?e e) (= ?s s)]
  [:test (and (< ?b ?c) (> (+ ?c ?d) (* ?e 3)))]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [Hit (= ?k k)])

;; THE ROW TABLE — mirrors where-multivar.wat's `build-rules` cond.
(def rows
  [[1  "three-var"     three-var]
   [2  "four-var"      four-var]
   [3  "five-var"      five-var]
   [4  "chain"         chain]
   [5  "sum-vars"      sum-vars]
   [6  "prod-vars"     prod-vars]
   [7  "repeat-var"    repeat-var]
   [8  "combo-fn"      combo-fn]
   [9  "mixed-type"    mixed-type]
   [10 "unused-binds"  unused-binds]
   [11 "five-fn"       five-fn]
   [12 "chain-arith"   chain-arith]])

;; seed-req i — the SAME formulas as wmv::seed, computed independently:
;;   k(i) = i,  a(i) = i mod 11,  b(i) = i mod 13,  c(i) = i mod 7,  d(i) = i mod 5,  e(i) = i mod 3
;;   s(i) = str(i)
(defn seed-req [i]
  (->Req i (mod i 11) (mod i 13) (mod i 7) (mod i 5) (mod i 3) (str i)))

(def seeds (mapv seed-req (range items)))

(defn run-row [[row nm rule]]
  (let [session (apply insert (mk-session [rule hit-q] :cache false) seeds)
        codes   (sort (map :?k (query (fire-rules session) hit-q)))]
    ;; Mirrors where-shapes.clj's run-row rendering EXACTLY — one leading space per element.
    (str "row " row " " nm " n=" (count codes) " ->"
         (apply str (map #(str " " %) codes)))))

;; `prn`, not `println` — see where-shapes.clj's note: the wat side EDN-encodes its output string,
;; so `prn` (which also quotes) is what makes the two outputs byte-identical.
(defn -main [& _] (doseq [r rows] (prn (run-row r))))

(-main)
