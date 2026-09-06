;; wat-scripts/perf/grid/retract-multiplicity.clj — the CLARA TWIN of retract-multiplicity.wat.
;; Read that file's header FIRST: the records, the seed (F(0)×2, F(k)×1 for k>0, G(k)×1), the one join,
;; fire-then-retract-F(0)-once-then-re-fire, and the multiplicity-preserving :derived encoding
;; are all documented there and mirrored here.
;;
;; ── ⛔ WHY THIS IS A STATIC `.clj` AND NOT A `gen-retract-multiplicity.sh` ──────────────────
;;
;; `run-all.sh:81-87` discovers a PERF axis as `<axis>.wat` WITH a `gen-<axis>.sh` twin, and
;; `:89-99` exits 2 for any discovered axis with no LADDER rung. `hunt_tooling_selftests.rs`
;; lifts that reconciliation onto the floor. A generator here would therefore drag a
;; CORRECTNESS proof onto the perf ladder (sizes are a published artifact that must not drift)
;; and onto `check-grid-speed.sh`, which treats `:accuracy :MISMATCH` as a gate failure —
;; turning the recorded violation this axis exists to emit into a red. A static `.clj` is
;; invisible to that discovery (same convention as `parametric-erasure.clj`).
;;
;; It does NOT self-invoke `(-main)` at load: `check-grid-three-way.sh` `require`s this into a
;; shared JVM and drives `-main` with the size args, so a load-time print would emit its row twice.
;;
;; ⛔ MULTIPLICITY IS THE WITNESS. `all-codes` is `(sort (map :?k …))` — no `set`, no `distinct`,
;; no `(count (set …))`. The where-family retract rows all collapse; copying that shape would
;; reproduce the blindness this axis exists to remove.

(ns retract-multiplicity
  (:require [clara.rules :refer [mk-session insert retract fire-rules query defrule defquery insert!]]))

(defrecord F [k])
(defrecord G [k])
(defrecord Out [k])

(defrule out
  [F (= ?k k)]
  [G (= ?k k)]
  => (insert! (->Out ?k)))

(defquery q [] [Out (= ?k k)])

;; Sorted bag of Out keys. sort does not dedup; two Out(1) stay two 1s.
(defn all-codes [s] (sort (map :?k (query s q))))

(defn seed-facts [items]
  (concat (mapcat (fn [k] [(->F k) (->G k)]) (range items))
          [(->F 0)]))

(defn -main [& args]
  (let [items (Long/parseLong (or (first args) "3"))
        seeds (seed-facts items)
        build (fn [] (apply insert (mk-session [out q] :cache false) seeds))
        after-one-retract (fn []
                            (let [s (fire-rules (build))]
                              (retract s (->F 0))))]
    (dotimes [_ 3] (count (all-codes (fire-rules (after-one-retract)))))
    (let [s (after-one-retract)
          t0 (System/nanoTime)
          f (fire-rules s)
          t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"retract-multiplicity\" :size [" items "] :derived ["
                     (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
