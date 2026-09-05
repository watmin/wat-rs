;; VIGILIA experiri probe — the join_keys_cache latch vs left_idx.
;;
;; SHAPE: [A ?k] :where [B ?k] [C ?k ?v]  — a guard followed by TWO fact conditions,
;; so the chain is  Root(A) -> Test(:where) -> HashJoin(B) -> HashJoin(C).
;; Pass 3.6 left-activates HashJoin(B); pass 3.7's `left_activate_join` left-activates
;; HashJoin(C). BOTH go through `keyed_join_persistent`, which writes
;; `join_keys_cache[C]` and NEVER writes `left_idx[C]`.
;;
;; Round 2 brings a DERIVED C for a key already seen. C's beta parent is a HashJoin, so
;; pass 3 (`hash_join_delta`) visits it; `join_keys_cache.contains_key(C)` is already
;; true, so `first_keying` is false and the catch-up (the only bulk builder of
;; `left_idx`) is skipped. Step 4's `left_idx.get(C)` is then a silent None and
;; `term2 = old_left join dr` never runs.
;;
;; CONTROL `plain` is the SAME data through a chain with NO guard, so pass 3 owns the
;; whole chain and its catch-up builds left_idx. It must be 2 on both engines: it proves
;; the fixture really reaches a second round with a non-empty dr and a non-empty old_left.

(:wat::core::defrecord :vlx::A  [k <- :wat::core::i64  g <- :wat::core::String])
(:wat::core::defrecord :vlx::A2 [k <- :wat::core::i64])
(:wat::core::defrecord :vlx::B  [k <- :wat::core::i64])
(:wat::core::defrecord :vlx::C  [k <- :wat::core::i64  v <- :wat::core::i64])
(:wat::core::defrecord :vlx::T  [k <- :wat::core::i64])
(:wat::core::defrecord :vlx::OutW [v <- :wat::core::i64])
(:wat::core::defrecord :vlx::OutP [v <- :wat::core::i64])
(:wat::core::defrecord :vlx::A3 [k <- :wat::core::i64])
(:wat::core::defrecord :vlx::Neg [k <- :wat::core::i64])
(:wat::core::defrecord :vlx::OutN [v <- :wat::core::i64])

;; Derives a SECOND C for a key round 1 already joined — one round later.
(:wat::rete::defrule :vlx::derive-c
  :when [(:vlx::T (?k <- :k))]
  :then [(:vlx::C :k ?k :v 20)])

;; THE SUBJECT: guard, then two fact conditions.
(:wat::rete::defrule :vlx::main-where
  :when [(:vlx::A (?k <- :k) (?g <- :g))
         (:wat::rete::where (:wat::rete::core::string::= ?g "yes"))
         (:vlx::B (?k <- :k))
         (:vlx::C (?k <- :k) (?v <- :v))]
  :then [(:vlx::OutW :v ?v)])

;; THE CONTROL: same three facts, no guard.
(:wat::rete::defrule :vlx::main-plain
  :when [(:vlx::A2 (?k <- :k))
         (:vlx::B (?k <- :k))
         (:vlx::C (?k <- :k) (?v <- :v))]
  :then [(:vlx::OutP :v ?v)])

;; THE SECOND SUBJECT: the filter is a `:not`, not a `:where`. Pass 3.6 walks
;; `filter_or_acc` = Test | Negation | Exists | Accumulate, so a `:not` followed by two
;; fact conditions reaches the same `keyed_join_persistent` latch.
(:wat::rete::defrule :vlx::main-not
  :when [(:vlx::A3 (?k <- :k))
         (:wat::rete::not (:vlx::Neg (?k <- :k)))
         (:vlx::B (?k <- :k))
         (:vlx::C (?k <- :k) (?v <- :v))]
  :then [(:vlx::OutN :v ?v)])

(:wat::rete::defquery :vlx::q-n :params [] :when [(?f <- :vlx::OutN)])
(:wat::rete::defquery :vlx::q-w :params [] :when [(?f <- :vlx::OutW)])
(:wat::rete::defquery :vlx::q-p :params [] :when [(?f <- :vlx::OutP)])
(:wat::rete::defquery :vlx::q-c :params [] :when [(?f <- :vlx::C)])

(:wat::core::defn :vlx::ins [s <- :wat::rete::Session f <- :wat::core::Record] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s f)
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
      (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None))))

(:wat::core::defn :vlx::staged [] -> :wat::rete::Session
  (:vlx::ins (:vlx::ins (:vlx::ins (:vlx::ins (:vlx::ins
    (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :vlx)
      (:wat::core::PersistentVector (:vlx::q-w) (:vlx::q-p) (:vlx::q-c) (:vlx::q-n)))
      ((:wat::rete::CompileOutcome::Compiled __session) __session)
      ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type)
        (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
    (:vlx::A :k 1 :g "yes")) (:vlx::A2 :k 1)) (:vlx::B :k 1)) (:vlx::C :k 1 :v 10)) (:vlx::T :k 1)))

(:wat::core::defn :vlx::staged2 [] -> :wat::rete::Session
  (:vlx::ins (:vlx::staged) (:vlx::A3 :k 1)))

(:wat::core::defn :vlx::counts [s <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector
    (:wat::core::length (:wat::rete::query s (:vlx::q-w)))
    (:wat::core::length (:wat::rete::query s (:vlx::q-p)))
    (:wat::core::length (:wat::rete::query s (:vlx::q-c)))
    (:wat::core::length (:wat::rete::query s (:vlx::q-n)))))

;; [guard-chain, no-guard-control, C population] x [native, oracle]
(:wat::core::defn :user::native-and-oracle [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::mapv
    (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)
    (:wat::core::PersistentVector/concat
      (:vlx::counts (:wat::core::match (:wat::rete::fire-rules (:vlx::staged2))
        ((:wat::rete::FireOutcome::Fired __fired) __fired)
        ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
          (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
        ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
          (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))
      (:vlx::counts (:wat::core::match (:wat::rete::fire-rules$oracle (:vlx::staged2))
        ((:wat::rete::FireOutcome::Fired __fired) __fired)
        ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
          (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None))
        ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
          (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))))
