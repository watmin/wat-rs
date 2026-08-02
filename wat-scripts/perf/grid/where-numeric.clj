;; wat-scripts/perf/grid/where-numeric.clj — THE `where`-CLAUSE EXPRESSIVITY CORPUS, NUMERIC TOWER
;; family, Clara side. Twin of where-numeric.wat — read its header first for the full rationale
;; (why row 11 is a deliberate crash, why a second raising row could not also be added).
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-numeric.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-numeric.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs
;;
;; ── FAITHFULNESS, NOT IDIOM ───────────────────────────────────────────────────────────────────
;;
;; `quot`/`rem`/`mod` are used AS Clojure's own builtins, not spelled out by hand — this is the
;; direct mirror, not an idiomisation: tests/clj_expr_oracle/{corpus.txt,golden.txt} already PINS
;; wat's `i64::quot`/`i64::rem`/`i64::mod` bit-identical to Clojure's `quot`/`rem`/`mod` across the
;; full sign matrix, so calling Clojure's own operators here tests exactly the same claim rows 1-3
;; and 5 test on the wat side, rather than re-deriving it by hand a second time (which is how a
;; hand-kept mirror rots). `i64::/` is the one exception: it TRUNCATES like `quot`, so row 4 and
;; row 11 use `quot`, never `/` (which would silently promote two ints to a RATIO).
;;
;; ROW 11 is expected to crash this process too (division by a bound var that is zero for some
;; facts) — verified standalone: `(quot ?a ?z)` with ?z=0 throws `ArithmeticException: Divide by
;; zero` inside `fire-rules`, identically to the wat side's `DivisionByZero`. Placed last so rows
;; 1-10 print first; `-main` does not catch it, by design — see where-numeric.wat's header.

