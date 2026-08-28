;; Probe 3: missing impl is OBSERVABLE error (no per-class impl for Unhandled).
(:wat::core::defrecord :myapp::Unhandled [v <- :wat::core::i64])

(:wat::core::defn :myapp::Formattable/format
  [self <- :wat::core::Record] -> :wat::core::String
  (:wat::core::let
    [classifier    (:wat::holon::extract-classifier self)
     mangled-str   (:wat::string::concat classifier "/Formattable-format")
     mangled-kw    (:wat::keyword::from-string mangled-str)]
    (:wat::core::apply  mangled-kw [self])))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:myapp::Formattable/format (:myapp::Unhandled :v 42)))
