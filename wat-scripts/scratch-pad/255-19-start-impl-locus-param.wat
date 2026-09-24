;; arc 255 Stone 255.19 — MEASUREMENT. Expands two real `defservice` forms at the FORM level
;; (a defservice cannot be runtime-macroexpanded) and prints every generated `start$impl*` /
;; `resume$impl*` defn head, so the abstract impl's LOCUS PARAMETER and its type binder are
;; READ, not argued. One monomorphic service and one parametric (`:- [K V]`, the stdlib shape).
(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Kv::GetRequest [k <- :wat::core::String])
   (:wat::core::defenum :probe::Kv::GetResponse :wat::enum::Pure
     :Ok              [v <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String]) expected <- :wat::core::String got <- :wat::core::String])]
  :features
  [(get [self <- :probe::Kv req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse :max-request-bytes 524288)])

(:wat::core::defn :probe::impl-heads [exp <- :wat::WatAST] -> :wat::core::nil
  (:wat::core::let [s (:wat::core::write-forms exp)
                    parts (:wat::string::split s "(:wat.core/defn ")]
    (:wat::core::foldl
      (:wat::core::fn [_acc <- :wat::core::nil p <- :wat::core::String] -> :wat::core::nil
        (:wat::core::if (:wat::string::contains? (:wat::string::subs p 0 (:wat::core::if (:wat::core::< (:wat::string::length p) 80) (:wat::string::length p) 80)) "$impl")
          (:wat::kernel::println
            (:wat::string::subs p 0 (:wat::core::if (:wat::core::< (:wat::string::length p) 260) (:wat::string::length p) 260)))
          nil))
      nil
      parts)))

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
    (:wat::core::do
      (:wat::kernel::println "=== MONO ===")
      (:probe::impl-heads mono)
      (:wat::kernel::println "=== PARA ===")
      (:probe::impl-heads para))))