(ns where-numeric
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(def items 200)                                    ;; the stream size, both sides

(defrecord Num [k a z x y])
(defrecord Hit [k])

;; THE SHARED LEADING CONDITION — every row binds all five fields off one [Num ...] pattern.

;; ROW 1 — quot, negative dividend. quot(a,7) < 0 => 94 of 200.
(defrule quot-neg
  [Num (= ?k k) (= ?a a) (= ?z z) (= ?x x) (= ?y y)]
  [:test (< (quot ?a 7) 0)]
  => (insert! (->Hit ?k)))

;; ROW 2 — rem, negative dividend (sign of the DIVIDEND). rem(a,7) < 0 => 86 of 200.
(defrule rem-neg
  [Num (= ?k k) (= ?a a) (= ?z z) (= ?x x) (= ?y y)]
  [:test (< (rem ?a 7) 0)]
  => (insert! (->Hit ?k)))

;; ROW 3 — mod, NEGATIVE DIVISOR (floored, sign of the DIVISOR). mod(a,-7) < -3 => 85 of 200.
(defrule mod-negdiv
  [Num (= ?k k) (= ?a a) (= ?z z) (= ?x x) (= ?y y)]
  [:test (< (mod ?a -7) -3)]
  => (insert! (->Hit ?k)))

;; ROW 4 — truncating division (`quot` mirrors wat's `i64::/`), NEGATIVE DIVISOR.
;; (a quot -3) > 0 => 98 of 200.
(defrule div-negdiv
  [Num (= ?k k) (= ?a a) (= ?z z) (= ?x x) (= ?y y)]
  [:test (> (quot ?a -3) 0)]
  => (insert! (->Hit ?k)))

;; ROW 5 — rem/mod DIVERGENCE as a predicate. rem(a,6) != mod(a,6) => 84 of 200.
(defrule rem-mod-diverge
  [Num (= ?k k) (= ?a a) (= ?z z) (= ?x x) (= ?y y)]
  [:test (not= (rem ?a 6) (mod ?a 6))]
  => (insert! (->Hit ?k)))

;; ROW 6 — comparison CHAIN: range (>=, <=) ANDed with an exclusion (not= on a mod).
;; -50<=a<=50 AND a mod 3 != 0 => 68 of 200.
(defrule chain
  [Num (= ?k k) (= ?a a) (= ?z z) (= ?x x) (= ?y y)]
  [:test (and (>= ?a -50) (and (<= ?a 50) (not= (mod ?a 3) 0)))]
  => (insert! (->Hit ?k)))

;; ROW 7 — the numeric tower's implicit-promotion boundary: `<` freely mixes an int bound var
;; against a float literal, mirroring wat's generic `:wat::core::<` exactly (no explicit
;; conversion on EITHER side, unlike row 8). a < 0.5 => 101 of 200.
(defrule gencmp
  [Num (= ?k k) (= ?a a) (= ?z z) (= ?x x) (= ?y y)]
  [:test (< ?a 0.5)]
  => (insert! (->Hit ?k)))

;; ROW 8 — f64-vs-i64 mixing via an EXPLICIT conversion (`double`, mirroring wat's `i64::to-f64`).
;; y > (double a) => 112 of 200.
(defrule fmix
  [Num (= ?k k) (= ?a a) (= ?z z) (= ?x x) (= ?y y)]
  [:test (> ?y (double ?a))]
  => (insert! (->Hit ?k)))

;; ROW 9 — f64 arithmetic composed with an f64 comparison. x*x > 100.0 => 119 of 200.
(defrule fsq
  [Num (= ?k k) (= ?a a) (= ?z z) (= ?x x) (= ?y y)]
  [:test (> (* ?x ?x) 100.0)]
  => (insert! (->Hit ?k)))

;; ROW 10 — generic `=` ANDed with a range test on `k` (not `a`). k mod 4 == 0 AND k >= 101 =>
;; 24 of 200.
(defrule mix-and
  [Num (= ?k k) (= ?a a) (= ?z z) (= ?x x) (= ?y y)]
  [:test (and (= (mod ?k 4) 0) (>= ?k 101))]
  => (insert! (->Hit ?k)))

;; ROW 11 — DIVISION BY A BOUND VAR THAT IS ZERO FOR SOME FACTS. `z` is zero for 40 of 200 facts
;; (i mod 5 == 2), first at i=2. `quot` mirrors wat's `i64::/`. This raises
;; `ArithmeticException: Divide by zero` and aborts `fire-rules` entirely — see the file header.
(defrule div-by-zero
  [Num (= ?k k) (= ?a a) (= ?z z) (= ?x x) (= ?y y)]
  [:test (> (quot ?a ?z) 1)]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [Hit (= ?k k)])

;; THE ROW TABLE — mirrors where-numeric.wat's `build-rules` cond.
(def rows
  [[1  "quot-neg"        quot-neg]
   [2  "rem-neg"         rem-neg]
   [3  "mod-negdiv"      mod-negdiv]
   [4  "div-negdiv"      div-negdiv]
   [5  "rem-mod-diverge" rem-mod-diverge]
   [6  "chain"           chain]
   [7  "gencmp"          gencmp]
   [8  "fmix"            fmix]
   [9  "fsq"             fsq]
   [10 "mix-and"         mix-and]])
;; Row 11 (div-by-zero) is RETIRED from the dispatch — see the .wat header. Its `defrule` is kept
;; above, unreferenced, as the executable record of the form; it is simply never run, because a
;; raising predicate aborts the whole program and would leave this pair permanently RED.

;; seed-req i — the SAME formulas as wnm::seed, computed independently:
;;   a(i) = i - 100
;;   z(i) = (i mod 5) - 2
;;   x(i) = i*0.25 - 25.0
;;   y(i) = i*0.1
(defn seed-req [i]
  (let [a (- i 100)
        z (- (mod i 5) 2)
        x (- (* i 0.25) 25.0)
        y (* i 0.1)]
    (->Num i a z x y)))

(def seeds (mapv seed-req (range items)))

(defn run-row [[row nm rule]]
  (let [session (apply insert (mk-session [rule hit-q] :cache false) seeds)
        codes   (sort (map :?k (query (fire-rules session) hit-q)))]
    ;; Mirrors where-shapes.clj's run-row EXACTLY — one leading space per element.
    (str "row " row " " nm " n=" (count codes) " ->"
         (apply str (map #(str " " %) codes)))))

;; `prn`, not `println` — matches wat's EDN-quoted String output; see where-shapes.clj's note.
;; Row 11 is NOT caught: `-main` throws through exactly like the wat side, by design.
(defn -main [& _] (doseq [r rows] (prn (run-row r))))

(-main)
