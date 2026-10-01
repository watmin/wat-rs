;; wat-tests/service-request-malformed.wat — arc 278 Stone 1: the DoS probe, INVERTED.
;;
;; THE VULNERABILITY (proven, both tiers, before this stone —
;; wat-scripts/scratch-pad/probe-arc278-wire-dos-service-killed.wat):
;;
;;     "attacker good  => Ok"
;;     "attacker BAD   => LOST (peer gone)"
;;     victim: connect REFUSED — service is GONE
;;
;; A client sends well-formed EDN with a wrong-typed body under a CORRECT tag —
;; `#…/PutRequest {:items [1 2 3]}` against `items <- (Vector :- [String])`. The wire accepted it
;; verbatim: the thread tier never decodes at all (`ReactorClass::InMemory` passes the Value
;; through crossbeam), and the process tier's decode is TAG-driven, not TARGET-driven
;; (`reconstruct_record` uses the declared fields for names and order only — the declared
;; field type is never compared to the decoded value). The handler then used the field AT ITS
;; DECLARED TYPE — `(string::length (nth items 0))`, legal and correct against the declaration
;; — and DETONATED, killing the service for everyone. One frame from any client. A denial of
;; service, with no bug in the handler.
;;
;; THE WALL: the op's declared `<Op>Request` record IS the whitelist — already authored,
;; nothing new to declare — and `:wat::edn::validate` is the deep shape check against it
;; (`:wat::core::conforms?` cannot serve: for an Aggregate it is a NOMINAL identity check
;; that never recurses into FIELDS, so the attacker's frame conforms? TRUE). The guard sits in
;; the generated dispatch arm beside the `:max-request-bytes` size guard — POST-DECODE, before
;; the handler — which is precisely why it covers BOTH TIERS.
;;
;; THIS TEST IS THE ACCEPTANCE BAR, and it is point (2) that is the whole strike:
;;   1. the attacker's malformed frame returns a NAMED `:RequestMalformed` — not a crash, not
;;      a raise, a value the caller's exhaustive match must face;
;;   2. a subsequent INNOCENT client `connect'`s and IS SERVED.
;; A bad caller, malicious or dumb, cannot crash anything.

;; ── the surface: the whitelist is `items <- (Vector :- [String])`, and nothing else ────────────
;; `:RequestMalformed` is the shape sibling of ruling A's `:RequestTooLarge` size variant:
;; `path` is STRUCTURED (segments — ["items" "[0]"]); `expected`/`got` are Strings (the
;; four-questions ruling: `got` is the EDN SHAPE that arrived, and an untyped wire value has
;; no declared type — structuring it would fabricate information).
(:wat::core::defsurface :wat-tests::MalBag :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::MalBag::PutRequest [items <- (:wat::core::Vector :- [:wat::core::String])])
   (:wat::core::defenum :wat-tests::MalBag::PutResponse :wat::enum::Pure
     :Ok               [n <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path     <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got      <- :wat::core::String])]
  :features
  [(put [self <- :wat-tests::MalBag  req <- :wat-tests::MalBag::PutRequest]
     -> :wat-tests::MalBag::PutResponse :max-request-bytes 4096)])

;; ── the service: the handler is UNCHANGED from the DoS reproduction ──────────────────────
;; It still uses the field at its declared type. That is the point: the handler is correct
;; against the declaration and must not have to defend itself. The wall is upstream of it.
(:wat::service::defservice :wat-tests::mal-bag
  :satisfies :wat-tests::MalBag
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  ;; NOTHING IS OPTED INTO HERE. Arc 278 Stone 1 shipped the wall behind a clause and defaulted
  ;; it off; Stone 2 annihilated the clause. This service declares a surface, a state, and a
  ;; handler — and the request-shape wall is generated into every one of its op arms regardless,
  ;; because that is what a service IS. The two deftests below are the proof.
  :impls
  [(put [s ctx req]
     (:wat::service::Outcome.Reply {:state s
       :reply (:wat-tests::MalBag::PutResponse.Ok
         {:n (:wat::string::length
           (:wat::core::nth (:wat-tests::MalBag::PutRequest/items req) 0))})}))])

;; ── the probe verbs ──────────────────────────────────────────────────────────────────────
;; One call → one label. The exhaustive match is the shield: `:RequestMalformed` is a variant
;; the caller CANNOT ignore (arc 109 — no wildcard arm), so a refusal can never be silent.
(:wat::core::defn :wat-tests::mal/try
  [c <- (:wat::kernel::Peer :- [:wat-tests::MalBag::Op :wat-tests::MalBag::Reply])
   req <- :wat-tests::MalBag::PutRequest] -> :wat::core::String
  (:wat::core::match (:wat-tests::MalBag/put c req)
    [:wat::kernel::RecvOutcome.Message {:msg resp}
      (:wat::core::match resp
        [:wat-tests::MalBag::PutResponse.Ok {:n n} "Ok"]
        [:wat-tests::MalBag::PutResponse.RequestTooLarge {:bytes b :cap cap} "TooLarge"]
        [:wat-tests::MalBag::PutResponse.RequestMalformed {:path path :expected expected :got got}
          (:wat::string::concat "Malformed"
            (:wat::string::concat (:wat::edn::write path)
              (:wat::string::concat "/" (:wat::string::concat expected
                (:wat::string::concat "/" got)))))])]
    [:wat::kernel::RecvOutcome.Lost {:cause cause} "LOST"]
    ;; arc 278 #73 — distinct from LOST (the peer died) and Closed (a clean hangup): the
    ;; substrate was asked to stop while this recv was parked; the peer was ALIVE.
    [:wat::kernel::RecvOutcome.Stopped {} "Stopped"]
    [:wat::kernel::RecvOutcome.Closed {} "Closed"]))

