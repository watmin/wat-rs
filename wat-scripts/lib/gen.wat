;; wat-scripts/lib/gen.wat — FINITE GENERATORS: the generic core of generative testing, in wat.
;;
;; A generator is an INDEXED SET, not a seeded random source:
;;
;;     Gen<T> = { card : i64,  at : i64 -> T }
;;
;; That one choice is what the whole design turns on, and it differs deliberately from the
;; QuickCheck / `clojure.test.check` lineage this borrows from (`gen/elements`, `gen/fmap`,
;; `gen/tuple` map onto the verbs below). Because `at` is a total function of an index, three
;; operations that are separate machinery there collapse into one here:
;;
;;   ENUMERATE  iterate 0..card                 — exhaustive whenever the space fits
;;   SAMPLE     pick any i < card               — uniform, and reproducible by construction
;;   SHRINK     walk a coordinate's digits down — index arithmetic, not tree surgery
;;
;; And a failing case gets a PERMANENT name. A `test.check` seed is meaningless the moment the
;; generator changes; a coordinate like `[3 1 0 2]` still dials in the same case.
;;
;; The cost, stated plainly: every dimension must be BOUNDED. You cannot generate an unbounded
;; structure. For differential testing against a slow reference that is a feature, not a limit —
;; it is what keeps the oracle affordable.
;;
;; `defstruct`, not `defrecord`: a Gen carries a FUNCTION, and the containment rule (arc 293.W)
;; holds that a pure aggregate must survive an EDN round-trip across a comms boundary. A
;; generator is local computation and never crosses one. The checker names this itself if you
;; try — it is a good error.
;;
;; Namespace: :user:: (the only writable prefix for scripts outside wat/ stdlib), name-prefixed
;; `gen-` in the wat-grep style.
;; Loaded by: (:wat::load-file! "../lib/gen.wat")

(:wat::core::defstruct :user::Gen :- [T]
  [card <- :wat::core::i64
   at   <- [:wat::core::i64 :-> T]])

;; ── index arithmetic ─────────────────────────────────────────────────────────
;; No native i64 mod/rem (only + - * /), so mod is the truncating-division idiom
;; the grid axes already use. Both args are non-negative at every call here.
(:wat::core::defn :user::gen-digit [i <- :wat::core::i64  base <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::- i (:wat::core::i64::* (:wat::core::i64::/ i base) base)))

(:wat::core::defn :user::gen-shift [i <- :wat::core::i64  base <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::/ i base))

;; ── the primitive generator ──────────────────────────────────────────────────
(:wat::core::defn :user::gen-ints [lo <- :wat::core::i64  hi <- :wat::core::i64]
  -> (:user::Gen :- [:wat::core::i64])
  (:user::Gen :card (:wat::core::i64::- hi lo)
              :at   (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64
                      (:wat::core::i64::+ lo i))))

;; ── fmap: reshape what a generator yields, keeping its cardinality ────────────
(:wat::core::defn :user::gen-fmap :- [A B]
  [f <- [A :-> B]  g <- (:user::Gen :- [A])] -> (:user::Gen :- [B])
  (:wat::core::let [inner (:user::Gen/at g)]
    (:user::Gen :card (:user::Gen/card g)
                :at   (:wat::core::fn [i <- :wat::core::i64] -> B (f (inner i))))))

;; ── the workhorse: a COORDINATE generator over mixed bases ───────────────────
;; `gen-coords [b0 b1 b2]` has card b0*b1*b2 and yields [d0 d1 d2] with di < bi —
;; positional notation in mixed radix. This is `gen/tuple` for the enumerable
;; case, and it is what a target actually wants: one index in, its own tuple of
;; dimension choices out, with no heterogeneous tuple type needed.
(:wat::core::defstruct :user::GenAcc
  [rem <- :wat::core::i64
   out <- (:wat::core::PersistentVector :- [:wat::core::i64])])

(:wat::core::defn :user::gen-coords [bases <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> (:user::Gen :- [(:wat::core::PersistentVector :- [:wat::core::i64])])
  (:user::Gen
    :card (:wat::core::foldl
            (:wat::core::fn [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::* a b))
            1 bases)
    :at (:wat::core::fn [i <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::i64])
          (:user::GenAcc/out
            (:wat::core::foldl
              (:wat::core::fn [acc <- :user::GenAcc  b <- :wat::core::i64] -> :user::GenAcc
                (:user::GenAcc
                  :rem (:user::gen-shift (:user::GenAcc/rem acc) b)
                  :out (:wat::core::PersistentVector/conj (:user::GenAcc/out acc)
                         (:user::gen-digit (:user::GenAcc/rem acc) b))))
              (:user::GenAcc :rem i :out (:wat::core::PersistentVector))
              bases)))))

;; ── the driver ───────────────────────────────────────────────────────────────
;; `prop` returns 0 for a pass and 1 for a failure, and OWNS its own reporting —
;; it is the only party that knows what its values mean. The driver's job is to
;; walk the space and tally, nothing more. A target that wants its coordinate in
;; the report generates coordinates (`gen-coords`) and prints them itself.
(:wat::core::defn :user::gen-check :- [T]
  [g <- (:user::Gen :- [T])  prop <- [T :-> :wat::core::i64]] -> :wat::core::i64
  (:wat::core::let [at (:user::Gen/at g)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::+ acc (prop (at i))))
      0
      (:wat::core::range 0 (:user::Gen/card g)))))
