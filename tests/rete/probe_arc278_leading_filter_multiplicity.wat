;; Fixture BESIDE probe_arc278_leading_filter_multiplicity.rs.
;;
;; THE CONTRACT: a LEADING (parentless) `:not` or `:exists` passes its token at
;; most ONCE per distinct inner binding, for the whole fire — not once per
;; fixpoint round. `fire/pass/filter.rs` states it outright: "ExistsNode binds
;; nothing and passes the token at most ONCE (no multiplicity)".
;;
;; WHY THIS EXISTS. That contract was broken and nothing in 5016 tests saw it.
;; The leading arms are re-evaluated every round of the delta fixpoint with no
;; round gating, and `wm.beta` is cumulative, so the token was appended again on
;; every round: a query over such a rule returned N rows where 1 is correct,
;; N being the number of rounds. Measured before the fix, by chain length:
;; 2 -> 2 rows, 3 -> 3, 4 -> 4, 6 -> 6. Exact, not approximate.
;;
;; WHY THE SUITE MISSED IT — and why this probe is shaped the way it is.
;; `production_delta` dedups DERIVED FACTS by value, so a rule's output set stays
;; correct and every oracle-differential passes regardless. The duplicates are
;; only observable through a QUERY that reads beta directly. And every existing
;; leading-`:not`/`:exists` test fires a single round, where N=1 and a per-round
;; re-emission is indistinguishable from a correct one.
;;
;; So this fixture is built to make the round count the ONLY variable: TWO
;; namespaces with identical queries and identical facts, differing solely in
;; the length of an inert S-chain that does nothing but force the fixpoint to
;; iterate. A fix that special-cases "round 0" passes `:lf2` and fails `:lf6`.
;;
;; No arithmetic anywhere: the `:then` purity fence rejects non-total ops like
;; `:wat::core::+`, so the chain is distinct record types rather than a counter.

;; ── :lf2 — a 2-link chain (2 rounds) ─────────────────────────────────────────

(:wat::core::defrecord :lf2::Wind [loc <- :wat::core::String])
(:wat::core::defrecord :lf2::Ghost [k <- :wat::core::i64])
(:wat::core::defrecord :lf2::S1 [k <- :wat::core::i64])
(:wat::core::defrecord :lf2::S2 [k <- :wat::core::i64])

(:wat::rete::defrule :lf2::r2 :when [(:lf2::S1 (?k <- :k))] :then [(:lf2::S2 :k ?k)])

;; Two Winds, ONE distinct loc => exactly one distinct inner binding.
(:wat::rete::defquery :lf2::q-exists :params []
  :when [(:wat::rete::exists (:lf2::Wind (?loc <- :loc)))])
;; Ghost is never asserted => the empty world matches with ONE empty token.
(:wat::rete::defquery :lf2::q-not :params []
  :when [(:wat::rete::not (:lf2::Ghost))])

;; ── :lf6 — a 6-link chain (6 rounds), otherwise identical ────────────────────

(:wat::core::defrecord :lf6::Wind [loc <- :wat::core::String])
(:wat::core::defrecord :lf6::Ghost [k <- :wat::core::i64])
(:wat::core::defrecord :lf6::S1 [k <- :wat::core::i64])
(:wat::core::defrecord :lf6::S2 [k <- :wat::core::i64])
(:wat::core::defrecord :lf6::S3 [k <- :wat::core::i64])
(:wat::core::defrecord :lf6::S4 [k <- :wat::core::i64])
(:wat::core::defrecord :lf6::S5 [k <- :wat::core::i64])
(:wat::core::defrecord :lf6::S6 [k <- :wat::core::i64])

(:wat::rete::defrule :lf6::r2 :when [(:lf6::S1 (?k <- :k))] :then [(:lf6::S2 :k ?k)])
(:wat::rete::defrule :lf6::r3 :when [(:lf6::S2 (?k <- :k))] :then [(:lf6::S3 :k ?k)])
(:wat::rete::defrule :lf6::r4 :when [(:lf6::S3 (?k <- :k))] :then [(:lf6::S4 :k ?k)])
(:wat::rete::defrule :lf6::r5 :when [(:lf6::S4 (?k <- :k))] :then [(:lf6::S5 :k ?k)])
(:wat::rete::defrule :lf6::r6 :when [(:lf6::S5 (?k <- :k))] :then [(:lf6::S6 :k ?k)])

(:wat::rete::defquery :lf6::q-exists :params []
  :when [(:wat::rete::exists (:lf6::Wind (?loc <- :loc)))])
(:wat::rete::defquery :lf6::q-not :params []
  :when [(:wat::rete::not (:lf6::Ghost))])

;; ── the witnesses ────────────────────────────────────────────────────────────

(:wat::core::defn :lf2::rows [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::let [rules (:wat::rete::collect-rules :lf2)
                    s0    (:wat::rete::compile-all rules
                            (:wat::core::PersistentVector (:lf2::q-exists) (:lf2::q-not)))
                    s1    (:wat::rete::insert-all s0
                            (:wat::core::PersistentVector (:lf2::Wind "MCI") (:lf2::Wind "MCI")))
                    s2    (:wat::rete::insert-all s1
                            (:wat::core::PersistentVector (:lf2::S1 1)))
                    fired (:wat::core::match (:wat::rete::fire-rules s2) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::mapv
      (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)
      (:wat::core::PersistentVector
        (:wat::core::length (:wat::rete::query fired (:lf2::q-exists)))
        (:wat::core::length (:wat::rete::query fired (:lf2::q-not)))))))

(:wat::core::defn :lf6::rows [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::let [rules (:wat::rete::collect-rules :lf6)
                    s0    (:wat::rete::compile-all rules
                            (:wat::core::PersistentVector (:lf6::q-exists) (:lf6::q-not)))
                    s1    (:wat::rete::insert-all s0
                            (:wat::core::PersistentVector (:lf6::Wind "MCI") (:lf6::Wind "MCI")))
                    s2    (:wat::rete::insert-all s1
                            (:wat::core::PersistentVector (:lf6::S1 1)))
                    fired (:wat::core::match (:wat::rete::fire-rules s2) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::mapv
      (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)
      (:wat::core::PersistentVector
        (:wat::core::length (:wat::rete::query fired (:lf6::q-exists)))
        (:wat::core::length (:wat::rete::query fired (:lf6::q-not)))))))

;; [exists@2rounds, not@2rounds, exists@6rounds, not@6rounds] — all must be 1.
(:wat::core::defn :user::leading-rows [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::into (:lf2::rows) (:lf6::rows)))
