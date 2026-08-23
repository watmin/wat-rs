;; wat-scripts/fixes/rename-list-to-seq.wat — arc 118, the eager-world rename.
;; The eager materialized namespace :wat::list::* graduates to :wat::seq::* (the
;; two-world split: :wat::seq::* = EAGER, :wat::stream::* = LAZY single-pass).
;; :wat::list::* today is 2 defaliases (reduce/fold -> :wat::core::foldl); they
;; become :wat::seq::reduce / :wat::seq::fold.
;;
;; ONE prefix rename, boundary-aware (the arc-283.1 hardening): catches every
;; :wat::list::<name> head + accessor + type-arg, never a prefix-sibling or an
;; unrelated path. The file move wat/list.wat -> wat/seq.wat and its stdlib
;; registration path are the manual seams the codemod cannot do.
;;
;; Usage (one EDN vector of EVERY path holding the prefix on stdin):
;;   printf '["wat/list.wat" "wat-tests/core/list-fold-aliases.wat"]\n' \
;;     | cargo wat ./wat-scripts/fixes/rename-list-to-seq.wat
;;
;; Idempotent: re-running yields zero changes (the old prefix is gone).

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  ;; NB: no trailing "::" — the prefix must end on a boundary char. ":wat::list"
  ;; matches ":wat::list::reduce" (next char "::" is non-ident); ":wat::list::"
  ;; would land the boundary on "r" (ident) and never match. "list" vs a sibling
  ;; "listicle" is still protected (the right-boundary check rejects it).
  (:wat::fix::rename-keyword-prefix ":wat::list" ":wat::seq" src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[renamed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
