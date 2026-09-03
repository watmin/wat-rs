;; strike-match-arm-is-not-a-call (D5) — SPELLING 2/3: the bare CORE head `:wat::core::match`.
;;
;; ⚠ THIS FIXTURE STILL FAILS after the cure — but for a DIFFERENT reason than before it, and that
;; CHANGE OF REASON is the whole assertion (see `probe_arc278_match_arm_is_not_a_call.rs`).
;;
;; `walk_nested_constructors` is reached by BOTH spellings VERBATIM — measured by instrumenting the
;; walker to print `items[0]`: a `:then` operand delivers `:wat::rete::core::match` and
;; `:wat::core::match` un-lowered, and at HEAD d10ae67c4 each produced the SAME phantom
;; `RhsArityMismatch` on `:mac::E::A` / `:mac::E::B`, at FREEZE, killing startup. That is why the
;; walker's guard resolves through `resolve_core_name` — one indirection, exactly as `purity.rs`'s
;; `classify_expr` match arm does — rather than testing the rete name alone: keyed on
;; `:wat::rete::core::match` alone, this spelling would still fabricate an insert of an enum
;; variant that appears nowhere below.
;;
;; The core spelling is nonetheless ILLEGAL in a `:then`: `wat/rete/compile.wat`'s then-item fence
;; admits only `:wat::rete::` ops. So the required behaviour is a refusal FROM THE FENCE, naming the
;; head it will not admit — never a fabricated arity error about a variant nobody constructed.
;;
;; NOT `.wat.bad`: that convention is "expected to fail to LOAD" and this file now loads clean — the
;; freeze wall is exactly what stopped lying. The refusal has moved to `compile-all`, at run time.

(:wat::core::defenum :mac::E :wat::enum::Pure :A :B)

(:wat::core::defrecord :mac::In  [k <- :wat::core::i64  v <- :mac::E])
(:wat::core::defrecord :mac::Out [k <- :wat::core::i64  ok <- :wat::core::bool])

(:wat::rete::defrule :mac::r
  :when [(:mac::In (?k <- :k) (?v <- :v))]
  :then [(:mac::Out :k ?k :ok (:wat::core::match ?v (:mac::E::A true) (:mac::E::B false)))])

(:wat::rete::defquery :mac::by-ok
  :params [?ok]
  :when [(:mac::Out (?ok <- :ok) (?k <- :k))])

(:wat::core::defn :mac::world [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules
    (:wat::core::match (:wat::rete::insert
      (:wat::core::match (:wat::rete::compile-all
                           (:wat::core::PersistentVector (:mac::r))
                           (:wat::core::PersistentVector (:mac::by-ok)))
        ((:wat::rete::CompileOutcome::Compiled __s) __s)
        ((:wat::rete::CompileOutcome::MayNotTerminate __r __f)
          (:wat::kernel::assertion-failed! "compile: may not terminate" :wat::core::None :wat::core::None)))
      (:mac::In :k 1 :v :mac::E::A)
      (:mac::In :k 2 :v :mac::E::B)
      (:mac::In :k 3 :v :mac::E::A))
      ((:wat::rete::InsertOutcome::Inserted __st) __st)
      ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c)
        (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None)))
    )
    ((:wat::rete::FireOutcome::Fired __f) __f)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r2)
      (:wat::kernel::assertion-failed! "fire: ceiling" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __c __s)
      (:wat::kernel::assertion-failed! "fire: round cap" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [w (:mac::world)]
    (:wat::kernel::println
      (:wat::core::String/concat
        (:wat::core::String/concat "true=" (:wat::core::i64::to-string
          (:wat::core::length (:wat::rete::query w (:mac::by-ok) :?ok true))))
        (:wat::core::String/concat " false=" (:wat::core::i64::to-string
          (:wat::core::length (:wat::rete::query w (:mac::by-ok) :?ok false))))))))
