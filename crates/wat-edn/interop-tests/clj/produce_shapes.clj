;; Shape matrix producer — Clojure side. Builds the same shape Map
;; using Clojure data + tagged-literal, emits via pr-str.
;;
;; Rust side (shape_matrix_reader) parses and asserts each shape.

(let [shapes
      {:primitive-i64           42
       :primitive-string        "hello"
       :primitive-keyword       :asset/BTC
       :primitive-bool          true
       :primitive-nil           nil
       :primitive-f64           2.5

       :collection-vector       [1 2 3]
       :collection-set          #{:a :b :c}
       :collection-map          {:k1 1 :k2 2}

       :nested-vec-of-vecs      [[1 2] [3 4]]
       :nested-map-of-vec       {:numbers [1 2 3]}

       :builtin-inst            #inst "2026-05-21T00:00:00.000-00:00"
       :builtin-uuid            #uuid "550e8400-e29b-41d4-a716-446655440000"

       :tagged-some-i64    (tagged-literal 'wat.core/Some 42)
       :tagged-none        (tagged-literal 'wat.core/None nil)
       :tagged-ok-string   (tagged-literal 'wat.core/Ok "fine")
       :tagged-err-map     (tagged-literal 'wat.core/Err
                                            {:code 500 :msg "boom"})
       :tagged-duration    (tagged-literal 'wat.time/Duration "PT5M")

       :nested-some-set-of-maps
       (tagged-literal 'wat.core/Some #{{:foo "baz"}})

       :nested-ok-vec-of-maps
       (tagged-literal 'wat.core/Ok [{:a 1} {:b 2}])

       :nested-some-some-i64
       (tagged-literal 'wat.core/Some
                       (tagged-literal 'wat.core/Some 42))

       :vec-of-options
       [(tagged-literal 'wat.core/Some 1)
        (tagged-literal 'wat.core/None nil)
        (tagged-literal 'wat.core/Some 2)]

       :map-with-tagged-keys
       {(tagged-literal 'wat.holon/Atom :role)
        (tagged-literal 'wat.holon/Atom :filler)}}]
  (binding [*print-dup* false]
    (print (pr-str shapes))
    (newline)))
