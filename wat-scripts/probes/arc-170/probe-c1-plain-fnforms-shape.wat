;; Ground truth — what does fn-forms render a PLAIN (non-kwargs) fn's own literal
;; param type annotations as? Mirrors spawn-runner's arity-6 c-nm derivation exactly.
;;
;; DISPOSITION (255.75) — repair: `c-ty` (the reified first param's type AST node) is a compound
;; `(Head :- [args])` list — `ast-name` correctly refuses it (same shape/same wall as
;; `probe-c1-ast-shape.wat`'s `arg2`, confirmed by hand-dump: `c-ty` is a 3-child list, head
;; keyword `:wat::kernel::Peer`, `:-`, then a 2-element vector of `:probe::Echo::Op` /
;; `:probe::Echo::Reply`). Repaired by reporting the structural shape instead of calling
;; `ast-name` on the compound node, and asserting the now-current ground truth.
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- wat.type/String])
             (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- wat.type/String] :RequestTooLarge [bytes <- wat.type/i64  cap <- wat.type/i64]
                                                                                                                :RequestMalformed [path <- (wat.type/Vector :- [wat.type/String])  expected <- wat.type/String  got <- wat.type/String])]
  :features [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::core::defn :probe::dial-work
  [peer <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])
   item <- wat.type/String]
  -> wat.type/String
  (:wat::core::match (:probe::Echo/echo peer (:probe::Echo::EchoRequest :msg item)) [:wat::kernel::RecvOutcome.Message {:msg __recv} (:wat::core::match __recv 
  [:probe::Echo::EchoResponse.Ok {:reply reply} reply]
  [:probe::Echo::EchoResponse.RequestTooLarge {:bytes bytes :cap cap}
    (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
  [:probe::Echo::EchoResponse.RequestMalformed {:path mpath :expected mexpected :got mgot}
    (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome.Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")]))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [work-name (:wat::keyword::from-string "user::bracket::work-fn")
     forms     (:wat::kernel::fn-forms :probe::dial-work work-name)
     def-node  (:wat::core::Option/expect (:wat::core::last forms) "no define")
     fn-form   (:wat::core::nth (:wat::core::ast->children def-node) 2)
     fn-ch     (:wat::core::ast->children fn-form)
     argspec   (:wat::core::nth fn-ch 1)
     arg-ch    (:wat::core::ast->children argspec)
     c-ty      (:wat::core::nth arg-ch 2)
     c-kind    (:wat::core::ast-kind c-ty)
     c-ch      (:wat::core::ast->children c-ty)
     c-nch     (:wat::core::length c-ch)
     c-head    (:wat::core::Option/expect (:wat::core::get c-ch 0) "no head")
     c-hname   (:wat::core::ast-name c-head)
     c-a1      (:wat::core::Option/expect (:wat::core::get c-ch 1) "no a1")
     c-a1name  (:wat::core::ast-name c-a1)
     c-a2      (:wat::core::Option/expect (:wat::core::get c-ch 2) "no a2")
     c-a2kind  (:wat::core::ast-kind c-a2)
     c-a2ch    (:wat::core::ast->children c-a2)
     c-a2nch   (:wat::core::length c-a2ch)
     c-a2e0    (:wat::core::Option/expect (:wat::core::get c-a2ch 0) "no e0")
     c-a2e0nm  (:wat::core::ast-name c-a2e0)
     c-a2e1    (:wat::core::Option/expect (:wat::core::get c-a2ch 1) "no e1")
     c-a2e1nm  (:wat::core::ast-name c-a2e1)]
    (:wat::core::do
      (:wat::kernel::println forms)
      (:wat::kernel::println (:wat::string::concat "c-kind: " c-kind))
      (:wat::kernel::println (:wat::string::concat "c-nch: " (:wat::string::interpolate "{n}" :n c-nch)))
      (:wat::kernel::println (:wat::string::concat "c-hname: " c-hname))
      (:wat::kernel::println (:wat::string::concat "c-a1name: " c-a1name))
      (:wat::kernel::println (:wat::string::concat "c-a2kind: " c-a2kind))
      (:wat::kernel::println (:wat::string::concat "c-a2nch: " (:wat::string::interpolate "{n}" :n c-a2nch)))
      (:wat::kernel::println (:wat::string::concat "c-a2e0nm: " c-a2e0nm))
      (:wat::kernel::println (:wat::string::concat "c-a2e1nm: " c-a2e1nm))
      (:wat::test::assert-eq c-kind "list")
      (:wat::test::assert-eq c-nch 3)
      (:wat::test::assert-eq c-hname ":wat::kernel::Peer")
      (:wat::test::assert-eq c-a1name ":-")
      (:wat::test::assert-eq c-a2kind "vector")
      (:wat::test::assert-eq c-a2nch 2)
      (:wat::test::assert-eq c-a2e0nm ":probe::Echo::Op")
      (:wat::test::assert-eq c-a2e1nm ":probe::Echo::Reply")
      (:wat::kernel::println "c1-plain-fnforms-shape: ok"))))
