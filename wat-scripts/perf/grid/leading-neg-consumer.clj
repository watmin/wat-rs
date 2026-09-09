;; wat-scripts/perf/grid/leading-neg-consumer.clj — the CLARA TWIN of
;; leading-neg-consumer.wat. Read that file's header FIRST: Wind(loc)×2 per loc, Tag(loc)
;; per loc, Bad NEVER seeded, an inert S1..S6 cascade forcing six fixpoint rounds
;; (leading-exists's exact shape), and the three-stage positive chain
;; Signal :- (exists Wind) [LEADING] -> Ok :- Signal, NOT Bad -> Final :- Ok, Tag
;; (neg-consumer's own two-stage positive-consumer idiom, now downstream of a leading gate).
;;
;; ── ⛔ WHY THIS IS A STATIC `.clj` AND NOT A `gen-leading-neg-consumer.sh` ──────────────
;;
;; `run-all.sh:81-87` discovers a PERF axis as `<axis>.wat` WITH a `gen-<axis>.sh` twin, and
;; `:89-99` exits 2 for any discovered axis with no LADDER rung. A generator here would drag
;; a CORRECTNESS proof onto the perf ladder (`check-grid-speed.sh` treats `:accuracy
;; :MISMATCH` as a gate failure). Static `.clj`, same convention as `neg-consumer` axes and
;; `userfn-head.clj`. (leading-exists itself IS `gen-`-shaped and rides the perf ladder;
;; this compound cell deliberately does not, per DESIGN's own "not in run-all.sh's ORDER".)
;;
;; It does NOT self-invoke `(-main)` at load: `check-grid-three-way.sh` `require`s this into
;; a shared JVM and drives `-main` with the size args, so a load-time print would emit its
;; row twice.
;;
;; ── Clara's :exists is accumulator-backed ───────────────────────────────────────────────
;;
;; `clara.rules.accumulators` must be required (a bare `:require [clara.rules ...]` dies
;; with ClassNotFoundException on the first `:exists`, not a wrong answer) — the same note
;; `gen-leading-exists.sh` carries.
;;
;; ⛔ MULTIPLICITY IS THE WITNESS. `all-codes` is `(sort (map …))` — no `set`, no `distinct`.
;; A per-round leak at Signal would show as `items * rounds` Final rows here, two hops
;; downstream of where leading-exists itself would have caught it.

(ns leading-neg-consumer
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators]))

(defrecord Wind [loc])
(defrecord Bad [loc])
(defrecord Tag [loc])
(defrecord Signal [loc])
(defrecord Ok [loc])
(defrecord Final [loc])
(defrecord S1 [k]) (defrecord S2 [k]) (defrecord S3 [k])
(defrecord S4 [k]) (defrecord S5 [k]) (defrecord S6 [k])

;; ★ THE LEADING GATE — no parent condition, binds ?loc per DISTINCT Wind loc.
(defrule signal
  [:exists [Wind (= ?loc loc)]]
  => (insert! (->Signal ?loc)))

;; ★ FIRST POSITIVE CONSUMER — Bad is never seeded, so this is vacuously true for every loc.
(defrule ok
  [Signal (= ?loc loc)]
  [:not [Bad (= ?loc loc)]]
  => (insert! (->Ok ?loc)))

;; ★ SECOND POSITIVE CONSUMER — the subject, two hops downstream of the leading gate.
(defrule final
  [Ok  (= ?loc loc)]
  [Tag (= ?loc loc)]
  => (insert! (->Final ?loc)))

;; The inert cascade — leading-exists's own five rules, unchanged, forcing six rounds.
(defrule r2 [S1 (= ?k k)] => (insert! (->S2 ?k)))
(defrule r3 [S2 (= ?k k)] => (insert! (->S3 ?k)))
(defrule r4 [S3 (= ?k k)] => (insert! (->S4 ?k)))
(defrule r5 [S4 (= ?k k)] => (insert! (->S5 ?k)))
(defrule r6 [S5 (= ?k k)] => (insert! (->S6 ?k)))

(defquery q-final [] [Final (= ?loc loc)])

;; Sorted bag of Final locs. sort does not dedup — a per-round leak stays visible.
(defn all-codes [s] (sort (map :?loc (query s q-final))))

(defn seed-facts [items]
  (concat (mapcat (fn [i] [(->Wind i) (->Wind i) (->Tag i)]) (range items))
          [(->S1 1)]))

(defn -main [& args]
  (let [items (Long/parseLong (or (first args) "20"))
        seeds (seed-facts items)
        build (fn [] (apply insert (mk-session 'leading-neg-consumer :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))
    (let [s (build)
          t0 (System/nanoTime)
          f (fire-rules s)
          t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"leading-neg-consumer\" :size [" items "] :derived ["
                    (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
