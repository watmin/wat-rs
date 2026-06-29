;; struct_restricted_field_allowed.wat — whitelisted caller can access restricted field.
(:wat::core::defstruct :my::Vault
  {:restricted-to  [:my::admin::]
   :field-metadata {:secret {:restricted-to [:my::auditor::]}}}
  [secret <- :wat::core::i64
   name   <- :wat::core::i64])
(:wat::core::defn :my::admin::mint [] -> :my::Vault
  (:my::Vault 0 0))
(:wat::core::defn :my::auditor::audit
  [v <- :my::Vault] -> :wat::core::i64
  (:my::Vault/secret v))
