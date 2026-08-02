;; wat-scripts/perf/grid/where-string.clj — THE `where`-CLAUSE EXPRESSIVITY CORPUS,
;; STRING-VERB family, Clara side.
;;
;; The twin of where-string.wat. Same fact stream, same predicates, same output format:
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-string.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-string.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs
;;
;; ── FAITHFULNESS, NOT IDIOM ───────────────────────────────────────────────────────────────────
;;
;; `:wat::core::String/contains?` mirrors as `clojure.string/includes?`, NEVER Clojure's own
;; `contains?` — that tests collection/map KEY membership (`(contains? [1 2 3] 0)` is about the
;; index, not the value), a completely different operation. Using the same NAME across languages
;; here would be the fudge; `includes?` is the substring test and is the faithful mirror of what
;; the wat verb actually does.
;;
;; `:wat::core::String/empty?` mirrors as `(zero? (count s))`, NEVER `clojure.string/blank?` —
;; `blank?` is true for whitespace-only strings too (`(blank? "  ")` => true), which
;; `str::is_empty()` on the wat side is not. That would silently change which facts match.
;;
;; `:wat::core::string::length` mirrors as `count` — for the BMP-only content this corpus seeds
;; (Latin ASCII + two hiragana chars, no surrogate pairs), Java's UTF-16-code-unit-based `count`
;; and wat's Unicode-scalar-based `chars().count()` coincide exactly.
;;
;; `:wat::core::string::subs` mirrors as `subs` — both are start-inclusive/end-exclusive and
;; char-indexed (again coinciding with wat's scalar-indexing for BMP-only content), and both RAISE
;; on an out-of-range end, which is exactly what row 11's `and`-guard exists to prevent hitting.
;;
;; Divison mirrors wat's truncating `i64::/` via `quot`, exactly as where-shapes.clj/where-boolean.clj do.
;;
;; ── WHY EVERY ROW GETS ITS OWN SESSION ────────────────────────────────────────────────────────
;;
;; `mk-session` is called with an EXPLICIT PRODUCTION LIST — never the namespace symbol — so all
;; twelve rules do not collapse into one session and union their derived sets. See
;; where-shapes.clj's header for the full rationale; identical here.

(ns where-string
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clojure.string :as str]))

(def items 400)                                    ;; 5*2*8*5 — CRT-clean, both sides

(defrecord Req [k n tag minlen padded])
(defrecord Hit [k])

;; row 9's user-defined pure fn, mirroring wst::feline? EXACTLY: (contains? s "cat") and
;; (length s > 3), via `includes?`/`count` (see header for why not `contains?`/`blank?`).
(defn feline? [s] (and (str/includes? s "cat") (> (count s) 3)))

