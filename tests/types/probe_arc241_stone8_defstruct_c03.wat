;; Contract 03: defstruct with :field-metadata.
(:wat::core::defstruct :my::Capability
  {:field-metadata {:witness {:restricted-to [:my::]}}}
  [witness <- :wat::core::i64
   data <- :wat::core::i64])
