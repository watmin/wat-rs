;; wat-scripts/fixes/type-member-colon-to-slash.wat — arc 255.4: one member join.
;; SCOPE: corpus
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; Unifies Type::member → Type/member via rename-keyword-exact (whole token).
;; Prefix rename is the wrong tool: `:wat::cache::Lru::` is not a token boundary
;; inside `:wat::cache::Lru::new` (the next char is a keyword constituent).
;;
;; Names are DERIVED from the live registry (wat_intrinsic + wat_dispatch +
;; wat/cache.wat defn) plus tracked call-heads (`:rust::test::*`,
;; `:my::Counter::surface-forms`). Do NOT rename Record::def (already retired
;; to defrecord, 293.2).
;;
;; Idempotent (re-run = 0 changes): after the rewrite the old token is gone.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/cache.wat" …]\n' | ./target/release/wat ./wat-scripts/fixes/type-member-colon-to-slash.wat

(:wat::core::defn :user::apply-renames
  [pairs <- (:wat::core::Vector :- [(:wat::core::Vector :- [:wat::core::String])])
   src   <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::core::empty? pairs)
    src
    (:wat::core::let [p (:wat::core::first pairs)]
      (:user::apply-renames
        (:wat::core::rest pairs)
        (:wat::fix::rename-keyword-exact
          (:wat::core::first p)
          (:wat::core::second p)
          src)))))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:user::apply-renames
    [[":wat::core::Bytes::to-hex" ":wat::core::Bytes/to-hex"]
     [":wat::core::Bytes::from-hex" ":wat::core::Bytes/from-hex"]
     [":wat::kernel::HandlePool::new" ":wat::kernel::HandlePool/new"]
     [":wat::kernel::HandlePool::pop" ":wat::kernel::HandlePool/pop"]
     [":wat::kernel::HandlePool::finish" ":wat::kernel::HandlePool/finish"]
     [":wat::cache::Lru::new" ":wat::cache::Lru/new"]
     [":wat::cache::Lru::put" ":wat::cache::Lru/put"]
     [":wat::cache::Lru::get" ":wat::cache::Lru/get"]
     [":wat::cache::Lru::len" ":wat::cache::Lru/len"]
     [":wat::cache::HolographicLru::new" ":wat::cache::HolographicLru/new"]
     [":wat::cache::HolographicLru::put" ":wat::cache::HolographicLru/put"]
     [":wat::cache::HolographicLru::get" ":wat::cache::HolographicLru/get"]
     [":wat::cache::HolographicLru::len" ":wat::cache::HolographicLru/len"]
     [":rust::cache::Lru::new" ":rust::cache::Lru/new"]
     [":rust::cache::Lru::put" ":rust::cache::Lru/put"]
     [":rust::cache::Lru::get" ":rust::cache::Lru/get"]
     [":rust::cache::Lru::len" ":rust::cache::Lru/len"]
     [":rust::cache::Lru::is_empty" ":rust::cache::Lru/is_empty"]
     [":rust::sqlite::Connection::open" ":rust::sqlite::Connection/open"]
     [":rust::sqlite::Connection::execute_ddl" ":rust::sqlite::Connection/execute_ddl"]
     [":rust::sqlite::Connection::execute" ":rust::sqlite::Connection/execute"]
     [":rust::sqlite::Connection::select" ":rust::sqlite::Connection/select"]
     [":rust::sqlite::Connection::pragma" ":rust::sqlite::Connection/pragma"]
     [":rust::sqlite::Connection::begin" ":rust::sqlite::Connection/begin"]
     [":rust::sqlite::Connection::commit" ":rust::sqlite::Connection/commit"]
     [":rust::sqlite::ReadConnection::open_readonly" ":rust::sqlite::ReadConnection/open_readonly"]
     [":rust::sqlite::ReadConnection::select" ":rust::sqlite::ReadConnection/select"]
     [":rust::test::Counter::new" ":rust::test::Counter/new"]
     [":rust::test::Counter::increment" ":rust::test::Counter/increment"]
     [":rust::test::Counter::read" ":rust::test::Counter/read"]
     [":rust::test::Ticket::new" ":rust::test::Ticket/new"]
     [":rust::test::Ticket::redeem" ":rust::test::Ticket/redeem"]
     [":rust::test::Greeting::new" ":rust::test::Greeting/new"]
     [":rust::test::Greeting::message" ":rust::test::Greeting/message"]
     [":rust::test::Greeting::year" ":rust::test::Greeting/year"]
     [":rust::test::Fallible::non_negative" ":rust::test::Fallible/non_negative"]
     [":rust::test::MathUtils::add" ":rust::test::MathUtils/add"]
     [":rust::test::MathUtils::maybe_double" ":rust::test::MathUtils/maybe_double"]
     [":rust::test::TupleUtils::sum2" ":rust::test::TupleUtils/sum2"]
     [":rust::test::TupleUtils::pair_of" ":rust::test::TupleUtils/pair_of"]
     [":rust::test::TupleUtils::describe" ":rust::test::TupleUtils/describe"]
     [":rust::test::VecUtils::sum" ":rust::test::VecUtils/sum"]
     [":rust::test::VecUtils::reverse" ":rust::test::VecUtils/reverse"]
     [":rust::test::VecUtils::sort" ":rust::test::VecUtils/sort"]
     [":my::Counter::surface-forms" ":my::Counter/surface-forms"]
     [":wat-tests::holon::Reject::bundle-or-fail" ":wat-tests::holon::Reject/bundle-or-fail"]
     [":wat-tests::holon::Reject::project-bundle-or-fail" ":wat-tests::holon::Reject/project-bundle-or-fail"]]
    src))

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
    (:wat::core::match (:wat::kernel::readln )
      [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
      [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
