;; Co-located fixture for wat_string_ops.rs — slurped via startup_beside(file!()).
;; Bool fns return bool. String fns return String.
;; Error fns (compute-split-empty-sep, compute-regex-invalid) error at eval time.

;; ─── contains? / starts-with? / ends-with? ───────────────────────────────────

(:wat::core::defn :my::compute-contains-hit [] -> :wat::core::bool
  (:wat::string::contains? "hello world" "world"))

(:wat::core::defn :my::compute-contains-miss [] -> :wat::core::bool
  (:wat::string::contains? "hello" "xyz"))

(:wat::core::defn :my::compute-starts-with-hit [] -> :wat::core::bool
  (:wat::string::starts-with? "foobar" "foo"))

(:wat::core::defn :my::compute-starts-with-miss [] -> :wat::core::bool
  (:wat::string::starts-with? "foobar" "bar"))

(:wat::core::defn :my::compute-ends-with-hit [] -> :wat::core::bool
  (:wat::string::ends-with? "foobar" "bar"))

(:wat::core::defn :my::compute-ends-with-miss [] -> :wat::core::bool
  (:wat::string::ends-with? "foobar" "foo"))

;; ─── string::length ──────────────────────────────────────────────────────────

(:wat::core::defn :my::compute-length-chars [] -> :wat::core::bool
  (:wat::core::= (:wat::string::length "héllo") 5))

;; ─── string::trim ────────────────────────────────────────────────────────────

(:wat::core::defn :my::compute-trim [] -> :wat::core::String
  (:wat::string::trim "   hello   "))

;; ─── string::split / join ────────────────────────────────────────────────────

(:wat::core::defn :my::compute-split-join [] -> :wat::core::String
  (:wat::core::let
    [pieces (:wat::string::split "a,b,c" ",")]
    (:wat::string::join "|" pieces)))

;; 279.3 — join is generic over element type, rendering each element
;; through the total `str`. Non-String elements (i64 here).
(:wat::core::defn :my::compute-join-non-string [] -> :wat::core::String
  (:wat::string::join "," [1 2 3]))

;; 279.3 ★ load-bearing — a String element renders BARE, not re-quoted
;; by the encoder. Must be "a-b", never "\"a\"-\"b\"".
(:wat::core::defn :my::compute-join-string-bare [] -> :wat::core::String
  (:wat::string::join "-" ["a" "b"]))

(:wat::core::defn :my::compute-split-empty-sep [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::string::split "abc" ""))

;; ─── regex::matches? ─────────────────────────────────────────────────────────

(:wat::core::defn :my::compute-regex-match [] -> :wat::core::bool
  (:wat::regex::matches? "[0-9]+" "order #42 shipped"))

(:wat::core::defn :my::compute-regex-no-match [] -> :wat::core::bool
  (:wat::regex::matches? "^foo$" "foobar"))

(:wat::core::defn :my::compute-regex-invalid [] -> :wat::core::bool
  (:wat::core::let
    [_ (:wat::regex::matches? "[unclosed" "x")]
    false))

