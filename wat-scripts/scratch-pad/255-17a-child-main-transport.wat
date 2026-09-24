;; arc 255 Stone 255.17a — MEASUREMENT. Expands two real `defservice` forms at the FORM level
;; (a defservice cannot be runtime-macroexpanded) and prints the generated `:user::main` child
;; form, so which type in it carries the transport slot is READ, not argued. One monomorphic
;; service (`:probe::kv`) and one parametric (`:wat::cache::lru-svc :- [K V]`, the stdlib shape).
(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Kv::GetRequest [k <- :wat::core::String])
   (:wat::core::defenum :probe::Kv::GetResponse :wat::enum::Pure
     :Ok              [v <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String]) expected <- :wat::core::String got <- :wat::core::String])]
  :features
  [(get [self <- :probe::Kv req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse :max-request-bytes 524288)])

(:wat::core::defn :probe::child-main-of [exp <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [s (:wat::core::write-forms exp)
                    parts (:wat::string::split s "(:wat.core/defn :user/main")]
    (:wat::core::if (:wat::core::< (:wat::core::count parts) 2)
      "<no child main found>"
      (:wat::string::concat "(:wat.core/defn :user/main" (:wat::core::nth parts 1)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [mono (:wat::core::macroexpand
            (:wat::core::quote
              (:wat::service::defservice :probe::kv
                :satisfies :probe::Kv :durable [] :ephemeral []
                :impls [(get [s ctx req] (:wat::service::Outcome.Reply {:state s :reply (:probe::Kv::GetResponse.Ok {:v "v"})}))])))
     para (:wat::core::macroexpand
            (:wat::core::quote
              (:wat::service::defservice :wat::cache::lru-svc :- [K V]
                :satisfies (:wat::cache::Cache :- [K V])
                :durable   [capacity <- :wat::core::i64]
                :ephemeral [cache <- (:wat::cache::Lru :- [K V])]
                :init (:wat::core::fn [record <- (:wat::cache::lru-svc::Record :- [K V])]
                        -> (:wat::cache::lru-svc::State :- [K V])
                        (:wat::cache::lru-svc::State
                          :durable record
                          :cache (:wat::cache::Lru/new (:wat::cache::lru-svc::Record/capacity record))))
                :impls [(get [s ctx req] (:wat::service::Outcome.Reply {:state s :reply req}))
                        (put [s ctx req] (:wat::service::Outcome.Reply {:state s :reply req}))])))]
    (:wat::core::let [_a (:wat::kernel::println "=== MONO child main ===")
                      _b (:wat::kernel::println (:probe::child-main-of mono))
                      _c (:wat::kernel::println "=== PARA child main ===")]
      (:wat::kernel::println (:probe::child-main-of para)))))
