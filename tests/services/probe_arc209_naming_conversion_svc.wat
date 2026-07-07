;; Arc 278 S4c migration: :ops RETIRED — the service wears a surface (:satisfies + :impls).
;; SUBJECT PRESERVED: a MULTI-WORD op must round-trip kebab<->pascal correctly through the
;; surface path. The impl clause names the op in kebab (`get-object`); the macro derives the
;; kebab client method `:my::svc/get-object` (service.wat:1044, op-str verbatim) AND the
;; pascal record/Op names `:my::Svc::GetObjectRequest` / `:my::Svc::Op::GetObject` via
;; `kebab->pascal-in` (service.wat:1041). If the multi-word conversion were broken (e.g.
;; "get-object" -> "Getobject"), the generated req-ty `:my::Svc::GetObjectRequest` would not
;; resolve to the user-declared record and startup would fail. Running the service end-to-end
;; therefore proves the kebab<->pascal derivation handles the multi-word op.
(:wat::core::defsurface :my::Svc :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :my::Svc::GetObjectRequest  [n <- :wat::core::i64])
   (:wat::core::defrecord :my::Svc::GetObjectResponse [value <- :wat::core::i64])]
  :features
  [(get-object [self <- :my::Svc  req <- :my::Svc::GetObjectRequest] -> :my::Svc::GetObjectResponse)])

(:wat::service::defservice :my::svc
  :satisfies :my::Svc
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get-object [s req]
     (:wat::service::Outcome::Reply s (:my::Svc::GetObjectResponse (:my::Svc::GetObjectRequest/n req))))])

;; End-to-end through the KEBAB client method `:my::svc/get-object` (multi-word); echoes the
;; request's n back as the response value (42), proving the whole multi-word wiring resolved.
(:wat::core::defn :user::req-id [] -> :wat::core::i64
  (:wat::core::let
    [h (:my::svc/start :locus (:wat::spawn::thread) :record (:my::svc::Record 0))
     c (:wat::kernel::connect' (:my::svc::Handle/addr h))
     r (:my::svc/get-object c (:my::Svc::GetObjectRequest 42))
     _ (:my::svc/stop h)]
    (:my::Svc::GetObjectResponse/value r)))
