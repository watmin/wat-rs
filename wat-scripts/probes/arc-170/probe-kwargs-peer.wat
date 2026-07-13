;; GROUNDING 2 — can a kwargs work-fn hold Peer' (impure) kwargs? i.e. is the <name>::Kwargs
;; bundle a struct that accepts resources (the dialed services), not a pure Record that'd reject them?
;; EXPECT (fix reachable): COMPILES — the work-fn declares kv/echo as Peer' kwargs, binds them in
;;   the body; the ::Kwargs bundle holds the impure peers.
;; EXPECT (wall): a nature/purity error rejecting Peer' in the kwargs bundle.
(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::Kv::GetReq  [k <- :wat::core::String])
             (:wat::core::defrecord :probe::Kv::GetResp [v <- :wat::core::String])]
  :features [(get [self <- :probe::Kv  req <- :probe::Kv::GetReq] -> :probe::Kv::GetResp)])
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::Echo::Req  [msg <- :wat::core::String])
             (:wat::core::defrecord :probe::Echo::Resp [reply <- :wat::core::String])]
  :features [(echo [self <- :probe::Echo  req <- :probe::Echo::Req] -> :probe::Echo::Resp)])

;; the bracket work-fn: item POSITIONAL, the services as Peer' KWARGS — bound directly in the body
(:wat::core::defn :probe::work
  [item <- :wat::core::String
   & [kv   <- :wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>
      echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>]]
  -> :wat::core::String
  (:probe::Echo::Resp/reply
    (:probe::Echo/echo echo
      (:probe::Echo::Req :msg
        (:probe::Kv::GetResp/v (:probe::Kv/get kv (:probe::Kv::GetReq :k item)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "kwargs work-fn with Peer' kwargs: defined + type-checked ok"))
