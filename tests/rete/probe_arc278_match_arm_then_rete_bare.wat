;; strike-match-arm-is-not-a-call (D5) — SPELLING 1/3: the bare RETE head, bare variant patterns.
;;
;; This is the exact shape that was REFUSED at HEAD d10ae67c4: `walk_nested_constructors` recursed
;; into the arm `(:mac::E::A true)`, resolved `:mac::E::A` as an enum-variant constructor head and
;; fired its 0 declared fields against the arm's length 1 — `RhsArityMismatch` naming a `:then`
;; insert of `:mac::E::A` that appears nowhere below.
;;
;; The three spellings must print IDENTICAL output. Not "all three compile": the walker fix could
;; have been "stop walking match forms", which also compiles. Agreement on the FIRED VALUES is what
;; says the match still means what it meant.

(:wat::core::defenum :mac::E :wat::enum::Pure :A :B)

(:wat::core::defrecord :mac::In  [k <- :wat::core::i64  v <- :mac::E])
(:wat::core::defrecord :mac::Out [k <- :wat::core::i64  ok <- :wat::core::bool])

(:wat::rete::defrule :mac::r
  :when [(:mac::In (?k :- :k) (?v :- :v))]
  :then [(:mac::Out :k ?k :ok (:wat::rete::core::match ?v [:mac::E.A {} true] [:mac::E.B {} false]))])

(:wat::rete::defquery :mac::by-ok
  :params [?ok]
  :when [(:mac::Out (?ok :- :ok) (?k :- :k))])

(:wat::core::defn :mac::world [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules
    (:wat::core::match (:wat::rete::insert
      (:wat::core::match (:wat::rete::compile-all
                           (:wat::core::PersistentVector (:mac::r))
                           (:wat::core::PersistentVector (:mac::by-ok)))
        [:wat::rete::CompileOutcome.Compiled {:session __s} __s]
        [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __f}
          (:wat::kernel::assertion-failed! :message "compile: may not terminate")])
      (:mac::In :k 1 :v :mac::E.A)
      (:mac::In :k 2 :v :mac::E.B)
      (:mac::In :k 3 :v :mac::E.A))
      [:wat::rete::InsertOutcome.Inserted {:session __st} __st]
      [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __a :used __b :staged __c}
        (:wat::kernel::assertion-failed! :message "insert: ceiling")])
    )
    [:wat::rete::FireOutcome.Fired {:value __f} __f]
    [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r2}
      (:wat::kernel::assertion-failed! :message "fire: ceiling")]
    [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s}
      (:wat::kernel::assertion-failed! :message "fire: round cap")]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [w (:mac::world)]
    (:wat::kernel::println
      (:wat::string::concat
        (:wat::string::concat "true=" (:wat::i64::to-string
          (:wat::core::length (:wat::rete::query w (:mac::by-ok) :?ok true))))
        (:wat::string::concat " false=" (:wat::i64::to-string
          (:wat::core::length (:wat::rete::query w (:mac::by-ok) :?ok false))))))))
