;; Symbol-spelled twin of wat_grep__count_rules.wat. One rule, one fact type.
;; :user::grep stays the driver's entry name.
(wat.rete/defrule probe.a87/node
  :when [(wat.grep/Node (?id :- :id))
         (wat.grep/Source (?f :- :file))]
  :then [(wat.grep/Match :file ?f :line ?id :col 0 :end-line ?id :end-col 0
          :rule "probe.a87/node"
          :captures (wat.rete.core/PersistentVector))])

(wat.core/defn user/grep [] :- (wat.type/PersistentVector :- [wat.rete/Rule])
  (wat.rete/collect-rules probe/a87))
