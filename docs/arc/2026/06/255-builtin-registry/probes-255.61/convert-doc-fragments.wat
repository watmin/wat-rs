;; Stone 255.61 — drive :wat::fix::fix-text on extracted doc fragments.
;; Same call to-faithful-clojure.wat makes (wat/fix.wat fix-text). One EDN
;; vector of absolute paths on stdin. Each file is one wat form, rewritten
;; in place. The mapping is the codemod's; this script does not add rules.
(:wat::core::defn :user::apply-each
  [paths <- (wat.type/Vector :- [wat.type/String])] -> wat.type/nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:wat::fix::fix-text (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[doc-fragment] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
      [:wat::kernel::ReadlnOutcome.Eof {}
        (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
