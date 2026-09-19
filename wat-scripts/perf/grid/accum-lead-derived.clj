;; wat-scripts/perf/grid/accum-lead-derived.clj — the CLARA TWIN of accum-lead-derived.wat.
;; Read that file's header FIRST: Step(0); Step(k) :- Step(k-1) for k in [1,depth];
;; Tally(n) :- [?n <- (acc/count) :from Step] — LEADING, no anchor, no join. The encoding
;; (enc kind*1e15 + level*1e9 + id), the sorted-NOT-deduped :derived, and the compound cell
;; this fixture closes — (record, absent, derived, leading, na), the intersection of
;; accum-over-derived's `A` mechanism and leading-exists's / accum-lead-rule-cascade's `L`
;; mechanism — are all documented there and mirrored here.
;;
;; ── ⛔ WHY THIS IS A STATIC `.clj` AND NOT A `gen-accum-lead-derived.sh` ────────────────────
;;
;; `run-all.sh:81-87` discovers a PERF axis as `<axis>.wat` WITH a `gen-<axis>.sh` twin, and
;; `:89-99` exits 2 for any discovered axis with no LADDER rung. A generator here would
;; therefore drag a CORRECTNESS proof onto the perf ladder and onto `check-grid-speed.sh`,
;; which treats `:accuracy :MISMATCH` as a gate failure. A static `.clj` is invisible to
;; that discovery (same convention as `accum-over-derived.clj` / `accum-lead-rule-cascade.clj`).
;;
;; Depth varies the RULE COUNT, so the step rules are generated at runtime (`eval` of
;; `defrule` into this namespace, then `mk-session 'accum-lead-derived`) rather than a
;; pre-max of inert rules. That is still this file, not a `gen-` script.
;;
;; It does NOT self-invoke `(-main)` at load: `check-grid-three-way.sh` `require`s this into
;; a shared JVM and drives `-main` with the size args, so a load-time print would emit its
;; row twice.
;;
;; ⛔ MULTIPLICITY IS THE WITNESS. `all-codes` is `(sort (concat …))` — no `set`, no
;; `distinct`. A leaked intermediate tally OR a per-round duplicate is an extra `enc(1,0,k)`
;; next to the survivor.

(ns accum-lead-derived
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Step [level])
(defrecord Tally [n])

;; ★ THE CELL. tally is a LEADING accumulate — its ONLY condition, no anchor, no join —
;; over Step, which the SAME ruleset derives via the generated step-N rules below.
(defrule tally
  [?n <- (acc/count) :from [Step]]
  => (insert! (->Tally ?n)))

(defquery q-step [] [Step (= ?level level)])
(defquery q-tally [] [Tally (= ?n n)])

(defn enc [kind level id] (+ (* kind 1000000000000000) (* level 1000000000) id))

;; Sorted bag of derived Step levels (level > 0) plus every Tally. sort does not dedup.
(defn all-codes [s]
  (sort
    (concat
      (map (fn [r] (enc 0 (:?level r) 0))
           (filter #(pos? (:?level %)) (query s q-step)))
      (map (fn [r] (enc 1 0 (:?n r)))
           (query s q-tally)))))

;; One `Step(k) :- Step(k-1)` per k in [1,depth], interned into THIS namespace so
;; `mk-session 'accum-lead-derived` sees them. Bound `*ns*` because `-main` is invoked
;; from a driver whose `*ns*` is not this one.
(defn install-step-rules! [depth]
  (binding [*ns* (the-ns 'accum-lead-derived)]
    (doseq [k (range 1 (inc depth))]
      ;; `[Step (= prev level)]` — type is the vector's first element, not a
      ;; wrapping list. `[(Step (= prev level))]` interned a rule that never fired.
      (eval (list 'defrule (symbol (str "step-" k))
                  (vector 'Step (list '= (dec k) 'level))
                  '=>
                  (list 'insert! (list '->Step k)))))))

(defn -main [& args]
  (let [depth (Long/parseLong (or (first args) "9"))]
    (install-step-rules! depth)
    (let [seeds [(->Step 0)]
          build (fn [] (apply insert (mk-session 'accum-lead-derived :cache false) seeds))]
      (dotimes [_ 3] (count (all-codes (fire-rules (build)))))
      (let [s (apply insert (mk-session 'accum-lead-derived :cache false) seeds)
            t0 (System/nanoTime)
            f (fire-rules s)
            t1 (System/nanoTime)
            codes (all-codes f)]
        (println (str "#grid/Result {:axis \"accum-lead-derived\" :size [" depth "] :derived ["
                      (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}"))))))
