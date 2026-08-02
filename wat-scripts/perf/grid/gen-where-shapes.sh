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
# ROWS 2-5 (2026-08-01, BRIEF-where-shapes-rows-2-5.md): record accessor · String verb · collection
# verb · a user-defined pure fn. All rows bind the SAME four fields (?k ?c ?n ?t) off one leading
# [Req ...] condition, mirroring where-shapes.wat's rule 1 (bind every field once, never again per
# row) — only the trailing :test differs.
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires). Emits one
# #grid/Result EDN line with :derived (sorted k) so the two outputs compare byte-for-byte.
set -euo pipefail
M="$1"; ROW="$2"

cat <<HEADER
(ns where-shapes (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord Client [rep])
(defrecord Req [k client name tags])
(defrecord Hit [k])

;; row 5's user-defined pure fn, mirroring wsh::big? EXACTLY: k mod 7 > 3 via quot, not idiomatic
;; mod, for the same reason row 1's arithmetic uses quot (Clojure's / on ints yields a ratio).
(defn big? [k] (> (- k (* (quot k 7) 7)) 3))
HEADER

# THE SHARED LEADING CONDITION — every row binds all four fields off one [Req ...] pattern, so a
# shape can only ever perturb its own trailing :test.
REQ_COND='[Req (= ?k k) (= ?c client) (= ?n name) (= ?t tags)]'

# THE ROW DISPATCH — mirrors where-shapes.wat's `build-rules`. One shape per grid cell, so a
# :MISMATCH names the failing shape. An unknown row EXITS NON-ZERO; it never falls back to row 1,
# because a silent fallback would report a green cell for a shape nobody ran.
case "$ROW" in
  # ROW 1 — arithmetic (the shape every Step-0 number came from).
  1) echo "(defrule arith $REQ_COND [:test (= 3 (- ?k (* (quot ?k 10) 10)))] => (clara.rules/insert! (->Hit ?k)))" ;;
  # ROW 2 — record accessor: rep(k) = (k mod 5) - 2; rep > 0 selects k mod 5 in {3,4}.
  2) echo "(defrule accessor $REQ_COND [:test (> (:rep ?c) 0)] => (clara.rules/insert! (->Hit ?k)))" ;;
  # ROW 3 — String verb: name(k) = \"ad\"+k when k mod 3 == 0 else \"zz\"+k.
  3) echo "(defrule string $REQ_COND [:test (clojure.string/starts-with? ?n \"ad\")] => (clara.rules/insert! (->Hit ?k)))" ;;
  # ROW 4 — collection verb: tags(k) has length (k mod 4); count > 1 selects k mod 4 in {2,3}.
  4) echo "(defrule collection $REQ_COND [:test (> (count ?t) 1)] => (clara.rules/insert! (->Hit ?k)))" ;;
  # ROW 5 — a user-defined pure fn, not an inline expression — the shape a compiled executor
  # cannot model and must hand back to the interpreter.
  5) echo "(defrule userfn $REQ_COND [:test (big? ?k)] => (clara.rules/insert! (->Hit ?k)))" ;;
  *) echo "gen-where-shapes: unknown row $ROW" >&2; exit 2 ;;
esac

cat <<FOOTER
(defquery hit-q [] [Hit (= ?k k)])
(defn all-codes [s] (sort (map (fn [r] (:?k r)) (query s hit-q))))
;; seed-req i — the SAME formulas as wsh::seed, computed independently rather than kept as a
;; hand-synced table (brief rule 3): rep = (i mod 5) - 2; name = "ad"+i / "zz"+i by i mod 3; tags =
;; a vector of length (i mod 4), contents [0, len).
(defn seed-req [i]
  (let [rep      (- (- i (* (quot i 5) 5)) 2)
        is-ad    (= 0 (- i (* (quot i 3) 3)))
        nm       (str (if is-ad "ad" "zz") i)
        tags-len (- i (* (quot i 4) 4))]
    (->Req i (->Client rep) nm (vec (range tags-len)))))
(defn -main [& _]
  (let [seeds (map seed-req (range $M))
        build (fn [] (apply insert (mk-session 'where-shapes :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"where-shapes\" :size [$M $ROW] :derived ["
                     (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
FOOTER
