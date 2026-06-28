;; Contract 04: defstruct with both form-level and field metadata.
(:wat::core::defstruct :my::Client
  {:restricted-to  [:my::]
   :field-metadata {:server-id {:restricted-to [:my::]}
                    :client-id {:restricted-to [:my::]}}}
  [server-id <- :wat::core::Uuid
   client-id <- :wat::core::Uuid
   public-data <- :wat::core::i64])
