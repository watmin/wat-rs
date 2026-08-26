(ns wat-edn.core-test
  "Locks the public API:
    - load-types! / list-types / type-fields
    - gen / emit / validate
    - read / read-str / read-typed
    - variant helpers
    - default reader for wat.* tags"
  (:require [clojure.test :refer [deftest is testing use-fixtures]]
            [wat-edn.core :as wat]))

;; Load fixture types before each test; clear after.
(use-fixtures :each
  (fn [f]
    (wat/clear-types!)
    (wat/load-types! "wat/shared.wat")
    (f)
    (wat/clear-types!)))

;; ─── Schema registration ───────────────────────────────────────

(deftest load-types-registers-structs
  (let [tags (set (wat/list-types))]
    (is (contains? tags 'enterprise.config/SizeAdjust))
    (is (contains? tags 'enterprise.observer.market/TradeSignal))
    (is (contains? tags 'enterprise.treasury.events/Fill))))

(deftest load-types-skips-non-struct-forms
  ;; The fixture has a (:wat::core::defn ...) inside; should NOT
  ;; have been registered as a type.
  (let [tags (set (wat/list-types))]
    (is (not (some #(= "TradeSignal/show" (str %)) tags)))))

(deftest type-fields-returns-schema
  (is (= {:asset "wat::core::keyword" :factor "wat::core::f64" :reason "wat::core::String"}
         (wat/type-fields 'enterprise.config/SizeAdjust))))

;; ─── Generators ────────────────────────────────────────────────

(deftest gen-builds-tagged-literal
  (let [v (wat/gen 'enterprise.config/SizeAdjust
                   {:asset :BTC :factor 1.5 :reason "vol-spike"})]
    (is (tagged-literal? v))
    (is (= 'enterprise.config/SizeAdjust (:tag v)))
    (is (= {:asset :BTC :factor 1.5 :reason "vol-spike"} (:form v)))))

(deftest gen-validates-primitive-types
  (testing "wrong type for :asset"
    (is (thrown? clojure.lang.ExceptionInfo
          (wat/gen 'enterprise.config/SizeAdjust
                   {:asset "BTC"   ; ← String, not Keyword
                    :factor 1.5
                    :reason "x"}))))
  (testing "missing field"
    (is (thrown? clojure.lang.ExceptionInfo
          (wat/gen 'enterprise.config/SizeAdjust
                   {:asset :BTC :factor 1.5}))))
  (testing "wrong type for :factor"
    (is (thrown? clojure.lang.ExceptionInfo
          (wat/gen 'enterprise.config/SizeAdjust
                   {:asset :BTC :factor "wat" :reason "x"})))))

(deftest gen-rejects-unknown-types
  (is (thrown? clojure.lang.ExceptionInfo
        (wat/gen 'unknown.module/Type {:foo 1}))))

(deftest emit-produces-edn-string
  (let [s (wat/emit 'enterprise.config/SizeAdjust
                    {:asset :BTC :factor 1.5 :reason "x"})]
    (is (string? s))
    (is (.startsWith s "#enterprise.config/SizeAdjust"))
    (is (.contains s ":asset :BTC"))))

;; ─── Typed read ────────────────────────────────────────────────

(deftest read-typed-returns-body
  (let [s (wat/emit 'enterprise.config/SizeAdjust
                    {:asset :BTC :factor 1.5 :reason "x"})
        body (wat/read-typed 'enterprise.config/SizeAdjust s)]
    (is (= {:asset :BTC :factor 1.5 :reason "x"} body))))

(deftest read-typed-rejects-wrong-tag
  (let [s (wat/emit 'enterprise.config/SizeAdjust
                    {:asset :BTC :factor 1.5 :reason "x"})]
    (is (thrown? clojure.lang.ExceptionInfo
          (wat/read-typed 'enterprise.observer.market/TradeSignal s)))))

(deftest read-typed-validates-fields
  ;; Hand-construct EDN with wrong field type
  (let [bad-edn "#enterprise.config/SizeAdjust {:asset 42, :factor 1.5, :reason \"x\"}"]
    (is (thrown? clojure.lang.ExceptionInfo
          (wat/read-typed 'enterprise.config/SizeAdjust bad-edn)))))

;; ─── Round-trip ────────────────────────────────────────────────

(deftest gen-emit-read-typed-round-trip
  (let [original {:asset :BTC :factor 2.5 :reason "drawdown breach"}
        s (wat/emit 'enterprise.config/SizeAdjust original)
        round (wat/read-typed 'enterprise.config/SizeAdjust s)]
    (is (= original round))))

;; ─── Default reader for wat.* tags ─────────────────────────────

(deftest default-reader-handles-wat-collections
  (is (= [1 2 3] (wat/read-str "#wat.core/Vec<i64> [1 2 3]")))
  (is (= {"a" 1} (wat/read-str "#wat.core/HashMap<String_i64> {\"a\" 1}"))))

(deftest default-reader-handles-sums
  ;; Arc 278 A.0 — canonical vector-bodied variant form.
  (is (wat/some-variant? (wat/read-str "#wat.core.Option/Some [42]")))
  (is (= 42 (wat/unwrap-some (wat/read-str "#wat.core.Option/Some [42]"))))
  (is (wat/none-variant? (wat/read-str "#wat.core.Option/None []"))))

(deftest variant-writers-emit-wat-tags
  (is (= "#wat.core.Option/Some [42]" (wat/write-str (wat/some-of 42))))
  (is (= "#wat.core.Option/None []" (wat/write-str (wat/none-of))))
  (is (= "#wat.core.Result/Ok [7]" (wat/write-str (wat/ok-of 7)))))
