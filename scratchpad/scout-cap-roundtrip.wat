;; scout-cap-roundtrip: faithful inner-form preservation across write-forms/read-string.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [form      (:wat::core::quote
                  (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::bool
                    (:wat::core::> n 3)))
     edn       (:wat::core::write-forms form)
     back      (:wat::core::read-string edn)
     kids      (:wat::core::ast->children back)
     inner     (:wat::core::Option/expect (:wat::core::get kids 0) "no child 0")
     edn-inner (:wat::core::write-forms inner)
     same      (:wat::core::= edn edn-inner)]
    (:wat::kernel::println (:wat::core::string::concat "EDN1=" edn))
    (:wat::kernel::println (:wat::core::string::concat "INNER=" edn-inner))
    (:wat::kernel::println (:wat::core::str same))))
