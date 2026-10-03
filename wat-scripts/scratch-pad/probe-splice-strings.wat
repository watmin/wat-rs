;; Probe: inside a defmacro body, build a PLAIN runtime (Vector :- [String]) (not WatAST),
;; then `~@` splice it into a `(:wat::core::Vector :wat::core::String ~@strs)` call form.
;; Tests whether splice auto-wraps each String element as a literal. Mirrors sieve-pred's
;; `~src` (a plain runtime String unquoted directly) but for SPLICE of a String Vector.

(:wat::core::defmacro :probe::mk-vec
  [] -> wat.type/AST
  (:wat::core::let
    [strs (wat.type/Vector :- [wat.type/String] "usr::Temp" "usr::Hot")]
    `(wat.type/Vector :- [wat.type/String] ~@strs)))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [v    (:probe::mk-vec)
     ok   (:wat::core::contains? v "usr::Hot")
     bad  (:wat::core::contains? v "nope")]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat "ok="  (:wat::core::str ok)))
      (:wat::kernel::println (:wat::string::concat "bad=" (:wat::core::str bad))))))
