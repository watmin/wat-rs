;; Show the EXACT head string the purity classifier sees, for :: vs dotted.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [colon (:wat::core::read-string "(:wat::core::> n 3)")
     dot   (:wat::core::read-string "(:wat.core/> n 3)")
     ck    (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children colon) 0) "c")
     dk    (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children dot) 0) "d")
     chead (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children ck) 0) "ch")
     dhead (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children dk) 0) "dh")]
    (:wat::kernel::println (:wat::core::string::concat "colon-head=" (:wat::core::ast-name chead)))
    (:wat::kernel::println (:wat::core::string::concat "dot-head="   (:wat::core::ast-name dhead)))))
