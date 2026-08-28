;; Probe: inside a defmacro body, build a PLAIN runtime (Vector :- [String]) (not WatAST),
;; then `~@` splice it into a `(:wat::core::Vector :wat::core::String ~@strs)` call form.
;; Tests whether splice auto-wraps each String element as a literal. Mirrors sieve-pred's
;; `~src` (a plain runtime String unquoted directly) but for SPLICE of a String Vector.

(:wat::core::defmacro :probe::mk-vec
  [] -> :wat::WatAST
  (:wat::core::let
    [strs (:wat::core::Vector :wat::core::String "usr::Temp" "usr::Hot")]
    `(:wat::core::Vector :wat::core::String ~@strs)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [v    (:probe::mk-vec)
     ok   (:wat::vec::contains? v "usr::Hot")
     bad  (:wat::vec::contains? v "nope")]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat "ok="  (:wat::core::str ok)))
      (:wat::kernel::println (:wat::string::concat "bad=" (:wat::core::str bad))))))
