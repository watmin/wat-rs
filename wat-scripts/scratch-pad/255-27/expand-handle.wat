;; 255.27 probe — expand a monomorphic defservice at the form level and print it, to READ the
;; emitted Handle struct and the start$impl-thread body.
(wat.core/defsurface probe/Kv :nature wat.kernel/Peer
  :messages
  [(wat.core/defrecord probe.Kv/GetRequest [k :- wat.type/String])
   (wat.core/defenum probe.Kv/GetResponse wat.enum/Pure
     :Ok              [v :- wat.type/String]
     :RequestTooLarge [bytes :- wat.type/i64 cap :- wat.type/i64]
     :RequestMalformed [path :- (wat.type/Vector :- [wat.type/String]) expected :- wat.type/String got :- wat.type/String])]
  :features
  [(get [self :- probe/Kv req :- probe.Kv/GetRequest] :- probe.Kv/GetResponse :max-request-bytes 524288)])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println
    (wat.core/write-forms
      (wat.core/macroexpand
        (wat.core/quote
          (wat.service/defservice probe/kv
            :satisfies probe/Kv :durable [] :ephemeral []
            :impls [(get [s ctx req] (wat.service/Outcome.Reply {:state s :reply (probe.Kv/GetResponse.Ok {:v "v"})}))]))))))
