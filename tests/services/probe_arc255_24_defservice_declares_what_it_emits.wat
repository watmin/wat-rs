;; tests/services/probe_arc255_24_defservice_declares_what_it_emits.wat — stone 255.24 (C-b2).
;;
;; Driven by `probe_arc255_24_defservice_declares_what_it_emits.rs`. Two things live here:
;;
;; 1. `:probe::emitted-defn-binders` — the EMITTED-FORM census as a value. It expands a monomorphic
;;    and a `:- [K V]` defservice at the FORM level (a defservice cannot be runtime-macroexpanded;
;;    the shape is `wat-scripts/scratch-pad/255-19-start-impl-locus-param.wat`'s) and returns, for
;;    every generated `defn` at the top of the expansion (and one `do` deep, where start/resume
;;    sit), `"<name> <binder>"` — the binder rendered, or `-` when the defn has none. Before 255.24
;;    every defn below whose signature names `K`/`V`/`T` read `-` (the letters were type
;;    parameters by SPELLING); after it, each declares exactly the letters its signature uses.
;;    The child `:user::main` is not at this depth (it sits inside `service-forms`' quoted body);
;;    it is excluded by the brief (C-b4) and the census names it separately.
;;
;; 2. `start$impl` of a `:- [K V]` service, called POSITIONALLY with a concrete locus: a thread
;;    locus gives `(Handle :- [String i64 Shared])`, a process locus `… Wire`, and an abstract
;;    `(Locus :- [T])` passes its `T` through. The refused claims are in
;;    `probe_arc255_24_start_impl_claims_the_other_transport.wat.bad`.
(:wat::core::defsurface :probe::Pair :- [K V] :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Pair::PutRequest [item <- :wat::core::i64])
   (:wat::core::defenum :probe::Pair::PutResponse :wat::enum::Pure
     :Ok               [echo <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(put [self <- (:probe::Pair :- [K V])  req <- :probe::Pair::PutRequest]
     -> :probe::Pair::PutResponse :max-request-bytes 1024)])

(:wat::service::defservice :probe::pair-svc :- [K V]
  :satisfies (:probe::Pair :- [K V])
  :durable   [k <- (:wat::core::Option :- [K])  v <- (:wat::core::Option :- [V])]
  :ephemeral []
  :impls
  [(put [s ctx req]
     (:wat::service::Outcome.Reply {:state s
       :reply (:probe::Pair::PutResponse.Ok {:echo (:probe::Pair::PutRequest/item req)})}))])

(:wat::core::defn :probe::seed [] -> (:probe::pair-svc::Record :- [:wat::core::String :wat::core::i64])
  (:probe::pair-svc::Record
    :k (:wat::core::Option.Some {:value "hi"})
    :v (:wat::core::Option.Some {:value 42})))

;; ── the start$impl rows (accepted) ──────────────────────────────────────────────────────────
(:wat::core::defn :probe::thread-is-shared []
  -> (:probe::pair-svc::Handle :- [:wat::core::String :wat::core::i64 :wat::kernel::Shared])
  (:probe::pair-svc/start$impl (:wat::spawn::thread) (:probe::seed)))

(:wat::core::defn :probe::process-is-wire []
  -> (:probe::pair-svc::Handle :- [:wat::core::String :wat::core::i64 :wat::kernel::Wire])
  (:probe::pair-svc/start$impl (:wat::spawn::process) (:probe::seed)))

(:wat::core::defn :probe::abstract-locus-passes-its-transport :- [T]
  [locus <- (:wat::spawn::Locus :- [T])]
  -> (:probe::pair-svc::Handle :- [:wat::core::String :wat::core::i64 T])
  (:probe::pair-svc/start$impl locus (:probe::seed)))

;; ── the emitted-form census, as a value ─────────────────────────────────────────────────────
(:wat::core::defn :probe::defn-row [form <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let [cs (:wat::core::ast->children form)]
    (:wat::core::if (:wat::core::< (:wat::core::length cs) 3)
      (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::if (:wat::core::= (:wat::core::ast-name (:wat::core::first cs)) ":wat::core::defn")
        (:wat::core::Vector :- [:wat::core::String]
          (:wat::string::concat
            (:wat::core::ast-name (:wat::core::nth cs 1))
            " "
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::nth cs 2)) "keyword")
              (:wat::core::if (:wat::core::= (:wat::core::ast-name (:wat::core::nth cs 2)) ":-")
                (:wat::core::write-forms (:wat::core::nth cs 3))
                "-")
              "-")))
        (:wat::core::Vector :- [:wat::core::String])))))

(:wat::core::defn :probe::is-list-headed? [form <- :wat::WatAST head <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind form) "list")
    (:wat::core::let [cs (:wat::core::ast->children form)]
      (:wat::core::if (:wat::core::empty? cs)
        false
        (:wat::core::= (:wat::core::ast-name (:wat::core::first cs)) head)))
    false))

(:wat::core::defn :probe::defn-rows [exp <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) c <- :wat::WatAST]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::if (:probe::is-list-headed? c ":wat::core::do")
        (:wat::core::foldl
          (:wat::core::fn [acc2 <- (:wat::core::Vector :- [:wat::core::String]) cc <- :wat::WatAST]
            -> (:wat::core::Vector :- [:wat::core::String])
            (:wat::core::if (:probe::is-list-headed? cc ":wat::core::defn")
              (:wat::core::concat acc2 (:probe::defn-row cc))
              acc2))
          acc
          (:wat::core::rest (:wat::core::ast->children c)))
        (:wat::core::if (:probe::is-list-headed? c ":wat::core::defn")
          (:wat::core::concat acc (:probe::defn-row c))
          acc)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::rest (:wat::core::ast->children exp))))

(:wat::core::defn :probe::emitted-defn-binders [] -> (:wat::core::Tuple :- [(:wat::core::Vector :- [:wat::core::String]) (:wat::core::Vector :- [:wat::core::String])])
  (:wat::core::Tuple
    (:probe::defn-rows
      (:wat::core::macroexpand
        (:wat::core::quote
          (:wat::service::defservice :probe::mono-svc
            :satisfies :probe::Mono :durable [] :ephemeral []
            :impls [(get [s ctx req] (:wat::service::Outcome.Reply {:state s :reply req}))]))))
    (:probe::defn-rows
      (:wat::core::macroexpand
        (:wat::core::quote
          (:wat::service::defservice :probe::pair-svc :- [K V]
            :satisfies (:probe::Pair :- [K V])
            :durable   [k <- (:wat::core::Option :- [K])  v <- (:wat::core::Option :- [V])]
            :ephemeral []
            :impls [(put [s ctx req] (:wat::service::Outcome.Reply {:state s :reply req}))]))))))

(:wat::core::defsurface :probe::Mono :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Mono::GetRequest [k <- :wat::core::String])
   (:wat::core::defenum :probe::Mono::GetResponse :wat::enum::Pure
     :Ok               [v <- :wat::core::String]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get [self <- :probe::Mono  req <- :probe::Mono::GetRequest] -> :probe::Mono::GetResponse :max-request-bytes 1024)])
