;; tests/collection/probe_arc216_stone2_vector_roundtrip.wat — co-located fixture.
;; Arc 216 Stone 2 — (Vec :- [T]) round-trip through HolonAST::Bundle of positional-Binds.

;; p1: forward round-trip length 3
(wat.core/defn t/p1-forward-rt-len [] :- wat.type/i64
  (wat.core/let
    [h (wat.holon/to-holon [1 2 3])
     v (wat.holon/from-holon h)]
    (wat.core/length v)))

;; p2a: round-trip length 3
(wat.core/defn t/p2a-rt-len [] :- wat.type/i64
  (wat.core/let
    [h (wat.holon/to-holon [1 2 3])
     v (wat.holon/from-holon h)]
    (wat.core/length v)))

;; p2b: round-trip first element = 1
(wat.core/defn t/p2b-rt-first [] :- wat.type/i64
  (wat.core/let
    [h (wat.holon/to-holon [1 2 3])
     v (wat.holon/from-holon h)]
    (wat.core/match
      (wat.core/get v 0)
      
      [wat.core/Option.Some {:value x} x]
      [wat.core/Option.None {} -1])))

;; p3: empty vec round-trip length 0
(wat.core/defn t/p3-empty-rt-len [] :- wat.type/i64
  (wat.core/let
    [h (wat.holon/to-holon [])
     v (wat.holon/from-holon h)]
    (wat.core/length v)))

;; p4a: single element round-trip length 1
(wat.core/defn t/p4a-single-rt-len [] :- wat.type/i64
  (wat.core/let
    [h (wat.holon/to-holon [42])
     v (wat.holon/from-holon h)]
    (wat.core/length v)))

;; p4b: single element round-trip get index 0 = 42
(wat.core/defn t/p4b-single-rt-elem [] :- wat.type/i64
  (wat.core/let
    [h (wat.holon/to-holon [42])
     v (wat.holon/from-holon h)]
    (wat.core/match
      (wat.core/get v 0)
      
      [wat.core/Option.Some {:value x} x]
      [wat.core/Option.None {} -1])))

;; p5a: (Vec :- [i64]) element at index 1 = 20
(wat.core/defn t/p5a-i64-elem1 [] :- wat.type/i64
  (wat.core/let
    [h (wat.holon/to-holon [10 20 30])
     v (wat.holon/from-holon h)]
    (wat.core/match
      (wat.core/get v 1)
      
      [wat.core/Option.Some {:value x} x]
      [wat.core/Option.None {} -1])))

;; p5b: (Vec :- [String]) round-trip length 3
(wat.core/defn t/p5b-str-rt-len [] :- wat.type/i64
  (wat.core/let
    [h (wat.holon/to-holon (wat.type/Vector :- [wat.type/String] "a" "b" "c"))
     v (wat.holon/from-holon h)]
    (wat.core/length v)))

;; p5c: (Vec :- [bool]) round-trip length 3
(wat.core/defn t/p5c-bool-rt-len [] :- wat.type/i64
  (wat.core/let
    [h (wat.holon/to-holon (wat.type/Vector :- [wat.type/bool] true false true))
     v (wat.holon/from-holon h)]
    (wat.core/length v)))

;; p6a: order preservation index 0 = 10
(wat.core/defn t/p6a-order-idx0 [] :- wat.type/i64
  (wat.core/let
    [h (wat.holon/to-holon [10 20 30])
     v (wat.holon/from-holon h)]
    (wat.core/match
      (wat.core/get v 0)
      
      [wat.core/Option.Some {:value x} x]
      [wat.core/Option.None {} -1])))

;; p6b: order preservation index 2 = 30
(wat.core/defn t/p6b-order-idx2 [] :- wat.type/i64
  (wat.core/let
    [h (wat.holon/to-holon [10 20 30])
     v (wat.holon/from-holon h)]
    (wat.core/match
      (wat.core/get v 2)
      
      [wat.core/Option.Some {:value x} x]
      [wat.core/Option.None {} -1])))

;; p7a: nested vector outer length 2
(wat.core/defn t/p7a-nested-outer-len [] :- wat.type/i64
  (wat.core/let
    [inner1 (wat.type/Vector :- [wat.type/i64] 1 2 3)
     inner2 (wat.type/Vector :- [wat.type/i64] 4 5)
     outer  (wat.type/Vector :- [wat.type/Infer] inner1 inner2)
     h      (wat.holon/to-holon outer)
     v      (wat.holon/from-holon h)]
    (wat.core/length v)))

;; p7b: nested vector arc 228 re-verify outer length 2
(wat.core/defn t/p7b-nested-arc228 [] :- wat.type/i64
  (wat.core/let
    [inner1 (wat.type/Vector :- [wat.type/i64] 1 2 3)
     inner2 (wat.type/Vector :- [wat.type/i64] 4 5)
     outer  (wat.type/Vector :- [wat.type/Infer] inner1 inner2)
     h      (wat.holon/to-holon outer)
     v      (wat.holon/from-holon h)]
    (wat.core/length v)))

;; p7c: nested vector inner element at [1][0] = 4
(wat.core/defn t/p7c-nested-inner-elem [] :- wat.type/i64
  (wat.core/let
    [inner1 (wat.type/Vector :- [wat.type/i64] 1 2 3)
     inner2 (wat.type/Vector :- [wat.type/i64] 4 5)
     outer  (wat.type/Vector :- [wat.type/Infer] inner1 inner2)
     h      (wat.holon/to-holon outer)
     v      (wat.holon/from-holon h)]
    (wat.core/match
      (wat.core/get v 1)
      
      [wat.core/Option.Some {:value inner}
        (wat.core/match
          (wat.core/get inner 0)
          
          [wat.core/Option.Some {:value x} x]
          [wat.core/Option.None {} -1])]
      [wat.core/Option.None {} -1])))

;; p8a: (Vec :- [(HashSet :- [i64])]) outer length 2
(wat.core/defn t/p8a-mixed-outer-len [] :- wat.type/i64
  (wat.core/let
    [s1 (wat.type/HashSet :- [wat.type/i64] 1 2 3)
     s2 (wat.type/HashSet :- [wat.type/i64] 4 5)
     v  (wat.type/Vector :- [wat.type/Infer] s1 s2)
     h  (wat.holon/to-holon v)
     rv (wat.holon/from-holon h)]
    (wat.core/length rv)))

;; p8b: (Vec :- [(HashSet :- [i64])]) arc 228 outer length 2
(wat.core/defn t/p8b-mixed-arc228 [] :- wat.type/i64
  (wat.core/let
    [s1 (wat.type/HashSet :- [wat.type/i64] 1 2 3)
     s2 (wat.type/HashSet :- [wat.type/i64] 4 5)
     v  (wat.type/Vector :- [wat.type/Infer] s1 s2)
     h  (wat.holon/to-holon v)
     rv (wat.holon/from-holon h)]
    (wat.core/length rv)))

;; p9a: atomizable passes — returns 1
(wat.core/defn t/p9a-atomizable-passes [] :- wat.type/i64
  (wat.core/let
    [h (wat.holon/to-holon [1 2 3])]
    1))

;; p9b: nested atomizable passes — returns 1
(wat.core/defn t/p9b-nested-atomizable [] :- wat.type/i64
  (wat.core/let
    [inner (wat.type/Vector :- [wat.type/i64] 1 2)
     outer (wat.type/Vector :- [wat.type/Infer] inner)
     h     (wat.holon/to-holon outer)]
    1))
