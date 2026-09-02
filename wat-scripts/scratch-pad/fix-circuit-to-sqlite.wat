;; fix-circuit-to-sqlite.wat — the measurement transform for THE ORDER item 4.
;; Self-hosted: wat rewriting wat. No hand-edit, no python, no sed (CLAUDE.md).
;;
;; Rewrites a copy of the fan-out circuit to run on `sqlite-store` instead of `mem-store`,
;; changing exactly ONE thing so the wall time is comparable. Both satisfy :wat::query::Store,
;; and the queue holds its store as a peer of the SURFACE (sqs.wat:104,:115), so nothing
;; downstream of the handle vector cares.
;;
;; Two rewrites:
;;   1. whole-name prefix rename  :wat::query::mem-store -> :wat::query::sqlite-store
;;      (covers ::Handle, /start, /grant, ::Handle/addr, ::Record — 19 sites)
;;   2. the Record's arguments, which differ in arity and cannot be a rename
;;
;; Usage (dry-run on a copy and diff BEFORE applying, per the fixes doctrine):
;;   printf '["wat-scripts/scratch-pad/probe-circuit-sqlite.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/scratch-pad/fix-circuit-to-sqlite.wat
;;
;; Idempotent: re-running yields zero changes (the old prefix is gone).

(:wat::core::defn :user::replace-all
  [src <- :wat::core::String  old <- :wat::core::String  new <- :wat::core::String]
  -> :wat::core::String
  (:wat::string::join new (:wat::string::split src old)))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:user::replace-all
    (:wat::fix::rename-keyword-prefix ":wat::query::mem-store" ":wat::query::sqlite-store" src)
    "(:wat::query::sqlite-store::Record :rows (:wat::core::PersistentVector))"
    "(:wat::query::sqlite-store::Record :path \":memory:\" :index-names (:wat::core::Vector :- [:wat::core::String] \"by-visible-at\"))"))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[swapped] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
