;; tests/rete/probe_constructor_meta_enum_variant_green.wat — BRIEF-construction-total-three-
;; walls.md #3, the "STILL WORKS" counterpart to `probe_constructor_meta_surface_total_enum.wat.bad`
;; (the wrong-arity REJECT proof). A CORRECT-arity nested tagged-enum-variant constructor
;; (`(:cg::Status::Active 7)` — `Active` declares exactly one field) must compile AND fire,
;; through both the oracle and the native kernel — the new freeze-time arity wall
;; (`walk_nested_constructors`, `src/rete/validate.rs`) must not reject a legal call.

(:wat::core::defenum :cg::Status :wat::enum::Pure
  :Active [level <- :wat::core::i64])

(:wat::core::defrecord :cg::Anchor [x <- :wat::core::i64])
(:wat::core::defrecord :cg::Wrap   [s <- :cg::Status])

(:wat::rete::defrule :cg::gather
  :when [(:cg::Anchor (?x <- :x))]
  :then [(:cg::Wrap :s (:cg::Status::Active 7))])

(:wat::rete::defquery :cg::q-Wrap
  :params []
  :when [(:cg::Wrap (?s <- :s))])


;; Fires via the WAT ORACLE.
(:wat::core::defn :user::run-oracle [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cg)
     session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cg::q-Wrap))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:cg::Anchor :x 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules$oracle session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     derived (:wat::rete::query fired (:cg::q-Wrap))
     r       (:wat::core::first derived)
     s       (:wat::core::Option/expect
               (:wat::core::PersistentMap/get r "?s")
               "q-Wrap: ?s")]
    (:wat::core::match s
      ((:cg::Status::Active lvl) lvl))))

;; Fires via the NATIVE KERNEL — same rule, same expected value, through the compiled RHS path.
(:wat::core::defn :user::run-native [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cg)
     session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cg::q-Wrap))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:cg::Anchor :x 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     derived (:wat::rete::query fired (:cg::q-Wrap))
     r       (:wat::core::first derived)
     s       (:wat::core::Option/expect
               (:wat::core::PersistentMap/get r "?s")
               "q-Wrap: ?s")]
    (:wat::core::match s
      ((:cg::Status::Active lvl) lvl))))
