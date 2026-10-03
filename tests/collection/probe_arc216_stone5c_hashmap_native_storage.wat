;; tests/collection/probe_arc216_stone5c_hashmap_native_storage.wat — co-located fixture.
;; Arc 216 Stone 216.5c — Value::wat__std__HashMap native storage refactor.

;; Probe 1a: keyword→i64 map length 3
(wat.core/defn t/p1a-kw-i64-len [] :- wat.type/i64
  (wat.core/length
    (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
      :foo 1 :bar 2 :baz 3)))

;; Probe 1b: String→bool map length 2
(wat.core/defn t/p1b-str-bool-len [] :- wat.type/i64
  (wat.core/length
    (wat.type/HashMap :- [wat.type/String wat.type/bool]
      "x" true "y" false)))

;; Probe 1c: i64→String map length 2
(wat.core/defn t/p1c-i64-str-len [] :- wat.type/i64
  (wat.core/length
    (wat.type/HashMap :- [wat.type/i64 wat.type/String]
      1 "one" 2 "two")))

;; Probe 2a: get hit returns Some(42)
(wat.core/defn t/p2a-get-hit [] :- wat.type/i64
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 42 :bar 99)]
    (wat.core/match (wat.core/get m :foo) 
      [wat.core/Option.Some {:value v} v]
      [_ -1])))

;; Probe 2b: get miss → key :missing not present
(wat.core/defn t/p2b-get-miss [] :- wat.type/bool
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 42)]
    (wat.core/not (wat.core/contains? m :missing))))

;; Probe 3a: assoc inserts new key → length 2
(wat.core/defn t/p3a-assoc-insert [] :- wat.type/i64
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 1)]
    (wat.core/let [m2 (wat.core/assoc m :bar 99)]
      (wat.core/length m2))))

;; Probe 3b: assoc overwrites existing key → 999
(wat.core/defn t/p3b-assoc-overwrite [] :- wat.type/i64
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 1)]
    (wat.core/let [m2 (wat.core/assoc m :foo 999)]
      (wat.core/match (wat.core/get m2 :foo) 
        [wat.core/Option.Some {:value v} v]
        [_ -1]))))

;; Probe 3c: assoc does not mutate original → original :foo = 1
(wat.core/defn t/p3c-assoc-immutable [] :- wat.type/i64
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 1)]
    (wat.core/let [_m2 (wat.core/assoc m :foo 999)]
      (wat.core/match (wat.core/get m :foo) 
        [wat.core/Option.Some {:value v} v]
        [_ -1]))))

;; Probe 4a: dissoc removes key → length 2
(wat.core/defn t/p4a-dissoc-remove [] :- wat.type/i64
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 1 :bar 2 :baz 3)]
    (wat.core/length
      (wat.core/dissoc m :foo))))

;; Probe 4b: dissoc missing key → length unchanged
(wat.core/defn t/p4b-dissoc-noop [] :- wat.type/i64
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 1 :bar 2)]
    (wat.core/length
      (wat.core/dissoc m :missing))))

;; Probe 5a: keys returns Vec of length 2
(wat.core/defn t/p5a-keys-len [] :- wat.type/i64
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 10 :bar 20)]
    (wat.core/length (wat.core/keys m))))

;; Probe 5b: keys returns actual keyword Values that round-trip through contains-key?
(wat.core/defn t/p5b-keys-values [] :- wat.type/bool
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 10)]
    (wat.core/let [ks (wat.core/keys m)]
      (wat.core/let [first-key (wat.core/match
                                     (wat.core/get ks 0) 
                                     [wat.core/Option.Some {:value k} k]
                                     [_ :missing])]
        (wat.core/contains? m first-key)))))

;; Probe 6: values returns Vec of length 3
(wat.core/defn t/p6-values-len [] :- wat.type/i64
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 10 :bar 20 :baz 30)]
    (wat.core/length (wat.core/values m))))

;; Probe 7a: contains-key? hit
(wat.core/defn t/p7a-contains-hit [] :- wat.type/bool
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 1 :bar 2)]
    (wat.core/contains? m :foo)))

