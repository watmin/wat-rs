#!/usr/bin/env bash
# gen-node-share.sh N M — emit the Clara translation of grid/node-share.wat (axis A8:
# node-sharing / rule-count) to stdout. SAME workload: A(k),B(k) for k in [0,M); N rules each
#   [A (= ?k k)] [B (= ?k k)] [:test (= (mod ?k N) I)] => (insert! (->Out ?k))
# all sharing the leading [A]⋈[B] join-prefix, differentiated only by the trailing per-rule :test.
# Clara's compiler dedups the shared prefix automatically (CLARA-TRANSLATIONS.md §A8, grounded in
# clara/rules/compiler.clj: `to-alpha-graph`'s condition-to-node-map merges common conditions into
# one alpha node; `add-conjunctions` assigns an existing beta id iff an identical node has the same
# parent-id set — the same two-part key (condition identity + parent identity) wat uses). Both
# engines therefore collapse [A]⋈[B] into ONE subtree fanning out to N per-rule continuations.
#
# Every k in [0,M) satisfies EXACTLY one rule (I == k mod N), so the derived set is
# {Out(k) : k in [0,M)} = [0 1 .. M-1] on BOTH sides, independent of N — a SINGLE derived-set
# sanity check, NOT an accuracy gate (CLARA-TRANSLATIONS.md §A8: "this is a speed/node-count axis,
# not an accuracy axis"). The measurement is fire-cost as rule-count N grows with a fixed prefix.
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires). Emits one
# #grid/Result EDN line with :derived (sorted k) so the two outputs compare byte-for-byte.
set -euo pipefail
N="$1"; M="$2"

cat <<HEADER
(ns node-share (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord A [k])
(defrecord B [k])
(defrecord Out [k])
HEADER

for i in $(seq 0 $((N - 1))); do
  echo "(defrule r$i [A (= ?k k)] [B (= ?k k)] [:test (= (mod ?k $N) $i)] => (clara.rules/insert! (->Out ?k)))"
done

cat <<FOOTER
(defquery out-q [] [Out (= ?k k)])
(defn all-codes [s] (sort (map (fn [r] (:?k r)) (query s out-q))))
(defn -main [& _]
  (let [seeds (mapcat (fn [i] [(->A i) (->B i)]) (range $M))
        build (fn [] (apply insert (mk-session 'node-share :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"node-share\" :size [$N $M] :derived ["
                     (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
FOOTER
