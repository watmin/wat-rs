;; Fixture BESIDE probe_arc278_join_carries_both_sides_into_the_rhs.rs.
;;
;; THE CONTRACT: a two-condition join instantiates its RHS with bindings from BOTH
;; sides, in the right slots.
;;
;; WHY THIS EXISTS. A `vocare` cast found four in-crate join tests
;; (`src/rete/kernel/tests.rs`) that hand-build a `Rule` with a deliberately EMPTY
;; `:rhs` and read `wm.beta` directly. Those are legitimate implementer-vantage unit
;; tests of the join itself — but with no `:rhs`, NO production ever runs, so nothing
;; in them can see the join→RHS boundary.
;;
;; And the caller-level join test that does exist does not close it either: the
;; `cold-and-windy` rule joins on `?loc` and its `:then` uses ONLY `?loc` — the JOIN
;; KEY. That variable is bound by the first condition and merely matched by the
;; second, so a bug that dropped or swapped the second side's bindings still yields
;; the right `?loc` and the test stays green.
;;
;; So this asserts the thing neither reaches: a RHS built from a NON-JOIN binding on
;; EACH side. The two values are deliberately distinguishable (5 and 40) and land in
;; typed slots, so a SWAP is a red, not a coincidence — 5/40 and 40/5 are different
;; facts. Both values are also absent from the join key, so nothing about `?loc`
;; being correct can mask either of them being wrong.

(:wat::core::defrecord :jb::Temp  [loc <- :wat::core::String  celsius <- :wat::core::i64])
(:wat::core::defrecord :jb::Wind  [loc <- :wat::core::String  kph     <- :wat::core::i64])
;; Both non-join bindings, kept apart by NAME as well as position.
(:wat::core::defrecord :jb::Both  [loc <- :wat::core::String
                                   celsius <- :wat::core::i64
                                   kph <- :wat::core::i64])

(:wat::rete::defrule :jb::both-sides
  :when [(:jb::Temp (?loc <- :loc) (?c <- :celsius))
         (:jb::Wind (?loc <- :loc) (?k <- :kph))]
  :then [(:jb::Both :loc ?loc :celsius ?c :kph ?k)])

(:wat::rete::defquery :jb::q :params [] :when [(?fact <- :jb::Both)])

(:wat::core::defn :jb::staged [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all
    (:wat::core::match (:wat::rete::insert-all
      (:wat::rete::compile-all (:wat::rete::collect-rules :jb)
                               (:wat::core::PersistentVector (:jb::q)))
      (:wat::core::PersistentVector (:jb::Temp :loc "MCI" :celsius 5))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
    (:wat::core::PersistentVector (:jb::Wind :loc "MCI" :kph 40))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :jb::readback [s <- :wat::rete::Session] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [rows (:wat::rete::query s (:jb::q))]
    (:wat::core::if (:wat::core::= (:wat::core::length rows) 1)
      (:wat::core::let [f (:wat::core::Option/expect
                            (:wat::core::PersistentMap/get (:wat::core::first rows) "?fact") "fact")]
        (:wat::core::PersistentVector (:wat::core::length rows) (:jb::Both/celsius f) (:jb::Both/kph f)))
      (:wat::core::PersistentVector (:wat::core::length rows) 0 0))))

;; [rows, celsius, kph] under native, then the same under $oracle. Expect 1/5/40 twice.
(:wat::core::defn :user::native-and-oracle [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::mapv
    (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)
    (:wat::core::PersistentVector/concat
      (:jb::readback (:wat::core::match (:wat::rete::fire-rules (:jb::staged)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))
      (:jb::readback (:wat::core::match (:wat::rete::fire-rules$oracle (:jb::staged)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))))
