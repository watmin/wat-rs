#!/usr/bin/env bash
# gen-bench.sh DEPTH WIDTH — emit a Clara deep-cascade bench .clj to stdout.
# Same workload as wat-scripts/perf/deep-cascade.wat: depth-N x width-M, every level a 2-way
# JOIN on the prior level's DERIVED facts (Stage@k-1 ⋈ Tag@k-1 on id -> Stage@k, Tag@k).
# Times Clara fire-rules only (mk-session compile + JIT warmed out first). Emits one EDN line.
set -euo pipefail
D="$1"; W="$2"
cat <<HEADER
(ns bench (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord Stage [level id])
(defrecord Tag [level id])
HEADER
for k in $(seq 1 "$D"); do p=$((k-1)); cat <<RULE
(defrule r$k [Stage (= ?id id) (= $p level)] [Tag (= ?id id) (= $p level)]
  => (clara.rules/insert! (->Stage $k ?id)) (clara.rules/insert! (->Tag $k ?id)))
RULE
done
cat <<FOOTER
(defquery deepest-q [] [Stage (= $D level) (= ?id id)])
(defn -main [& _]
  (let [seeds (mapcat (fn [i] [(->Stage 0 i) (->Tag 0 i)]) (range $W))
        build (fn [] (apply insert (mk-session 'bench :cache false) seeds))]
    (dotimes [_ 3] (count (query (fire-rules (build)) deepest-q)))   ; JIT + compile warmup
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)
          cnt (count (query f deepest-q))]
      (println (str "#clara/Result {:depth $D :width $W :deepest " cnt " :clara-ns " (- t1 t0) "}")))))
FOOTER
