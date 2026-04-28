(ns wat-edn.scanner-test
  "Locks the wat scanner against fixture .wat content. Verifies
  it extracts the right struct declarations and ignores
  non-struct forms."
  (:require [clojure.test :refer [deftest is testing]]
            [wat-edn.scanner :as scanner]))

(def simple-wat
  "(:wat::core::struct :myapp::Order
     (id :i64)
     (name :String))")

(def two-structs-wat
  "(:wat::core::struct :a::B
     (x :i64))
   (:wat::core::struct :c::d::E
     (y :String)
     (z :Keyword))")

(def with-noise-wat
  "(:wat::core::use! :rust::lru::LruCache)
   (:wat::core::struct :myapp::Real
     (id :i64))
   (:wat::core::define (:myapp::compute (n :i64) -> :i64) n)
   (:wat::core::struct :myapp::AnotherReal
     (name :String))")

(deftest extracts-single-struct
  (let [r (scanner/extract-structs simple-wat)]
    (is (= 1 (count r)))
    (is (= 'myapp/Order (:tag (first r))))
    (is (= {:id "i64" :name "String"}
           (:fields (first r))))))

(deftest extracts-multiple-structs
  (let [r (scanner/extract-structs two-structs-wat)]
    (is (= 2 (count r)))
    (is (= #{'a/B 'c.d/E} (set (map :tag r))))))

(deftest skips-non-struct-forms
  (let [r (scanner/extract-structs with-noise-wat)
        tags (set (map :tag r))]
    (is (= 2 (count r)))
    (is (contains? tags 'myapp/Real))
    (is (contains? tags 'myapp/AnotherReal))
    ;; Should NOT include the use! or define forms
    (is (not (some #(= 'myapp/compute %) tags)))))

(deftest wat-path-becomes-edn-tag
  (let [r (scanner/extract-structs
           "(:wat::core::struct :enterprise::observer::market::TradeSignal (asset :Keyword))")]
    (is (= 'enterprise.observer.market/TradeSignal
           (:tag (first r))))))

(deftest empty-wat-source-returns-empty
  (is (= [] (scanner/extract-structs "")))
  (is (= [] (scanner/extract-structs "; just a comment\n")))
  (is (= [] (scanner/extract-structs "  \n\t  "))))

(deftest comments-are-skipped
  (let [r (scanner/extract-structs
           "; comment before
            (:wat::core::struct :x::Y
              ; comment in body
              (a :i64))
            ; comment after")]
    (is (= 1 (count r)))
    (is (= 'x/Y (:tag (first r))))
    (is (= {:a "i64"} (:fields (first r))))))

(deftest commas-are-whitespace
  (let [r (scanner/extract-structs
           "(:wat::core::struct :x::Y, (a :i64), (b :String))")]
    (is (= {:a "i64" :b "String"} (:fields (first r))))))

(deftest strings-are-skipped
  ;; A string with parens inside shouldn't confuse the scanner.
  (let [r (scanner/extract-structs
           "(:wat::core::struct :x::Y (msg :String))
            (:wat::core::define (:x::greet -> :String) \"hello (world)\")")]
    (is (= 1 (count r)))))

(deftest field-types-include-generics
  (let [r (scanner/extract-structs
           "(:wat::core::struct :myapp::Order
              (items :Vec<i64>)
              (until :Option<wat::time::Instant>))")]
    (is (= "Vec<i64>" (get (:fields (first r)) :items)))
    ;; Note: nested `::` in type spec is preserved as-is in the
    ;; type string (Clojure consumers use it as opaque).
    (is (= "Option<wat::time::Instant>"
           (get (:fields (first r)) :until)))))
