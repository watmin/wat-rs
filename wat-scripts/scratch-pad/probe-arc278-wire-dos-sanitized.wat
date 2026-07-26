;; Arc 278 Stone 1 — probe-arc278-wire-dos-service-killed.wat, INVERTED.
;;
;; Byte-for-byte the DoS reproduction beside it, with exactly two things added:
;;   1. `:RequestMalformed` on the op's Response enum (the refusal the caller must face);
;;   2. `:sanitize-requests :all` on the service (the transitional Stone-1 gate).
;; The handler is UNCHANGED — it still uses the field at its declared type. That is the
;; point: the handler is correct against the declaration and must not defend itself.
;;
;; BEFORE (the sibling probe, still reproducible — it has neither of the two):
;;   "attacker good  => Ok"
;;   "attacker BAD   => LOST (peer gone)"
;;   victim: connect REFUSED — service is GONE
;;
;; AFTER (this probe): the attacker gets a NAMED refusal carrying the exact coordinate,
;; and the victim is SERVED. A bad caller, malicious or dumb, cannot crash anything.

(:wat::core::defsurface :dos2::Bag :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :dos2::Bag::PutRequest [items <- :wat::core::Vector<wat::core::String>])
   (:wat::core::defenum :dos2::Bag::PutResponse :wat::enum::Pure
     :Ok               [n <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path     <- :wat::core::Vector<wat::core::String>
                        expected <- :wat::core::String
                        got      <- :wat::core::String])]
  :features
  [(put [self <- :dos2::Bag  req <- :dos2::Bag::PutRequest]
     -> :dos2::Bag::PutResponse :max-request-bytes 4096)])

(:wat::service::defservice :dos2::bag-svc
  :satisfies :dos2::Bag
  :durable   [n <- :wat::core::i64]
  :ephemeral []
  :sanitize-requests :all
  :impls
  [(put [s req]
     ;; uses the field AT ITS DECLARED TYPE — correct against the declaration, unchanged
     (:wat::service::Outcome::Reply s
       (:dos2::Bag::PutResponse::Ok
         (:wat::core::string::length
           (:wat::core::nth (:dos2::Bag::PutRequest/items req) 0)))))])

(:wat::core::defn :dos2/try
  [c <- :wat::kernel::Peer'<dos2::Bag::Op,dos2::Bag::Reply>  label <- :wat::core::String
   req <- :dos2::Bag::PutRequest] -> :wat::core::nil
  (:wat::core::match (:dos2::Bag/put c req)
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:dos2::Bag::PutResponse::Ok n)
          (:wat::kernel::println (:wat::core::string::concat label " => Ok")))
        ((:dos2::Bag::PutResponse::RequestTooLarge b cap)
          (:wat::kernel::println (:wat::core::string::concat label " => TooLarge")))
        ((:dos2::Bag::PutResponse::RequestMalformed path expected got)
          (:wat::kernel::println
            (:wat::core::string::concat label
              (:wat::core::string::concat " => MALFORMED at "
                (:wat::core::string::concat (:wat::edn::write path)
                  (:wat::core::string::concat " expected="
                    (:wat::core::string::concat expected
                      (:wat::core::string::concat " got=" got))))))))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::println (:wat::core::string::concat label " => LOST (peer gone)")))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::println (:wat::core::string::concat label " => Closed")))))

(:wat::core::defn :dos2/run [locus <- :wat::spawn::Locus  tier <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let
    [h (:dos2::bag-svc/start :locus locus :record (:dos2::bag-svc::Record :n 0))
     good (:dos2::Bag::PutRequest :items (:wat::core::Vector :wat::core::String "abcd"))
     bad  (:wat::edn::read "#dos2.Bag/PutRequest {:items [1 2 3]}")
     ;; ATTACKER connection
     a (:wat::core::match (:wat::kernel::connect' (:dos2::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "refused" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "rejected" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "failed" :wat::core::None :wat::core::None)))
     _ (:dos2/try a (:wat::core::string::concat tier " attacker good ") good)
     _ (:dos2/try a (:wat::core::string::concat tier " attacker BAD  ") bad)
     ;; a SECOND, INNOCENT client connects AFTER the bad frame — THIS dial is the whole strike
     b (:wat::core::match (:wat::kernel::connect' (:dos2::bag-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused f)  (:wat::kernel::assertion-failed! "victim: connect REFUSED — service is GONE" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected f) (:wat::kernel::assertion-failed! "victim: connect REJECTED — service is GONE" :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed f)   (:wat::kernel::assertion-failed! "victim: connect FAILED — service is GONE" :wat::core::None :wat::core::None)))
     _ (:dos2/try b (:wat::core::string::concat tier " victim   good ") good)
     _ (:dos2::bag-svc/stop h)]
    nil))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:dos2/run (:wat::spawn::thread)  "thread ")
     _ (:dos2/run (:wat::spawn::process) "process")]
    nil))
