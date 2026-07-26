;; SEVERITY: after a mistyped frame, does the service still serve OTHER clients?
(:wat::core::defsurface :dos::Bag :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :dos::Bag::PutRequest [items <- :wat::core::Vector<wat::core::String>])
   (:wat::core::defenum :dos::Bag::PutResponse :wat::enum::Pure
     :Ok              [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(put [self <- :dos::Bag  req <- :dos::Bag::PutRequest]
     -> :dos::Bag::PutResponse :max-request-bytes 4096)])

(:wat::service::defservice :dos::bag-svc
  :satisfies :dos::Bag
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :impls
  [(put [s req]
     ;; uses the field AT ITS DECLARED TYPE — correct against the declaration
     (:wat::service::Outcome::Reply s
       (:dos::Bag::PutResponse::Ok
         (:wat::core::string::length
           (:wat::core::nth (:dos::Bag::PutRequest/items req) 0)))))])

(:wat::core::defn :dos/try
  [c <- :wat::kernel::Peer'<dos::Bag::Op,dos::Bag::Reply>  label <- :wat::core::String
   req <- :dos::Bag::PutRequest] -> :wat::core::nil
  (:wat::core::match (:dos::Bag/put c req)
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:dos::Bag::PutResponse::Ok n)
          (:wat::kernel::println (:wat::core::string::concat label " => Ok")))
        ((:dos::Bag::PutResponse::RequestTooLarge b cap)
          (:wat::kernel::println (:wat::core::string::concat label " => TooLarge")))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::println (:wat::core::string::concat label " => LOST (peer gone)")))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::println (:wat::core::string::concat label " => Closed")))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:dos::bag-svc/start :locus (:wat::spawn::process) :record (:dos::bag-svc::Record :n 0))
     good (:dos::Bag::PutRequest :items (:wat::core::Vector :wat::core::String "abcd"))
     bad  (:wat::edn::read "#dos.Bag/PutRequest {:items [1 2 3]}")
     ;; ATTACKER connection
     a (:wat::core::match (:wat::kernel::connect' (:dos::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "refused" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "rejected" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "failed" :wat::core::None :wat::core::None)))
     _ (:dos/try a "attacker good " good)
     _ (:dos/try a "attacker BAD  " bad)
     ;; a SECOND, INNOCENT client connects AFTER the bad frame
     b (:wat::core::match (:wat::kernel::connect' (:dos::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "victim: connect REFUSED — service is GONE" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "victim: connect REJECTED — service is GONE" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "victim: connect FAILED — service is GONE" :wat::core::None :wat::core::None)))]
    (:dos/try b "victim   good " good)))
