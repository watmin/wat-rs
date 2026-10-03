;; CLAIM: a map-worker process bracket, over a locus that ALSO hosts a live defservice,
;; produces [2 4 6 8 10] — the service's presence doesn't disturb the plain-fn pool path.
(wat.core/defsurface probe/Echo :nature wat.kernel/Peer
  :messages
  [(wat.core/defrecord probe.Echo/EchoRequest  [msg   :- wat.type/String])
   (wat.core/defenum probe.Echo/EchoResponse wat.enum/Pure :Ok [reply :- wat.type/String] :RequestTooLarge [bytes :- wat.type/i64  cap :- wat.type/i64]
                                                                                                      :RequestMalformed [path :- (wat.type/Vector :- [wat.type/String])  expected :- wat.type/String  got :- wat.type/String])]
  :features
  [(echo [self :- probe/Echo  req :- probe.Echo/EchoRequest] :- probe.Echo/EchoResponse :max-request-bytes 524288)])
(wat.service/defservice probe/echo
  :satisfies probe/Echo  :durable [] :ephemeral []
  :impls [(echo [s ctx req] (wat.service/Outcome.Reply {:state s
                          :reply (probe.Echo/EchoResponse.Ok {:reply (probe.Echo.EchoRequest/msg req)})}))])
(wat.core/defn probe/double [n :- wat.type/i64] :- wat.type/i64 (wat.i64/* n 2))
(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [nums (wat.type/Vector :- [wat.type/i64] 1 2 3 4 5)
     pr   (wat.bracket/map (wat.spawn/process) nums probe/double)]
    (wat.core/do
      (wat.test/assert-eq pr (wat.type/Vector :- [wat.type/i64] 2 4 6 8 10))
      (wat.kernel/println (wat.edn/write pr)))))
