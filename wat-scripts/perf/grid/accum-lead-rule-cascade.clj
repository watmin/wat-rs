;; wat-scripts/perf/grid/accum-lead-rule-cascade.clj — the CLARA TWIN of
;; accum-lead-rule-cascade.wat. Read that file's header FIRST: Reading(v) × items,
;; Anchor(k) × anchors, inert Link cascade of `depth`, and
;; Busy(k,n) :- [?n <- (acc/count) :from Reading] AND Anchor(k) — a leading
;; accumulate in a RULE, then a join. The cascade is unread. Constancy of
;; :derived across depth IS the assertion.
;;
;; ── ⛔ WHY THIS IS A STATIC `.clj` AND NOT A `gen-accum-lead-rule-cascade.sh` ────
;;
;; `run-all.sh:81-87` discovers a PERF axis as `<axis>.wat` WITH a `gen-<axis>.sh` twin.
;; A generator would drag a CORRECTNESS proof onto the perf ladder. Static `.clj`,
;; same convention as `userfn-head.clj` / `accum-over-derived.clj`.
;;
;; Depth varies the RULE COUNT, so the Link rules are generated at runtime (`eval` of
;; `defrule` into this namespace) rather than a pre-max of inert rules.
;;
;; It does NOT self-invoke `(-main)` at load: `check-grid-three-way.sh` `require`s this
;; into a shared JVM and drives `-main`.
;;
;; ⛔ THE JOIN AFTER THE LEADING ACCUMULATE IS THE CELL. Dropping Anchor or answering
;; via a query would model a covered sibling (where-accum-lead / where-accum-lead-cascade)
;; and would agree for the wrong reason.
;;
;; ⛔ MULTIPLICITY IS THE WITNESS. `all-codes` is `(sort (map …))` — no `set`, no
;; `distinct`. Extra Busy copies from a round leak are extra elements.

(ns accum-lead-rule-cascade
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Reading [v])
(defrecord Anchor [k])
(defrecord Link [level])
(defrecord Busy [k n])

;; ★ leading accumulate, then the join. Acc first, Anchor second.
(defrule busy
  [?n <- (acc/count) :from [Reading]]
  [Anchor (= ?k k)]
  => (insert! (->Busy ?k ?n)))

(defquery q-busy [] [Busy (= ?k k) (= ?n n)])

(defn enc [k n] (+ (* k 1000000000000000) n))

(defn all-codes [s]
  (sort (map (fn [r] (enc (:?k r) (:?n r))) (query s q-busy))))

(defn install-link-rules! [depth]
  (binding [*ns* (the-ns 'accum-lead-rule-cascade)]
    (doseq [k (range 1 (inc depth))]
      (eval (list 'defrule (symbol (str "link-" k))
                  (vector 'Link (list '= (dec k) 'level))
                  '=>
                  (list 'insert! (list '->Link k)))))))

(defn -main [& args]
  (let [items   (Long/parseLong (or (first args) "3"))
        anchors (Long/parseLong (or (second args) "2"))
        depth   (Long/parseLong (or (nth args 2 nil) "3"))]
    (install-link-rules! depth)
    (let [seeds (concat (map ->Reading (range items))
                        (map ->Anchor (range anchors))
                        [(->Link 0)])
          build (fn [] (apply insert (mk-session 'accum-lead-rule-cascade :cache false) seeds))]
      (dotimes [_ 3] (count (all-codes (fire-rules (build)))))
      (let [s (apply insert (mk-session 'accum-lead-rule-cascade :cache false) seeds)
            t0 (System/nanoTime)
            f (fire-rules s)
            t1 (System/nanoTime)
            codes (all-codes f)]
        (println (str "#grid/Result {:axis \"accum-lead-rule-cascade\" :size ["
                      items " " anchors " " depth "] :derived ["
                      (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}"))))))
