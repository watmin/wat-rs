;; Arc 278 #57 round 1a — sanity probe for the nine newly-minted monomorphic
;; rete aliases (String/*, string::{length,trim,to-lowercase}, i64::to-f64).
;; Not a test file itself (see tests/rete for the durable gate); this is a
;; loadable, type-checked reference proving the new spellings resolve.
(def :probe-string-concat (:wat::rete::string::concat "a" "b"))
(def :probe-string-starts-with (:wat::rete::string::starts-with? "abc" "a"))
(def :probe-string-ends-with (:wat::rete::string::ends-with? "abc" "c"))
(def :probe-string-contains (:wat::rete::string::contains? "abc" "b"))
(def :probe-string-empty (:wat::rete::string::empty? ""))
(def :probe-string-length (:wat::rete::string::length "abc"))
(def :probe-string-trim (:wat::rete::string::trim "  abc  "))
(def :probe-string-to-lowercase (:wat::rete::string::to-lowercase "ABC"))
(def :probe-i64-to-f64 (:wat::rete::i64::to-f64 42))
