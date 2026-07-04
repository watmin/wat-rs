;; RED probe (one-shot) — proves the REPL seam: readln → read-string → eval! → println.
;; No loop yet: isolate the four-form chain so any failure names exactly which seam broke.
;;
;; Feed it (on stdin) an EDN string whose text is a wat expression's source:
;;   printf '"(:wat::core::+ 1 2)"\n' | target/release/wat crates/wat-edn/demo/probe-oneshot.wat
;; Expect on stdout:  3
;;
;;   readln -> String   : block, read one EDN-decoded string (the expression source)
;;   read-string        : parse that source with wat's own reader → an AST
;;   eval!              : constrained eval of the AST against the frozen world
;;   println            : EDN-encode the result value to stdout

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::eval-ast!
      (:wat::core::first
        (:wat::core::read-string
          (:wat::kernel::readln -> :wat::core::String))))))
