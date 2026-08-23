;; wat-scripts/fix-macro-param-types.wat — the arc-251 fix-macro-param-types codemod,
;; run over real wat source files IN WAT, through the wat CLI. The migration tool,
;; self-hosted: no Rust harness.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/core.wat" "wat/Record.wat"]\n' | cargo wat ./wat-scripts/fixes/fix-macro-param-types.wat
;;
;; readln parses the line as a Vector<String>; for each path:
;;   read-file → :wat::fix::fix-macro-param-types → write-file.
;; The rewrite is comment-faithful and idempotent (re-running yields zero changes), so it
;; is safe to run over a clean tree.

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:wat::fix::fix-macro-param-types (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[fixed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
