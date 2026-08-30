;; tests/rete/probe_construction_headline_green.wat — BRIEF-construction-inside-a-fn.md, the
;; HEADLINE Stone B was blocked on (ac90d262): a `defn` that CONSTRUCTS and RETURNS a fresh
;; record from bound `:then` terms, not merely extracts an existing one
;; (`probe_arc278_then_user_forms_userfn.wat`'s own doc explains why THAT fixture had to work
;; around this exact gap). `:cg::make-rate`'s body is `(:cg::Rate :count c :window w)`, which
;; macro-expands to `:wat::core::kwargs-construct` — the verb classification this brief closes.

(:wat::core::defrecord :cg::Anchor [x <- :wat::core::i64])
(:wat::core::defrecord :cg::Rate   [count <- :wat::core::i64 window <- :wat::core::i64])

(:wat::rete::core::defn :cg::make-rate
  [c <- :wat::core::i64
   w <- :wat::core::i64]
  -> :cg::Rate
  (:cg::Rate :count c :window w))

(:wat::rete::defrule :cg::gather
  :when [(:cg::Anchor (?x <- :x))]
  :then [(:cg::make-rate 7 9)])

(:wat::rete::defquery :cg::q-Rate
  :params []
  :when [(:cg::Rate (?count <- :count))])


;; Fires via the WAT ORACLE.
(:wat::core::defn :user::run-oracle [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cg)
     session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cg::q-Rate))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:cg::Anchor :x 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules$oracle session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     derived (:wat::rete::query fired (:cg::q-Rate))
     r       (:wat::core::first derived)]
    (:wat::core::Option/expect
      (:wat::core::PersistentMap/get r "?count")
      "q-Rate: ?count")))

;; Fires via the NATIVE KERNEL — same rule, same expected value, through the compiled RHS path.
(:wat::core::defn :user::run-native [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cg)
     session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cg::q-Rate))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:cg::Anchor :x 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     derived (:wat::rete::query fired (:cg::q-Rate))
     r       (:wat::core::first derived)]
    (:wat::core::Option/expect
      (:wat::core::PersistentMap/get r "?count")
      "q-Rate: ?count")))
