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
     (:wat::service::Outcome::Reply s
       (:wat-tests::MalBag::PutResponse::Ok
         (:wat::string::length
           (:wat::core::nth (:wat-tests::MalBag::PutRequest/items req) 0)))))])

;; ── the probe verbs ──────────────────────────────────────────────────────────────────────
;; One call → one label. The exhaustive match is the shield: `:RequestMalformed` is a variant
;; the caller CANNOT ignore (arc 109 — no wildcard arm), so a refusal can never be silent.
(:wat::core::defn :wat-tests::mal/try
  [c <- (:wat::kernel::Peer :- [:wat-tests::MalBag::Op :wat-tests::MalBag::Reply])
   req <- :wat-tests::MalBag::PutRequest] -> :wat::core::String
  (:wat::core::match (:wat-tests::MalBag/put c req)
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat-tests::MalBag::PutResponse::Ok n) "Ok")
        ((:wat-tests::MalBag::PutResponse::RequestTooLarge b cap) "TooLarge")
        ((:wat-tests::MalBag::PutResponse::RequestMalformed path expected got)
          (:wat::string::concat "Malformed"
            (:wat::string::concat (:wat::edn::write path)
              (:wat::string::concat "/" (:wat::string::concat expected
                (:wat::string::concat "/" got))))))))
    ((:wat::kernel::RecvOutcome::Lost cause) "LOST")
    ;; arc 278 #73 — distinct from LOST (the peer died) and Closed (a clean hangup): the
    ;; substrate was asked to stop while this recv was parked; the peer was ALIVE.
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::Closed "Closed")))

(:wat::core::defn :wat-tests::mal/dial
  [a <- (:wat::kernel::Address :- [:wat-tests::MalBag::Op :wat-tests::MalBag::Reply])]
  -> (:wat::kernel::Peer :- [:wat-tests::MalBag::Op :wat-tests::MalBag::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)
      (:wat::kernel::assertion-failed! "victim: connect REFUSED — the service is GONE (the DoS is back)" :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c)
      (:wat::kernel::assertion-failed! "victim: connect REJECTED — the service is GONE (the DoS is back)" :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)
      (:wat::kernel::assertion-failed! "victim: connect FAILED — the service is GONE (the DoS is back)" :wat::core::None :wat::core::None))))

;; The whole run, as one string: attacker-good | attacker-BAD | victim-good.
;; The victim's `connect'` happens AFTER the malformed frame — that dial is the assertion.
(:wat::core::defn :wat-tests::mal/run
  [locus <- :wat::spawn::Locus] -> :wat::core::String
  (:wat::core::let
    [h    (:wat-tests::mal-bag/start :locus locus :record (:wat-tests::mal-bag::Record :n 0))
     good (:wat-tests::MalBag::PutRequest :items (:wat::core::Vector :wat::core::String "abcd"))
     ;; the attacker's frame: correct TAG, wrong-typed BODY
     bad  (:wat::edn::read "#wat-tests.MalBag/PutRequest {:items [1 2 3]}")
     a    (:wat-tests::mal/dial (:wat-tests::mal-bag::Handle/addr h))
     r1   (:wat-tests::mal/try a good)
     r2   (:wat-tests::mal/try a bad)
     ;; a SECOND, INNOCENT client connects AFTER the malformed frame
     b    (:wat-tests::mal/dial (:wat-tests::mal-bag::Handle/addr h))
     r3   (:wat-tests::mal/try b good)
     _    (:wat-tests::mal-bag/stop h)]
    (:wat::string::concat r1
      (:wat::string::concat " | " (:wat::string::concat r2
        (:wat::string::concat " | " r3))))))

;; ── thread tier ──────────────────────────────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::service::request-malformed-on-thread

  (:wat::test::assert-eq
    (:wat-tests::mal/run (:wat::spawn::thread))
    "Ok | Malformed[\"items\" \"[0]\"]/:wat::core::String/Integer | Ok"))

;; ── process tier ─────────────────────────────────────────────────────────────────────────
;; The SAME expectation, one token apart. Tier-generality is the requirement: a Rust-side
;; decode fix would pass this on the process tier and fail on the thread tier, which never
;; decodes at all.
(:wat::test::deftest :wat-tests::service::request-malformed-on-process

  (:wat::test::assert-eq
    (:wat-tests::mal/run (:wat::spawn::process))
    "Ok | Malformed[\"items\" \"[0]\"]/:wat::core::String/Integer | Ok"))
