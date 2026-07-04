#!/usr/bin/env bash
# gen-asym-join.sh ITEMS — emit the Clara translation of grid/asym-join.wat (axis A2:
# asymmetric-arrival joins) to stdout. SAME workload: A(k) for k in [0,ITEMS); R1: A(?k)->B(?k);
# R2: B(?k) JOIN A(?k) -> C(?k). The rule shape is verbatim the repo's R18 reference
# wat-scripts/fixes/rete-truth-maintenance-probes/chain.clj (CLARA-TRANSLATIONS.md §A2).
#
# CAVEAT (load-bearing, CLARA-TRANSLATIONS.md §A2): Clara has NO arrival-order hazard by
# construction — HashJoinNode left/right-activate always read the other side's COMPLETE persistent
# memory, so any insertion order is trivially correct. This axis uses Clara purely as the
# GROUND-TRUTH ORACLE for the now-fixed wat P6 ordering bug; the speed number is a raw-throughput
# comparison, NOT a "Clara handles arrival better/worse" claim (that axis does not exist for Clara).
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires).
# Emits one #grid/Result EDN line with :derived canonically encoded exactly like the wat side
# (B: 0*1000000+k, C: 1*1000000+k, sorted ascending) so the two outputs compare byte-for-byte.
set -euo pipefail
M="$1"

cat <<FOOTER
(ns asym-join (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord A [k])
(defrecord B [k])
(defrecord C [k])

;; R1: A -> B (derive B from every input A)
(defrule r1 [A (= ?k k)] => (clara.rules/insert! (->B ?k)))
;; R2: B JOIN A (derived B joined with input A on the same k) -> C
(defrule r2 [B (= ?k k)] [A (= ?k k)] => (clara.rules/insert! (->C ?k)))

(defquery b-q [] [B (= ?k k)])
(defquery c-q [] [C (= ?k k)])

(defn all-codes [s]
  (sort
    (concat
      (map (fn [r] (+ (* 0 1000000) (:?k r))) (query s b-q))
      (map (fn [r] (+ (* 1 1000000) (:?k r))) (query s c-q)))))

(defn -main [& _]
  (let [seeds (map ->A (range $M))
        build (fn [] (apply insert (mk-session 'asym-join :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"asym-join\" :size [$M] :derived ["
                     (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
FOOTER
