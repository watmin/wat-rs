;; wat-scripts/fixes/strip-expect-ascription.wat — the `-> :T` annihilation codemod
;; for Option/expect + Result/expect, run over real wat source IN WAT, through the CLI.
;; The migration tool, self-hosted (use-the-tool, not hand-fix); kept for future readers.
;;
;; Arc 258 sub-strike 1 (the clean kills): Option/expect + Result/expect drop their
;; `-> :T` return ascription — the unwrapped type is now INFERRED from the (Option :- [T]) /
;; (Result :- [T E]) argument (the recv'/select' pattern, 258.5b). The checker/runtime were
;; changed to the bare 2-arg layout; this codemod rewrites every CALL SITE:
;;
;;   (:wat::core::Option/expect -> :T <opt> <msg>)  →  (:wat::core::Option/expect <opt> <msg>)
;;   (:wat::core::Result/expect -> :T <res> <msg>)  →  (:wat::core::Result/expect <res> <msg>)
;;
;; It is a THIN wrapper over the GENERIC, reusable `:wat::fix::strip-arrow-ascription`
;; (wat/fix.wat) — head-set-parameterized, so the `if` / `match` `-> :T` kills reuse the
;; SAME tool with their own head-sets. Comment-faithful + idempotent; touches ONLY the two
;; expect heads (NOT `if`, `match`, `apply`, `readln`, fn-return `-> :T`, or 251 forms).
;;
;; Usage (one EDN vector of EVERY path holding the forms on stdin):
;;   printf '["wat/Record.wat" ...]\n' \
;;     | cargo wat ./wat-scripts/fixes/strip-expect-ascription.wat

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::strip-arrow-ascription src
    (:wat::core::Vector :wat::core::String
      ":wat::core::Option/expect"
      ":wat::core::Result/expect")))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[stripped] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
