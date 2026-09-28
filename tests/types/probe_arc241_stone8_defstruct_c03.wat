;; Contract 03: defstruct with :field-metadata.
(:wat::core::defstruct :my::Capability
  {:field-metadata {:witness {:restricted-to [:my::]}}}
  [witness <- wat.type/i64
   data <- wat.type/i64])
