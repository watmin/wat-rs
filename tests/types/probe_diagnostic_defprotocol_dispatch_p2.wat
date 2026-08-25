;; Probe 2: open extension — per-class impl defined AFTER dispatcher still routes.
(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :myapp::Formattable/format
  [self <- :wat::core::Record] -> :wat::core::String
  (:wat::core::let
    [classifier    (:wat::holon::extract-classifier self)
     mangled-str   (:wat::string::concat classifier "/Formattable-format")
     mangled-kw    (:wat::core::keyword/from-string mangled-str)]
    (:wat::core::apply  mangled-kw [self])))

(:wat::core::defn :myapp::Voltage/Formattable-format
  [self <- :wat::core::Record] -> :wat::core::String
  "voltage-after-dispatcher")

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:myapp::Formattable/format (:myapp::Voltage :magnitude 5.0)))
