;; wat-scripts/fixes/rename-wat-record-to-core-record.wat — arc 293 holder-vocab move,
;; run over real wat source files IN WAT, through the wat CLI. The migration tool,
;; self-hosted: no Rust harness, no hand-edit of wat source (use-the-tool, not hand-fix).
;;
;; Gives the record holder-root its honest `:wat::core::` prefix — sibling of
;; `:wat::core::Struct` (minted in 293 decl-a). The old `:wat::core::Record` symbol ceases to exist:
;;   :wat::core::Record            -> :wat::core::Record
;;   :wat::core::Record::of        -> :wat::core::Record::of        (of-func ctor — shares the prefix)
;;   :wat::core::Record/field-at   -> :wat::core::Record/field-at   (accessor primitive — shares the prefix)
;;
;; ONE full-name PREFIX rename. The prefix `:wat::core::Record` IS the full name, which still catches the
;; `::of` and `/field-at` suffixes that share it — and CANNOT touch `:wat::holon::Record` (a different
;; prefix: `:wat::holon::Record` does not start with `:wat::core::Record`).
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/rete.wat" "wat/spawn.wat" ...]\n' | cargo wat ./wat-scripts/fixes/rename-wat-record-to-core-record.wat
;;
;; The rewrite is comment-faithful and idempotent (re-running yields zero changes — the old prefix
;; is gone), so it is safe to run over a clean tree.

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":wat::core::Record" ":wat::core::Record" src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[renamed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
