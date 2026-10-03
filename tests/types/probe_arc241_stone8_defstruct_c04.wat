;; Contract 04: defstruct with both form-level and field metadata.
(wat.core/defstruct my/Client
  {:restricted-to  [my]
   :field-metadata {:server-id {:restricted-to [my]}
                    :client-id {:restricted-to [my]}}}
  [server-id :- wat.uuid/UUID
   client-id :- wat.uuid/UUID
   public-data :- wat.type/i64])
