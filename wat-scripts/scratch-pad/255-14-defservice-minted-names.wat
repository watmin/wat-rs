;; arc 255 Stone 255.14 — MEASUREMENT, not a cure.
;; Q1 of the brief: "whether the ~140 runtime-minted service members all come from THAT
;; template (`wat/service.wat:2021-2023`, `{b}/{op-str}`)". The eighth draw reported the
;; number and said it narrowed BY ARGUMENT, not measurement. This expands a real
;; `defservice` at the FORM level (CLAUDE.md: a defservice cannot be runtime-macroexpanded)
;; and prints every emitted top-level form's HEAD and NAME, so the minted population is
;; READ rather than argued.
(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Kv::GetRequest [k <- :wat::core::String])
   (:wat::core::defenum :probe::Kv::GetResponse :wat::enum::Pure
     :Ok              [v <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String]) expected <- :wat::core::String got <- :wat::core::String])
   (:wat::core::defrecord :probe::Kv::PutRequest [k <- :wat::core::String v <- :wat::core::String])
   (:wat::core::defenum :probe::Kv::PutResponse :wat::enum::Pure
     :Ok              [ok <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String]) expected <- :wat::core::String got <- :wat::core::String])]
  :features
  [(get [self <- :probe::Kv req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse :max-request-bytes 524288)
   (put [self <- :probe::Kv req <- :probe::Kv::PutRequest] -> :probe::Kv::PutResponse :max-request-bytes 524288)])

(:wat::core::defn :probe::show [] -> :wat::core::nil
  (:wat::core::let
    [exp (:wat::core::macroexpand
           (:wat::core::quote
             (:wat::service::defservice :probe::kv
               :satisfies :probe::Kv :durable [] :ephemeral []
               :impls [(get [s ctx req] (:wat::service::Outcome.Reply {:state s :reply (:probe::Kv::GetResponse.Ok {:v "v"})}))
                       (put [s ctx req] (:wat::service::Outcome.Reply {:state s :reply (:probe::Kv::PutResponse.Ok {:ok 1})}))])))]
    (:wat::kernel::println
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::String f <- :wat::WatAST] -> :wat::core::String
          (:wat::core::let [ch (:wat::core::ast->children f)]
            (:wat::core::if (:wat::core::< (:wat::core::count ch) 1)
              (:wat::string::concat acc "<atom>  |  " (:wat::core::write-forms f) "\n")
              (:wat::string::concat acc
                (:wat::core::ast-name (:wat::core::first ch))
                "  |  "
                (:wat::core::if (:wat::core::< (:wat::core::count ch) 2)
                  "<no name slot>"
                  (:wat::core::write-forms (:wat::core::nth ch 1)))
                "\n"))))
        ""
        (:wat::core::ast->children exp)))))
(:wat::core::defn :user::main [] -> :wat::core::nil (:probe::show))
