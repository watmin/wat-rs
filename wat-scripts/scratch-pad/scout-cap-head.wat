;; Show the EXACT head string the purity classifier sees, for :: vs dotted.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [colon (:wat::core::match (:wat::core::read-string "(:wat::core::> n 3)") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     dot   (:wat::core::match (:wat::core::read-string "(:wat.core/> n 3)") ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     ck    (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children colon) 0) "c")
     dk    (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children dot) 0) "d")
     chead (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children ck) 0) "ch")
     dhead (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children dk) 0) "dh")]
    (:wat::kernel::println (:wat::string::concat "colon-head=" (:wat::core::ast-name chead)))
    (:wat::kernel::println (:wat::string::concat "dot-head="   (:wat::core::ast-name dhead)))))
