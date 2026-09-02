;; Arc 170 wrong-service compile error — POSITIVE control.
;; Two services (:probe::echo' / :probe::kv'); :user::main ascribes ONLY the
;; correctly-typed coord (echo handle's auto-emitted Dialable/coord -> Echo address).
;; No hand-written defsurface/extend-type for Dialable — it is BAKED (wat/capability.wat)
;; and AUTO-EMITTED per-service (wat/service.wat). EXPECT: freezes clean.

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
     :Ok              [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable []  :ephemeral []
  :impls [(echo [s ctx req]
            (:wat::service::Outcome::Continue s
              (:wat::core::Some (:probe::Echo::Reply::Echo (:probe::Echo::EchoResponse::Ok (:probe::Echo::EchoRequest/msg req)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::echo::Op])])))])

(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Kv::GetRequest  [k <- :wat::core::String])
   (:wat::core::defenum :probe::Kv::GetResponse :wat::enum::Pure
     :Ok              [v <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get [self <- :probe::Kv  req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::kv
  :satisfies :probe::Kv  :durable []  :ephemeral []
  :impls [(get [s ctx req]
            (:wat::service::Outcome::Continue s
              (:wat::core::Some (:probe::Kv::Reply::Get (:probe::Kv::GetResponse::Ok (:probe::Kv::GetRequest/k req)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Kv::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::kv::Op])])))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh  (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     kvh (:probe::kv/start   :locus (:wat::spawn::process) :record (:probe::kv::Record))
     ok  (:wat::core::ann-form (:wat::capability::Dialable/coord eh)
           (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply]))]
    (:wat::kernel::println "measured")))
