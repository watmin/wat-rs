;; wat-scripts/perf/grid/userfn-accum-derived.clj — the CLARA TWIN of
;; userfn-accum-derived.wat. Read that file's header FIRST: Seed anchors an accumulate over
;; Step (Step(k):-Step(k-1) makes Step derived within the SAME ruleset — the shape is
;; accum-over-derived.clj's, verbatim), and the accumulate's `:then` head is a call to a
;; user fn `mk-tally`, not a bare constructor (userfn-head.clj's inlining convention).
;;
;; ── ⛔ WHY THIS IS A STATIC `.clj` AND NOT A `gen-userfn-accum-derived.sh` ─────────────
;;
;; `run-all.sh:81-87` discovers a PERF axis as `<axis>.wat` WITH a `gen-<axis>.sh` twin, and
;; `:89-99` exits 2 for any discovered axis with no LADDER rung. A generator here would
;; therefore drag a CORRECTNESS proof onto the perf ladder and onto `check-grid-speed.sh`,
;; which treats `:accuracy :MISMATCH` as a gate failure. A static `.clj` is invisible to
;; that discovery (same convention as `accum-over-derived.clj` / `accum-lead-derived.clj`).
;;
;; Depth varies the RULE COUNT, so the step rules are generated at runtime exactly as
;; accum-over-derived.clj's `install-step-rules!` does (`eval` of `defrule` into THIS
;; namespace, then `mk-session 'userfn-accum-derived`), not a pre-max of inert rules.
;;
;; It does NOT self-invoke `(-main)` at load: `check-grid-three-way.sh` `require`s this into
;; a shared JVM and drives `-main` with the size args, so a load-time print would emit its
;; row twice.
;;
;; ── MODELLING POINT — THE USER FN IS INLINED (same as userfn-head.clj) ─────────────────
;;
;; Clara 0.24.0 has no `:then`-head-is-a-fn. wat's `:cad::tally` RHS is
;; `(:cad::mk-tally ?n)`, and `mk-tally` is
;;
;;   (:wat::rete::core::defn :cad::mk-tally [n <- :wat::core::i64] -> :cad::Tally
;;     (:cad::Tally :n n))
;;
;; The body is exactly `(->Tally n)`. The Clara `tally` rule inlines that construction as
;; `(insert! (->Tally ?n))`. If `mk-tally` ever computes, this file is no longer the same
;; rule and must be rewritten, not trusted.
;;
;; ⛔ MULTIPLICITY IS THE WITNESS. `all-codes` is `(sort (concat …))` — no `set`, no
;; `distinct`. A leaked intermediate tally is an extra `enc(1,0,k)` next to the survivor.

(ns userfn-accum-derived
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Seed [id])
(defrecord Step [level])
(defrecord Tally [n])

;; Seed is the ANCHOR (nonleading, like accum-over-derived) — the `:then` head is the
;; user-fn call inlined, per the modelling point above.
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
  (binding [*ns* (the-ns 'userfn-accum-derived)]
    (doseq [k (range 1 (inc depth))]
      (eval (list 'defrule (symbol (str "step-" k))
                  (vector 'Step (list '= (dec k) 'level))
                  '=>
                  (list 'insert! (list '->Step k)))))))

(defn -main [& args]
  (let [depth (Long/parseLong (or (first args) "9"))]
    (install-step-rules! depth)
    (let [seeds [(->Seed 0) (->Step 0)]
          build (fn [] (apply insert (mk-session 'userfn-accum-derived :cache false) seeds))]
      (dotimes [_ 3] (count (all-codes (fire-rules (build)))))
      (let [s (apply insert (mk-session 'userfn-accum-derived :cache false) seeds)
            t0 (System/nanoTime)
            f (fire-rules s)
            t1 (System/nanoTime)
            codes (all-codes f)]
        (println (str "#grid/Result {:axis \"userfn-accum-derived\" :size [" depth "] :derived ["
                      (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}"))))))
