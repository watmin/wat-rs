;; ⛔ THIS PROBE REFUTES THREE RECORDED BLOCKERS. Run 2026-08-17, output "3,4".
;;
;; src/collection/infer.rs:638 says `Seqable` is "the type wat cannot currently spell" and lists
;; three blockers, "none is a small fix". Measured against the disk, all three are STALE:
;;
;;   1. "no defsurface :nature admits a builtin container (only Record and Peer')"
;;        -> REFUTED: `:nature :wat::core::Struct` + extend-type on TWO builtins below.
;;   2. "no builtin (Vector/PersistentVector/List) satisfies any surface today"
;;        -> REFUTED twice over, below.
;;   3. "wat has no ad-hoc unions - a bound over four concrete builtins is structurally a union"
;;        -> DISSOLVED: it is not a union. It is N extend-types of ONE surface. Clojure's ISeq.
;;
;; The comment was written 2026-07-31. `SCORE-293.4d` -- whose acceptance demo extends
;; `:wat::core::Vector` to a user surface and calls a fn typed by that surface -- went GREEN
;; 2026-06-28, A MONTH EARLIER. tests/types/probe_arc293_acceptance_demo.wat is that test; this
;; file is the same program with Shape->Seqable and area->as-vec.
;;
;; ⚠ WHAT THIS DOES *NOT* PROVE: only Vector + PersistentVector are extended here (List and
;; Stream are untested); the surface is NOT parametric (hardcoded Vector<i64>, not Seqable<T>);
;; and per-element dispatch cost is UNMEASURED. Those three are the real remaining work.

;; DISCONFIRMING PROBE — can `Seqable` be spelled in wat TODAY?
;; Mirrors tests/types/probe_arc293_acceptance_demo.wat exactly, renamed.
(:wat::core::defsurface :sq::Seqable
  :nature :wat::core::Struct
  :features [(as-vec [self <- :sq::Seqable] -> (:wat::core::Vector :- [:wat::core::i64]))])

(:wat::core::extend-type :wat::core::Vector :sq::Seqable
  (as-vec [self] -> (:wat::core::Vector :- [:wat::core::i64]) self))

(:wat::core::extend-type :wat::core::PersistentVector :sq::Seqable
  (as-vec [self] -> (:wat::core::Vector :- [:wat::core::i64])
    (:wat::core::into (:wat::core::Vector :wat::core::i64) self)))

;; the payoff: ONE function over ANY Seqable — what join/map/filter want
(:wat::core::defn :sq::count-of [s <- :sq::Seqable] -> :wat::core::i64
  (:wat::core::length (:sq::Seqable/as-vec s)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::string::join ","
      (:wat::core::Vector :wat::core::i64
        (:sq::count-of (:wat::core::Vector :wat::core::i64 10 20 30))
        (:sq::count-of (:wat::core::PersistentVector 1 2 3 4))))))
