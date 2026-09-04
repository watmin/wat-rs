;; wat-scripts/fixes/wait-ns-to-wait.wat — Stone B: the wait names its verb.
;;
;; Self-hosted, comment-faithful fix-wat codemod. The queue's ReceiveRequest
;; carried `wait-ns <- i64` where 0 meant "do not wait, sweep" and a positive
;; meant "park up to this long". One field, two verbs. This rewrite is the
;; value-dependent flip at every construction kwarg:
;;
;;   :wait-ns 0          -> :wait (:queue::Queue::Wait::Immediate)
;;   :wait-ns 250000000  -> :wait (:queue::Queue::Wait::UpTo (:wat::time::Millisecond 250))
;;   :wait-ns 50000000   -> :wait (:queue::Queue::Wait::UpTo (:wat::time::Millisecond 50))
;;
;; A non-integer sibling (a parameter) is LEFT for a hand-typed helper — those
;; helpers take a Queue::Wait now, and collapsing them is Stone D.
;; An integer that is not one of the three known literals is a STOP-2: the
;; flip cannot be guessed.
;;
;; Finder (`wat --grep`): every KEYWORD leaf named `:wait-ns`. Comments and
;; string literals are not keywords. Count occurrences, not lines.
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>  -> :user::grep  (prints every Match, unapplied)
;;   `wat` <this file>         -> :user::main  (rewrites files in place)
;;
;; Usage — finder:
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/wait-ns-to-wait.wat
;;
;; Usage — dry-run (copy, then apply, then diff):
;;   cp <file> /tmp/pilot.wat && printf '["/tmp/pilot.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/wait-ns-to-wait.wat
;;   diff <file> /tmp/pilot.wat
;;
;; Usage — apply (list EVERY path the finder named):
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/wait-ns-to-wait.wat
;;
;; Idempotent: after a literal site is rewritten it is no longer a `:wait-ns`
;; keyword, so a re-run emits 0 edits. Parameter sites stay `:wait-ns` until
;; the hand-typed helper pass.

;; ── finder ───────────────────────────────────────────────────────────────────

(:wat::rete::defrule :wn::wait-ns-kw
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wait-ns"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "wait-ns-to-wait"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :wn))

;; ── applier: AST walk, value-dependent flip ──────────────────────────────────

(:wat::core::defn :wn::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :wn::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :wn::node-text
  [n <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::string::subs src (:wn::start-off n lines) (:wn::end-off n lines)))

(:wat::core::defn :wn::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :wn::empty-edits []
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))

(:wat::core::defn :wn::replacement-for [lit <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= lit "0")
    ":wait (:queue::Queue::Wait::Immediate)"
    (:wat::core::if (:wat::core::= lit "250000000")
      ":wait (:queue::Queue::Wait::UpTo (:wat::time::Milliseconds 250))"
      (:wat::core::if (:wat::core::= lit "50000000")
        ":wait (:queue::Queue::Wait::UpTo (:wat::time::Milliseconds 50))"
        ""))))

(:wat::core::defn :wn::pair-edit
  [kw <- :wat::WatAST  val <- :wat::WatAST
   src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind val) "int")
    (:wat::core::let
      [lit (:wn::node-text val src lines)
       neu (:wn::replacement-for lit)]
      (:wat::core::if (:wat::core::= neu "")
        (:wat::kernel::assertion-failed!
          (:wat::core::format "wait-ns-to-wait: unknown integer literal {n} — STOP-2, cannot guess the flip"
            :n lit)
          :wat::core::None :wat::core::None)
        (:wat::core::let
          [off (:wn::start-off kw lines)
           end (:wn::end-off val lines)
           old (:wat::string::subs src off end)]
          (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
            (:wat::core::Tuple off old neu)))))
    (:wn::empty-edits)))

(:wat::core::defn :wn::list-edits
  [ch <- (:wat::core::Vector :- [:wat::WatAST])
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let [n (:wat::core::length ch)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                       i   <- :wat::core::i64]
        -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
        (:wat::core::if (:wat::core::>= i (:wat::i64::- n 1))
          acc
          (:wat::core::let
            [cur (:wat::core::Option/expect (:wat::core::get ch i) "wn list cur")
             nxt (:wat::core::Option/expect (:wat::core::get ch (:wat::i64::+ i 1)) "wn list nxt")]
            (:wat::core::if (:wat::core::= (:wn::kw-name cur) ":wait-ns")
              (:wat::core::concat acc (:wn::pair-edit cur nxt src lines))
              acc))))
      (:wn::empty-edits)
      (:wat::core::range 0 n))))

(:wat::core::defn :wn::edits
  [node <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::concat
        (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
          (:wn::list-edits ch src lines)
          (:wn::empty-edits))
        (:wn::edits-seq ch src lines)))
    (:wn::empty-edits)))

(:wat::core::defn :wn::edits-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wn::empty-edits)
    (:wat::core::concat
      (:wn::edits (:wat::core::first items) src lines)
      (:wn::edits-seq (:wat::core::into [] (:wat::core::rest items)) src lines))))

(:wat::core::defn :wn::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             ((:wat::core::ReadOutcome::Forms __forms) __forms)
             ((:wat::core::ReadOutcome::Malformed __cause)
               (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     forms (:wat::core::ast->children tree)
     eds   (:wn::edits-seq forms src lines)
     sorted (:wat::core::sort
              (:wat::core::fn [a <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                               b <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                -> :wat::core::bool
                (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
              eds)]
    (:wat::fix::fix-text-apply src sorted)))

(:wat::core::defn :wn::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:wn::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[wait-ns-to-wait] " path))
        (:wn::apply-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wn::apply-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
