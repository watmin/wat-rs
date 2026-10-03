;; parametric_enum_walkstep_continue.wat — WalkStep::Continue parametric inference.
(wat.core/defn my.test/wrap [n :- wat.type/i64] :- (wat.eval/WalkStep :- [wat.type/i64]) (wat.eval/WalkStep.Continue {:acc n}))
(wat.core/defn my/compute [] :- wat.type/i64
  (wat.core/let
    [wrapped (my.test/wrap 7)]
    7))
