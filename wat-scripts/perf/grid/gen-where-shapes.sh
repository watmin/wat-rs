#!/usr/bin/env bash
# gen-where-shapes.sh M ROW — emit the Clara translation of grid/where-shapes.wat (the `where`-clause
# EXPRESSIVITY axis) to stdout. SAME workload: Req(k) for k in [0,M); one rule
#   [Req (= ?k k)] [:test <the row's predicate>] => (insert! (->Hit ?k))
#
# THE POINT OF THIS AXIS: wat's `where` admits only PURE functions and Clara's `:test` admits
# arbitrary Clojure. This measures whether the SAME CONSTRAINT, expressed in each, derives the same
# facts — and the purity is what makes ours compilable (task #49a), which Clara's can never be.
#
# ROW 1 — arithmetic, 4 i64 ops over 1 bound var. The Clara side mirrors wat's ARITHMETIC EXACTLY
# — `(- ?k (* (quot ?k 10) 10))`, not idiomatic `(mod ?k 10)` — so the row measures the constraint
# rather than a translation choice. `quot` (not `/`) is the faithful counterpart of wat's `i64::/`:
# Clojure's `/` on two integers yields a RATIO, so `/` here would silently change the semantics.
# For k >= 0 both forms are exactly k mod 10.
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires). Emits one
# #grid/Result EDN line with :derived (sorted k) so the two outputs compare byte-for-byte.
set -euo pipefail
M="$1"; ROW="$2"

cat <<HEADER
(ns where-shapes (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord Req [k])
(defrecord Hit [k])
HEADER

# THE ROW DISPATCH — mirrors where-shapes.wat's `build-rules`. One shape per grid cell, so a
# :MISMATCH names the failing shape. An unknown row EXITS NON-ZERO; it never falls back to row 1,
# because a silent fallback would report a green cell for a shape nobody ran.
case "$ROW" in
  # ROW 1 — arithmetic (the shape every Step-0 number came from).
  1) echo '(defrule arith [Req (= ?k k)] [:test (= 3 (- ?k (* (quot ?k 10) 10)))] => (clara.rules/insert! (->Hit ?k)))' ;;
  *) echo "gen-where-shapes: unknown row $ROW" >&2; exit 2 ;;
esac

cat <<FOOTER
(defquery hit-q [] [Hit (= ?k k)])
(defn all-codes [s] (sort (map (fn [r] (:?k r)) (query s hit-q))))
(defn -main [& _]
  (let [seeds (map ->Req (range $M))
        build (fn [] (apply insert (mk-session 'where-shapes :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"where-shapes\" :size [$M $ROW] :derived ["
                     (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
FOOTER
