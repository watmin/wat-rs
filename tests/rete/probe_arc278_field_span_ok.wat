;; strike-field-span — THE CONTROL. Every shape the four fixtures beside it misspell, spelled
;; correctly: an inline constraint on a real field, a bind of a real field, a kwargs `:then`
;; naming real fields, and a nested constructor naming a real field.
;;
;; Without it a refusal above is indistinguishable from a fixture that was malformed for some
;; unrelated reason — "it refused" is also what a broken fixture looks like. This one must COMPILE
;; AND FIRE, and it prints the number of derived facts so a silent no-match cannot pass either.

(:wat::core::defrecord :fso::Src   [k <- :wat::core::i64])
(:wat::core::defrecord :fso::Inner [x <- :wat::core::i64])
(:wat::core::defrecord :fso::Outer [k <- :wat::core::i64  inner <- :fso::Inner])

(:wat::rete::defrule :fso::r
  :when [(:fso::Src (?k <- :k) (?b <- :k) (:wat::rete::core::i64::= :k 5))]
  :then [(:fso::Outer :k ?k :inner (:fso::Inner :x ?b))])

(:wat::rete::defquery :fso::q :params [] :when [(?f <- :fso::Outer)])

(:wat::core::defn :fso::fire [] -> :wat::core::i64
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :fso) (:wat::core::PersistentVector (:fso::q)))
          ((:wat::rete::CompileOutcome::Compiled __s) __s)
          ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
            (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None)))
     s1 (:wat::core::match (:wat::rete::insert s0 (:fso::Src :k 5))
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
      (:fso::q)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:fso::fire)))
