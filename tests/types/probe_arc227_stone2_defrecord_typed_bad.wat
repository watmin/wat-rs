;; Negative fixture: constructor rejects wrong type (String where f64 expected).
(:wat::core::defrecord :test::Voltage [value <- :wat::core::f64])
(:wat::core::defn :user::compute [] -> :wat::holon::HolonAST (:test::Voltage "not-a-float"))
