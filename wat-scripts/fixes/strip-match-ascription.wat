;; wat-scripts/fixes/strip-match-ascription.wat — the `-> :T` annihilation codemod for
;; `match`, run over real wat source IN WAT, through the CLI. Self-hosted migration tool;
;; kept for future readers.
;;
;; Arc 258 sub-strike 2: `match` drops its `-> :T` result ascription — the result type is
;; now INFERRED by unifying the arm bodies (the mechanism `if` already uses). The
;; checker/runtime change to the bare layout is per-form; this rewrites every CALL SITE:
;;
;;   (:wat::core::match <scrut> -> :T (pat body) ...)  →  (:wat::core::match <scrut> (pat body) ...)
;;
;; A THIN wrapper over the GENERIC, reusable `:wat::fix::strip-arrow-ascription`
;; (wat/fix.wat) — the SAME tool sub-strike 1 used for Option/expect + Result/expect, here
;; with the `match` head-set. The generic is position-agnostic, so it deletes the `->` +
;; type wherever they sit (child[2] for match — after the scrutinee). Comment-faithful +
;; idempotent; touches ONLY the `match` head.
;;
;; ⚠ BOOTSTRAP ORDER (this run): run BEFORE the new infer_match lands, while the current
;; binary still accepts `match -> :T` — so the corpus goes bare FIRST, then the new
;; bare-unify checker is written against the already-bare corpus (no stash-dance needed).
;; See the BOOTSTRAP header in wat/fix.wat for the general checker-change case.
;;
;; Usage (one EDN vector of EVERY path holding `match -> :T` on stdin):
;;   printf '["wat/service.wat" ...]\n' \
;;     | cargo wat ./wat-scripts/fixes/strip-match-ascription.wat

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::strip-arrow-ascription src
    (:wat::core::Vector :wat::core::String ":wat::core::match")))

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
