(ns wat-edn.json-test
  "Tests for the JSON-to-EDN bridge. Round-trip identity and
  cross-language wire-compatibility (the JSON shapes match what
  the Rust side emits)."
  (:require [clojure.test :refer [deftest is testing]]
            [wat-edn.core :as wat]
            [wat-edn.json :as wj])
  (:import [java.util Date UUID]))

(defn round-trip
  "Convert v → JSON → back; assert equality."
  [v]
  (let [json (wj/to-json-string v)
        v2   (wj/from-json-string json)]
    (is (= v v2) (str "round-trip failed for " (pr-str v) "; json was: " json))
    v2))

(deftest primitives
  (round-trip nil)
  (round-trip true)
  (round-trip false)
  (round-trip 42)
  (round-trip -7)
  (round-trip 3.14)
  (round-trip "hello"))

(deftest large-integers-via-string
  ;; Values outside JS safe-integer range should serialize as strings.
  (let [n 9007199254740993                       ; 2^53 + 1
        json (wj/to-json-string n)]
    (is (.contains json "\"")
        (str "expected string-encoded large int, got: " json))))

(deftest keywords-roundtrip
  (round-trip :foo)
  (round-trip :ns/foo))

(deftest symbols-roundtrip-via-sentinel
  (round-trip 'foo)
  (round-trip 'ns/foo))

(deftest collections
  (round-trip [1 2 3])
  (round-trip [])
  (round-trip #{1 2 3})
  (round-trip {:a 1 :b 2}))

(deftest nested
  (round-trip [{:a 1} {:b 2}])
  (round-trip {:k [1 2 3]})
  (round-trip #{[1 2] [3 4]}))

(deftest map-with-vector-key
  (round-trip {[1 2] :pair, [3 4] :other}))

(deftest tagged-literals
  (let [t (tagged-literal 'myapp/Order {:id 1 :name "x"})
        back (round-trip t)]
    (is (tagged-literal? back))
    (is (= 'myapp/Order (:tag back)))))

(deftest nan-and-infinity
  (let [back (wj/from-json-string (wj/to-json-string Double/NaN))]
    (is (Double/isNaN back)))
  (is (= Double/POSITIVE_INFINITY
         (wj/from-json-string (wj/to-json-string Double/POSITIVE_INFINITY))))
  (is (= Double/NEGATIVE_INFINITY
         (wj/from-json-string (wj/to-json-string Double/NEGATIVE_INFINITY)))))

(deftest inst-and-uuid
  (let [d (Date.)
        u (UUID/fromString "550e8400-e29b-41d4-a716-446655440000")]
    (is (= (.getTime d)
           (.getTime (wj/from-json-string (wj/to-json-string d)))))
    (is (= u (wj/from-json-string (wj/to-json-string u))))))

(deftest set-via-sentinel
  (let [json (wj/to-json-string #{1 2 3})]
    (is (.contains json "#set"))
    (is (= #{1 2 3} (wj/from-json-string json)))))

(deftest variant-helpers-survive
  (round-trip (wat/some-of 42))
  (round-trip (wat/none-of))
  (round-trip (wat/ok-of "fine"))
  (round-trip (wat/err-of "boom")))

(deftest wire-shape-matches-spec
  ;; Verify the JSON shapes match the documented wire convention,
  ;; so the Rust side parses them.
  (testing "keyword → ':foo' string"
    (is (= "\":foo\"" (wj/to-json-string :foo))))

  (testing "set → {#set: [...]}"
    (is (.contains (wj/to-json-string #{1 2}) "#set")))

  (testing "tagged → {#tag: ..., body: ...}"
    (let [s (wj/to-json-string (tagged-literal 'a/B {:x 1}))]
      (is (.contains s "#tag"))
      (is (.contains s "a/B"))
      (is (.contains s "body"))))

  (testing "inst → {#inst: \"...\"}"
    (is (.contains (wj/to-json-string (Date.)) "#inst"))))

(deftest pretty-json
  (let [v {:asset :BTC :tags #{:vip :early}}
        compact (wj/to-json-string v)
        pretty  (wj/to-json-pretty v)]
    (is (not (.contains compact "\n")))
    (is (.contains pretty "\n"))
    ;; Both must round-trip to the same value.
    (is (= v (wj/from-json-string compact)))
    (is (= v (wj/from-json-string pretty)))))

(deftest pretty-edn
  ;; The wat/pretty-edn function should produce parseable output.
  (let [v {:asset :BTC :tags #{:vip :early} :nested [1 [2 [3 4]]]}
        s (wat/pretty-edn v)]
    (is (string? s))
    (is (.contains s "\n"))
    ;; Pretty-printed EDN parses back to the same value.
    (is (= v (clojure.edn/read-string s)))))

(deftest realistic-blob
  (let [v {:asset :BTC
           :side  :Buy
           :size  0.025
           :tags  #{:vip :early}
           :id    (UUID/fromString "550e8400-e29b-41d4-a716-446655440000")
           :nested {[1 2] :pair}}
        back (round-trip v)]
    (is (= v back))))
