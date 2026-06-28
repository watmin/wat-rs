;; Co-located fixture for wat_string_ops.rs — slurped via startup_beside(file!()).
;; Bool fns return bool. String fns return String.
;; Error fns (compute-split-empty-sep, compute-regex-invalid) error at eval time.

;; ─── contains? / starts-with? / ends-with? ───────────────────────────────────

(:wat::core::defn :my::compute-contains-hit [] -> :wat::core::bool
  (:wat::core::string::contains? "hello world" "world"))

(:wat::core::defn :my::compute-contains-miss [] -> :wat::core::bool
  (:wat::core::string::contains? "hello" "xyz"))

(:wat::core::defn :my::compute-starts-with-hit [] -> :wat::core::bool
  (:wat::core::string::starts-with? "foobar" "foo"))

(:wat::core::defn :my::compute-starts-with-miss [] -> :wat::core::bool
  (:wat::core::string::starts-with? "foobar" "bar"))

(:wat::core::defn :my::compute-ends-with-hit [] -> :wat::core::bool
  (:wat::core::string::ends-with? "foobar" "bar"))

(:wat::core::defn :my::compute-ends-with-miss [] -> :wat::core::bool
  (:wat::core::string::ends-with? "foobar" "foo"))

;; ─── string::length ──────────────────────────────────────────────────────────

(:wat::core::defn :my::compute-length-chars [] -> :wat::core::bool
  (:wat::core::= (:wat::core::string::length "héllo") 5))

;; ─── string::trim ────────────────────────────────────────────────────────────

(:wat::core::defn :my::compute-trim [] -> :wat::core::String
  (:wat::core::string::trim "   hello   "))

;; ─── string::split / join ────────────────────────────────────────────────────

(:wat::core::defn :my::compute-split-join [] -> :wat::core::String
  (:wat::core::let
    [pieces (:wat::core::string::split "a,b,c" ",")]
    (:wat::core::string::join "|" pieces)))

(:wat::core::defn :my::compute-split-empty-sep [] -> :wat::core::Vector<wat::core::String>
  (:wat::core::string::split "abc" ""))

;; ─── regex::matches? ─────────────────────────────────────────────────────────

(:wat::core::defn :my::compute-regex-match [] -> :wat::core::bool
  (:wat::core::regex::matches? "[0-9]+" "order #42 shipped"))

(:wat::core::defn :my::compute-regex-no-match [] -> :wat::core::bool
  (:wat::core::regex::matches? "^foo$" "foobar"))

(:wat::core::defn :my::compute-regex-invalid [] -> :wat::core::bool
  (:wat::core::let
    [_ (:wat::core::regex::matches? "[unclosed" "x")]
    false))

