;; wat-scripts/fixes/bare-variant-to-qualified.wat — arc 296 N.
;; SCOPE: corpus
;;
;; Exact-token rename of the five illegal bare spellings to their qualified
;; Type::Variant FQDNs. Span-faithful (rename-keyword-exact). Idempotent:
;; after the rewrite the old token is gone, so a re-run matches nothing.
;;
;;   :wat::core::Some  ->  :wat::core::Option::Some
;;   :wat::core::None  ->  :wat::core::Option::None
;;   :wat::core::Ok    ->  :wat::core::Result::Ok
;;   :wat::core::Err   ->  :wat::core::Result::Err
;;   :None             ->  :wat::core::Option::None
;;
;; Patterns AND constructions: a keyword leaf is a keyword leaf.
;; printf '[...paths...]\n' | ./target/release/wat ./wat-scripts/fixes/bare-variant-to-qualified.wat

(:wat::core::defn :user::rename-five [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [s1 (:wat::fix::rename-keyword-exact ":wat::core::Some" ":wat::core::Option::Some" src)
     s2 (:wat::fix::rename-keyword-exact ":wat::core::None" ":wat::core::Option::None" s1)
     s3 (:wat::fix::rename-keyword-exact ":wat::core::Ok" ":wat::core::Result::Ok" s2)
     s4 (:wat::fix::rename-keyword-exact ":wat::core::Err" ":wat::core::Result::Err" s3)]
    (:wat::fix::rename-keyword-exact ":None" ":wat::core::Option::None" s4)))

(:wat::core::defn :user::rewrite-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::rename-five (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[bare-variant] " path))
        (:user::rewrite-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [paths (:wat::core::match (:wat::kernel::readln)
             [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
             [:wat::kernel::ReadlnOutcome.Eof {}
               (:wat::kernel::assertion-failed! :message "readln: end of input")]
             [:wat::kernel::ReadlnOutcome.Stopped {}
               (:wat::kernel::assertion-failed! :message "readln: stop requested")])]
    (:user::rewrite-each paths)))
