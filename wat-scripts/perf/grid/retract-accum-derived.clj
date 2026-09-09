;; wat-scripts/perf/grid/retract-accum-derived.clj — the CLARA TWIN of
;; retract-accum-derived.wat. Read that file's header FIRST: accum-over-derived's exact
;; shape (Seed anchors an accumulate over Step; Step(k):-Step(k-1) derives Step), with
;; Step(0) — the accumulate's own `:from` source AND the cascade's root — duplicated
;; (retract-multiplicity's technique), then ONE copy retracted before re-fire.
;;
;; ── ⛔ WHY THIS IS A STATIC `.clj` AND NOT A `gen-retract-accum-derived.sh` ─────────────
;;
;; `run-all.sh:81-87` discovers a PERF axis as `<axis>.wat` WITH a `gen-<axis>.sh` twin. A
;; generator would drag a CORRECTNESS proof onto the perf ladder and onto
;; `check-grid-speed.sh`, which treats `:accuracy :MISMATCH` as a gate failure. Static
;; `.clj`, same convention as `accum-over-derived.clj` / `retract-multiplicity.clj`.
;;
;; Depth varies the RULE COUNT, so the step rules are generated at runtime exactly
;; `accum-over-derived.clj`'s `install-step-rules!`.
;;
;; It does NOT self-invoke `(-main)` at load: `check-grid-three-way.sh` `require`s this
;; into a shared JVM and drives `-main`.
;;
;; ⛔ MULTIPLICITY IS THE WITNESS. `all-codes` is `(sort (concat …))` — no `set`, no
;; `distinct`. Clara's `insert!` is a LOGICAL (justified) insert: a Step(k) derived from a
;; retracted Step(0) token should be automatically retracted by Clara's own truth
;; maintenance once that Step(0) token's support is gone — this file is the REFEREE for
;; whether wat's own bookkeeping matches that.

(ns retract-accum-derived
  (:require [clara.rules :refer [mk-session insert retract fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Seed [id])
(defrecord Step [level])
(defrecord Tally [n])

;; Seed is the ANCHOR (nonleading), exactly accum-over-derived's own tally rule, unchanged.
(defrule tally
  [Seed (= ?id id)]
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

;; One `Step(k) :- Step(k-1)` per k in [1,depth], interned into THIS namespace, exactly
;; accum-over-derived.clj's `install-step-rules!`.
(defn install-step-rules! [depth]
  (binding [*ns* (the-ns 'retract-accum-derived)]
    (doseq [k (range 1 (inc depth))]
      (eval (list 'defrule (symbol (str "step-" k))
                  (vector 'Step (list '= (dec k) 'level))
                  '=>
                  (list 'insert! (list '->Step k)))))))

(defn -main [& args]
  (let [depth (Long/parseLong (or (first args) "9"))]
    (install-step-rules! depth)
    ;; Seed(0), Step(0) TWICE — the duplicate. Duplicate ONLY the retracted key, exactly
    ;; retract-multiplicity's own care.
    (let [seeds [(->Seed 0) (->Step 0) (->Step 0)]
          build (fn [] (apply insert (mk-session 'retract-accum-derived :cache false) seeds))
          after-one-retract (fn []
                              (let [s (fire-rules (build))]
                                (retract s (->Step 0))))]
      (dotimes [_ 3] (count (all-codes (fire-rules (after-one-retract)))))
      (let [s (after-one-retract)
            t0 (System/nanoTime)
            f (fire-rules s)
            t1 (System/nanoTime)
            codes (all-codes f)]
        (println (str "#grid/Result {:axis \"retract-accum-derived\" :size [" depth "] :derived ["
                      (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}"))))))
