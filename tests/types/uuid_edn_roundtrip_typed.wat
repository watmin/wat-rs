;; uuid_edn_roundtrip_typed.wat — Uuid survives edn::write + edn::read roundtrip.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [u        (:wat::uuid::v4)
     edn-form (:wat::edn::write u)
     back     (:wat::edn::read edn-form)]
    (:wat::core::if (:wat::core::= back u) 
      (:wat::kernel::println "EDN-ROUNDTRIP-OK")
      (:wat::kernel::println "EDN-ROUNDTRIP-FAIL"))))
