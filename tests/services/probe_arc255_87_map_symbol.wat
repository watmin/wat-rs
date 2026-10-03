;; Symbol spelling of probe-c1-clean-surface. The work function is `probe/work`,
;; not `:probe::work`. Both must mint `:probe::work::kwargs-check`.
(wat.core/defsurface probe/Echo :nature wat.kernel/Peer
  :messages
  [(wat.core/defrecord probe.Echo/EchoRequest  [msg   :- wat.type/String])
   (wat.core/defenum probe.Echo/EchoResponse wat.enum/Pure :Ok [reply :- wat.type/String] :RequestTooLarge [bytes :- wat.type/i64  cap :- wat.type/i64]
                                                                                                      :RequestMalformed [path :- (wat.type/Vector :- [wat.type/String])  expected :- wat.type/String  got :- wat.type/String])]
  :features
  [(echo [self :- probe/Echo  req :- probe.Echo/EchoRequest] :- probe.Echo/EchoResponse :max-request-bytes 524288)])

(wat.service/defservice probe/echo
  :satisfies probe/Echo  :durable []  :ephemeral []
  :impls [(echo [s ctx req]
            (wat.service/Outcome.Reply {:state s
              :reply (probe.Echo/EchoResponse.Ok {:reply (wat.string/concat "echo:" (probe.Echo.EchoRequest/msg req))})}))])

(wat.core/defn probe/work
  [item :- wat.type/String
   & [echo :- (wat.kernel/Peer :- [probe.Echo/Op probe.Echo/Reply])]]
  :- wat.type/String
  (wat.core/match
    (probe.Echo/echo echo (probe.Echo/EchoRequest :msg item)) [wat.kernel/RecvOutcome.Message {:msg __recv} (wat.core/match __recv
  [probe.Echo/EchoResponse.Ok {:reply reply} reply]
  [probe.Echo/EchoResponse.RequestTooLarge {:bytes bytes :cap cap}
    (wat.kernel/assertion-failed! :message "unexpected RequestTooLarge")]
  [probe.Echo/EchoResponse.RequestMalformed {:path mpath :expected mexpected :got mgot}
    (wat.kernel/assertion-failed! :message "unexpected RequestMalformed")])] [wat.kernel/RecvOutcome.Lost {:cause __cause} (wat.kernel/assertion-failed! :message (wat.kernel.LociDiedError/message __cause))] [wat.kernel/RecvOutcome.Stopped {} (wat.kernel/assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [wat.kernel/RecvOutcome.Closed {} (wat.kernel/assertion-failed! :message "recv': peer closed")]))

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [eh    (probe.echo/start :locus (wat.spawn/process) :record (probe.echo/Record))
     out   (wat.bracket/map (wat.spawn/process) ["a" "b" "c"] probe/work :echo eh)]
    (wat.kernel/println out)))
