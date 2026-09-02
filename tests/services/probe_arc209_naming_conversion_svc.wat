;; Arc 278 S4c migration: :ops RETIRED — the service wears a surface (:satisfies + :impls).
;; SUBJECT PRESERVED: a MULTI-WORD op must round-trip kebab<->pascal correctly through the
;; surface path. The impl clause names the op in kebab (`get-object`); the macro derives the
;; kebab client method `:my::svc/get-object` (service.wat:1044, op-str verbatim) AND the
;; pascal record/Op names `:my::Svc::GetObjectRequest` / `:my::Svc::Op::GetObject` via
;; `kebab->pascal-in` (service.wat:1041). If the multi-word conversion were broken (e.g.
;; "get-object" -> "Getobject"), the generated req-ty `:my::Svc::GetObjectRequest` would not
;; resolve to the user-declared record and startup would fail. Running the service end-to-end
;; therefore proves the kebab<->pascal derivation handles the multi-word op.
(:wat::core::defsurface :my::Svc :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::Svc::GetObjectRequest  [n <- :wat::core::i64])
   (:wat::core::defenum :my::Svc::GetObjectResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get-object [self <- :my::Svc  req <- :my::Svc::GetObjectRequest] -> :my::Svc::GetObjectResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::svc
  :satisfies :my::Svc
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get-object [s ctx req]
     (:wat::service::Outcome::Continue s (:wat::core::Some (:my::Svc::Reply::GetObject (:my::Svc::GetObjectResponse::Ok (:my::Svc::GetObjectRequest/n req)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:my::Svc::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:my::svc::Op])])))])

;; End-to-end through the KEBAB client method `:my::svc/get-object` (multi-word); echoes the
;; request's n back as the response value (42), proving the whole multi-word wiring resolved.
(:wat::core::defn :user::req-id [] -> :wat::core::i64
  (:wat::core::let
    [h (:my::svc/start :locus (:wat::spawn::thread) :record (:my::svc::Record :count 0))
     c (:wat::core::match (:wat::kernel::connect (:my::svc::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     r (:my::svc/get-object c (:my::Svc::GetObjectRequest :n 42))
     _ (:my::svc/stop h)]
    (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:my::Svc::GetObjectResponse::Ok value) value)
      ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
      ((:my::Svc::GetObjectResponse::RequestTooLarge bytes cap)
        (:wat::kernel::assertion-failed! "req-id: unexpected RequestTooLarge"
          :wat::core::None :wat::core::None))
      ((:my::Svc::GetObjectResponse::RequestMalformed mpath mexpected mgot)
        (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
