#!/usr/bin/env bash
# gen-user-reduce.sh LOCS READS — emit the Clara translation of grid/user-reduce.wat (axis A6:
# user reducers / custom accumulator) to stdout. SAME workload: Station(loc) for loc in [0,LOCS);
# Reading(loc, (mod (+ loc j) 7)) for j in [0,READS); Agg(loc, s) :- Station(loc) AND
# (?s <- sum-of-squares :from Reading(loc)), where sum-of-squares = Σ v².
#
# SHAPE CAVEAT (CLARA-TRANSLATIONS.md A6c): wat's custom fold is a BATCH fold over PV<T>; Clara's
# `acc/accum` is a STREAMING reduce (reduce-fn per fact, + combine-fn). The two mechanisms differ —
# this axis compares FINAL RESULTS ONLY. sum-of-squares IS incrementally decomposable, so both
# engines compute the IDENTICAL population Σv² per location. `accum`'s reduce-fn receives each
# Reading FACT (verified empirically against clara 0.24.0), so it extracts :value itself.
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires).
# Emits one #grid/Result EDN line with :derived canonically encoded exactly like the wat side
# (loc * 1,000,000 + s, sorted ascending) so the two outputs compare byte-for-byte.
set -euo pipefail
L="$1"; R="$2"

cat <<FOOTER
(ns user-reduce (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
                          [clara.rules.accumulators :as acc]))
(defrecord Station [loc])
(defrecord Reading [loc value])
(defrecord Agg [loc sos])

;; the USER custom fold: Σ v² over the gathered Readings. accum's reduce-fn gets each Reading fact,
;; so it extracts :value. combine-fn + merges partial sums; convert-return-fn is identity.
(def sum-of-squares
  (acc/accum {:initial-value 0
              :reduce-fn (fn [total reading] (+ total (* (:value reading) (:value reading))))
              :combine-fn +
              :convert-return-fn identity}))

(defrule flag
  [Station (= ?loc loc)]
  [?s <- sum-of-squares :from [Reading (= ?loc loc)]]
  => (insert! (->Agg ?loc ?s)))

(defquery q-agg [] [Agg (= ?loc loc) (= ?s sos)])

(defn all-codes [s]
  (sort (map (fn [r] (+ (* (:?loc r) 1000000) (:?s r))) (query s q-agg))))

(defn -main [& _]
  (let [seeds (mapcat
                (fn [loc]
                  (cons (->Station loc)
                        (map (fn [j] (->Reading loc (mod (+ loc j) 7))) (range $R))))
                (range $L))
        build (fn [] (apply insert (mk-session 'user-reduce :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"user-reduce\" :size [$L $R] :derived ["
                     (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
FOOTER
