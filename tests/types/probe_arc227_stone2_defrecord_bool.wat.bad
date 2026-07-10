;; Negative fixture: constructor rejects bool where f64 expected.
(:wat::core::defrecord :test::Measured [value <- :wat::core::f64])
(:wat::core::defn :user::compute [] -> :wat::holon::HolonAST (:test::Measured true))
