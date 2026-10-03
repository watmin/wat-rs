;; probe-m1-service-pid.wat — LINCHPIN probe for the "service is a blessed concurrency
;; entry point" claim: can the owner read a started SERVICE's pid deterministically,
;; via peer-pid on the Handle's owner-side lineage peer (Handle/handle)?
;;
;; If YES → grant/revoke a service's pid needs NO spawn-program', NO racy post-spawn hook:
;;   (:probe::echo'::Handle/handle eh)  is a spawn-derived Process' peer → peer-pid → (Some pid).
;; This is the pid path a SERVICE-prober M1 needs (dogfooding service+bracket only).
;;
;; CLAIM (shape, not digits — a pid can't be pinned): peer-pid on a started service's
;; Handle/handle lineage peer is Option.Some of a positive i64.

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
              :reply (probe.Echo/EchoResponse.Ok {:reply (probe.Echo.EchoRequest/msg req)})}))])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [eh  (probe.echo/start :locus (wat.spawn/process) :record (probe.echo/Record))
     lp  (probe.echo.Handle/handle eh)
     lp-pid (wat.kernel/peer-pid lp)
     _   (wat.core/match lp-pid
           [wat.core/Option.Some {:value v} (wat.test/assert-true (wat.i64/> v 0))]
           [wat.core/Option.None {}
             (wat.kernel/assertion-failed! :message "service pid via Handle/handle: expected Some, got None")])
     _   (wat.kernel/println "service pid via Handle/handle:")
     _   (wat.kernel/println lp-pid)]
    nil))
