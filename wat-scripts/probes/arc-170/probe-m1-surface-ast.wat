;; Inspect a defsurface form's AST children (from fn-forms output) to find its :messages.

(wat.core/defsurface probe/Echo :nature wat.kernel/Peer
  :messages
  [(wat.core/defrecord probe.Echo/EchoRequest  [msg   :- wat.type/String])
   (wat.core/defenum probe.Echo/EchoResponse wat.enum/Pure :Ok [reply :- wat.type/String] :RequestTooLarge [bytes :- wat.type/i64  cap :- wat.type/i64]
                                                                                                      :RequestMalformed [path :- (wat.type/Vector :- [wat.type/String])  expected :- wat.type/String  got :- wat.type/String])]
  :features
  [(echo [self :- probe/Echo  req :- probe.Echo/EchoRequest] :- probe.Echo/EchoResponse :max-request-bytes 524288)])

(wat.service/defservice probe/echo
  :satisfies probe/Echo  :durable [] :ephemeral []
  :impls [(echo [s ctx req]
            (wat.service/Outcome.Reply {:state s
              :reply (probe.Echo/EchoResponse.Ok {:reply (wat.string/concat "echo:" (probe.Echo.EchoRequest/msg req))})}))])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [wf (wat.core/fn [c :- (wat.kernel/Peer :- [probe.Echo/Op probe.Echo/Reply])  s :- wat.type/String]
             :- wat.type/String
           (wat.core/match (probe.Echo/echo c (probe.Echo/EchoRequest :msg s)) [wat.kernel/RecvOutcome.Message {:msg __recv} (wat.core/match __recv
  [probe.Echo/EchoResponse.Ok {:reply reply} reply]
  [probe.Echo/EchoResponse.RequestTooLarge {:bytes bytes :cap cap}
    (wat.kernel/assertion-failed! :message "unexpected RequestTooLarge")]
  [probe.Echo/EchoResponse.RequestMalformed {:path mpath :expected mexpected :got mgot}
    (wat.kernel/assertion-failed! :message "unexpected RequestMalformed")])] [wat.kernel/RecvOutcome.Lost {:cause __cause} (wat.kernel/assertion-failed! :message (wat.kernel.LociDiedError/message __cause))] [wat.kernel/RecvOutcome.Stopped {} (wat.kernel/assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [wat.kernel/RecvOutcome.Closed {} (wat.kernel/assertion-failed! :message "recv': peer closed")]))
     forms (wat.kernel/fn-forms wf (wat.keyword/from-string "user::bracket::work-fn"))]
    (wat.core/foldl
      (wat.core/fn [_a :- wat.type/nil  f :- wat.type/AST] :- wat.type/nil
        (wat.core/let
          [ch (wat.core/ast->children f)
           hd (wat.core/ast-name (wat.core/first ch))]
          (wat.core/if (wat.core/= hd ":wat::core::defsurface")
            (wat.core/do
              (wat.kernel/println "=== defsurface children ast-kinds ===")
              (wat.core/foldl
                (wat.core/fn [_b :- wat.type/nil  cc :- wat.type/AST] :- wat.type/nil
                  (wat.kernel/println (wat.core/ast-kind cc)))
                nil
                ch))
            nil)))
      nil
      forms)))
