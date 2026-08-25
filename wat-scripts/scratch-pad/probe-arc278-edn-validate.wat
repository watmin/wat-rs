;; Arc 278 the REQUEST-MALFORMED wall (Stone 1) — the VALIDATOR, on its own.
;;
;; What `:wat::core::conforms?` structurally cannot do, and `:wat::edn::validate` does:
;; a DEEP walk of a value against a DECLARED type. conforms? on an Aggregate is a
;; NOMINAL identity check (runtime.rs conforms_check, TypeDef::Aggregate arm →
;; concrete_type_name_matches) — it never recurses into the record's FIELDS. So a
;; well-formed EDN frame with a wrong-typed body under a CORRECT tag sails through it.
;; That gap is the wire denial of service (probe-arc278-wire-dos-service-killed.wat).
;;
;; validate reuses `edn_shim::edn_to_typed_value` — the deep walker that has had ZERO
;; production callers since arc 258 Stone 258.5b deleted its last one on the
;; trusted-wire premise. Nothing new is validated; the two halves are connected.

(:wat::core::defrecord :vprobe::PutRequest [items <- (:wat::core::Vector :- [:wat::core::String])])

(:wat::core::defn :vprobe::render [v <- :wat::edn::Validation] -> :wat::core::String
  (:wat::core::match v
    (:wat::edn::Validation::Valid "VALID")
    ((:wat::edn::Validation::Invalid path expected got)
      (:wat::string::concat "INVALID at "
        (:wat::string::concat (:wat::edn::write path)
          (:wat::string::concat " expected="
            (:wat::string::concat expected
              (:wat::string::concat " got=" got))))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [good (:vprobe::PutRequest :items (:wat::core::Vector :wat::core::String "abcd"))
     ;; the attacker's frame: correct TAG, wrong-typed BODY
     bad  (:wat::edn::read "#vprobe/PutRequest {:items [1 2 3]}")
     _ (:wat::kernel::println
         (:wat::string::concat "good => " (:vprobe::render (:wat::edn::validate good :vprobe::PutRequest))))
     _ (:wat::kernel::println
         (:wat::string::concat "bad  => " (:vprobe::render (:wat::edn::validate bad :vprobe::PutRequest))))
     ;; conforms? — the SAME bad value — is the gap, shown side by side
     _ (:wat::kernel::println
         (:wat::string::concat "bad conforms? => "
           (:wat::core::show (:wat::core::conforms? bad :vprobe::PutRequest))))]
    nil))
