;; tests/collection/probe_arc216_stone5c_hashmap_native_storage.wat — co-located fixture.
;; Arc 216 Stone 216.5c — Value::wat__std__HashMap native storage refactor.

;; Probe 1a: keyword→i64 map length 3
(:wat::core::defn :t::p1a-kw-i64-len [] -> :wat::core::i64
  (:wat::core::HashMap/length
    (:wat::core::HashMap :wat::core::keyword :wat::core::i64
      :foo 1 :bar 2 :baz 3)))

;; Probe 1b: String→bool map length 2
(:wat::core::defn :t::p1b-str-bool-len [] -> :wat::core::i64
  (:wat::core::HashMap/length
    (:wat::core::HashMap :wat::core::String :wat::core::bool
      "x" true "y" false)))

;; Probe 1c: i64→String map length 2
(:wat::core::defn :t::p1c-i64-str-len [] -> :wat::core::i64
  (:wat::core::HashMap/length
    (:wat::core::HashMap :wat::core::i64 :wat::core::String
      1 "one" 2 "two")))

;; Probe 2a: get hit returns Some(42)
(:wat::core::defn :t::p2a-get-hit [] -> :wat::core::i64
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 42 :bar 99)]
    (:wat::core::match (:wat::core::HashMap/get m :foo) 
      ((:wat::core::Some v) v)
      (_ -1))))

;; Probe 2b: get miss → key :missing not present
(:wat::core::defn :t::p2b-get-miss [] -> :wat::core::bool
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 42)]
    (:wat::core::not (:wat::core::HashMap/contains-key? m :missing))))

;; Probe 3a: assoc inserts new key → length 2
(:wat::core::defn :t::p3a-assoc-insert [] -> :wat::core::i64
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 1)]
    (:wat::core::let [m2 (:wat::core::HashMap/assoc m :bar 99)]
      (:wat::core::HashMap/length m2))))

;; Probe 3b: assoc overwrites existing key → 999
(:wat::core::defn :t::p3b-assoc-overwrite [] -> :wat::core::i64
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 1)]
    (:wat::core::let [m2 (:wat::core::HashMap/assoc m :foo 999)]
      (:wat::core::match (:wat::core::HashMap/get m2 :foo) 
        ((:wat::core::Some v) v)
        (_ -1)))))

;; Probe 3c: assoc does not mutate original → original :foo = 1
(:wat::core::defn :t::p3c-assoc-immutable [] -> :wat::core::i64
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 1)]
    (:wat::core::let [_m2 (:wat::core::HashMap/assoc m :foo 999)]
      (:wat::core::match (:wat::core::HashMap/get m :foo) 
        ((:wat::core::Some v) v)
        (_ -1)))))

;; Probe 4a: dissoc removes key → length 2
(:wat::core::defn :t::p4a-dissoc-remove [] -> :wat::core::i64
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 1 :bar 2 :baz 3)]
    (:wat::core::HashMap/length
      (:wat::core::HashMap/dissoc m :foo))))

;; Probe 4b: dissoc missing key → length unchanged
(:wat::core::defn :t::p4b-dissoc-noop [] -> :wat::core::i64
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 1 :bar 2)]
    (:wat::core::HashMap/length
      (:wat::core::HashMap/dissoc m :missing))))

;; Probe 5a: keys returns Vec of length 2
(:wat::core::defn :t::p5a-keys-len [] -> :wat::core::i64
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 10 :bar 20)]
    (:wat::core::Vector/length (:wat::core::HashMap/keys m))))

;; Probe 5b: keys returns actual keyword Values that round-trip through contains-key?
(:wat::core::defn :t::p5b-keys-values [] -> :wat::core::bool
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 10)]
    (:wat::core::let [ks (:wat::core::HashMap/keys m)]
      (:wat::core::let [first-key (:wat::core::match
                                     (:wat::core::Vector/get ks 0) 
                                     ((:wat::core::Some k) k)
                                     (_ :missing))]
        (:wat::core::HashMap/contains-key? m first-key)))))

;; Probe 6: values returns Vec of length 3
(:wat::core::defn :t::p6-values-len [] -> :wat::core::i64
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 10 :bar 20 :baz 30)]
    (:wat::core::Vector/length (:wat::core::HashMap/values m))))

;; Probe 7a: contains-key? hit
(:wat::core::defn :t::p7a-contains-hit [] -> :wat::core::bool
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 1 :bar 2)]
    (:wat::core::HashMap/contains-key? m :foo)))

