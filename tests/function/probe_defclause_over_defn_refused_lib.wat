;; THE LIBRARY. Loaded by every consumer fixture beside this one through an
;; InMemoryLoader keyed "lib.wat". Its own internal call site — `:mylib::caller`
;; calling `:mylib::greet` — is what a consumer's `defclause` used to break.
(:wat::core::defn :mylib::greet  [s <- :wat::core::String] -> :wat::core::String s)
(:wat::core::defn :mylib::caller [] -> :wat::core::String (:mylib::greet "hi"))
