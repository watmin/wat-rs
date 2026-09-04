;; wat-scripts/fixes/alarm-after-to-delay.wat — Stone C: Alarm's field after → delay.
;;
;; Self-hosted, comment-faithful fix-wat codemod. FORM, not token: `:after` is a
;; bare keyword and some occurrences are not Alarm's (metric names, etc.).
;; The finder distinguishes them by parentage. If it cannot, that is STOP-1.
;;
;; Three Alarm forms, one rewrite:
;;   (:wat::service::Alarm :after DUR :op OP)     keyword kwarg
;;   (:wat::service::Alarm/after alarm)           accessor
;;   (defrecord :wat::service::Alarm … [after ←]) field binder (symbol)
;;
;; NOT rewritten: `:wat::kernel::after` (different full name), `:user::metric
;; :after`, any other `:after` whose list-head is not Alarm.
;;
;; TWO ENTRY POINTS:
;;   `wat --grep` <this file>  -> :user::grep
;;   `wat` <this file>         -> :user::main
;;
;; Usage — finder:
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/alarm-after-to-delay.wat
;;
;; Usage — apply (list EVERY path the Alarm-form rules named):
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/alarm-after-to-delay.wat
;;
;; Idempotent: after rewrite the keywords/symbols are no longer the old names.

;; ── finder ───────────────────────────────────────────────────────────────────

;; TOKEN census — every KEYWORD leaf named `:after`. Hypothesis, not the rewrite
;; population. Compare against alarm-ctor-after to prove the form filter works.
(:wat::rete::defrule :aa::token-after
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":after"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "token-after"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

;; FORM: a `:after` keyword whose enclosing list is headed by `:wat::service::Alarm`.
(:wat::rete::defrule :aa::alarm-ctor-after
  :when [(:wat::grep::Node   (?id <- :id) (?p <- :parent) (?i <- :index) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::grep::Node   (?hid <- :id) (?p <- :parent) (?hi <- :index) (?hk <- :kind))
         (:wat::grep::Named  (?hid <- :id) (?hn <- :name))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":after"))
         (:wat::rete::where (:wat::rete::i64::= ?hi 0))
         (:wat::rete::where (:wat::rete::string::= ?hn ":wat::service::Alarm"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "alarm-ctor-after"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

;; Accessor: unique full name, not a bare `:after`.
(:wat::rete::defrule :aa::alarm-accessor
  :when [(:wat::grep::Node   (?id <- :id) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::rete::where (:wat::rete::string::= ?k "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::service::Alarm/after"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "alarm-accessor"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

;; Defrecord field binder: SYMBOL `after` inside a vector whose enclosing list is
;; `(defrecord :wat::service::Alarm …)`.
(:wat::rete::defrule :aa::alarm-field
  :when [(:wat::grep::Node   (?id <- :id) (?vec <- :parent) (?k <- :kind))
         (:wat::grep::Named  (?id <- :id) (?n <- :name))
         (:wat::grep::Span   (?id <- :id) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))
         (:wat::grep::Node   (?vec <- :id) (?rec <- :parent) (?vk <- :kind))
         (:wat::grep::Node   (?hid <- :id) (?rec <- :parent) (?hi <- :index))
         (:wat::grep::Named  (?hid <- :id) (?hn <- :name))
         (:wat::grep::Node   (?tid <- :id) (?rec <- :parent) (?ti <- :index))
         (:wat::grep::Named  (?tid <- :id) (?tn <- :name))
         (:wat::rete::where (:wat::rete::string::= ?k "symbol"))
         (:wat::rete::where (:wat::rete::string::= ?n "after"))
         (:wat::rete::where (:wat::rete::string::= ?vk "vector"))
         (:wat::rete::where (:wat::rete::i64::= ?hi 0))
         (:wat::rete::where (:wat::rete::string::= ?hn ":wat::core::defrecord"))
         (:wat::rete::where (:wat::rete::i64::= ?ti 1))
         (:wat::rete::where (:wat::rete::string::= ?tn ":wat::service::Alarm"))]
  :then [(:wat::grep::Match :file ?f :line ?l :col ?c :end-line ?el :end-col ?ec
           :rule "alarm-field"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "old" :value ?n)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :aa))

;; ── applier ──────────────────────────────────────────────────────────────────

(:wat::core::defn :aa::empty-edits []
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))

(:wat::core::defn :aa::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :aa::sym-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "symbol")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :aa::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :aa::leaf-edit
  [n <- :wat::WatAST  old <- :wat::core::String  neu <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
    (:wat::core::Tuple (:aa::start-off n lines) old neu)))

(:wat::core::defn :aa::alarm-ctor-edits
  [ch <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     n   <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::if (:wat::core::= (:aa::kw-name n) ":after")
        (:wat::core::concat acc (:aa::leaf-edit n ":after" ":delay" lines))
        acc))
    (:aa::empty-edits)
    ch))

(:wat::core::defn :aa::vector-after-symbol-edits
  [ch <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     n   <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::if (:wat::core::= (:aa::sym-name n) "after")
        (:wat::core::concat acc (:aa::leaf-edit n "after" "delay" lines))
        acc))
    (:aa::empty-edits)
    ch))

(:wat::core::defn :aa::alarm-defrecord?
  [ch <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
    false
    (:wat::core::if (:wat::core::= (:aa::kw-name (:wat::core::first ch)) ":wat::core::defrecord")
      (:wat::core::= (:aa::kw-name (:wat::core::nth ch 1)) ":wat::service::Alarm")
      false)))

(:wat::core::defn :aa::edits
  [node <- :wat::WatAST
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::let [ch (:wat::core::ast->children node)
                      k  (:wat::core::ast-kind node)
                      here (:wat::core::if (:wat::core::= k "list")
                             (:wat::core::if (:wat::core::empty? ch)
                               (:aa::empty-edits)
                               (:wat::core::if (:wat::core::= (:aa::kw-name (:wat::core::first ch)) ":wat::service::Alarm")
                                 (:aa::alarm-ctor-edits ch lines)
                                 (:wat::core::if (:aa::alarm-defrecord? ch)
                                   (:aa::defrecord-field-edits ch lines)
                                   (:aa::empty-edits))))
                             (:aa::empty-edits))]
      (:wat::core::concat here (:aa::edits-seq ch src lines)))
    (:aa::empty-edits)))

(:wat::core::defn :aa::defrecord-field-edits
  [ch <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                     n   <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "vector")
        (:wat::core::concat acc (:aa::vector-after-symbol-edits (:wat::core::ast->children n) lines))
        acc))
    (:aa::empty-edits)
    ch))

(:wat::core::defn :aa::edits-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:aa::empty-edits)
    (:wat::core::concat
      (:aa::edits (:wat::core::first items) src lines)
      (:aa::edits-seq (:wat::core::into [] (:wat::core::rest items)) src lines))))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     tree  (:wat::core::match (:wat::core::read-string src)
             ((:wat::core::ReadOutcome::Forms __forms) __forms)
             ((:wat::core::ReadOutcome::Malformed __cause)
               (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     forms (:wat::core::ast->children tree)
     eds   (:aa::edits-seq forms src lines)
     sorted (:wat::core::sort
              (:wat::core::fn [a <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                               b <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                -> :wat::core::bool
                (:wat::core::> (:wat::core::first a) (:wat::core::first b)))
              eds)
     walked (:wat::fix::fix-text-apply src sorted)]
    (:wat::fix::rename-keyword-exact ":wat::service::Alarm/after" ":wat::service::Alarm/delay" walked)))

(:wat::core::defn :aa::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[alarm-after-to-delay] " path))
        (:aa::apply-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:aa::apply-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
