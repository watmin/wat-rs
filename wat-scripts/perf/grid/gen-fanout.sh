#!/usr/bin/env bash
# gen-fanout.sh ITEMS — emit the Clara translation of grid/fanout.wat (axis A1: fan-out /
# low-selectivity join) to stdout. SAME workload: fanout FIXED at F=20 (R4's exact P9/P10/P11
# bench configuration, "echo '[100 20]'"); keys = ITEMS / F^2 (the single "items" dial IS the
# target derived-Pair count — R4 recorded this axis in PAIRS, e.g. "40k", not in keys or fanout
# directly). For every key k in [0,keys): Left(k,f)+Right(k,f) for f in [0,F); one join rule:
#   Pair(k,l,r) :- Left(k,l) AND Right(k,r)     (F Lefts x F Rights per key -> F^2 pairs/key)
# keys*F^2 = ITEMS exactly at every rung of the ladder (10000/20000/40000 -> keys 25/50/100),
# reproducing R4's 40,000-pair cell (keys=100, fanout=20) at the top rung — the only recorded
# Clara win in the project's history (REALIZATIONS.md:201).
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires). Emits
# one #grid/Result EDN line with :derived canonically encoded exactly like the wat side
# (key*1000000 + lid*1000 + rid, sorted ascending) so the two outputs compare byte-for-byte.
set -euo pipefail
ITEMS="$1"
F=20
KEYS=$((ITEMS / (F * F)))

cat <<PROGRAM
(ns fanout (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord Left  [key lid])
(defrecord Right [key rid])
(defrecord Pair  [key lid rid])
(defrule r-fan [Left (= ?k key) (= ?l lid)] [Right (= ?k key) (= ?r rid)]
  => (clara.rules/insert! (->Pair ?k ?l ?r)))
(defquery q-pair [] [Pair (= ?k key) (= ?l lid) (= ?r rid)])
(defn enc [k l r] (+ (* k 1000000) (* l 1000) r))
(defn all-codes [s] (sort (map (fn [p] (enc (:?k p) (:?l p) (:?r p))) (query s q-pair))))
(defn -main [& _]
  (let [seeds (mapcat (fn [k] (mapcat (fn [f] [(->Left k f) (->Right k f)]) (range $F))) (range $KEYS))
        build (fn [] (apply insert (mk-session 'fanout :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [base (mk-session 'fanout :cache false)
          t0 (System/nanoTime)
          s (apply insert base seeds)
          t1 (System/nanoTime)
          f (fire-rules s)
          t2 (System/nanoTime)
          codes (all-codes f)
          t3 (System/nanoTime)]
      (println (str "#grid/Result {:axis \"fanout\" :size [$ITEMS] :derived ["
                     (clojure.string/join " " codes)
                     "] :clara-ns " (- t2 t1)
                     " :insert-ns " (- t1 t0)
                     " :fire-ns " (- t2 t1)
                     " :query-ns " (- t3 t2)
                     " :protocol-ns " (- t3 t0) "}")))))
PROGRAM