;; THE SHARED LEADING CONDITION — every row binds all five fields off one [Req ...] pattern.
;; (Written out per rule because Clara's defrule takes the pattern literally.)

;; ROW 1 — String/starts-with?. True only for r=1 ("cat" itself) => 80/400.
(defrule starts-with
  [Req (= ?k k) (= ?n n) (= ?tag tag) (= ?minlen minlen) (= ?padded padded)]
  [:test (str/starts-with? ?n "cat")]
  => (insert! (->Hit ?k)))

;; ROW 2 — String/ends-with?. True for r=1 and r=3, NOT r=2 => 160/400.
(defrule ends-with
  [Req (= ?k k) (= ?n n) (= ?tag tag) (= ?minlen minlen) (= ?padded padded)]
  [:test (str/ends-with? ?n "cat")]
  => (insert! (->Hit ?k)))

;; ROW 3 — String/contains?, via `includes?`. True for r=1, r=2, r=3 => 240/400.
(defrule contains
  [Req (= ?k k) (= ?n n) (= ?tag tag) (= ?minlen minlen) (= ?padded padded)]
  [:test (str/includes? ?n "cat")]
  => (insert! (->Hit ?k)))

;; ROW 4 — String/empty?, via `(zero? (count s))`. True only for r=0 => 80/400.
(defrule empty-str
  [Req (= ?k k) (= ?n n) (= ?tag tag) (= ?minlen minlen) (= ?padded padded)]
  [:test (zero? (count ?n))]
  => (insert! (->Hit ?k)))

;; ROW 5 — string::length vs a BOUND var (?minlen), not a constant => 180/400 (see wat side's CRT
;; derivation over k mod 40; identical arithmetic here via `count`).
(defrule length-bound
  [Req (= ?k k) (= ?n n) (= ?tag tag) (= ?minlen minlen) (= ?padded padded)]
  [:test (> (count ?n) ?minlen)]
  => (insert! (->Hit ?k)))

;; ROW 6 — argument built AT TEST TIME: needle = (str ?tag "t"), mirroring
;; (:wat::core::String/concat ?tag "t") exactly (2-arg concat, matching String/concat's arity).
;; needle="cat" (even k) or "xyt" (odd k, never found) => 120/400.
(defrule dynamic-arg
  [Req (= ?k k) (= ?n n) (= ?tag tag) (= ?minlen minlen) (= ?padded padded)]
  [:test (str/includes? ?n (str ?tag "t"))]
  => (insert! (->Hit ?k)))

;; ROW 7 — String verb inside a boolean composition: contains AND (NOT starts-with) => 160/400.
(defrule compose-bool
  [Req (= ?k k) (= ?n n) (= ?tag tag) (= ?minlen minlen) (= ?padded padded)]
  [:test (and (str/includes? ?n "cat") (not (str/starts-with? ?n "cat")))]
  => (insert! (->Hit ?k)))

;; ROW 8 — String verb feeding an i64 comparison, composed with `and`: contains AND (minlen > 3)
;; => 120/400.
(defrule compose-i64
  [Req (= ?k k) (= ?n n) (= ?tag tag) (= ?minlen minlen) (= ?padded padded)]
  [:test (and (str/includes? ?n "cat") (> ?minlen 3))]
  => (insert! (->Hit ?k)))

;; ROW 9 — the user-defined pure fn. True for r=2, r=3 => 160/400.
(defrule userfn
  [Req (= ?k k) (= ?n n) (= ?tag tag) (= ?minlen minlen) (= ?padded padded)]
  [:test (feline? ?n)]
  => (insert! (->Hit ?k)))

;; ROW 10 — a String verb feeding ANOTHER String verb's argument: (starts-with? (lower-case ?n)
;; "dog"). Only r=4 ("DOG") lowercases to "dog" and matches => 80/400.
(defrule lowercase-chain
  [Req (= ?k k) (= ?n n) (= ?tag tag) (= ?minlen minlen) (= ?padded padded)]
  [:test (str/starts-with? (str/lower-case ?n) "dog")]
  => (insert! (->Hit ?k)))

;; ROW 11 — SHORT-CIRCUIT-SENSITIVE. `(subs ?n 0 3)` raises `StringIndexOutOfBoundsException` when
;; `(count ?n) < 3` (r=0, the empty string). Guarded behind `(>= (count ?n) 3)` in the same `and`
;; — a non-short-circuiting `and` would ABORT on the first r=0 fact rather than derive a wrong
;; count. Of what clears the guard, only r=1's first three chars are "cat" => 80/400.
(defrule shortcircuit-subs
  [Req (= ?k k) (= ?n n) (= ?tag tag) (= ?minlen minlen) (= ?padded padded)]
  [:test (and (>= (count ?n) 3) (str/starts-with? (subs ?n 0 3) "cat"))]
  => (insert! (->Hit ?k)))

;; ROW 12 — string::trim feeding `=`, over the whitespace edge. padded = "  cat  " (even k) or
;; "  dog  " (odd k); trimming and comparing to "cat" selects exactly the even half => 200/400.
(defrule trim-eq
  [Req (= ?k k) (= ?n n) (= ?tag tag) (= ?minlen minlen) (= ?padded padded)]
  [:test (= (str/trim ?padded) "cat")]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [Hit (= ?k k)])

;; THE ROW TABLE — mirrors where-string.wat's `build-rules` cond.
(def rows
  [[1  "starts-with"        starts-with]
   [2  "ends-with"          ends-with]
   [3  "contains"           contains]
   [4  "empty"              empty-str]
   [5  "length-bound"       length-bound]
   [6  "dynamic-arg"        dynamic-arg]
   [7  "compose-bool"       compose-bool]
   [8  "compose-i64"        compose-i64]
   [9  "userfn"             userfn]
   [10 "lowercase-chain"    lowercase-chain]
   [11 "shortcircuit-subs"  shortcircuit-subs]
   [12 "trim-eq"            trim-eq]])

;; seed-req i — the SAME formulas as wst::seed, computed independently rather than kept as a
;; hand-synced table:
;;   r(i)       = i mod 5   — selects n's category (see where-string.wat's header for the five)
;;   is-even(i) = i mod 2 == 0 — selects tag/padded
;;   minlen(i)  = i mod 8   — the per-fact bound threshold
(defn seed-req [i]
  (let [r        (- i (* (quot i 5) 5))
        is-even  (= 0 (- i (* (quot i 2) 2)))
        nm       (case r 0 "" 1 "cat" 2 "zzcatzz" 3 "ねこcat" "DOG")
        tg       (if is-even "ca" "xy")
        ml       (- i (* (quot i 8) 8))
        pd       (if is-even "  cat  " "  dog  ")]
    (->Req i nm tg ml pd)))

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
