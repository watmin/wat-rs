;; wat-scripts/fixes/declare-queue-drop-knobs.wat — arc 278, the queue can drop too.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; `:queue::queue::Record` gained three `:durable` fields (`drop-recv-bp`, `drop-ack-bp`,
;; `drop-seed`). Every kwargs constructor must supply them. THIS codemod inserts
;;
;;   :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0
;;
;; immediately after the `:store-addr <value>` pair inside a `:queue::queue::Record`
;; construction. Defaults are zero so aiming at one verb never darkens another.
;;
;; Idempotent: a constructor that already has a `:drop-recv-bp` keyword among its
;; children is left byte-untouched. A list whose head is not exactly
;; `:queue::queue::Record` is never edited.
;;
;; Comment/format faithful (span edits via fix-text-apply).
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>  -> :user::grep  (prints every Match, unapplied)
;;   `wat` <this file>         -> :user::main  (rewrites files in place)
;;
;; Usage — finder:
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/declare-queue-drop-knobs.wat
;;
;; Usage — dry-run:
;;   cp <file> /tmp/pilot.wat && printf '["/tmp/pilot.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/declare-queue-drop-knobs.wat
;;   diff <file> /tmp/pilot.wat
;;
;; Usage — apply (list EVERY path):
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/declare-queue-drop-knobs.wat

;; ── finder ───────────────────────────────────────────────────────────────────

(:wat::rete::defrule :qd::queue-record-kw
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":queue::queue::Record"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "declare-queue-drop-knobs"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :qd))

;; ── applier ──────────────────────────────────────────────────────────────────

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::ctor-head?
  [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:user::kw-name n) ":queue::queue::Record"))

(:wat::core::defn :user::has-drop-recv-bp?
  [ch <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  i <- :wat::core::i64] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::core::= (:user::kw-name (:wat::core::nth ch i)) ":drop-recv-bp")))
    false
    (:wat::core::range 0 (:wat::core::length ch))))

(:wat::core::defn :user::store-addr-value
  [ch <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Option :- [:wat::WatAST])  i <- :wat::core::i64]
      -> (:wat::core::Option :- [:wat::WatAST])
      (:wat::core::match acc
        ((:wat::core::Some v) (:wat::core::Some v))
        (:wat::core::None
          (:wat::core::if
            (:wat::core::= (:user::kw-name (:wat::core::nth ch i)) ":store-addr")
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
      (:wat::core::if (:user::has-drop-recv-bp? ch)
        (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
        (:wat::core::match (:user::store-addr-value ch)
          (:wat::core::None
            (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))
          ((:wat::core::Some val)
            (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
              (:wat::core::Tuple (:user::end-off val lines) "" " :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0"))))))))

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
        (:wat::kernel::println (:wat::string::concat "[queue-drop-knobs] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
