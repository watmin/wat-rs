;; tests/function/recursive_patterns_t7.wat — linear_shadowing (second binding wins)
(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
              [row
                (wat.core/Option.Some {:value (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 5 7)})
               v
                (wat.core/match row 
                  [wat.core/Option.Some {:value (x x)} x]
                  [wat.core/Option.None {} 0])]
              (wat.kernel/println (wat.i64/to-string v))))
