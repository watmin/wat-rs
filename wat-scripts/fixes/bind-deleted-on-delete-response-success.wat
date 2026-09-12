;; wat-scripts/fixes/bind-deleted-on-delete-response-success.wat — the-store-says-what-it-deleted.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; `Store::DeleteResponse::Success` gained `deleted <- i64`. Every MATCH ARM whose pattern is
;; the nullary `(…::DeleteResponse::Success)` must bind the field:
;;
;;   (…::DeleteResponse::Success)  →  (…::DeleteResponse::Success _)
;;
;; THE RULE is structural, not a file allow-list. `(…::Success)` is the same token in a
;; construction and in an arm-head, and probe-entry-three-of-ten.wat contains both.
;; Rewrite ONLY a list whose sole child is that keyword and which is itself the FIRST
;; child of a match-arm list. Never in value position. Idempotent: a head that already
;; has a binder is untouched. Constructions (mem.wat, sqlite-store.wat, the probe's
;; hand-built reply) are logic and are not this file's job.
;;
;; Comment/format faithful (span edits via fix-text-apply).
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>  -> :user::grep  (prints every Match, unapplied)
;;   `wat` <this file>         -> :user::main  (rewrites files in place)
;;
;; Usage — finder:
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/bind-deleted-on-delete-response-success.wat
;;
;; Usage — dry-run:
;;   cp <file> /tmp/pilot.wat && printf '["/tmp/pilot.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/bind-deleted-on-delete-response-success.wat
;;   diff <file> /tmp/pilot.wat
;;
;; Usage — apply (list EVERY path across wat/, wat-scripts/, tests/, docs/):
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/bind-deleted-on-delete-response-success.wat

;; ── finder (rete) ────────────────────────────────────────────────────────────

(:wat::core::defrecord :ds::MatchList [id <- :wat::core::i64])
(:wat::core::defrecord :ds::HasBinder [id <- :wat::core::i64])

(:wat::rete::defrule :ds::match-list
  :when [(:wat::grep::Node  (?h <- :id) (?m <- :parent) (?i <- :index) (?kind <- :kind))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::match"))]
  :then [(:ds::MatchList ?m)])

;; pattern list already has a binder (child at index 1) — leave it.
(:wat::rete::defrule :ds::has-binder
  :when [(:wat::grep::Node (?c <- :id) (?p <- :parent) (?i <- :index))
         (:wat::rete::where (:wat::rete::i64::= ?i 1))]
  :then [(:ds::HasBinder ?p)])

;; THE REPORT — match arm whose pattern is a list whose head keyword contains
;; DeleteResponse::Success and which has no binder yet.
(:wat::rete::defrule :ds::nullary-arm
  :when [(:ds::MatchList (?m <- :id))
         (:wat::grep::Node (?arm <- :id) (?m <- :parent) (?ai <- :index))
         (:wat::rete::where (:wat::rete::i64::>= ?ai 2))
         (:wat::grep::Node (?pat <- :id) (?arm <- :parent) (?pi <- :index) (?pk <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?pi 0))
         (:wat::rete::where (:wat::rete::string::= ?pk "list"))
         (:wat::grep::Node  (?hk <- :id) (?pat <- :parent) (?hi <- :index) (?hknd <- :kind))
         (:wat::grep::Named (?hk <- :id) (?hn <- :name))
         (:wat::rete::where (:wat::rete::i64::= ?hi 0))
         (:wat::rete::where (:wat::rete::string::= ?hknd "keyword"))
         (:wat::rete::where (:wat::rete::string::contains? ?hn "DeleteResponse::Success"))
         (:wat::rete::not (:ds::HasBinder (?pat <- :id)))
         (:wat::grep::Span (?pat <- :id) (?ln <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?ln :col ?c :end-line ?el :end-col ?ec
           :rule "bind-deleted-on-delete-response-success"
           :captures (:wat::rete::core::PersistentVector
                       (:wat::grep::Capture :name "head" :value ?hn)))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :ds))

;; ── applier ──────────────────────────────────────────────────────────────────

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :user::head-kw-name [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch) "" (:user::kw-name (:wat::core::first ch))))
    ""))

(:wat::core::defn :user::success-nullary-pat?
  [pat <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "list")
    (:wat::core::let [ch (:wat::core::ast->children pat)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 1)
        (:wat::string::contains? (:user::kw-name (:wat::core::first ch)) "DeleteResponse::Success")
        false))
    false))

(:wat::core::defn :user::arm-pattern-edits
  [arm <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
    (:wat::core::let [ch (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::empty? ch)
        (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
        (:wat::core::if (:user::success-nullary-pat? (:wat::core::first ch))
          (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
            (:wat::core::Tuple
              (:user::end-off (:wat::core::first (:wat::core::ast->children (:wat::core::first ch))) lines)
              ""
              " _"))
          (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))))
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])))

(:wat::core::defn :user::match-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let [ch (:wat::core::ast->children node)]
    (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::foldl
        (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                         arm <- :wat::WatAST]
          -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
          (:wat::core::concat acc (:user::arm-pattern-edits arm lines)))
        (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
        (:wat::core::into [] (:wat::core::drop ch 2))))))

(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let
      [own (:wat::core::if (:wat::core::= (:user::head-kw-name node) ":wat::core::match")
              (:user::match-edits node lines)
              (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))]
      (:wat::core::concat own (:user::seq-edits (:wat::core::ast->children node) lines)))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::seq-edits (:wat::core::ast->children node) lines)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
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
        (:wat::kernel::println (:wat::string::concat "[bind-deleted] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
