;; uuid_edn_write_reader_literal.wat — edn::write produces #uuid "..." (44 chars).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [u        (:wat::core::Uuid/v4)
     edn-form (:wat::edn::write u)
     len      (:wat::string::length edn-form)]
    (:wat::core::if (:wat::core::= len 44) 
      (:wat::kernel::println "EDN-LEN-OK")
      (:wat::kernel::println "EDN-LEN-FAIL"))))
