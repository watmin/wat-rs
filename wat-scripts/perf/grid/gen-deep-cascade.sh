#!/usr/bin/env bash
# gen-deep-cascade.sh DEPTH WIDTH — emit the Clara translation of grid/deep-cascade.wat (axis A0:
# deep forward-chain cascade) to stdout. SAME workload: Node(0,i)+Tag(0,i) seeded for i in
# [0,WIDTH); DEPTH rules, one per level k in [1,DEPTH]:
#   Node(k,id),Tag(k,id) :- Node(k-1,id) AND Tag(k-1,id)
# Every id survives every level (the joins never drop anyone) — mirrors the wat side's
# :dc::build-rule exactly (the level literals splice in as bare numbers, one rule per level, same
# shape gen-node-share.sh uses to emit a variable number of defrule forms).
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires). Emits
# one #grid/Result EDN line with :derived canonically encoded exactly like the wat side
# (kind*1e15 + level*1e9 + id, kind 0=Node/1=Tag, level>0 only — level 0 is the seeded input,
# excluded from the witness) so the two outputs compare byte-for-byte.
set -euo pipefail
D="$1"; W="$2"

cat <<HEADER
(ns deep-cascade (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord Node [level id])
(defrecord Tag  [level id])
HEADER

for k in $(seq 1 "$D"); do
  prev=$((k - 1))
  echo "(defrule cascade-$k [Node (= ?id id) (= $prev level)] [Tag (= ?id id) (= $prev level)] => (clara.rules/insert! (->Node $k ?id)) (clara.rules/insert! (->Tag $k ?id)))"
done

cat <<FOOTER
(defquery q-node [] [Node (= ?level level) (= ?id id)])
(defquery q-tag  [] [Tag  (= ?level level) (= ?id id)])
(defn enc [kind level id] (+ (* kind 1000000000000000) (* level 1000000000) id))
(defn all-codes [s]
  (sort
    (concat
      (map (fn [r] (enc 0 (:?level r) (:?id r))) (filter #(pos? (:?level %)) (query s q-node)))
      (map (fn [r] (enc 1 (:?level r) (:?id r))) (filter #(pos? (:?level %)) (query s q-tag))))))
(defn -main [& _]
  (let [seeds (concat (map #(->Node 0 %) (range $W)) (map #(->Tag 0 %) (range $W)))
        build (fn [] (apply insert (mk-session 'deep-cascade :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [base (mk-session 'deep-cascade :cache false)
          t0 (System/nanoTime)
          s (apply insert base seeds)
          t1 (System/nanoTime)
          f (fire-rules s)
          t2 (System/nanoTime)
          codes (all-codes f)
          t3 (System/nanoTime)]
      (println (str "#grid/Result {:axis \"deep-cascade\" :size [$D $W] :derived ["
                     (clojure.string/join " " codes)
                     "] :clara-ns " (- t2 t1)
                     " :insert-ns " (- t1 t0)
                     " :fire-ns " (- t2 t1)
                     " :query-ns " (- t3 t2)
                     " :protocol-ns " (- t3 t0) "}")))))
FOOTER
