;; BENCH — 118.B7. Did collapsing `reduce` to two `Seqable<T>` clauses cost anything on an EAGER
;; container?
;;
;; `reduce`'s 3-arity body is now `(foldl f init coll)` with `coll` declared `Seqable<T>`. The
;; question this answers: does declaring the parameter as the SURFACE change what the native does?
;; It must not — `Seqable<T>` is a STATIC type, the value arriving is a concrete Vector, and
;; `foldl` classifies the VALUE. So `reduce` should cost `foldl` plus one clause dispatch, and no
;; more.
;;
;; ⛔ THE FAILURE THIS GUARDS. The obvious way to write that body is
;; `(foldl f init (Seqable/seq coll))` — normalise, then fold. That type-checks, and it forces every
;; eager reduce onto the lazy Stream path for a Stream it never needed. The arm added to
;; `extract_lazyable_elem` (accepting `Seqable<T>` itself) is what makes the direct spelling legal;
;; without it the normalising spelling is the ONLY one that compiles, and the tax comes back in
;; through the front door. If this bench ever shows `reduce` tracking the SPEC rather than `foldl`,
;; that is what happened.
;;
;; Shape discipline: fixed n, BOTH block orderings, non-vacuity proving all arms agree.

(:wat::core::defn :bench::add [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::i64::+ acc x))

(:wat::core::defn :bench::via-reduce [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::reduce :bench::add 0 v))

(:wat::core::defn :bench::via-foldl [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl :bench::add 0 v))

;; the wat oracle, as the SLOW reference — reduce must track foldl, not this.
(:wat::core::defn :bench::via-spec [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl-spec :bench::add 0 v))

(:wat::core::defn :bench::ns [t0 <- :wat::time::Instant t1 <- :wat::time::Instant] -> :wat::core::i64
  (:wat::core::i64::- (:wat::time::epoch-nanos t1) (:wat::time::epoch-nanos t0)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n  200000
     v  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::range 0 n))
     a0 (:wat::time::now) ra (:bench::via-reduce v) a1 (:wat::time::now)
     b0 (:wat::time::now) rb (:bench::via-foldl v)  b1 (:wat::time::now)
     c0 (:wat::time::now) rc (:bench::via-spec v)   c1 (:wat::time::now)
     d0 (:wat::time::now) rd (:bench::via-foldl v)  d1 (:wat::time::now)
     e0 (:wat::time::now) re (:bench::via-reduce v) e1 (:wat::time::now)]
    (:wat::kernel::println
      (:wat::core::string::interpolate
        "n={n} NONVACUITY ra={ra} rb={rb} rc={rc} rd={rd} re={re} | reduce={ad}ms foldl={bd}ms spec={cd}ms | foldl={dd}ms reduce={ed}ms"
        :n n :ra ra :rb rb :rc rc :rd rd :re re
        :ad (:wat::core::i64::/ (:bench::ns a0 a1) 1000000)
        :bd (:wat::core::i64::/ (:bench::ns b0 b1) 1000000)
        :cd (:wat::core::i64::/ (:bench::ns c0 c1) 1000000)
        :dd (:wat::core::i64::/ (:bench::ns d0 d1) 1000000)
        :ed (:wat::core::i64::/ (:bench::ns e0 e1) 1000000)))))
