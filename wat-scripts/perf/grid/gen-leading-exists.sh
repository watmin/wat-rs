#!/usr/bin/env bash
# gen-leading-exists.sh ITEMS — emit the Clara translation of grid/leading-exists.wat
# (a LEADING :exists observed through a QUERY, across a multi-round fixpoint) to stdout.
#
# SAME workload: Wind(loc) for loc in [0,ITEMS), each asserted TWICE; one S1 seed carried
# to S6 by five inert cascade rules. The witness is the sorted DISTINCT ?loc bound by a
# leading :exists query — exactly [0..ITEMS) on both sides.
#
# WHY CLARA IS THE RIGHT REFERENCE HERE, verified rather than assumed (Clara 0.24.0 on
# this machine): a leading `[:exists [Wind (= ?loc loc)]]` in a defquery BINDS ?loc
# outward and yields ONE row per DISTINCT loc — 5 Winds over 3 distinct locs => 3 rows,
# {:?loc "A"}. That is the same semantics wat implements ("two Winds at MCI => one
# {?loc MCI}"), so the two :derived vectors are comparable byte-for-byte.
#
# WHAT THIS AXIS IS FOR. A defect found 2026-08-24: wat's leading :not/:exists emitted one
# token PER FIXPOINT ROUND into a cumulative beta, so this query returned ROUNDS x ITEMS
# rows. Clara does NOT share the flaw (1 activation with an unrelated cascade running,
# measured), and neither does the wat $oracle (immune by construction — it rebuilds alpha
# and beta from empty every fire). Both references were right; the corpus simply had no
# case of this shape. Now it does.
#
# NOTE ON THE CASCADE: the S1..S6 chain is inert and touches nothing Wind-related. Clara is
# incremental rather than round-based, so the chain does not create "rounds" for Clara the
# way it does for wat's batch fixpoint — it is carried across so BOTH sides run the identical
# workload, and so the wat side's row count is provably independent of it.
#
# Clara's :exists is implemented over accumulators, so clara.rules.accumulators must be
# loaded — without it the run dies with ClassNotFoundException, not a wrong answer.
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires).
set -euo pipefail
M="$1"

cat <<PROGRAM
(ns leading-exists
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators]))
(defrecord Wind [loc])
(defrecord S1 [k]) (defrecord S2 [k]) (defrecord S3 [k])
(defrecord S4 [k]) (defrecord S5 [k]) (defrecord S6 [k])
(defrule r2 [S1 (= ?k k)] => (insert! (->S2 ?k)))
(defrule r3 [S2 (= ?k k)] => (insert! (->S3 ?k)))
(defrule r4 [S3 (= ?k k)] => (insert! (->S4 ?k)))
(defrule r5 [S4 (= ?k k)] => (insert! (->S5 ?k)))
(defrule r6 [S5 (= ?k k)] => (insert! (->S6 ?k)))
(defquery q-exists [] [:exists [Wind (= ?loc loc)]])
(defn all-locs [s] (sort (map :?loc (query s q-exists))))
(defn -main [& _]
  (let [winds (mapcat (fn [i] [(->Wind i) (->Wind i)]) (range $M))
        seeds (concat winds [(->S1 1)])
        build (fn [] (apply insert (mk-session 'leading-exists :cache false) seeds))]
    (dotimes [_ 3] (count (all-locs (fire-rules (build)))))   ; JIT + compile warmup
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)
          locs (all-locs f)]
      (println (str "#grid/Result {:axis \"leading-exists\" :size [$M] :derived ["
                     (clojure.string/join " " locs) "] :clara-ns " (- t1 t0) "}")))))
PROGRAM
