;; CLAIM: a user-defined defsurface's generated `surface-forms` accessor resolves and
;; returns exactly 1 carrier form (AMEND-255.4's "the rename reaches beyond the builtin
;; families" finding — :my::Counter/surface-forms is a USER type's member, not a builtin).
(wat.core/defsurface my/Counter :nature wat.kernel/Peer
  :messages
  [(wat.core/defrecord my.Counter/GetRequest  [])
   (wat.core/defenum my.Counter/GetResponse wat.enum/Pure :Ok [value :- wat.type/i64] :RequestTooLarge [bytes :- wat.type/i64  cap :- wat.type/i64]
                                                                                                  :RequestMalformed [path :- (wat.type/Vector :- [wat.type/String])  expected :- wat.type/String  got :- wat.type/String])]
  :features
  [(get [self :- my/Counter  req :- my.Counter/GetRequest] :- my.Counter/GetResponse :max-request-bytes 524288)])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [forms (my.Counter/surface-forms)]
    (wat.core/do
      (wat.test/assert-eq (wat.core/length forms) 1)
      (wat.kernel/println (wat.i64/to-string (wat.core/length forms))))))
