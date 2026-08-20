#!/usr/bin/env bash
# gen-accum.sh GROUPS READINGS — emit the Clara translation of grid/accum.wat (axis A5:
# accumulate / exists) to stdout. SAME workload: Group(g) for g in [0,GROUPS); Reading(g,v) with
# W=READINGS readings per group, v = val(g,j) = (g*31 + j*17) mod 1000 (byte-identical to the wat
# side's :acc::val); five rules over an AccumulateNode / :exists:
#   CountF(g,n) :- Group(g) [?n <- (acc/count)  :from Reading(g)]        n = W
#   SumF(g,n)   :- Group(g) [?n <- (acc/sum :v) :from Reading(g,v)]      n = Σ v
#   MinF(g,n)   :- Group(g) [?n <- (acc/min :v) :from Reading(g,v)]      n = min v
#   MaxF(g,n)   :- Group(g) [?n <- (acc/max :v) :from Reading(g,v)]      n = max v
#   ExistsF(g)  :- Group(g) [:exists [Reading (= ?g g)]]                 fires (W>=1)
# Clara's built-in accumulators (clara.rules.accumulators, the shipped 0.24.0 source) compute the
# SAME bag-semantics folds; :exists is Clara's compile-time acc/exists (count>0). Same final set,
# different desugaring — the point of this axis.
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires).
# Emits one #grid/Result EDN line with :derived canonically encoded exactly like the wat side
# (kind*1e15 + g*1e9 + val, sorted ascending) so the two outputs compare byte-for-byte.
set -euo pipefail
G="$1"; W="$2"

cat <<HEADER
(ns accum (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]
                    [clara.rules.accumulators :as acc]))
(defrecord Group  [g])
(defrecord Reading [g v])
(defrecord CountF [g n])
(defrecord SumF   [g n])
(defrecord MinF   [g n])
(defrecord MaxF   [g n])
(defrecord ExistsF [g])

(defrule count-rule
  [Group (= ?g g)]
  [?n <- (acc/count) :from [Reading (= ?g g)]]
  => (clara.rules/insert! (->CountF ?g ?n)))
(defrule sum-rule
  [Group (= ?g g)]
  [?n <- (acc/sum :v) :from [Reading (= ?g g)]]
  => (clara.rules/insert! (->SumF ?g ?n)))
(defrule min-rule
  [Group (= ?g g)]
  [?n <- (acc/min :v) :from [Reading (= ?g g)]]
  => (clara.rules/insert! (->MinF ?g ?n)))
(defrule max-rule
  [Group (= ?g g)]
  [?n <- (acc/max :v) :from [Reading (= ?g g)]]
  => (clara.rules/insert! (->MaxF ?g ?n)))
(defrule exists-rule
  [Group (= ?g g)]
  [:exists [Reading (= ?g g)]]
  => (clara.rules/insert! (->ExistsF ?g)))

(defquery q-count [] [CountF (= ?g g) (= ?n n)])
(defquery q-sum   [] [SumF   (= ?g g) (= ?n n)])
(defquery q-min   [] [MinF   (= ?g g) (= ?n n)])
(defquery q-max   [] [MaxF   (= ?g g) (= ?n n)])
(defquery q-exists [] [ExistsF (= ?g g)])

(defn rval [g j] (mod (+ (* g 31) (* j 17)) 1000))
(defn enc [kind g v] (+ (* kind 1000000000000000) (* g 1000000000) v))
(defn all-codes [s]
  (sort
    (concat
      (map (fn [r] (enc 0 (:?g r) (:?n r))) (query s q-count))
      (map (fn [r] (enc 1 (:?g r) (:?n r))) (query s q-sum))
      (map (fn [r] (enc 2 (:?g r) (:?n r))) (query s q-min))
      (map (fn [r] (enc 3 (:?g r) (:?n r))) (query s q-max))
      (map (fn [r] (enc 4 (:?g r) 0))       (query s q-exists)))))
(defn -main [& _]
  (let [groups (map ->Group (range $G))
        reads  (for [g (range $G) j (range $W)] (->Reading g (rval g j)))
        seeds  (concat groups reads)
        build  (fn [] (apply insert (mk-session 'accum :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [base (mk-session 'accum :cache false)
          t0 (System/nanoTime)
          s (apply insert base seeds)
          t1 (System/nanoTime)
          f (fire-rules s)
          t2 (System/nanoTime)
          codes (all-codes f)
          t3 (System/nanoTime)]
      (println (str "#grid/Result {:axis \"accum\" :size [$G $W] :derived ["
                     (clojure.string/join " " codes)
                     "] :clara-ns " (- t2 t1)
                     " :insert-ns " (- t1 t0)
                     " :fire-ns " (- t2 t1)
                     " :query-ns " (- t3 t2)
                     " :protocol-ns " (- t3 t0) "}")))))
HEADER
