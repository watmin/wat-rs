;; C1 probe — inspect ast-kind/ast-name/ast->children on field-types-of's canonical
;; wat.type/ forms, so we know how to rebuild ":wat::kernel::Peer'<S,R>"-style
;; colon-angle-bracket keyword strings from them (matching the established idiom
;; the existing arity-6 AST-walk uses).
;;
;; DISPOSITION (255.75) — repair, and the goal above is now partly STALE: arc 109 "annihilate the
;; angle bracket" retired the colon-angle-bracket keyword-string spelling this probe was
;; reconnoitering toward, and 255.73's `probe-s3b-astsplice.wat` repair proved the working
;; mechanism splices the raw type AST nodes directly (quasiquote), never building an angle-bracket
;; NAME at all. The crash this probe hit (`ast-name requires a Symbol, Keyword, or StringLit
;; node`, line 37, calling `ast-name` on `arg2`) is `ast-name` correctly refusing a COMPOUND node
;; — measured here (`(:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])` is a 3-child
;; `list` node: a `symbol` head, a `:-` keyword, then a `vector` of the two type args — `arg2` IS
;; that vector, never nameable). Repaired by reporting `arg2`'s structural shape (kind + child
;; count + each element's own kind/name) instead of calling `ast-name` on the compound node, and
;; asserting the now-current, correct ground truth.
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- wat.type/String])
             (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- wat.type/String] :RequestTooLarge [bytes <- wat.type/i64  cap <- wat.type/i64]
                                                                                                                :RequestMalformed [path <- (wat.type/Vector :- [wat.type/String])  expected <- wat.type/String  got <- wat.type/String])]
  :features [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::core::defn :probe::work
  [item <- wat.type/String
   & [echo <- (:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])]]
  -> wat.type/String
  (:wat::core::match
    (:probe::Echo/echo echo (:probe::Echo::EchoRequest :msg item)) [:wat::kernel::RecvOutcome.Message {:msg __recv} (:wat::core::match __recv 
  [:probe::Echo::EchoResponse.Ok {:reply reply} reply]
  [:probe::Echo::EchoResponse.RequestTooLarge {:bytes bytes :cap cap}
    (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
  [:probe::Echo::EchoResponse.RequestMalformed {:path mpath :expected mexpected :got mgot}
    (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome.Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")]))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [types    (:wat::runtime::field-types-of :probe::work::Kwargs)
     ty0      (:wat::core::first types)
     kind0    (:wat::core::ast-kind ty0)
     ch0      (:wat::core::ast->children ty0)
     nch0     (:wat::core::length ch0)
     head     (:wat::core::first ch0)
     hkind    (:wat::core::ast-kind head)
     hname    (:wat::core::ast-name head)
     arg1     (:wat::core::Option/expect (:wat::core::get ch0 1) "no arg1")
     a1kind   (:wat::core::ast-kind arg1)
     a1name   (:wat::core::ast-name arg1)
     arg2     (:wat::core::Option/expect (:wat::core::get ch0 2) "no arg2")
     a2kind   (:wat::core::ast-kind arg2)
     a2ch     (:wat::core::ast->children arg2)
     a2nch    (:wat::core::length a2ch)
     a2e0     (:wat::core::Option/expect (:wat::core::get a2ch 0) "no a2e0")
     a2e0name (:wat::core::ast-name a2e0)
     a2e1     (:wat::core::Option/expect (:wat::core::get a2ch 1) "no a2e1")
     a2e1name (:wat::core::ast-name a2e1)
     names    (:wat::runtime::field-names-of :probe::work::Kwargs)
     nm0      (:wat::core::first names)]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat "ty0-kind: " kind0))
      (:wat::kernel::println (:wat::string::concat "nch0: " (:wat::string::interpolate "{n}" :n nch0)))
      (:wat::kernel::println (:wat::string::concat "head-kind: " hkind))
      (:wat::kernel::println (:wat::string::concat "head-name: " hname))
      (:wat::kernel::println (:wat::string::concat "arg1-kind: " a1kind))
      (:wat::kernel::println (:wat::string::concat "arg1-name: " a1name))
      (:wat::kernel::println (:wat::string::concat "arg2-kind: " a2kind))
      (:wat::kernel::println (:wat::string::concat "arg2-nch: " (:wat::string::interpolate "{n}" :n a2nch)))
      (:wat::kernel::println (:wat::string::concat "arg2-elem0-name: " a2e0name))
      (:wat::kernel::println (:wat::string::concat "arg2-elem1-name: " a2e1name))
      (:wat::kernel::println (:wat::keyword::to-string nm0))
      (:wat::test::assert-eq kind0 "list")
      (:wat::test::assert-eq hkind "symbol")
      (:wat::test::assert-eq hname "wat.kernel/Peer")
      (:wat::test::assert-eq a1kind "keyword")
      (:wat::test::assert-eq a1name ":-")
      (:wat::test::assert-eq a2kind "vector")
      (:wat::test::assert-eq a2nch 2)
      (:wat::test::assert-eq a2e0name "probe.Echo/Op")
      (:wat::test::assert-eq a2e1name "probe.Echo/Reply")
      (:wat::kernel::println "c1-ast-shape: ok"))))
