;; v7: does MATCHING (not just constructing) a program-locally-declared enum, inside an ORDINARY
;; public op (no -on-connect/-on-disconnect declared at all), break :locus process child startup?
(:wat::core::defsurface :probe::Mini :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Mini::PingRequest [])
   (:wat::core::defenum :probe::Mini::PingResponse :wat::enum::Pure
     :Ok               [ok <- :wat::core::bool]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :probe::Mini  req <- :probe::Mini::PingRequest] -> :probe::Mini::PingResponse :max-request-bytes 524288)])

(:wat::core::defenum :probe::Mini::Tag :wat::enum::Pure
  :Closed   []
  :Lost     []
  :Rejected [])

(:wat::service::defservice :probe::mini
  :satisfies :probe::Mini
  :durable   [tag <- :probe::Mini::Tag]
  :ephemeral []
  :init (:wat::core::fn [record <- :probe::mini::Record] -> :probe::mini::State
          (:probe::mini::State :durable record))
  :impls
  [(ping [s ctx req]
     (:wat::core::let
       [t (:probe::mini::Record/tag (:probe::mini::State/durable s))
        ok (:wat::core::match t
             ((:probe::Mini::Tag::Closed) true)
             ((:probe::Mini::Tag::Lost) false)
             ((:probe::Mini::Tag::Rejected) false))]
       (:wat::service::Outcome::Reply s (:probe::Mini::PingResponse::Ok ok))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:probe::mini/start :locus (:wat::spawn::process) :record (:probe::mini::Record :tag (:probe::Mini::Tag::Closed)))
     c (:wat::core::match (:wat::kernel::connect (:probe::mini::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     r (:probe::Mini/ping c (:probe::Mini::PingRequest))]
    (:wat::kernel::println "ok")))
