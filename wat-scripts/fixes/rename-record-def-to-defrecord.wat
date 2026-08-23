;; wat-scripts/fixes/rename-record-def-to-defrecord.wat — arc 293.2-rename: Record::def reaches final names.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; Renames the aggregate-trio record macro heads to their final canonical forms:
;;   :wat::holon::Record::def  ->  :wat::holon::defrecord   (reclaimed name; hard-cut at Stone 234.6)
;;   :wat::core::Record::def         ->  :wat::core::defrecord    (peer to :wat::core::defstruct)
;;
;; SURGICAL: only the `::def` macro head moves. The siblings are UNTOUCHED:
;;   :wat::core::Record::of         (the ctor primitive)
;;   :wat::core::Record/field-at    (the accessor)
;;   :wat::core::Record             (the holder TYPE / lattice root)
;;
;; The prefix match uses the FULL old name (`:wat::core::Record::def`, not bare `:wat::core::Record`),
;; so `rename-keyword-prefix` is boundary-aware and cannot eat the siblings or the type.
;;
;; ORDER: holon-first (:wat::holon::Record::def → :wat::holon::defrecord) because the
;; holon prefix `:wat::holon::Record::def` is a different namespace from `:wat::core::Record::def`
;; (they are disjoint), but holon-first is the safe order.
;;
;; The codemod is idempotent (re-run = 0 changes). Kept in wat-scripts/fixes/ as a recorded
;; migration alongside rename-kernel-to-spawn.wat.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/Record.wat" "wat/core.wat" ...]\n' | cargo wat ./wat-scripts/fixes/rename-record-def-to-defrecord.wat

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":wat::holon::Record::def" ":wat::holon::defrecord"
    (:wat::fix::rename-keyword-prefix ":wat::core::Record::def" ":wat::core::defrecord"
      src)))

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
