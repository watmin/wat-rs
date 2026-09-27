;; Types the corpus compares that are not in the stdlib. The agreement test
;; loads this file so those names resolve. Shapes match the corpus declarations.
(:wat::core::defrecord :my::Pt [x <- :wat::core::i64 y <- :wat::core::i64])
(:wat::holon::defrecord :my::HPt [x <- :wat::core::i64 y <- :wat::core::i64])
(:wat::core::defrecord :test::rd::Pt [x <- :wat::core::i64 y <- :wat::core::i64])
(:wat::holon::defrecord :test::rd::HPt [x <- :wat::core::i64 y <- :wat::core::i64])
(:wat::core::defstruct :my::Point [x <- :wat::core::f64 y <- :wat::core::f64])
(:wat::core::defenum :my::Color :wat::enum::Pure :Red :Blue :Green)
(:wat::core::defenum :eq::Method :wat::enum::Pure :GET :POST :PUT :DELETE)
(:wat::core::defenum :eq::Status :wat::enum::Pure :OPEN :CLOSED)
(:wat::core::defenum :user::NodeKind :wat::enum::Pure
  :IntLit []
  :Map []
  :Set [])
(:wat::core::defenum :probe255rf::VariantEx :wat::enum::Pure :V [sk <- :wat::core::i64])
