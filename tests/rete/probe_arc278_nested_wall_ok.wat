;; strike-nested-wall — THE CONTROL, and it is load-bearing for all four kind-probes.
;;
;; Same shapes they misspell, spelled correctly: a two-field nested constructor written as kwargs,
;; every field name declared and every declared field supplied. Without it, "the fixture refused"
;; is indistinguishable from "the fixture was malformed in some way I did not intend" — and after a
;; strike that makes a wall REFUSE for the first time, that is the confusion most likely to pass.
;;
;; It must COMPILE, FIRE, and derive exactly one fact.

(:wat::core::defrecord :nwo::Src   [k <- :wat::core::i64])
(:wat::core::defrecord :nwo::Inner [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :nwo::Outer [k <- :wat::core::i64  inner <- :nwo::Inner])

(:wat::rete::defrule :nwo::r
  :when [(:nwo::Src (?k <- :k))]
  :then [(:nwo::Outer :k ?k :inner (:nwo::Inner :x ?k :y ?k))])

(:wat::rete::defquery :nwo::q :params [] :when [(?f <- :nwo::Outer)])

(:wat::core::defn :nwo::fire [] -> :wat::core::i64
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :nwo) (:wat::core::PersistentVector (:nwo::q)))
          ((:wat::rete::CompileOutcome::Compiled __s) __s)
          ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
            (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None)))
     s1 (:wat::core::match (:wat::rete::insert s0 (:nwo::Src :k 5))
          ((:wat::rete::InsertOutcome::Inserted __st) __st)
          ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c)
            (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query
      (:wat::core::match (:wat::rete::fire-rules s1)
        ((:wat::rete::FireOutcome::Fired __f) __f)
        ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r2)
          (:wat::kernel::assertion-failed! "fire: ceiling" :wat::core::None :wat::core::None))
        ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
          (:wat::kernel::assertion-failed! "fire: round cap" :wat::core::None :wat::core::None)))
      (:nwo::q)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:nwo::fire)))
