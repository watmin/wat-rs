;; wat-scripts/fixes/add-event-id-to-metric-log-ctors.wat — excursus 001 SORTKEY migration.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; Scope gained `event-id <- :wat::core::Uuid` (spliced into Metric and Log). Every kwargs
;; constructor of those two records must supply the field. THIS codemod inserts
;;
;;   :event-id (:wat::uuid::nil)
;;
;; immediately after the `:time-ns <value>` pair inside a `:wat::telemetry::Metric` or
;; `:wat::telemetry::Log` construction. Fixtures that build records by hand use nil; the
;; producer (`wat/telemetry/span.wat`) already mints `(:wat::uuid::v4)` beside `now` and
;; is left untouched because it already carries `:event-id` (idempotency).
;;
;; Idempotent: a constructor that already has an `:event-id` keyword among its children is
;; left byte-untouched. A list whose head is not exactly those two names is never edited.
;;
;; Comment/format faithful (span edits via fix-text-apply). Usage:
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/add-event-id-to-metric-log-ctors.wat

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::ctor-head?
  [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [name (:user::kw-name n)]
    (:wat::core::if (:wat::core::= name ":wat::telemetry::Metric") true
      (:wat::core::= name ":wat::telemetry::Log"))))

(:wat::core::defn :user::has-event-id?
  [ch <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  i <- :wat::core::i64] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::core::= (:user::kw-name (:wat::core::nth ch i)) ":event-id")))
    false
    (:wat::core::range 0 (:wat::core::length ch))))

;; time-ns-value — the child immediately after `:time-ns`, or None if the keyword is absent.
(:wat::core::defn :user::time-ns-value
  [ch <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Option :- [:wat::WatAST])  i <- :wat::core::i64]
      -> (:wat::core::Option :- [:wat::WatAST])
      (:wat::core::match acc
        ((:wat::core::Some v) (:wat::core::Some v))
        (:wat::core::None
          (:wat::core::if
            (:wat::core::= (:user::kw-name (:wat::core::nth ch i)) ":time-ns")
            (:wat::core::get ch (:wat::core::+ i 1))
            :wat::core::None))))
    :wat::core::None
    (:wat::core::range 0 (:wat::core::length ch))))

(:wat::core::defn :user::ctor-edit
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let [ch (:wat::core::ast->children node)]
    (:wat::core::if
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 2) true
        (:wat::core::not (:user::ctor-head? (:wat::core::first ch))))
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::if (:user::has-event-id? ch)
        (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
        (:wat::core::match (:user::time-ns-value ch)
          (:wat::core::None
            (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))
          ((:wat::core::Some val)
            (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
              (:wat::core::Tuple (:user::end-off val lines) "" " :event-id (:wat::uuid::nil)"))))))))

(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::concat
      (:user::ctor-edit node lines)
      (:user::seq-edits (:wat::core::ast->children node) lines))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::seq-edits (:wat::core::ast->children node) lines)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lines)))
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    items))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             ((:wat::core::ReadOutcome::Forms __forms) __forms)
             ((:wat::core::ReadOutcome::Malformed __cause)
               (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     forms (:wat::core::ast->children tree)
     eds   (:user::seq-edits forms lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[event-id] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
