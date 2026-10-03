;; struct_restricted_field_allowed.wat — whitelisted caller can access restricted field.
(wat.core/defstruct my/Vault
  {:restricted-to  [my.admin]
   :field-metadata {:secret {:restricted-to [my.auditor]}}}
  [secret :- wat.type/i64
   name   :- wat.type/i64])
(wat.core/defn my.admin/mint [] :- my/Vault
  (my/Vault :secret 0 :name 0))
(wat.core/defn my.auditor/audit
  [v :- my/Vault] :- wat.type/i64
  (my.Vault/secret v))
