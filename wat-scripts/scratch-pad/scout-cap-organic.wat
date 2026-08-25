;; scout-cap-organic: the ORGANIC capture path a macro would expand to.
;; User writes (fn ...) organically; macro wraps it as (write-forms (quote (fn ...))).
;; Result is a String field (fork-safe). Server does read-string to rebuild the form.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; what the macro emits at the call site — NO hand-typed string, NO user quote visible:
     captured (:wat::core::write-forms
                 (:wat::core::quote
                    (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::bool
                      (:wat::core::> n 3))))
     ;; server side rebuild:
     form     (:wat::core::match (:wat::core::read-string captured) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     kids     (:wat::core::ast->children form)
     inner    (:wat::core::Option/expect (:wat::core::get kids 0) "no child 0")
     pure     (:wat::rete::pure? inner)
     det      (:wat::rete::deterministic? inner)]
    (:wat::kernel::println (:wat::string::concat "CAPTURED-STRING=" captured))
    (:wat::kernel::println (:wat::core::str pure))
    (:wat::kernel::println (:wat::core::str det))))
