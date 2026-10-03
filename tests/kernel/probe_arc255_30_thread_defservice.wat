;; Stone 255.30 — a defservice on a thread locus runs. Status.Started carries
;; a Shared address, which 255.29 made data.
(wat.core/defsurface p30/Echo :nature wat.kernel/Peer
  :messages
  [(wat.core/defrecord p30.Echo/EchoRequest [msg :- wat.type/String])
   (wat.core/defenum p30.Echo/EchoResponse wat.enum/Pure
     :Ok [reply :- wat.type/String]
     :RequestTooLarge [bytes :- wat.type/i64 cap :- wat.type/i64]
     :RequestMalformed [path :- (wat.type/Vector :- [wat.type/String])
                        expected :- wat.type/String
                        got :- wat.type/String])]
  :features
  [(echo [self :- p30/Echo req :- p30.Echo/EchoRequest] :- p30.Echo/EchoResponse
     :max-request-bytes 524288)])

(wat.service/defservice p30/echo
  :satisfies p30/Echo
  :durable []
  :ephemeral []
  :impls
  [(echo [s ctx req]
     (wat.service/Outcome.Reply {:state s
       :reply (p30.Echo/EchoResponse.Ok
         {:reply (wat.string/concat "echo:" (p30.Echo.EchoRequest/msg req))})}))])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [h (p30.echo/start :locus (wat.spawn/thread) :record (p30.echo/Record))
     a (p30.echo.Handle/addr h)
     p (wat.core/match (wat.kernel/connect a)
         [wat.kernel/ConnectOutcome.Connected {:peer peer} peer]
         [wat.kernel/ConnectOutcome.Closed {:cause c}
           (wat.kernel/assertion-failed! :message (wat.kernel.Failure/message c))]
         [wat.kernel/ConnectOutcome.Undialable {:cause c}
           (wat.kernel/assertion-failed! :message (wat.kernel.Failure/message c))] [wat.kernel/ConnectOutcome.WrongPeer {:cause c}
           (wat.kernel/assertion-failed! :message (wat.kernel.Failure/message c))]
         [wat.kernel/ConnectOutcome.Failed {:cause c}
           (wat.kernel/assertion-failed! :message (wat.kernel.Failure/message c))])
     r (wat.core/match (p30.Echo/echo p (p30.Echo/EchoRequest :msg "hi"))
         [wat.kernel/RecvOutcome.Message {:msg m}
           (wat.core/match m
             [p30.Echo/EchoResponse.Ok {:reply reply} reply]
             [p30.Echo/EchoResponse.RequestTooLarge {:bytes _b :cap _c}
               (wat.kernel/assertion-failed! :message "too-large")]
             [p30.Echo/EchoResponse.RequestMalformed {:path _p :expected _e :got _g}
               (wat.kernel/assertion-failed! :message "malformed")])]
         [wat.kernel/RecvOutcome.Lost {:cause c}
           (wat.kernel/assertion-failed! :message (wat.kernel.LociDiedError/message c))]
         [wat.kernel/RecvOutcome.Stopped {}
           (wat.kernel/assertion-failed! :message "stopped")]
         [wat.kernel/RecvOutcome.Closed {}
           (wat.kernel/assertion-failed! :message "closed")])]
    (wat.kernel/println r)))
