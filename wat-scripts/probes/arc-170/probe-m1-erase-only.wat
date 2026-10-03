;; Can a concrete (Address' :- [S R]) be erased to bare Address' via ann-form, stored, and
;; sent as a bare-D PoolMsg::Setup? Test the TYPE questions only (no child).
;; CLAIM (shape, not digits — an Address' can't be pinned): the erase->store->construct
;; pipeline produces an actual :probe::PoolMsg.Setup variant (never silently falls through
;; to .Work or fails to construct at all).

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

;; bare-D PoolMsg (the parent-side shape)
(wat.core/defenum probe/PoolMsg :- [I] wat.enum/Pure
  :Setup [addr :- wat.kernel/Address]
  :Work  [s    :- I])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [eh  (probe.echo/start :locus (wat.spawn/process) :record (probe.echo/Record))
     ea  (probe.echo.Handle/addr eh)                       ;; concrete (Address' :- [Op Reply])
     eab (wat.core/ann-form ea wat.kernel/Address)      ;; erase -> bare Address'
     v   (wat.type/Vector :- [wat.kernel/Address] eab)       ;; store bare in (Vector :- [Address'])
     msg (probe/PoolMsg.Setup {:addr (wat.core/first v)})]       ;; bare-D Setup constructor
    (wat.core/do
      (wat.core/match msg
        [probe/PoolMsg.Setup {:addr _} nil]
        [probe/PoolMsg.Work {:s _}
          (wat.kernel/assertion-failed! :message "erase-only: expected PoolMsg.Setup, got PoolMsg.Work")])
      (wat.kernel/println "erase-ok"))))
