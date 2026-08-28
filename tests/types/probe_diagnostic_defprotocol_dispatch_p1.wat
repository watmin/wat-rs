;; Probe 1: dispatcher routes to per-type impl based on first-arg classifier.
(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::core::defrecord :myapp::Celsius [degrees <- :wat::core::f64])

(:wat::core::defn :myapp::Voltage/Formattable-format
  [self <- :wat::core::Record] -> :wat::core::String
  "voltage-formatted")

(:wat::core::defn :myapp::Celsius/Formattable-format
  [self <- :wat::core::Record] -> :wat::core::String
  "celsius-formatted")

(:wat::core::defn :myapp::Formattable/format
  [self <- :wat::core::Record] -> :wat::core::String
  (:wat::core::let
    [classifier    (:wat::holon::extract-classifier self)
     mangled-str   (:wat::string::concat classifier "/Formattable-format")
     mangled-kw    (:wat::keyword::from-string mangled-str)]
    (:wat::core::apply  mangled-kw [self])))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [v  (:myapp::Voltage :magnitude 5.0)
     c  (:myapp::Celsius :degrees 20.0)
     vf (:myapp::Formattable/format v)
     cf (:myapp::Formattable/format c)
     joined (:wat::string::concat vf "|")]
    (:wat::string::concat joined cf)))
