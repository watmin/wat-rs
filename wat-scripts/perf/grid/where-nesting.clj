;; wat-scripts/perf/grid/where-nesting.clj — the NESTING-DEPTH family, Clara side.
;; Twin of where-nesting.wat. Same fact stream, same predicates, same output format:
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-nesting.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-nesting.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs
;;
;; See where-nesting.wat's header for the family's rationale and the depth-boundary finding (none
;; found; tried to 20,000 outside this corpus). Per-row sessions (never one shared namespace-wide
;; session) for the same reason as where-shapes.clj: a UNION of derived sets could not name the row
;; that diverged.
;;
;; FAITHFULNESS: `quot`, never `/` (Clojure's `/` on two ints yields a RATIO; wat's `i64::/`
;; truncates). Every mod is spelled out as `(- x (* (quot x n) n))`, mirroring the wat side's
;; expansion rather than collapsing to `mod`.

(ns where-nesting
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(def items 200)

(defrecord Req [k m])
(defrecord Hit [k])

;; depth chain c1..c10 — mirrors :wnst::c1..:wnst::c10 EXACTLY, including the call structure (each
;; cN calls c(N-1), never inlined), so purity/nesting-depth is not the only thing under test — the
;; ARITHMETIC itself must also agree.
(defn c1  [k] (- k (* (quot k 13) 13)))
(defn c2  [k] (+ (c1 k) 3))
(defn c3  [k] (+ (c2 k) 3))
(defn c4  [k] (+ (c3 k) 3))
(defn c5  [k] (+ (c4 k) 3))
(defn c6  [k] (+ (c5 k) 3))
(defn c7  [k] (+ (c6 k) 3))
(defn c8  [k] (+ (c7 k) 3))
(defn c9  [k] (+ (c8 k) 3))
(defn c10 [k] (+ (c9 k) 3))

;; row 7 — two bound vars.
(defn twoarg [a b] (> (+ a b) 113))

;; row 8 — argument-is-a-call: wrap(c2(k)).
(defn wrap [v] (= 0 (- v (* (quot v 4) 4))))

;; row 9 — diamond: f calls g and h, both call hub.
(defn hub [k] (- k (* (quot k 17) 17)))
(defn g   [k] (+ (hub k) 2))
(defn h   [k] (* (hub k) 2))
(defn f   [k] (and (> (g k) 5) (< (h k) 25)))

;; rows 10/11 — score returns i64; is-good calls score and returns bool.
(defn score [k]
  (let [v (* k 3)]
    (- v (* (quot v 11) 11))))
(defn is-good [k]
  (let [sc (score k)]
    (= 0 (- sc (* (quot sc 2) 2)))))

;; THE SHARED LEADING CONDITION — every row binds both fields off one [Req ...] pattern.

;; ROW 1 — depth-2 chain. c2 > 10 -> 75 of 200.
(defrule depth2
  [Req (= ?k k) (= ?m m)]
  [:test (> (c2 ?k) 10)]
  => (insert! (->Hit ?k)))

;; ROW 2 — depth-3 chain. c3 > 15 -> 45 of 200.
(defrule depth3
  [Req (= ?k k) (= ?m m)]
  [:test (> (c3 ?k) 15)]
  => (insert! (->Hit ?k)))

;; ROW 3 — depth-4 chain. c4 > 15 -> 90 of 200.
(defrule depth4
  [Req (= ?k k) (= ?m m)]
  [:test (> (c4 ?k) 15)]
  => (insert! (->Hit ?k)))

;; ROW 4 — depth-5 chain. c5 > 20 -> 60 of 200.
(defrule depth5
  [Req (= ?k k) (= ?m m)]
  [:test (> (c5 ?k) 20)]
  => (insert! (->Hit ?k)))

;; ROW 5 — depth-10 chain, the "keeps going past 5" witness. c10 > 32 -> 105 of 200.
(defrule depth10
  [Req (= ?k k) (= ?m m)]
  [:test (> (c10 ?k) 32)]
  => (insert! (->Hit ?k)))

;; ROW 6 — deep INLINE arithmetic tree, 6 levels, zero fn calls. g6 > 157 -> 94 of 200.
(defrule inline-tree
  [Req (= ?k k) (= ?m m)]
  [:test (> (* (+ (quot (- (* (+ ?k 10) 2) 15) 3) 7) 2) 157)]
  => (insert! (->Hit ?k)))

;; ROW 7 — TWO bound vars. (?k+?m) > 113 -> 108 of 200.
(defrule two-arg
  [Req (= ?k k) (= ?m m)]
  [:test (twoarg ?k ?m)]
  => (insert! (->Hit ?k)))

;; ROW 8 — argument IS a call: (wrap (c2 ?k)) -> 46 of 200.
(defrule arg-is-call
  [Req (= ?k k) (= ?m m)]
  [:test (wrap (c2 ?k))]
  => (insert! (->Hit ?k)))

;; ROW 9 — mutual/chained helpers: f calls g and h, both call hub. -> 108 of 200.
(defrule diamond
  [Req (= ?k k) (= ?m m)]
  [:test (f ?k)]
  => (insert! (->Hit ?k)))

;; ROW 10 — non-bool (i64) return, compared from outside. score > 6 -> 72 of 200.
(defrule int-then-compare
  [Req (= ?k k) (= ?m m)]
  [:test (> (score ?k) 6)]
  => (insert! (->Hit ?k)))

;; ROW 11 — CONTRAST with row 10: bool returned directly, no external comparison. -> 109 of 200.
(defrule bool-direct
  [Req (= ?k k) (= ?m m)]
  [:test (is-good ?k)]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [Hit (= ?k k)])

;; THE ROW TABLE — mirrors where-nesting.wat's `build-rules` cond.
(def rows
  [[1  "depth2"           depth2]
   [2  "depth3"           depth3]
   [3  "depth4"           depth4]
   [4  "depth5"           depth5]
   [5  "depth10"          depth10]
   [6  "inline-tree"      inline-tree]
   [7  "two-arg"          two-arg]
   [8  "arg-is-call"      arg-is-call]
   [9  "diamond"          diamond]
   [10 "int-then-compare" int-then-compare]
   [11 "bool-direct"      bool-direct]])

;; seed-req i — the SAME formulas as :wnst::seed, computed independently:
;;   k(i) = i
;;   m(i) = (7i + 11) mod 40
(defn seed-req [i]
  (let [mraw (+ (* 7 i) 11)
        m    (- mraw (* (quot mraw 40) 40))]
    (->Req i m)))

(def seeds (mapv seed-req (range items)))

(defn run-row [[row nm rule]]
  (let [session (apply insert (mk-session [rule hit-q] :cache false) seeds)
        codes   (sort (map :?k (query (fire-rules session) hit-q)))]
    (str "row " row " " nm " n=" (count codes) " ->"
         (apply str (map #(str " " %) codes)))))

;; `prn`, not `println` — matches where-shapes.clj's rationale exactly (wat's println EDN-encodes
;; the String, so it arrives quoted; `prn` quotes too).
(defn -main [& _] (doseq [r rows] (prn (run-row r))))

(-main)
