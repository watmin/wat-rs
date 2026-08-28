;; wat-scripts/scratch-pad/255-stone-o-iv-c-0-require-family-wrong-type.wat — arc 255
;; Stone O-iv-c-0, acceptance row 0. Trigger ONE wrong-typed-argument TypeMismatch
;; from EACH of the nine `require_*` fns in src/holon/require.rs (require_hologram,
;; require_fn, require_vector, require_subspace, require_reckoner, require_engram,
;; require_engram_library, require_string, require_numeric), printing kind+message
;; (the message already renders `got` via ValueSnapshot's Display — same pattern as
;; 255-stone-h-1a-holon-wrong-arity.wat) so the before/after (Value → &Value) diff is
;; byte-for-byte. NOTHING MOVED: this is a read-only probe. Scratch, per
;; holon/CLAUDE.md's .wat scratch convention.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "── require_hologram: (:wat::holon::Hologram/len 5) ──")
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::holon::Hologram/len 5)))
      ((:wat::core::Ok _) (:wat::kernel::println "UNEXPECTED: ok"))
      ((:wat::core::Err e)
        (:wat::kernel::println (:wat::string::concat "kind=" (:wat::core::EvalError/kind e)
                                  " message=" (:wat::core::EvalError/message e)))))

    (:wat::kernel::println "── require_fn: (:wat::holon::Hologram/make 5) ──")
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::holon::Hologram/make 5)))
      ((:wat::core::Ok _) (:wat::kernel::println "UNEXPECTED: ok"))
      ((:wat::core::Err e)
        (:wat::kernel::println (:wat::string::concat "kind=" (:wat::core::EvalError/kind e)
                                  " message=" (:wat::core::EvalError/message e)))))

    (:wat::kernel::println "── require_vector: (:wat::holon::vector-bytes 5) ──")
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::holon::vector-bytes 5)))
      ((:wat::core::Ok _) (:wat::kernel::println "UNEXPECTED: ok"))
      ((:wat::core::Err e)
        (:wat::kernel::println (:wat::string::concat "kind=" (:wat::core::EvalError/kind e)
                                  " message=" (:wat::core::EvalError/message e)))))

    (:wat::kernel::println "── require_subspace: (:wat::holon::OnlineSubspace/dim 5) ──")
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::holon::OnlineSubspace/dim 5)))
      ((:wat::core::Ok _) (:wat::kernel::println "UNEXPECTED: ok"))
      ((:wat::core::Err e)
        (:wat::kernel::println (:wat::string::concat "kind=" (:wat::core::EvalError/kind e)
                                  " message=" (:wat::core::EvalError/message e)))))

    (:wat::kernel::println "── require_reckoner: (:wat::holon::Reckoner/observe 5 1 0 1.0) ──")
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::holon::Reckoner/observe 5 1 0 1.0)))
      ((:wat::core::Ok _) (:wat::kernel::println "UNEXPECTED: ok"))
      ((:wat::core::Err e)
        (:wat::kernel::println (:wat::string::concat "kind=" (:wat::core::EvalError/kind e)
                                  " message=" (:wat::core::EvalError/message e)))))

    (:wat::kernel::println "── require_engram: (:wat::holon::Engram/name 5) ──")
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::holon::Engram/name 5)))
      ((:wat::core::Ok _) (:wat::kernel::println "UNEXPECTED: ok"))
      ((:wat::core::Err e)
        (:wat::kernel::println (:wat::string::concat "kind=" (:wat::core::EvalError/kind e)
                                  " message=" (:wat::core::EvalError/message e)))))

    (:wat::kernel::println "── require_engram_library: (:wat::holon::EngramLibrary/add 5 \"name\" 1) ──")
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::holon::EngramLibrary/add 5 "name" 1)))
      ((:wat::core::Ok _) (:wat::kernel::println "UNEXPECTED: ok"))
      ((:wat::core::Err e)
        (:wat::kernel::println (:wat::string::concat "kind=" (:wat::core::EvalError/kind e)
                                  " message=" (:wat::core::EvalError/message e)))))

    (:wat::kernel::println "── require_string: (:wat::holon::EngramLibrary/add (:wat::holon::EngramLibrary/new 10000) 5 1) ──")
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote
                          (:wat::holon::EngramLibrary/add (:wat::holon::EngramLibrary/new 10000) 5 1)))
      ((:wat::core::Ok _) (:wat::kernel::println "UNEXPECTED: ok"))
      ((:wat::core::Err e)
        (:wat::kernel::println (:wat::string::concat "kind=" (:wat::core::EvalError/kind e)
                                  " message=" (:wat::core::EvalError/message e)))))

    (:wat::kernel::println "── require_numeric: (:wat::holon::Thermometer \"x\" 0.0 10.0) ──")
    (:wat::core::match (:wat::eval-ast! (:wat::core::quote (:wat::holon::Thermometer "x" 0.0 10.0)))
      ((:wat::core::Ok _) (:wat::kernel::println "UNEXPECTED: ok"))
      ((:wat::core::Err e)
        (:wat::kernel::println (:wat::string::concat "kind=" (:wat::core::EvalError/kind e)
                                  " message=" (:wat::core::EvalError/message e)))))))
