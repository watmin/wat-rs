;; RUNAWAY, WITH THE CAP TUNED DOWN — proves the knob reaches the engine.
;;
;; `N(k)` derives `N(k+1)` with no guard, so every round mints a structurally novel fact and the
;; dedup that bounds a Datalog fixpoint never bites. This is the exact shape
;; `DESIGN-STONE-4b-cascade-fixpoint` NAMED and deferred a cap for ("let need reveal"); the need
;; revealed. Before the cap (2026-08-27) this died on
;; `memory allocation of 545259536 bytes failed` — no wat error, no span, no rule named, and with
;; no ulimit that is the machine's memory.
;;
;; Its twin `probe_arc278_fixpoint_round_cap_deep.wat` is THIS FILE PLUS ONE `:where`, and must
;; still succeed. The pair is the test: the cap must catch divergence without capping depth.
;; A round count cannot distinguish DEEP from DIVERGENT — transitive closure over a 50_000-node
;; path is legitimate Datalog needing 50_000 rounds, while the cap must stay low enough to fire
;; before the allocator does. No single number is right for both, which is why this is a
;; per-program value in the `dim-count` mould rather than a hard constant.
(:wat::config::set-max-fire-rounds! 25)

(:wat::core::defrecord :cap::N [k <- :wat::core::i64])

(:wat::rete::defrule :cap::grow
  :when [(:cap::N (?k <- :k))]
  :then [(:cap::N :k (:wat::rete::core::i64::+ ?k 1 :undefined 0))])

(:wat::rete::defquery :cap::q :params [] :when [(?fact <- :cap::N)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::i64::to-string
      (:wat::core::length
        (:wat::rete::query
          (:wat::rete::fire-rules
            (:wat::rete::insert
              (:wat::rete::compile-all
                (:wat::core::PersistentVector (:cap::grow))
                (:wat::core::PersistentVector (:cap::q)))
              (:cap::N :k 0)))
          (:cap::q))))))
