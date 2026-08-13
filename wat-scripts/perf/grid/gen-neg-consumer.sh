#!/usr/bin/env bash
# gen-neg-consumer.sh ITEMS — emit the Clara translation of grid/neg-consumer.wat (the axis that
# crosses the negation/positive seam) to stdout.
#
# SAME workload as the wat side:
#   Item(k)  k in [0,ITEMS)      input
#   Bad(k)   every EVEN k        input
#   Tag(k)   every k             input — the "ruling table" join partner
#   Ok(k)    :- Item(k), NOT Bad(k)    the GATE
#   Final(k) :- Ok(k), Tag(k)          the POSITIVE CONSUMER of a post-negation fact
#
# :derived is the sorted Final keys. Correct answer: exactly the ODD k.
#
# ★ WHY THIS AXIS IS THE ONE THAT CONVICTS. Clara reaches the answer by INCREMENTAL truth
# maintenance: each Bad insert live-retracts the Ok token already fired for that ?k, and the
# Final rule re-fires as its support changes — there is no stratum assignment to get wrong.
# wat reaches it (or fails to) by STRATIFIED batch fixpoint, and that is exactly where task
# #94 lives: the stratifier orders by negation dependency only, so `final` — which negates
# nothing — is placed below the rule that produces Ok, fires before Ok exists, and never
# re-fires. Measured 2026-08-13, ITEMS=8: wat native [] AND wat oracle [], Clara [1 3 5 7].
#
# Both wat impls being EMPTY IDENTICALLY is the load-bearing observation: `oracle == native`
# passes, so our own dual-impl differential cannot see this class at all. Only the external
# peer can (R61 PAR NON ARGVIT, with our own oracle in the peer's seat).
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires).
set -euo pipefail
M="$1"

cat <<PROGRAM
(ns neg-consumer (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord Item [k])
(defrecord Bad [k])
(defrecord Tag [k])
(defrecord Ok [k])
(defrecord Final [k])
(defrule ok    [Item (= ?k k)] [:not [Bad (= ?k k)]] => (clara.rules/insert! (->Ok ?k)))
(defrule final [Ok (= ?k k)]   [Tag (= ?k k)]        => (clara.rules/insert! (->Final ?k)))
(defquery q [] [Final (= ?k k)])
(defn all-codes [s] (sort (map :?k (query s q))))
(defn -main [& _]
  (let [items (map ->Item (range $M))
        tags  (map ->Tag  (range $M))
        bads  (map ->Bad  (filter even? (range $M)))
        seeds (concat items tags bads)
        build (fn [] (apply insert (mk-session 'neg-consumer :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"neg-consumer\" :size [$M] :derived ["
                     (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
PROGRAM
