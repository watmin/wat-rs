;; wat-scripts/scratch-pad/probe-arc278-wire-type-enforcement.wat
;;
;; arc 278 — MEASUREMENT probe (BRIEF-wire-type-enforcement-probe.md, stone 1 of (A)).
;;
;; THE QUESTION: does a service's wire decode ENFORCE the DECLARED type of an op's
;; request payload? `PutRequest [items <- (Vector :- [String])]` — deliver `{:items [1 2 3]}`
;; (well-formed EDN, i64s where String is declared). Does the server REJECT with a
;; named/located failure, or ACCEPT and hand the handler a mistyped value?
;;
;; This is a MEASUREMENT, not a fix. It prints what the substrate DOES and exits 0
;; either way — the verdict is in the printed transcript, not in an assertion.
;;
;; HOW IT WALKS THE REAL PATH (not a decoder unit test):
;;   - a real `defservice` on a real locus (BOTH tiers: thread AND process),
;;   - reached by a real `connect'` on the `Handle/addr`,
;;   - the payload crosses via the GENERATED, TYPED client fn (`Bag/put`) → the
;;     production `send'`/`recv'` verbs → the serve loop's inbound decode.
;;
;; HOW A MISTYPED PAYLOAD GETS UNDERNEATH THE TYPED CLIENT:
;;   `:wat::edn::read` is registered with a POLYMORPHIC FRESH-VAR return
;;   (`src/check.rs:19203` — `type_params: ["T"], params: [String], ret: t_var()`),
;;   documented as trust-the-caller: "the caller's binding context unifies with
;;   whatever shape the parsed value takes; runtime mismatches surface as
;;   pattern-match / accessor errors at the USE site" (`edn::render::eval_edn_read`'s doc comment).
;;   So `(:wat::edn::read "#probe-wire.Bag/PutRequest {:items [1 2 3]}")` type-checks
;;   as a `PutRequest` and at runtime produces a `PutRequest`-classed aggregate whose
;;   `items` field holds i64s. That is a REAL production verb, not a test hook — no
;;   decoder is called directly and nothing is faked.
;;
;; THE TELL: the handler echoes back `(:wat::edn::write items)` — the SERVER's own
;; rendering of what it actually received. `"[\"a\" \"b\"]"` = strings arrived;
;; `"[1 2 3]"` = i64s arrived under a `(Vector :- [String])` declaration, i.e. the wire did
;; NOT enforce the declared type.

;; ── the surface: one op, one field, declared (Vector :- [String]) ──────────────────
(:wat::core::defsurface :probe-wire::Bag :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe-wire::Bag::PutRequest
     [items <- (:wat::core::Vector :- [:wat::core::String])])
   (:wat::core::defenum :probe-wire::Bag::PutResponse :wat::enum::Pure
     ;; `seen` is the SERVER's own edn::write of the field it received — the tell.
     :Ok              [seen <- :wat::core::String]
     ;; ruling A — every serviceable op-Response carries the protocol-tier variant.
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(put [self <- :probe-wire::Bag  req <- :probe-wire::Bag::PutRequest]
     -> :probe-wire::Bag::PutResponse :max-request-bytes 4096)])

;; ── the service ──────────────────────────────────────────────────────────────
(:wat::service::defservice :probe-wire::bag-svc
  :satisfies :probe-wire::Bag
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :impls
  [(put [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe-wire::Bag::Reply::Put (:probe-wire::Bag::PutResponse::Ok
         (:wat::edn::write (:probe-wire::Bag::PutRequest/items req))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe-wire::Bag::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe-wire::bag-svc::Op])])))])

;; ── one round-trip, reporting whatever comes back ────────────────────────────
(:wat::core::defn :probe-wire::round-trip
  [c     <- (:wat::kernel::Peer :- [:probe-wire::Bag::Op :probe-wire::Bag::Reply])
   label <- :wat::core::String
   req   <- :probe-wire::Bag::PutRequest]
  -> :wat::core::nil
  (:wat::core::match (:probe-wire::Bag/put c req)
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:probe-wire::Bag::PutResponse::Ok seen)
          (:wat::kernel::println
            (:wat::string::concat label " => Ok, server saw items = " seen)))
        ((:probe-wire::Bag::PutResponse::RequestTooLarge bytes cap)
          (:wat::kernel::println
            (:wat::string::concat label " => RequestTooLarge")))
        ((:probe-wire::Bag::PutResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::println
        (:wat::string::concat label " => RecvOutcome::Lost: "
          (:wat::kernel::LociDiedError/message cause))))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::println
        (:wat::string::concat label " => RecvOutcome::Stopped")))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::println
        (:wat::string::concat label " => RecvOutcome::Closed")))))

;; ── one tier: stand up, connect, send a GOOD payload then a MISTYPED one ─────
(:wat::core::defn :probe-wire::measure-tier
  [locus <- :wat::spawn::Locus
   tier  <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::let
    [h (:probe-wire::bag-svc/start :locus locus
         :record (:probe-wire::bag-svc::Record :n 0))
     c (:wat::core::match (:wat::kernel::connect (:probe-wire::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)
           (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message f) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f)
           (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message f) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)
           (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message f) :wat::core::None :wat::core::None)))
     ;; CONTROL — a well-typed request, built by the normal ctor.
     good (:probe-wire::Bag::PutRequest
            :items (:wat::core::Vector :- [:wat::core::String] "a" "b"))
     ;; Show the exact wire form the ctor produces, so the hand-written EDN below
     ;; is provably the SAME tag with a wrong-typed body.
     _ (:wat::kernel::println
         (:wat::string::concat tier " control wire form = " (:wat::edn::write good)))
     _ (:probe-wire::round-trip c (:wat::string::concat tier " control  ") good)
     ;; THE PROBE — well-formed EDN, WRONG TYPE: i64s where (Vector :- [String]) is declared.
     bad (:wat::edn::read "#probe-wire.Bag/PutRequest {:items [1 2 3]}")
     _ (:wat::kernel::println
         (:wat::string::concat tier " mistyped wire form = " (:wat::edn::write bad)))
     _ (:probe-wire::round-trip c (:wat::string::concat tier " MISTYPED ") bad)]
    nil))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:probe-wire::measure-tier (:wat::spawn::thread)  "[thread] ")
    (:probe-wire::measure-tier (:wat::spawn::process) "[process]")))
