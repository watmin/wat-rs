;; probe-m1-service-pid.wat — LINCHPIN probe for the "service is a blessed concurrency
;; entry point" claim: can the owner read a started SERVICE's pid deterministically,
;; via peer-pid on the Handle's owner-side lineage peer (Handle/handle)?
;;
;; If YES → grant/revoke a service's pid needs NO spawn-program', NO racy post-spawn hook:
;;   (:probe::echo'::Handle/handle eh)  is a spawn-derived Process' peer → peer-pid → (Some pid).
;; This is the pid path a SERVICE-prober M1 needs (dogfooding service+bracket only).
;;
;; EXPECT (green): "service pid via Handle/handle:" then "#wat.core.Option/Some [<pid>]".

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s ctx req]
            (:wat::service::Outcome::Continue s
              (:wat::core::Some (:probe::Echo::Reply::Echo (:probe::Echo::EchoResponse::Ok (:probe::Echo::EchoRequest/msg req)))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Echo::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::echo::Op])])))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh  (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     lp  (:probe::echo::Handle/handle eh)
     _   (:wat::kernel::println "service pid via Handle/handle:")
     _   (:wat::kernel::println (:wat::kernel::peer-pid lp))]
    nil))
