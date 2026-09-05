;; d2-derived-fact-axis.wat — RECONNAISSANCE for arc 278 D2's STOP-1: did the cure move any FACT?
;;
;; D2 duplicated TOKENS in a HashJoin's persistent right index. `seen_insert` dedups the derived
;; FACT set, so every prior end-to-end drive of D2 was blind to it by construction. The cure
;; (`session.rs`, `JoinRightIndex`) stops the duplication. The question this file answers is the
;; INVERSE one: now that the duplicate tokens are gone, does anything OBSERVABLE move?
;;
;; Two observables, because they fail differently:
;;   * `hits` / `hits2` — the derived FACT set. Deduped, so this was ALREADY correct and must not
;;     move. Movement here would be the cure dropping a real derivation.
;;   * `chain-rows` — a query whose `:when` MIRRORS the join chain, so it yields one row per
;;     TOKEN, not per fact. This is the multiplicity-sensitive column the grid does not have.
;;
;; Each is read twice: native `fire-rules` and the wat spec `fire-rules$oracle`. The oracle is
;; re-run-from-scratch, so it has no persistent right index and cannot carry this defect.
;;
;;   ./target/release/wat wat-scripts/scratch-pad/d2-derived-fact-axis.wat
;;
;; The shape is `filter -> HashJoin(a) -> HashJoin(b)` with the TWO-WAVE stagger — wave 1 seeds
;; A/B/C so the maintainer keys both joins, wave 2 arrives as DERIVED facts so `hash_join_delta`
;; sees a non-empty dright on an already-keyed join. Without the stagger the two writers never
;; meet on one index and the probe measures nothing (that vacuous partition is recorded in
;; `src/rete/kernel/tests/right_index_counter_invariant.rs`).

(:wat::core::defrecord :d2p::A [k <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :d2p::B [k <- :wat::core::i64])
(:wat::core::defrecord :d2p::C [k <- :wat::core::i64])
(:wat::core::defrecord :d2p::D [k <- :wat::core::i64])
(:wat::core::defrecord :d2p::M [k <- :wat::core::i64])
(:wat::core::defrecord :d2p::Hit  [k <- :wat::core::i64])
(:wat::core::defrecord :d2p::Hit2 [k <- :wat::core::i64])

(:wat::rete::defrule :d2p::derive-a
  :when [(:d2p::M (?k <- :k))]
  :then [(:d2p::A ?k (:wat::rete::core::i64::+ ?k 1 :undefined 0))])

(:wat::rete::defrule :d2p::derive-b
  :when [(:d2p::M (?k <- :k))]
  :then [(:d2p::B ?k)])

(:wat::rete::defrule :d2p::derive-c
  :when [(:d2p::M (?k <- :k))]
  :then [(:d2p::C ?k)])

(:wat::rete::defrule :d2p::derive-d
  :when [(:d2p::M (?k <- :k))]
  :then [(:d2p::D ?k)])

(:wat::rete::defrule :d2p::chain
  :when [(:d2p::A (?k <- :k) (?v <- :v))
         (:wat::rete::where (:wat::rete::core::i64::> ?v 0))
         (:d2p::B (?k <- :k))
         (:d2p::C (?k <- :k))]
  :then [(:d2p::Hit ?k)])

(:wat::rete::defrule :d2p::chain2
  :when [(:d2p::A (?k <- :k) (?v <- :v))
         (:wat::rete::where (:wat::rete::core::i64::> ?v 0))
         (:d2p::B (?k <- :k))
         (:d2p::D (?k <- :k))]
  :then [(:d2p::Hit2 ?k)])

;; The FACT observable — deduped by `seen_insert`, so blind to multiplicity by construction.
(:wat::rete::defquery :d2p::q-hit  :params [] :when [(:d2p::Hit  (?k <- :k))])
(:wat::rete::defquery :d2p::q-hit2 :params [] :when [(:d2p::Hit2 (?k <- :k))])

;; ★ The TOKEN observable — this `:when` mirrors `chain`'s own join chain, so a doubled right
;; bucket yields a doubled row count here even though the fact set is unchanged.
(:wat::rete::defquery :d2p::q-chain
  :params []
  :when [(:d2p::A (?k <- :k) (?v <- :v))
         (:wat::rete::where (:wat::rete::core::i64::> ?v 0))
         (:d2p::B (?k <- :k))
         (:d2p::C (?k <- :k))])

(:wat::core::defn :d2p::ins-a [s <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s (:d2p::A :k k :v (:wat::core::i64::+ k 1))) ((:wat::rete::InsertOutcome::Inserted __x) __x) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! "insert a: ceiling" :wat::core::None :wat::core::None))))
(:wat::core::defn :d2p::ins-b [s <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s (:d2p::B k)) ((:wat::rete::InsertOutcome::Inserted __x) __x) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! "insert b: ceiling" :wat::core::None :wat::core::None))))
(:wat::core::defn :d2p::ins-c [s <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s (:d2p::C k)) ((:wat::rete::InsertOutcome::Inserted __x) __x) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! "insert c: ceiling" :wat::core::None :wat::core::None))))
(:wat::core::defn :d2p::ins-m [s <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s (:d2p::M k)) ((:wat::rete::InsertOutcome::Inserted __x) __x) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! "insert m: ceiling" :wat::core::None :wat::core::None))))

