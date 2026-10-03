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

(:wat::core::defrecord :jb::Temp  [loc <- wat.type/String  celsius <- wat.type/i64])
(:wat::core::defrecord :jb::Wind  [loc <- wat.type/String  kph     <- wat.type/i64])
;; Both non-join bindings, kept apart by NAME as well as position.
(:wat::core::defrecord :jb::Both  [loc <- wat.type/String
                                   celsius <- wat.type/i64
                                   kph <- wat.type/i64])

(:wat::rete::defrule :jb::both-sides
  :when [(:jb::Temp (?loc :- :loc) (?c :- :celsius))
         (:jb::Wind (?loc :- :loc) (?k :- :kph))]
  :then [(:jb::Both :loc ?loc :celsius ?c :kph ?k)])

(:wat::rete::defquery :jb::q :params [] :when [(?fact :- :jb::Both)])

(:wat::core::defn :jb::staged [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all
    (:wat::core::match (:wat::rete::insert-all
      (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :jb)
                               (wat.type/PersistentVector :- [:wat::rete::Query] (:jb::q))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
      (wat.type/PersistentVector :- [:jb::Temp] (:jb::Temp :loc "MCI" :celsius 5))) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
    (wat.type/PersistentVector :- [:jb::Wind] (:jb::Wind :loc "MCI" :kph 40))) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")]))

(:wat::core::defn :jb::readback [s <- :wat::rete::Session] -> (wat.type/PersistentVector :- [wat.type/i64])
  (:wat::core::let [rows (:wat::rete::query s (:jb::q))]
    (:wat::core::if (:wat::core::= (:wat::core::length rows) 1)
      (:wat::core::let [f (:wat::core::Option/expect
                            (:wat::core::get (:wat::core::first rows) "?fact") "fact")]
        (wat.type/PersistentVector :- [wat.type/i64] (:wat::core::length rows) (:jb::Both/celsius f) (:jb::Both/kph f)))
      (wat.type/PersistentVector :- [wat.type/i64] (:wat::core::length rows) 0 0))))

;; [rows, celsius, kph] under native, then the same under $oracle. Expect 1/5/40 twice.
(:wat::core::defn :user::native-and-oracle [] -> (wat.type/Vector :- [wat.type/i64])
  (:wat::core::mapv
    (:wat::core::fn [n <- wat.type/i64] -> wat.type/i64 n)
    (:wat::core::into
      (:jb::readback (:wat::core::match (:wat::rete::fire-rules (:jb::staged)) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")]))
      (:jb::readback (:wat::core::match (:wat::rete::fire-rules$oracle (:jb::staged)) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))))
