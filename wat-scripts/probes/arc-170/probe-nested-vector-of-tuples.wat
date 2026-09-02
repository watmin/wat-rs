;; probe-nested-vector-of-tuples.wat — the NESTED case Part 3 (retiring as-capability)
;; depends on: process/uses-pairs takes (Vector :- [(:wat::core::Tuple :- [:wat::core::keyword :wat::capability::Capability])]); the macro splices a
;; Vector literal whose ELEMENTS are (:wat::core::Tuple k v) ctor calls. Each Tuple element
;; must up-cast RECURSIVELY (Vector's check_vector_literal_against dispatches each item
;; through check_compound_against_expected, not bare infer) — a Handle inside a Tuple
;; inside a Vector literal, direct construction, no as-capability wrapping.
;; EXPECT (green): "nested-upcast: ok"
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
             (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                                :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo :satisfies :probe::Echo :durable [] :ephemeral []
  :impls [(echo [s ctx req] (:wat::service::Outcome::Continue s
            (:wat::core::Some (:probe::Echo::Reply::Echo (:probe::Echo::EchoResponse::Ok (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::echo::Op])])))])

(:wat::core::defn :probe::as-pairs [hs <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::keyword :wat::capability::Capability])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::keyword :wat::capability::Capability])])
  hs)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ;; direct Handle, no as-capability — Vector of Tuple ctor calls, both levels up-cast.
     hs (:probe::as-pairs [(:wat::core::Tuple :echo eh)])]
    (:wat::kernel::println "nested-upcast: ok")))
