;; tests/comms/probe_ex003_stone_q_printing_a_newtype_does_not_panic.wat — co-located fixture for
;; probe_ex003_stone_q_printing_a_newtype_does_not_panic.rs (startup_beside). No placeholder main at
;; the top level; the inner :user::main is a spawned CHILD's entrypoint.
;;
;; Excursus 003 stone Q — printing a newtype does not panic (the-little-wat F-030). The driven
;; repro (`the-little-wat/probes/ml/newtype-value.wat`, `(:wat::kernel::println (:u::N 5))`) panicked
;; at `crates/wat-edn/src/value.rs:330`: the writer rendered a newtype like a struct, keyed by field
;; NAMES — a newtype's one field is synthesized-named "0" (`register_newtype_methods` /
;; `eval_struct_new`, `src/record/construct.rs:103`), and `:0` is not a legal EDN keyword.
;;
;; BEFORE this stone: `(:wat::edn::write (:q::N 5))` panicked (`Keyword::new("0")`) instead of
;; returning `5`. The typed READER already treats a newtype as transparent at the EDN layer
;; (`edn_to_typed_value_inner`, `src/edn/render.rs`: "newtypes coerce against their inner declared
;; shape — the wat-side wrapper is invisible at the EDN layer") — the writer now matches it: a
;; newtype value renders as its INNER value's EDN.
;;
;; ⚠ The wrapper's invisibility at the EDN layer is BILATERAL and pre-dates this stone: decoding
;; through ANY untyped door (`:wat::edn::read`, wire receipt, a record field) also yields the bare
;; INNER value, never the newtype's own `Value::Aggregate` wrapper `struct-new` builds at
;; construction. So `back` below is `:wat::core::i64`, not `:q::N` — comparing it against a freshly
;; CONSTRUCTED `(:q::N 5)` with `=` does not type-check (the two sides are different runtime Value
;; shapes even though the checker treats `:q::N` as equatable via its inner type — a separate,
;; pre-existing asymmetry this stone does not touch). The round trip is proved the way that
;; asymmetry allows: re-wrapping the read-back inner value through the newtype's OWN constructor
;; reconstructs the original value, and `:wat::edn::validate` confirms the written EDN is a legal
;; `:q::N`. The wire probe (c) proves the same thing across a process boundary by comparing EDN TEXT
;; (what the writer emits, not a materialized post-decode Value) — the writer is this stone's
;; subject; the decode-side rewrap gap is not.

(:wat::core::newtype :q::N :wat::core::i64)

;; (a) Round trip: write, read back (untyped — the bare inner value comes back), re-wrap through the
;; newtype's own constructor, compare. Also validates the written EDN against the `:q::N` slot.
(:wat::core::defn :q::probe-round-trip [] -> :wat::core::String
  (:wat::core::let
    [v     (:q::N 5)
     s     (:wat::edn::write v)
     back  (:q::N (:wat::edn::read s))
     valid (:wat::core::match (:wat::edn::validate v :q::N)
             [:wat::edn::Validation.Valid {} "Valid"]
             [:wat::edn::Validation.Invalid {:path _p :expected e :got g}
               (:wat::core::format "Invalid: expected {e} got {g}" :e e :g g)])]
    (:wat::core::format "written: {s}; round trip equal: {e}; validate: {val}"
      :s s
      :e (:wat::core::= v back)
      :val valid)))

;; (b) The typed door: `:wat::edn::validate` renders the value and decodes it against the declared
;; type through `edn_to_typed_value`. Before this stone the RENDER step itself panicked, so this
;; never ran at all.
(:wat::core::defn :q::probe-validate [] -> :wat::core::String
  (:wat::core::match (:wat::edn::validate (:q::N 5) :q::N)
    [:wat::edn::Validation.Valid {} "Valid"]
    [:wat::edn::Validation.Invalid {:path _p :expected e :got g}
      (:wat::core::format "Invalid: expected {e} got {g}" :e e :g g)]))

;; (c) A newtype value over a process wire (the child's socket-tier `send` on its self-peer; the
;; parent's untyped `decode_trusted_wire`). BEFORE this stone the child's `send` panicked the same
;; way `println` did — `value_to_wire_edn_string` shares the writer with `println`. Compared as EDN
;; TEXT (see the file header on why: the arrived Value is the bare inner `i64`, not a re-wrapped
;; `:q::N`, so a `Value`-level `=` against a freshly-constructed `:q::N` cannot type-check).
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
        (:wat::core::format "Message; arrived edn: {a}; expected edn: {e}"
          :a (:wat::edn::write m)
          :e (:wat::edn::write (:q::N 5)))]
      [:wat::kernel::RecvOutcome.Lost {:cause c}
        (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))