;; Probe 7b: contains-key? miss
(wat.core/defn t/p7b-contains-miss [] :- wat.type/bool
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 1 :bar 2)]
    (wat.core/contains? m :missing)))

;; Probe 8a: length of 4-entry map
(wat.core/defn t/p8a-length-four [] :- wat.type/i64
  (wat.core/length
    (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
      :a 1 :b 2 :c 3 :d 4)))

;; Probe 8b: length of empty map
(wat.core/defn t/p8b-length-empty [] :- wat.type/i64
  (wat.core/length
    (wat.type/HashMap :- [wat.type/keyword wat.type/i64])))

;; Probe 9a: empty? true for empty map
(wat.core/defn t/p9a-empty-true [] :- wat.type/bool
  (wat.core/empty?
    (wat.type/HashMap :- [wat.type/keyword wat.type/i64])))

;; Probe 9b: empty? false for non-empty map
(wat.core/defn t/p9b-empty-false [] :- wat.type/bool
  (wat.core/empty?
    (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
      :foo 1)))

;; Probe 10a: nested HashMap contains-key? :inner
(wat.core/defn t/p10a-nested-contains [] :- wat.type/bool
  (wat.core/let
    [inner (wat.type/HashMap :- [wat.type/keyword wat.type/i64] :x 42)
     outer (wat.type/HashMap :- [wat.type/keyword wat.type/Infer] :inner inner)]
    (wat.core/contains? outer :inner)))

;; Probe 10b: nested HashMap get :inner then get :x → 42
(wat.core/defn t/p10b-nested-get [] :- wat.type/i64
  (wat.core/let
    [inner (wat.type/HashMap :- [wat.type/keyword wat.type/i64] :x 42)
     outer (wat.type/HashMap :- [wat.type/keyword wat.type/Infer] :inner inner)]
    (wat.core/match (wat.core/get outer :inner) 
      [wat.core/Option.Some {:value inner2}
        (wat.core/match (wat.core/get inner2 :x) 
          [wat.core/Option.Some {:value v} v]
          [_ -2])]
      [_ -1])))

;; Probe 11a: (HashMap :- [(HashSet :- [i64]) String]) length 1
(wat.core/defn t/p11a-hashset-key-len [] :- wat.type/i64
  (wat.core/let [k (wat.type/HashSet :- [wat.type/i64] 1 2 3)]
    (wat.core/length
      (wat.type/HashMap :- [wat.type/Infer wat.type/String] k "hello"))))

;; Probe 11b: HashSet-as-K found by contains-key?
(wat.core/defn t/p11b-hashset-key-contains [] :- wat.type/bool
  (wat.core/let
    [k     (wat.type/HashSet :- [wat.type/i64] 7 8 9)
     m     (wat.type/HashMap :- [wat.type/Infer wat.type/String] k "found-it")
     probe (wat.type/HashSet :- [wat.type/i64] 7 8 9)]
    (wat.core/contains? m probe)))

;; Probe 12a: forward round-trip length 2
(wat.core/defn t/p12a-rt-forward [] :- wat.type/i64
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 42 :bar 99)]
    (wat.core/let [h (wat.holon/to-holon m)]
      (wat.core/let [back (wat.holon/from-holon h)]
        (wat.core/length back)))))

;; Probe 12b: reverse round-trip contains-key? :foo
(wat.core/defn t/p12b-rt-contains [] :- wat.type/bool
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 42 :bar 99)]
    (wat.core/let [h (wat.holon/to-holon m)]
      (wat.core/let [m2 (wat.holon/from-holon h)]
        (wat.core/contains? m2 :foo)))))

;; Probe 12c: round-trip length preserved
(wat.core/defn t/p12c-rt-len [] :- wat.type/i64
  (wat.core/let [m (wat.type/HashMap :- [wat.type/keyword wat.type/i64]
                                 :foo 42 :bar 99)]
    (wat.core/let [h (wat.holon/to-holon m)]
      (wat.core/let [m2 (wat.holon/from-holon h)]
        (wat.core/length m2)))))