;; Probe 7b: contains-key? miss
(:wat::core::defn :t::p7b-contains-miss [] -> :wat::core::bool
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 1 :bar 2)]
    (:wat::core::HashMap/contains-key? m :missing)))

;; Probe 8a: length of 4-entry map
(:wat::core::defn :t::p8a-length-four [] -> :wat::core::i64
  (:wat::core::HashMap/length
    (:wat::core::HashMap :wat::core::keyword :wat::core::i64
      :a 1 :b 2 :c 3 :d 4)))

;; Probe 8b: length of empty map
(:wat::core::defn :t::p8b-length-empty [] -> :wat::core::i64
  (:wat::core::HashMap/length
    (:wat::core::HashMap :wat::core::keyword :wat::core::i64)))

;; Probe 9a: empty? true for empty map
(:wat::core::defn :t::p9a-empty-true [] -> :wat::core::bool
  (:wat::core::HashMap/empty?
    (:wat::core::HashMap :wat::core::keyword :wat::core::i64)))

;; Probe 9b: empty? false for non-empty map
(:wat::core::defn :t::p9b-empty-false [] -> :wat::core::bool
  (:wat::core::HashMap/empty?
    (:wat::core::HashMap :wat::core::keyword :wat::core::i64
      :foo 1)))

;; Probe 10a: nested HashMap contains-key? :inner
(:wat::core::defn :t::p10a-nested-contains [] -> :wat::core::bool
  (:wat::core::let
    [inner (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :x 42)
     outer (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :inner inner)]
    (:wat::core::HashMap/contains-key? outer :inner)))

;; Probe 10b: nested HashMap get :inner then get :x → 42
(:wat::core::defn :t::p10b-nested-get [] -> :wat::core::i64
  (:wat::core::let
    [inner (:wat::core::HashMap :wat::core::keyword :wat::core::i64 :x 42)
     outer (:wat::core::HashMap :wat::core::keyword :wat::type::Infer :inner inner)]
    (:wat::core::match (:wat::core::HashMap/get outer :inner) 
      ((:wat::core::Some inner2)
        (:wat::core::match (:wat::core::HashMap/get inner2 :x) 
          ((:wat::core::Some v) v)
          (_ -2)))
      (_ -1))))

;; Probe 11a: (HashMap :- [(HashSet :- [i64]) String]) length 1
(:wat::core::defn :t::p11a-hashset-key-len [] -> :wat::core::i64
  (:wat::core::let [k (:wat::core::HashSet :wat::core::i64 1 2 3)]
    (:wat::core::HashMap/length
      (:wat::core::HashMap :wat::type::Infer :wat::core::String k "hello"))))

;; Probe 11b: HashSet-as-K found by contains-key?
(:wat::core::defn :t::p11b-hashset-key-contains [] -> :wat::core::bool
  (:wat::core::let
    [k     (:wat::core::HashSet :wat::core::i64 7 8 9)
     m     (:wat::core::HashMap :wat::type::Infer :wat::core::String k "found-it")
     probe (:wat::core::HashSet :wat::core::i64 7 8 9)]
    (:wat::core::HashMap/contains-key? m probe)))

;; Probe 12a: forward round-trip length 2
(:wat::core::defn :t::p12a-rt-forward [] -> :wat::core::i64
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 42 :bar 99)]
    (:wat::core::let [h (:wat::holon::to-holon m)]
      (:wat::core::let [back (:wat::holon::from-holon h)]
        (:wat::core::HashMap/length back)))))

;; Probe 12b: reverse round-trip contains-key? :foo
(:wat::core::defn :t::p12b-rt-contains [] -> :wat::core::bool
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 42 :bar 99)]
    (:wat::core::let [h (:wat::holon::to-holon m)]
      (:wat::core::let [m2 (:wat::holon::from-holon h)]
        (:wat::core::HashMap/contains-key? m2 :foo)))))

;; Probe 12c: round-trip length preserved
(:wat::core::defn :t::p12c-rt-len [] -> :wat::core::i64
  (:wat::core::let [m (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                 :foo 42 :bar 99)]
    (:wat::core::let [h (:wat::holon::to-holon m)]
      (:wat::core::let [m2 (:wat::holon::from-holon h)]
        (:wat::core::HashMap/length m2)))))
