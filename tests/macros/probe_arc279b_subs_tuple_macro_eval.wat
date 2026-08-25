;; tests/macros/probe_arc279b_subs_tuple_macro_eval.wat — co-located fixture for
;; probe_arc279b_subs_tuple_macro_eval.rs, slurped via startup_beside(file!()).
;;
;; A macro that, at expand time, walks the chars of a string literal carrying a Tuple(kept, n-open)
;; accumulator: appends each non-{ char to kept, increments n-open on each {. Emits the
;; string literal "<kept>|<n-open>".
(:wat::core::defmacro :user::strip-braces
  [s <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::let
    [str   (:wat::core::ast-name s)
     len   (:wat::string::length str)
     ;; Arc 118.2a — `map` flipped LAZY; this macro evaluates at expand time (pure-total
     ;; program-body, bootstrap-adjacent), so `foldl`+`conj` (Rust-native) stand in.
     chars (:wat::core::foldl
             (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
               (:wat::core::conj acc (:wat::string::subs str i (:wat::core::i64::+ i 1))))
             (:wat::core::Vector :wat::core::String)
             (:wat::core::range 0 len))
     final (:wat::core::foldl
             (:wat::core::fn [acc <- :wat::core::Tuple
                              c   <- :wat::core::String]
               -> :wat::core::Tuple
               (:wat::core::let
                 [kept   (:wat::core::first acc)
                  nopen  (:wat::core::second acc)]
                 (:wat::core::if
                   (:wat::core::= c "{")
                   
                   (:wat::core::Tuple kept (:wat::core::i64::+ nopen 1))
                   (:wat::core::Tuple (:wat::string::concat kept c) nopen))))
             (:wat::core::Tuple "" 0)
             chars)
     kept   (:wat::core::first final)
     nopen  (:wat::core::second final)
     out    (:wat::string::concat kept
              (:wat::string::concat "|" (:wat::core::i64::to-string nopen)))]
    (:wat::core::first
      (:wat::core::ast->children
        (:wat::core::match (:wat::core::read-string
          (:wat::string::concat "\"" (:wat::string::concat out "\""))) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::core::macro-error (:wat::string::concat "expand-time read-string failed: " (:wat::core::Error/message __cause)))))))))

(:wat::core::defn :user::probe [] -> :wat::core::String (:user::strip-braces "a{b{c"))