(:wat::core::defn :wat-tests::mal/dial
  [a <- (:wat::kernel::Address :- [:wat-tests::MalBag::Op :wat-tests::MalBag::Reply])]
  -> (:wat::kernel::Peer :- [:wat-tests::MalBag::Op :wat-tests::MalBag::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    [:wat::kernel::ConnectOutcome.Connected {:peer p} p]
    [:wat::kernel::ConnectOutcome.Refused {:cause c}
      (:wat::kernel::assertion-failed! :message "victim: connect REFUSED — the service is GONE (the DoS is back)")]
    [:wat::kernel::ConnectOutcome.Rejected {:cause c}
      (:wat::kernel::assertion-failed! :message "victim: connect REJECTED — the service is GONE (the DoS is back)")]
    [:wat::kernel::ConnectOutcome.Failed {:cause c}
      (:wat::kernel::assertion-failed! :message "victim: connect FAILED — the service is GONE (the DoS is back)")]))

;; Excursus 003 strike T2 — the attacker-BAD probe RETIRED, both tiers, measured (not assumed):
;;
;; `bad` was built with `(:wat::edn::read "#wat-tests.MalBag/PutRequest {:items [1 2 3]}")` — an
;; IN-PROCESS decode, same door on every tier. Strike T's strict decode (`value_conforms`,
;; `src/edn/render.rs`) is NOT special-cased to `decode_trusted_wire`; `:wat::edn::read` goes
;; through the identical `reconstruct_record`/`reconstruct_struct` family, so this line now
;; REFUSES at construction, on BOTH tiers, before `mal/run` ever reaches `dial`/`try` — driven:
;; `cargo nextest run --release -p wat -E 'test(request_malformed)'` at the pre-T2 tree raised
;; `malformed :wat::edn::read form: ... :wat-tests::MalBag::PutRequest.items.[0] declared as
;; :wat::core::String, but the decoded value is :wat::core::i64` at THIS line, on both deftests.
;;
;; T's own brief measured the THREAD tier alone as unbuildable ("no wat path can build a
;; wrong-typed request value" — the thread tier has no wire to attack in the first place, so it
;; was never going to survive strict decode regardless of tier). T2 measured further: the SAME is
;; now true of the PROCESS tier, because there is no OTHER honest way left to construct a
;; wrong-shaped `PutRequest` in wat — the checker refuses it statically at any ordinary
;; constructor call (`:wat-tests::MalBag::PutRequest :items <wrong-typed-value>` does not
;; type-check), and the one runtime decode door (`:wat::edn::read`) is exactly what strict decode
;; now forbids using as a substitute. A raw-socket test (writing the malformed frame directly to
;; an established process-tier connection's fd, bypassing wat's typed layer entirely — the shape
;; `tests/comms/probe_arc278_over_budget_recovers.rs`'s `sender.raw_fds()[0]` takes for the
;; comms layer in isolation) would need either the connected peer's raw fd exposed to wat (it is
;; not — T's own finding) or a Rust-level test helper that reaches a REAL spawned `defservice`'s
;; accepted socket from outside wat — new kernel/test surface this strike does not mint without
;; the builder's sign-off (see the strike's report).
;;
;; The SERVER-SIDE contract T2 exists to restore IS proven, just not from here:
;; `src/kernel/message.rs`'s `kernel::message::excursus_003_t2_gates::{gt2a,gt2b,gt2c}` build the
;; malformed WIRE BYTES directly (the real writer, `value_to_wire_edn_string`, handed a `Value`
;; the Rust-level test built with the raw `AggregateValue`/`EnumValue` constructors — never a
;; wat-level `:wat::edn::read` and never an in-process `Value` handed to the server), feed them
;; through the REAL `decode_trusted_wire` → `decode_client_message_event` → `:wat::edn::validate`
;; path, and assert the exact `:RequestMalformed` shape this file's retired assertion used to
;; pin. The probes below keep what is STILL true: the service serves a well-formed request, and
;; keeps serving a second client afterward.
(:wat::core::defn :wat-tests::mal/run
  [locus <- :wat::spawn::Locus] -> :wat::core::String
  (:wat::core::let
    [h    (:wat-tests::mal-bag/start :locus locus :record (:wat-tests::mal-bag::Record :n 0))
     good (:wat-tests::MalBag::PutRequest :items (:wat::core::Vector :- [:wat::core::String] "abcd"))
     a    (:wat-tests::mal/dial (:wat-tests::mal-bag::Handle/addr h))
     r1   (:wat-tests::mal/try a good)
     b    (:wat-tests::mal/dial (:wat-tests::mal-bag::Handle/addr h))
     r2   (:wat-tests::mal/try b good)
     _    (:wat-tests::mal-bag/stop h)]
    (:wat::string::concat r1 (:wat::string::concat " | " r2))))

;; ── thread tier ──────────────────────────────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::service::request-malformed-on-thread

  (:wat::test::assert-eq
    (:wat-tests::mal/run (:wat::spawn::thread))
    "Ok | Ok"))

;; ── process tier ─────────────────────────────────────────────────────────────────────────
;; The SAME expectation, one token apart. Tier-generality is the requirement: a Rust-side
;; decode fix would pass this on the process tier and fail on the thread tier, which never
;; decodes at all.
(:wat::test::deftest :wat-tests::service::request-malformed-on-process

  (:wat::test::assert-eq
    (:wat-tests::mal/run (:wat::spawn::process))
    "Ok | Ok"))
