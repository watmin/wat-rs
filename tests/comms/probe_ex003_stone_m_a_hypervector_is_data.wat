;; tests/comms/probe_ex003_stone_m_a_hypervector_is_data.wat — co-located fixture for
;; probe_ex003_stone_m_a_hypervector_is_data.rs (startup_beside). No placeholder main at the top
;; level; each inner :user::main is a spawned CHILD's entrypoint.
;;
;; Excursus 003 stone M — a `:wat::holon::Vector` (a VSA hypervector, `holon::Vector` = `Vec<i8>`)
;; is DATA. It renders as its components, `#wat.holon/Vector [i8 …]`, and reads back equal.
;;
;; BEFORE this stone (measured at d29585eb1): the writer rendered `#wat.holon/Vector {:dim N}` —
;; neither the data nor nil — and no reader decoded the tag, so a Box holding a hypervector over a
;; process wire arrived as `RecvOutcome.Lost` "… unknown tag #wat.holon/Vector" (stone H's
;; measurement (c)), and `(:wat::edn::read (:wat::edn::write v))` raised the same unknown tag.

(:wat::core::defrecord :m::Box :- [T] [x <- :T])

(:wat::core::defn :m::hv [] -> :wat::holon::Vector
  (:wat::holon::encode (:wat::holon::to-holon "x")))

;; (a) Round trip: write, read back, compare (`PartialEq` on the data slice). Also reports the
;; dimension and the EDN's size in bytes.
(:wat::core::defn :m::probe-round-trip [] -> :wat::core::String
  (:wat::core::let
    [v    (:m::hv)
     s    (:wat::edn::write v)
     back (:wat::edn::read s)]
    (:wat::core::format "round trip equal: {e}; dim: {d}; edn bytes: {n}"
      :e (:wat::core::= v back)
      :d (:wat::config::dim-count)
      :n (:wat::string::length s))))

;; (b) The typed door: `:wat::edn::validate` renders the value and decodes it against the declared
;; type through `edn_to_typed_value`. Before: a `:wat::holon::Vector` slot had no arm and fell to
;; the registry lookup — `Invalid`.
(:wat::core::defn :m::probe-validate [] -> :wat::core::String
  (:wat::core::match (:wat::edn::validate (:m::hv) :wat::holon::Vector)
    [:wat::edn::Validation.Valid {} "Valid"]
    [:wat::edn::Validation.Invalid {:path _p :expected e :got g}
      (:wat::core::format "Invalid: expected {e} got {g}" :e e :g g)]))

;; (c) A Box holding a hypervector over a process wire (the child's socket-tier `send` on its
;; self-peer; the parent's untyped `decode_trusted_wire`). The parent compares what arrived with the
;; same hypervector computed locally.
(:wat::core::defn :m::arrived-equal [b <- (:m::Box :- [:wat::holon::Vector])] -> :wat::core::String
  (:wat::core::format "Message; arrived equal: {e}" :e (:wat::core::= (:m::Box/x b) (:m::hv))))

(:wat::core::defn :m::probe-wire [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defrecord :m::Box :- [T] [x <- :T])
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::match
                 (:wat::kernel::send
                   (:wat::program::self-peer (:m::Box :- [:wat::holon::Vector]) :wat::core::i64)
                   (:m::Box :x (:wat::holon::encode (:wat::holon::to-holon "x"))))
                 [:wat::kernel::SendOutcome.Sent {} nil]
                 [:wat::kernel::SendOutcome.Closed {} nil]
                 [:wat::kernel::SendOutcome.Lost {:cause _c} nil]
                 [:wat::kernel::SendOutcome.Stopped {} nil]))))]
    (:wat::core::match (:wat::kernel::recv svc)
      [:wat::kernel::RecvOutcome.Message {:msg m} (:m::arrived-equal m)]
      [:wat::kernel::RecvOutcome.Lost {:cause c}
        (:wat::core::format "Lost: {m}" :m (:wat::kernel::LociDiedError/message c))]
      [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
      [:wat::kernel::RecvOutcome.Closed {} "Closed"])))

;; (d) The refusals (the OLD `{:dim N}` form; a component outside i8) are unit tests beside the reader
;; (`src/edn/render.rs`, `stone_m_*`): raised through `:wat::edn::read`, their face embeds the reader's
;; Rust `file:line` in the message string, which no golden can pin.
