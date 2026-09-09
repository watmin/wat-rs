;; wat-scripts/perf/grid/retract-lead-accum.clj — the CLARA TWIN of retract-lead-accum.wat.
;; Read that file's header FIRST: accum-lead-rule-cascade's exact shape (leading
;; acc/count :from Reading, joined to Anchor, plus an inert Link cascade), with Reading(0)
;; duplicated (retract-multiplicity's technique) and then ONE copy retracted before re-fire.
;;
;; ── ⛔ WHY THIS IS A STATIC `.clj` AND NOT A `gen-retract-lead-accum.sh` ────────────────
;;
;; `run-all.sh:81-87` discovers a PERF axis as `<axis>.wat` WITH a `gen-<axis>.sh` twin. A
;; generator would drag a CORRECTNESS proof onto the perf ladder and onto
;; `check-grid-speed.sh`, which treats `:accuracy :MISMATCH` as a gate failure. Static
;; `.clj`, same convention as `accum-lead-rule-cascade.clj` / `retract-multiplicity.clj`.
;;
;; Depth varies the RULE COUNT, so the Link rules are generated at runtime (`eval` of
;; `defrule` into this namespace), exactly `accum-lead-rule-cascade.clj`'s
;; `install-link-rules!`.
;;
;; It does NOT self-invoke `(-main)` at load: `check-grid-three-way.sh` `require`s this
;; into a shared JVM and drives `-main`.
;;
;; ⛔ THE JOIN AFTER THE LEADING ACCUMULATE IS THE CELL — same reasoning as
;; accum-lead-rule-cascade.clj. Dropping Anchor would model a covered sibling.
;;
;; ⛔ MULTIPLICITY IS THE WITNESS. `all-codes` is `(sort (map …))` — no `set`, no
;; `distinct`. retract-multiplicity's own care applies: Reading(0) is the ONLY duplicated
;; key, so the justified derived-multiplicity split (Clara bag vs wat set on DERIVED facts)
;; cannot dominate — Reading is a plain base fact, never derived, in this fixture.

(ns retract-lead-accum
  (:require [clara.rules :refer [mk-session insert retract fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Reading [v])
(defrecord Anchor [k])
(defrecord Link [level])
(defrecord Busy [k n])

;; ★ leading accumulate, then the join. Acc first, Anchor second — accum-lead-rule-cascade's
;; exact rule.
(defrule busy
  [?n <- (acc/count) :from [Reading]]
  [Anchor (= ?k k)]
  => (insert! (->Busy ?k ?n)))

(defquery q-busy [] [Busy (= ?k k) (= ?n n)])

(defn enc [k n] (+ (* k 1000000000000000) n))

(defn all-codes [s]
  (sort (map (fn [r] (enc (:?k r) (:?n r))) (query s q-busy))))

(defn install-link-rules! [depth]
  (binding [*ns* (the-ns 'retract-lead-accum)]
    (doseq [k (range 1 (inc depth))]
      (eval (list 'defrule (symbol (str "link-" k))
                  (vector 'Link (list '= (dec k) 'level))
                  '=>
                  (list 'insert! (list '->Link k)))))))

;; Reading(i) once each for i in [0,items), then ONE extra Reading(0) — the duplicate.
(defn seed-facts [items anchors]
  (concat (map ->Reading (range items))
          [(->Reading 0)]
          (map ->Anchor (range anchors))
          [(->Link 0)]))

(defn -main [& args]
  (let [items   (Long/parseLong (or (first args) "3"))
        anchors (Long/parseLong (or (second args) "2"))
        depth   (Long/parseLong (or (nth args 2 nil) "3"))]
    (install-link-rules! depth)
    (let [seeds (seed-facts items anchors)
          build (fn [] (apply insert (mk-session 'retract-lead-accum :cache false) seeds))
          after-one-retract (fn []
                              (let [s (fire-rules (build))]
                                (retract s (->Reading 0))))]
      (dotimes [_ 3] (count (all-codes (fire-rules (after-one-retract)))))
      (let [s (after-one-retract)
            t0 (System/nanoTime)
            f (fire-rules s)
            t1 (System/nanoTime)
            codes (all-codes f)]
        (println (str "#grid/Result {:axis \"retract-lead-accum\" :size ["
                      items " " anchors " " depth "] :derived ["
                      (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}"))))))
