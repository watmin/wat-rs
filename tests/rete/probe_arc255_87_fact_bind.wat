;; Stone 255.87 #3 — a symbol type in `(?p :- wat.grep/Node)` is a fact-bind.
;; The keyword type `:wat::grep::Node` stays one. A real accumulator stays an
;; accumulator. The three rules are the floor's compile path: a keyword fact
;; pattern (the fmt/grep shape), a keyword fact-bind, and a symbol fact-bind.
;; `probe/hold` returns 1023 when every bit is set.

(wat.core/defrecord probe.a3/Hit [id :- wat.type/i64])

(wat.core/defn probe.a3/form [src :- wat.type/String] :- wat.type/AST
  (wat.core/match (wat.core/read-string src)
    [wat.core/ReadOutcome.Forms {:forms forms}
     (wat.core/first (wat.core/ast->children forms))]
    [wat.core/ReadOutcome.Malformed {:cause c}
     (wat.kernel/assertion-failed! :message (wat.core.Error/message c))]))

(wat.core/defn probe.a3/bit [b :- wat.type/bool n :- wat.type/i64] :- wat.type/i64
  (wat.core/if b n 0))

(wat.rete/defrule probe.a3/kw-fact
  :when [(wat.grep/Node (?h :- :id))
         (wat.rete/where (wat.rete.i64/= ?h ?h))]
  :then [(probe.a3/Hit :id 1)])

(wat.rete/defrule probe.a3/kw-bind
  :when [(?p :- wat.grep/Node)]
  :then [(probe.a3/Hit :id 1)])

(wat.rete/defrule probe.a3/sym-bind
  :when [(?p :- wat.grep/Node)]
  :then [(probe.a3/Hit :id 1)])

(wat.core/defn probe/hold [] :- wat.type/i64
  (wat.core/let [sym  (probe.a3/form "(?p :- wat.grep/Node)")
                 kw   (probe.a3/form "(?p :- :wat::grep::Node)")
                 fld  (probe.a3/form "(?h :- :id)")
                 acc  (probe.a3/form "(?c :- (wat.rete/acc/count) :from (:wat::grep::Node))")
                 fact (probe.a3/form "(:wat::grep::Node (?h :- :id))")
                 ok   (wat.core/match (wat.rete/compile (wat.rete/collect-rules probe/a3))
                        [wat.rete/CompileOutcome.Compiled {:session __session} true]
                        [wat.rete/CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} false])]
    (wat.i64/+
      (wat.i64/+
        (wat.i64/+
          (probe.a3/bit (wat.rete/cond-is-fact-bind sym) 1)
          (probe.a3/bit (wat.core/not (wat.rete/cond-is-accumulate sym)) 2))
        (wat.i64/+
          (probe.a3/bit (wat.rete/cond-is-fact-bind kw) 4)
          (probe.a3/bit (wat.core/not (wat.rete/cond-is-accumulate kw)) 8)))
      (wat.i64/+
        (wat.i64/+
          (wat.i64/+
            (probe.a3/bit (wat.core/not (wat.rete/cond-is-fact-bind fld)) 16)
            (probe.a3/bit (wat.rete/cond-is-accumulate fld) 32))
          (wat.i64/+
            (probe.a3/bit (wat.core/not (wat.rete/cond-is-fact-bind acc)) 64)
            (probe.a3/bit (wat.rete/cond-is-accumulate acc) 128)))
        (wat.i64/+
          (probe.a3/bit (wat.core/not (wat.rete/cond-is-accumulate fact)) 256)
          (probe.a3/bit ok 512))))))
