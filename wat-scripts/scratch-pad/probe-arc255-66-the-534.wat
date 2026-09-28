;; Stone 255.66 — the 255.64 startup shapes, in the new spelling, against today's stdlib.
;; The first refusal was conj at wat/bracket.wat:356: the collection's head was
;; compared as the raw string "wat::core::Vector" while the form had been read
;; as wat.type/Vector. format_type had already denoted the head, so the message
;; printed the old spelling of a head the match had not accepted.
(wat.core/defn user/conj-ast [x :- wat.type/AST] :- (wat.type/Vector :- [wat.type/AST])
  (wat.core/conj (wat.type/Vector :- [wat.type/AST]) x))

(wat.core/defn user/nth-i64 [] :- wat.type/i64
  (wat.core/nth (wat.type/Vector :- [wat.type/i64] 1 2) 0))

(wat.core/defn user/fold-i64 [] :- wat.type/i64
  (wat.core/foldl
    (wat.core/fn [a :- wat.type/i64 b :- wat.type/i64] :- wat.type/i64
      (wat.core/+ a b))
    0
    (wat.type/Vector :- [wat.type/i64] 1 2)))

(wat.core/defn user/mapv-i64 [] :- (wat.type/Vector :- [wat.type/i64])
  (wat.core/mapv
    (wat.core/fn [n :- wat.type/i64] :- wat.type/i64 n)
    (wat.type/Vector :- [wat.type/i64] 1)))

(wat.core/defn user/empty-peers []
  :- (wat.type/Vector :- [(wat.kernel/Peer :- [wat.type/i64 wat.type/i64])])
  (wat.type/Vector :- [(wat.kernel/Peer :- [wat.type/i64 wat.type/i64])]))

(wat.core/defn user/select-empty [] :- wat.type/nil
  (wat.core/match (wat.kernel/select (user/empty-peers))
    [:wat::spawn::ServiceEvent.Message {:idx _idx :msg _msg} nil]
    [:wat::spawn::ServiceEvent.Closed {:idx _idx} nil]
    [:wat::spawn::ServiceEvent.Lost {:idx _idx :cause _cause} nil]
    [:wat::spawn::ServiceEvent.Malformed {:idx _idx :cause _cause} nil]
    [_ nil]))
