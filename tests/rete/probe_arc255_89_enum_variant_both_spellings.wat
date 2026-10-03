;; Both spellings of a variant the enum does not declare. One file, one rule each.
;; The keyword `:evt::G::Hii` and the symbol `evt.G/Hii` are one mistake.
(wat.core/defenum evt/G wat.enum/Pure :Hi :Lo)
(wat.core/defrecord evt/Req [k :- wat.type/i64 grade :- evt/G])
(wat.core/defrecord evt/Hit [k :- wat.type/i64])

(wat.rete/defrule evt/symbol-typo
  :when [(evt/Req (?k :- :k) (wat.rete.core.enum/= :grade evt.G/Hii))]
  :then [(evt/Hit :k ?k)])

(wat.rete/defrule evt/keyword-typo
  :when [(evt/Req (?k :- :k) (wat.rete.core.enum/= :grade :evt::G::Hii))]
  :then [(evt/Hit :k ?k)])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println "admitted"))
