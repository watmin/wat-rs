#!/usr/bin/env bash
# gen-negation.sh ITEMS — emit the Clara translation of grid/negation.wat (axis A3: plain :not
# negation, single-stratum) to stdout. SAME workload: Item(k) for k in [0,ITEMS); Bad(k) seeded
# for every EVEN k; Ok(k) :- Item(k) AND NOT Bad(k). Ok fires for exactly the ODD k.
#
# Clara resolves the negation via its INCREMENTAL truth-maintenance machinery (each Bad insert
# live-retracts any Ok token already fired for that ?k); wat resolves the SAME final set via a
# single stratified batch fixpoint (the Ok rule sits one stratum above the base Bad facts). Same
# answer, different mechanism — that equivalence, and the mechanism gap, is the point of this axis
# (see CLARA-TRANSLATIONS.md A3).
#
# Bad is seeded as an INPUT fact here (not derived) — that is what makes the negation single-stratum
# and distinguishes A3 from the multi-stratum foundation axis (strat-neg). The Clara side seeds Bad
# the same way (a plain fact, no mark-bad rule).
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires). Emits one
# #grid/Result EDN line with :derived (the sorted Ok keys) so the two outputs compare byte-for-byte.
set -euo pipefail
M="$1"

cat <<PROGRAM
(ns negation (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord Item [k])
(defrecord Bad [k])
(defrecord Ok [k])
(defrule ok [Item (= ?k k)] [:not [Bad (= ?k k)]] => (clara.rules/insert! (->Ok ?k)))
(defquery q [] [Ok (= ?k k)])
(defn all-codes [s] (sort (map :?k (query s q))))
(defn -main [& _]
  (let [items (map ->Item (range $M))
        bads  (map ->Bad (filter even? (range $M)))
        seeds (concat items bads)
        build (fn [] (apply insert (mk-session 'negation :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"negation\" :size [$M] :derived ["
                     (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
PROGRAM
