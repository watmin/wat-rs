;; scout-cap-ednwire: does the GENERAL wire codec (edn::write/edn::read — the
;; SAME codec a process-peer message uses) preserve a quoted (fn ...) form?
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [form (:wat::core::quote
             (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::bool
               (:wat::core::> n 3)))
     wire (:wat::edn::write form)]
    (:wat::kernel::println (:wat::string::concat "WIRE=" wire))
    ;; read it back through the general codec, then try to write-forms it:
    (:wat::core::let [back (:wat::edn::read wire)]
      (:wat::kernel::pprintln back))))
