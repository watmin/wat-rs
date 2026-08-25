;; uuid_to_string_roundtrip.wat — Uuid/to-string round-trip.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [u        (:wat::core::Uuid/v4)
     s        (:wat::core::Uuid/to-string u)
     reparsed (:wat::core::Uuid/from-string s)]
    (:wat::core::do
      (:wat::core::if (:wat::core::= (:wat::string::length s) 36) 
        (:wat::kernel::println "LEN-36-OK")
        (:wat::kernel::println "LEN-36-FAIL"))
      (:wat::core::match reparsed 
        ((:wat::core::Some u2)
          (:wat::core::if (:wat::core::= (:wat::core::Uuid/to-string u2) s) 
            (:wat::kernel::println "ROUNDTRIP-OK")
            (:wat::kernel::println "ROUNDTRIP-FAIL")))
        (:wat::core::None (:wat::kernel::println "ROUNDTRIP-NONE"))))))
