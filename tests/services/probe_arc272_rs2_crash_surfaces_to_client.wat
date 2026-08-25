;; Arc 278 S4c migration: :ops RETIRED — the service wears a surface (:satisfies + :impls).
;; SUBJECT UNCHANGED: a service with one op whose handler CRASHES (assertion-failed! raises
;; inside the serve loop). The far-side crash must SURFACE to the client as a raise.
(:wat::core::defsurface :my::Svc :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::Svc::BoomRequest  [])
   (:wat::core::defenum :my::Svc::BoomResponse :wat::enum::Pure
     :Ok              [ok <- :wat::core::bool]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(boom [self <- :my::Svc  req <- :my::Svc::BoomRequest] -> :my::Svc::BoomResponse :max-request-bytes 524288)])

(:wat::service::defservice :my::svc
  :satisfies :my::Svc
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(boom [s ctx req]
     (:wat::kernel::assertion-failed!
       "boom — the handler crashed on purpose"
       (:wat::core::Some "boom")
       (:wat::core::Some "ok")))])

;; arc 278 recv'-wall: recv' surfaces the far-side crash as a MATCHABLE RecvOutcome::Lost VALUE
;; (never a raise — a raise unwinds past the reader, which is the mask the wall kills). The client
;; gets a reason-free 500 (the crash reason is administrative, on the owner's channel). We MATCH and
;; RETURN a marker: "LOST:<administrative msg>" on the crash, "MESSAGE"/"CLOSED" otherwise — the .rs
;; asserts the crash surfaced as ::Lost (not a mute ::Closed, not a fake ::Message, not a hang).
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [h  (:my::svc/start :locus (:wat::spawn::thread) :record (:my::svc::Record :count 0))
     c  (:wat::core::match (:wat::kernel::connect (:my::svc::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _s (:wat::kernel::send c (:my::Svc::Op::Boom (:my::Svc::BoomRequest)))]
    (:wat::core::match (:wat::kernel::recv c)
      ((:wat::kernel::RecvOutcome::Message _m) "MESSAGE")
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::string::concat "LOST:" (:wat::kernel::LociDiedError/message cause)))
      (:wat::kernel::RecvOutcome::Stopped "STOPPED")
      (:wat::kernel::RecvOutcome::Closed "CLOSED"))))
