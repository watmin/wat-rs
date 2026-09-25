;; tests/comms/probe_ex003_stone_q_printing_a_newtype_does_not_panic.wat — co-located fixture for
;; probe_ex003_stone_q_printing_a_newtype_does_not_panic.rs (startup_beside). No placeholder main at
;; the top level; the inner :user::main is a spawned CHILD's entrypoint.
;;
;; Excursus 003 stone Q/R — printing a newtype does not panic, and IS TAGGED (the-little-wat
;; F-030). Stone Q fixed the panic (`(:wat::kernel::println (:u::N 5))` crashed at
;; `crates/wat-edn/src/value.rs:330`: `Keyword::new("0")`, a newtype's synthesized field name is
;; not a legal EDN keyword) by writing a newtype as its BARE inner value (`5`), on the
;; orchestrator's mistaken reading of the typed reader's OLD comment ("the wat-side wrapper is
;; invisible at the EDN layer"). That broke round-trip identity: the untagged aggregate read back
;; as a bare `i64` everywhere, and `(= (:q::N 5) m)` after a wire trip raised `TypeMismatch` (one
;; side `Aggregate`, one side `i64`) — measured by this very probe's (now corrected) wire case.
;;
;; Stone R corrects it, per the builder's ruling: *"they look like records - records must always
;; be tagged .... i don't know how an untagged thing could ever be emitted"* — a newtype is a
;; record-shaped value and is ALWAYS tagged:
;;
;;   (:wat::edn::write (:q::N 5))  →  "#q/N 5"   (never bare "5")
;;
;; The decode-side asymmetry stone Q's header documented ("`back` is `:wat::core::i64`, not
;; `:q::N`") is GONE: both the untyped reader (`:wat::edn::read`) and the typed reader (a
;; newtype-typed wire slot) now resolve the tag and rebuild the NEWTYPE itself, so a round trip
;; and a wire crossing compare EQUAL directly with `=` — no more re-wrap-through-the-constructor
;; workaround, no more EDN-text comparison. Same idiom every other aggregate roundtrip test in
;; this tree uses (e.g. `tests/types/probe_arc234_7a_base_record_roundtrip.wat`'s `(= p p2)`).

(:wat::core::newtype :q::N :wat::core::i64)

;; (a) Round trip: write (now tagged), read back (now the NEWTYPE itself, not the bare inner),
;; compare directly. Also validates the written EDN against the `:q::N` slot.
(:wat::core::defn :q::probe-round-trip [] -> :wat::core::String
  (:wat::core::let
    [v     (:q::N 5)
     s     (:wat::edn::write v)
     back  (:wat::edn::read s)
     valid (:wat::core::match (:wat::edn::validate v :q::N)
             [:wat::edn::Validation.Valid {} "Valid"]
             [:wat::edn::Validation.Invalid {:path _p :expected e :got g}
               (:wat::core::format "Invalid: expected {e} got {g}" :e e :g g)])]
    (:wat::core::format "written: {s}; round trip equal: {e}; validate: {val}"
      :s s
      :e (:wat::core::= v back)
      :val valid)))

;; (b) The typed door: `:wat::edn::validate` renders the value and decodes it against the declared
;; type through `edn_to_typed_value`. Before stone Q the RENDER step itself panicked, so this never
;; ran at all; before stone R it ran but decoded to the bare inner rather than the newtype.
(:wat::core::defn :q::probe-validate [] -> :wat::core::String
  (:wat::core::match (:wat::edn::validate (:q::N 5) :q::N)
    [:wat::edn::Validation.Valid {} "Valid"]
    [:wat::edn::Validation.Invalid {:path _p :expected e :got g}
      (:wat::core::format "Invalid: expected {e} got {g}" :e e :g g)]))

;; (c) A newtype value over a process wire (the child's socket-tier `send` on its self-peer; the
;; parent's untyped `decode_trusted_wire`). BEFORE stone Q the child's `send` panicked the same
;; way `println` did — `value_to_wire_edn_string` shares the writer with `println`. BEFORE stone
;; R the arrived value decoded to the bare inner `i64`, so `(= m (:q::N 5))` raised `TypeMismatch`
;; at runtime (measured — see the file header). Stone R's untyped-reader newtype route rebuilds
;; the NEWTYPE on receipt, so the direct value comparison now succeeds.
(:wat::core::defn :q::probe-wire [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::newtype :q::N :wat::core::i64)
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::match
                 (:wat::kernel::send
                   (:wat::program::self-peer :q::N :wat::core::i64)
                   (:q::N 5))
                 [:wat::kernel::SendOutcome.Sent {} nil]
                 [:wat::kernel::SendOutcome.Closed {} nil]
                 [:wat::kernel::SendOutcome.Lost {:cause _c} nil]
                 [:wat::kernel::SendOutcome.Stopped {} nil]))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg m}
        (:wat::core::format "Message; arrived equal: {e}"
          :e (:wat::core::= m (:q::N 5)))]
      [:wat::kernel::RecvOutcome.Lost {:cause c}
        (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))