(:wat::core::defn :d2p::wave1 [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session
      (:d2p::ins-c (:d2p::ins-b (:d2p::ins-a acc k) k) k))
    s
    (:wat::core::range 0 n)))

(:wat::core::defn :d2p::wave2 [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::Session  k <- :wat::core::i64] -> :wat::rete::Session
      (:d2p::ins-m acc k))
    s
    (:wat::core::range n (:wat::core::i64::* n 2))))

(:wat::core::defn :d2p::seed [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session
  (:d2p::wave2 (:d2p::wave1 s n) n))

(:wat::core::defn :d2p::rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:d2p::derive-a) (:d2p::derive-b) (:d2p::derive-c) (:d2p::derive-d)
    (:d2p::chain) (:d2p::chain2)))

(:wat::core::defn :d2p::queries [] -> (:wat::core::PersistentVector :- [:wat::rete::Query])
  (:wat::core::PersistentVector (:d2p::q-hit) (:d2p::q-hit2) (:d2p::q-chain)))

(:wat::core::defn :d2p::fresh [n <- :wat::core::i64] -> :wat::rete::Session
  (:d2p::seed
    (:wat::core::match (:wat::rete::compile-all (:d2p::rules) (:d2p::queries))
      ((:wat::rete::CompileOutcome::Compiled __s) __s)
      ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None)))
    n))

(:wat::core::defn :d2p::fire-native [n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules (:d2p::fresh n))
    ((:wat::rete::FireOutcome::Fired __f) __f)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r) (:wat::kernel::assertion-failed! "fire: ceiling" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __c __s) (:wat::kernel::assertion-failed! "fire: round cap" :wat::core::None :wat::core::None))))

(:wat::core::defn :d2p::fire-oracle [n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules$oracle (:d2p::fresh n))
    ((:wat::rete::FireOutcome::Fired __f) __f)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r) (:wat::kernel::assertion-failed! "fire: ceiling" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __c __s) (:wat::kernel::assertion-failed! "fire: round cap" :wat::core::None :wat::core::None))))

(:wat::core::defn :d2p::line
  [label <- :wat::core::String  hits <- :wat::core::i64  hits2 <- :wat::core::i64  rows <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::String/concat
      (:wat::core::String/concat label " Hit=")
      (:wat::core::String/concat
        (:wat::core::i64::to-string hits)
        (:wat::core::String/concat
          (:wat::core::String/concat " Hit2=" (:wat::core::i64::to-string hits2))
          (:wat::core::String/concat " chain-rows=" (:wat::core::i64::to-string rows)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n 6
     nat (:d2p::fire-native n)
     ora (:d2p::fire-oracle n)]
    (:wat::core::do
      (:d2p::line "native"
        (:wat::core::length (:wat::rete::query nat (:d2p::q-hit)))
        (:wat::core::length (:wat::rete::query nat (:d2p::q-hit2)))
        (:wat::core::length (:wat::rete::query nat (:d2p::q-chain))))
      (:d2p::line "oracle"
        (:wat::core::length (:wat::rete::query ora (:d2p::q-hit)))
        (:wat::core::length (:wat::rete::query ora (:d2p::q-hit2)))
        (:wat::core::length (:wat::rete::query ora (:d2p::q-chain)))))))
