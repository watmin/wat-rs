#!/usr/bin/env bash
# gen-strat-neg.sh STRATA ITEMS — emit the Clara translation of grid/strat-neg.wat (axis A4:
# stratified negation) to stdout. SAME workload: Item(k) for k in [0,ITEMS); S0(k) :- Item(k)
# AND (k mod 2 == 0); Si(k) :- Item(k) AND NOT S(i-1)(k) for i in [1,STRATA). Clara resolves the
# non-monotonic negation-over-derived-facts via its TMS/retraction machinery (fire-rules runs the
# whole network to its natural fixpoint, re-evaluating :not against whatever S(i-1) facts already
# exist) — wat resolves the SAME semantics via explicit stratification (fire-stratified). Same
# final answer, different mechanism — that equivalence is the point of this axis.
#
# Unlike strat-neg.wat (which pre-declares a static S0..S9 ceiling — wat's stratifier needs
# distinct TYPES per level and defrecord is compile-time), Clara's defrecord/defrule/defquery are
# ordinary top-level forms this generator can emit exactly STRATA of, at any N — no ceiling here.
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires).
# Emits one #grid/Result EDN line with :derived canonically encoded exactly like the wat side
# (stratum * 1,000,000 + k, sorted ascending) so the two outputs compare byte-for-byte.
set -euo pipefail
N="$1"; M="$2"

cat <<HEADER
(ns strat-neg (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord Item [k])
HEADER

for i in $(seq 0 $((N - 1))); do
  echo "(defrecord S$i [k])"
done

echo "(defrule r0 [Item (= ?k k) (= (mod k 2) 0)] => (clara.rules/insert! (->S0 ?k)))"
for i in $(seq 1 $((N - 1))); do
  p=$((i - 1))
  cat <<RULE
(defrule r$i [Item (= ?k k)] [:not [S$p (= ?k k)]] => (clara.rules/insert! (->S$i ?k)))
RULE
done

for i in $(seq 0 $((N - 1))); do
  echo "(defquery q$i [] [S$i (= ?k k)])"
done

QVEC="[$(for i in $(seq 0 $((N - 1))); do printf 'q%s ' "$i"; done)]"

cat <<FOOTER
(def all-queries $QVEC)
(defn all-codes [s]
  (sort
    (mapcat
      (fn [i q] (map (fn [r] (+ (* i 1000000) (:?k r))) (query s q)))
      (range $N)
      all-queries)))
(defn -main [& _]
  (let [seeds (map ->Item (range $M))
        build (fn [] (apply insert (mk-session 'strat-neg :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"strat-neg\" :size [$N $M] :derived ["
                     (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
FOOTER
