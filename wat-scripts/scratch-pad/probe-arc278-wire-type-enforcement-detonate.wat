;; wat-scripts/scratch-pad/probe-arc278-wire-type-enforcement-detonate.wat
;;
;; arc 278 — MEASUREMENT probe, part 2 (companion to
;; `probe-arc278-wire-type-enforcement.wat`; same BRIEF).
;;
;; Part 1 established that the wire ACCEPTS `{:items [1 2 3]}` under a declared
;; `items <- (Vector :- [String])` and hands the handler i64s (the handler's own
;; `edn::write` echoed `[1 2 3]`).
;;
;; THIS probe measures the CONSEQUENCE: what happens when the handler actually
;; USES the field at its declared type. The handler calls
;; `(:wat::core::string::length (:wat::core::nth items 0))` — legal, checked, and
;; correct against the DECLARATION. If the wire had enforced the type this could
;; never fail. The transcript records what the caller sees instead.
;;
;; MEASUREMENT ONLY — prints and exits; no assertion, nothing is fixed.

(:wat::core::defsurface :probe-det::Bag :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe-det::Bag::PutRequest
     [items <- (:wat::core::Vector :- [:wat::core::String])])
   (:wat::core::defenum :probe-det::Bag::PutResponse :wat::enum::Pure
     :Ok              [len <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(put [self <- :probe-det::Bag  req <- :probe-det::Bag::PutRequest]
     -> :probe-det::Bag::PutResponse :max-request-bytes 4096)])

(:wat::service::defservice :probe-det::bag-svc
  :satisfies :probe-det::Bag
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :impls
  ;; The handler uses `items[0]` AS A STRING — exactly what the declaration promises.
  [(put [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe-det::Bag::Reply::Put (:probe-det::Bag::PutResponse::Ok
         (:wat::string::length
           (:wat::core::nth (:probe-det::Bag::PutRequest/items req) 0))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe-det::Bag::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe-det::bag-svc::Op])])))])

(:wat::core::defn :probe-det::round-trip
  [c     <- (:wat::kernel::Peer :- [:probe-det::Bag::Op :probe-det::Bag::Reply])
   label <- :wat::core::String
   req   <- :probe-det::Bag::PutRequest]
  -> :wat::core::nil
  (:wat::core::match (:probe-det::Bag/put c req)
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:probe-det::Bag::PutResponse::Ok len)
          (:wat::kernel::println
            (:wat::string::concat label " => Ok, string::length = "
              (:wat::i64::to-string len))))
        ((:probe-det::Bag::PutResponse::RequestTooLarge bytes cap)
          (:wat::kernel::println
            (:wat::string::concat label " => RequestTooLarge")))
        ((:probe-det::Bag::PutResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ;; NB (measured): on this path the payload that actually arrives in the `Lost`
    ;; arm at runtime is a `:wat::kernel::Failure`, NOT the declared
    ;; `:wat::kernel::LociDiedError` — calling `LociDiedError/message` on it raises
    ;; a `TypeMismatch`. That is a SEPARATE substrate wrinkle, out of this probe's
    ;; scope; the arm prints a static label so the measurement transcript stays clean.
    ;; The reason text observed in that raise was:
    ;;   "service peer lost (reason on the owner's crash channel)" (wat/spawn.wat:351)
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::println
        (:wat::string::concat label
          " => RecvOutcome::Lost — THE SERVICE DIED serving this request")))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::println
        (:wat::string::concat label " => RecvOutcome::Stopped")))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::println
        (:wat::string::concat label " => RecvOutcome::Closed")))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:probe-det::bag-svc/start :locus (:wat::spawn::process)
         :record (:probe-det::bag-svc::Record :n 0))
     c (:wat::core::match (:wat::kernel::connect (:probe-det::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)
           (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message f) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f)
           (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message f) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)
           (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message f) :wat::core::None :wat::core::None)))
     good (:probe-det::Bag::PutRequest
            :items (:wat::core::Vector :- [:wat::core::String] "abcd"))
     _ (:probe-det::round-trip c "[process] control " good)
     bad (:wat::edn::read "#probe-det.Bag/PutRequest {:items [1 2 3]}")
     _ (:probe-det::round-trip c "[process] MISTYPED" bad)]
    nil))
