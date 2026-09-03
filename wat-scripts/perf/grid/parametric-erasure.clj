;; wat-scripts/perf/grid/parametric-erasure.clj — the CLARA TWIN of parametric-erasure.wat.
;; Read that file's header FIRST: the record shapes, the seed formula, the three rules and the
;; canonical `:derived` encoding are all documented there and mirrored here rule for rule.
;;
;; ── ⛔ WHY A CLARA TWIN OF A *PARAMETRIC* AXIS EXISTS AT ALL ────────────────────────────────────
;;
;; `parametric-erasure.wat` carries arc 278's D7 shape — a silent fact-drop that shows up as a
;; NATIVE-vs-ORACLE divergence — and it was authored without a `.clj`, on the reasoning that
;; "Clojure `defrecord` has no type parameters, so the parametric declaration is not expressible".
;; That reasoning is struck, and the twin below is why it does not survive contact:
;;
;;   Clara referees RULE SEMANTICS, not wat's type system. The erasure is what wat does to the
;;   DECLARATION — `(:pe::Box :- [T] [k <- i64  v <- :T])` collapses `Box[i64]`, `Box[String]` and
;;   `Box[Tag]` into ONE runtime class `pe::Box`. What reaches the RETE network is a bag of
;;   ordinary facts of one class whose `v` fields have different runtime types, and Clojure —
;;   dynamically typed — expresses that as its NATIVE case: one `defrecord Box [k v]` whose
;;   instances hold a Long, a String, and a Tag record.
;;
;; So the twin cannot reproduce wat's *declaration*, and does not try. It reproduces the DERIVED
;; SET, and the derived set is the independent ground truth that would have named D7 without
;; anyone finding it by hand. Leaving this axis unrefereed would put a hole in the differential
;; corpus exactly where the known bug lives, with a green light over it.
;;
;; ── ⛔ WHY THIS IS A STATIC `.clj` AND NOT A `gen-parametric-erasure.sh` ────────────────────────
;;
;; `run-all.sh:81-87` discovers a PERF axis as `<axis>.wat` WITH a `gen-<axis>.sh` twin (the
;; requirement is the `[ -f "$GRID_DIR/gen-$stem.sh" ] || continue` at `:85`), and `:89-99` exits 2
;; for any discovered axis with no LADDER rung. A generator here would therefore
;; drag a CORRECTNESS shape onto the perf ladder, whose sizes are a published artifact that must
;; not drift. A static `.clj` is invisible to that discovery (checked: nothing in the tree globs
;; `*.clj` broadly; `check-where-shapes.sh:140` globs `where-*.wat` only), and the 38 static `.clj`
;; already in this directory are the convention it follows.
;;
;; ── HOW IT RUNS ────────────────────────────────────────────────────────────────────────────────
;;
;; Standalone (namespace mode — note the file must be reachable as `parametric_erasure.clj` on the
;; classpath for `-m`, dashes becoming underscores; `check-grid-three-way.sh` stages it that way):
;;
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}} :paths ["."]}' \
;;             -M -m parametric-erasure 200
;;
;; It does NOT self-invoke `(-main)` at load, unlike the `where-*.clj` rows: those are run as
;; scripts, this one is `require`d into a shared JVM alongside the eleven generated axis programs
;; and driven by an explicit `-main` call, so a load-time print would emit its row twice.
;;
;; ── FAITHFULNESS TO THE WAT SIDE ───────────────────────────────────────────────────────────────
;;
;;   Box(k,v)   v cycles by (k mod 3): 0 -> the Long k          (wat: Box[i64],    packable)
;;                                     1 -> (str k)             (wat: Box[String], not packable)
;;                                     2 -> (->Tag k)           (wat: Box[Tag],    not packable,
;;                                                               and a DIFFERENT erasure from
;;                                                               the String one)
;;   Plain(k)   the non-parametric, uniformly-packable control, seeded for every k.
;;   Hit(k)      :- Box(k,v)                 tag 0
;;   PlainHit(k) :- Plain(k)                 tag 1
;;   Pair(k)     :- Box(k,v) AND Plain(k)    tag 2   — ★ the JOIN arm; on the wat side Box's alpha
;;                                                     delta reaches this as SLOT INDICES, which is
;;                                                     what turns D7's aliasing into a WRONG
;;                                                     BINDING rather than a merely missing fact.
;;
;; `?v` is bound and unused in both r-box and r-pair, exactly as in the wat rules — it is the
;; heterogeneous field, and binding it is what makes the erased class's mixed packability reach
;; the join at all.
;;
;; wat's `(:wat::core::i64::to-string i)` is Clojure's `(str i)`; wat's `:pe::i64-mod` is
;; truncating-division modulo over non-negative operands, identical to Clojure's `mod` here (every
;; seed index is >= 0 — see where-multivar.clj's note on the floor-vs-truncate subtlety).
;;
;; `:derived` is canonically encoded EXACTLY as the wat side does it — `tag * 1,000,000 + k`,
;; sorted ascending, space-joined inside one bracket — so the two `#grid/Result` lines compare
;; byte-for-byte. It is the FULL SET, never a count: D7 produced a right-sized WRONG answer.

(ns parametric-erasure
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

;; The erased class. wat declares ONE parametric record and gets one runtime class; Clojure
;; declares one record and gets one class. Same population of facts, said the native way in each.
(defrecord Box [k v])

;; The record-valued filler — a second, independent non-packable erasure, so the axis is not
;; pinned to String on either side.
(defrecord Tag [n])

;; The non-parametric, uniformly-packable neighbour. A live control, not decoration: a cure that
;; narrowed batching to nothing would satisfy every equality on Box while silently deleting the
;; fast path, and Plain is what keeps that visible.
(defrecord Plain [k])

(defrecord Hit [k])
(defrecord PlainHit [k])
(defrecord Pair [k])

(defrule r-box
  [Box (= ?k k) (= ?v v)]
  => (insert! (->Hit ?k)))

(defrule r-plain
  [Plain (= ?k k)]
  => (insert! (->PlainHit ?k)))

;; ★ THE JOIN ARM.
(defrule r-pair
  [Box (= ?k k) (= ?v v)]
  [Plain (= ?k k)]
  => (insert! (->Pair ?k)))

(defquery q-hit   [] [Hit      (= ?k k)])
(defquery q-plain [] [PlainHit (= ?k k)])
(defquery q-pair  [] [Pair     (= ?k k)])

(defn encode [tag k] (+ (* tag 1000000) k))

;; ONE class, THREE erasures, cycling by (i mod 3) — a packable instance sits both BEFORE and
;; AFTER an erased one, so neither interleaving is privileged.
(defn box-for [i]
  (case (mod i 3)
    0 (->Box i i)
    1 (->Box i (str i))
    (->Box i (->Tag i))))

(defn seed-facts [items]
  (mapcat (fn [i] [(box-for i) (->Plain i)]) (range items)))

;; The full derived set, canonically encoded and sorted ascending. NOT A COUNT.
(defn all-codes [s]
  (sort (concat (map (fn [r] (encode 0 (:?k r))) (query s q-hit))
                (map (fn [r] (encode 1 (:?k r))) (query s q-plain))
                (map (fn [r] (encode 2 (:?k r))) (query s q-pair)))))

(defn -main [& args]
  (let [items (Long/parseLong (or (first args) "200"))
        seeds (seed-facts items)
        build (fn [] (apply insert (mk-session [r-box r-plain r-pair q-hit q-plain q-pair]
                                               :cache false)
                            seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"parametric-erasure\" :size [" items "] :derived ["
                    (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
