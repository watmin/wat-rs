;; wat-scripts/fixes/redial-failed-names-the-variant.wat — a-dial-failure-says-which-of-three.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files.
;;
;; THE SHAPE (uniform in the three service scripts):
;;
;;   (:wat::core::match (:wat::kernel::connect ADDR)
;;     ((:wat::kernel::ConnectOutcome::Connected p) p)
;;     (_ (:wat::kernel::assertion-failed!
;;          "SITE failed — peer is dead, not a broken pipe" :None :None)))
;;
;; becomes one line (require-stopped's shape — whole outcome in, peer out):
;;
;;   (:wat::service::redial-failed! "SITE" (:wat::kernel::connect ADDR))
;;
;; SITE is the old string with the suffix " failed — peer is dead, not a broken pipe"
;; stripped, so the label survives. The helper names Refused / Rejected / Failed and
;; carries (Failure/message c). It still RAISES. The 18 sites that already name the
;; three variants are left byte-untouched (they have no `_` catch-all).
;;
;; ⛔ The census is the authority, not the brief's "35". A `_` over ConnectOutcome
;; whose body is NOT that dead-peer raise is reported, not rewritten (STOP-2).
;;
;; IDEMPOTENT: a rewritten call is not a match, so a second run emits 0 edits.
;;
;; TWO ENTRY POINTS:
;;   wat --grep <this file>  -> :user::grep
;;   wat <this file>         -> :user::main  (rewrites files in place; prints each hit)
;;
;; Usage — finder:
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/redial-failed-names-the-variant.wat
;;
;; Usage — dry-run:
;;   cp <file> /tmp/pilot.wat && printf '["/tmp/pilot.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/redial-failed-names-the-variant.wat
;;   diff <file> /tmp/pilot.wat
;;
;; Usage — apply (list EVERY path):
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/redial-failed-names-the-variant.wat

(:wat::core::def :user::dead-peer-suffix
  " failed — peer is dead, not a broken pipe")

;; ── grep (rete) ──────────────────────────────────────────────────────────────

(:wat::core::defrecord :to::MatchList [id <- :wat::core::i64])
(:wat::core::defrecord :to::ConnectedArm [id <- :wat::core::i64])
(:wat::core::defrecord :to::DeadPeerCatch [id <- :wat::core::i64])

(:wat::rete::defrule :to::match-list
  :when [(:wat::grep::Node  (?h <- :id) (?m <- :parent) (?i <- :index) (?kind <- :kind))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::match"))]
  :then [(:to::MatchList ?m)])

(:wat::rete::defrule :to::connected-arm
  :when [(:to::MatchList (?m <- :id))
         (:wat::grep::Node (?arm <- :id) (?m <- :parent) (?ai <- :index))
         (:wat::rete::where (:wat::rete::i64::>= ?ai 2))
         (:wat::grep::Node (?pat <- :id) (?arm <- :parent) (?pi <- :index) (?pk <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?pi 0))
         (:wat::rete::where (:wat::rete::string::= ?pk "list"))
         (:wat::grep::Node  (?hk <- :id) (?pat <- :parent) (?hi <- :index) (?hknd <- :kind))
         (:wat::grep::Named (?hk <- :id) (?hn <- :name))
         (:wat::rete::where (:wat::rete::i64::= ?hi 0))
         (:wat::rete::where (:wat::rete::string::= ?hknd "keyword"))
         (:wat::rete::where (:wat::rete::string::contains? ?hn "ConnectOutcome::Connected"))]
  :then [(:to::ConnectedArm ?m)])

;; `_` arm whose assertion-failed! string is the dead-peer lie.
(:wat::rete::defrule :to::dead-peer-catch
  :when [(:to::MatchList (?m <- :id))
         (:wat::grep::Node (?arm <- :id) (?m <- :parent) (?ai <- :index))
         (:wat::rete::where (:wat::rete::i64::>= ?ai 2))
         (:wat::grep::Node  (?pat <- :id) (?arm <- :parent) (?pi <- :index) (?pk <- :kind))
         (:wat::grep::Named (?pat <- :id) (?pn <- :name))
         (:wat::rete::where (:wat::rete::i64::= ?pi 0))
         (:wat::rete::where (:wat::rete::string::= ?pk "symbol"))
         (:wat::rete::where (:wat::rete::string::= ?pn "_"))
         (:wat::grep::Node (?body <- :id) (?arm <- :parent) (?bi <- :index) (?bk <- :kind))
         (:wat::rete::where (:wat::rete::i64::= ?bi 1))
         (:wat::rete::where (:wat::rete::string::= ?bk "list"))
         (:wat::grep::Node  (?hd <- :id) (?body <- :parent) (?hi <- :index))
         (:wat::grep::Named (?hd <- :id) (?hn <- :name))
         (:wat::rete::where (:wat::rete::i64::= ?hi 0))
         (:wat::rete::where (:wat::rete::string::= ?hn ":wat::kernel::assertion-failed!"))
         (:wat::grep::Node  (?st <- :id) (?body <- :parent) (?si <- :index) (?sk <- :kind))
         (:wat::grep::Named (?st <- :id) (?sn <- :name))
         (:wat::rete::where (:wat::rete::i64::= ?si 1))
         (:wat::rete::where (:wat::rete::string::= ?sk "string"))
         (:wat::rete::where (:wat::rete::string::contains? ?sn "peer is dead, not a broken pipe"))]
  :then [(:to::DeadPeerCatch ?m)])

(:wat::rete::defrule :to::collapsing
  :when [(:to::ConnectedArm (?m <- :id))
         (:to::DeadPeerCatch (?m <- :id))
         (:wat::grep::Span (?m <- :id) (?ln <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?ln :col ?c :end-line ?el :end-col ?ec
           :rule "collapsing-connect-catchall"
           :captures (:wat::rete::core::PersistentVector))])

(:wat::core::defn :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::rete::collect-rules :to))

;; ── applier ──────────────────────────────────────────────────────────────────

(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::node-text
  [n <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::string::subs src (:user::start-off n lines) (:user::end-off n lines)))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :user::head-kw [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch) "" (:user::kw-name (:wat::core::first ch))))
    ""))

(:wat::core::defn :user::sym-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "symbol")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :user::site-of [msg <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::string::ends-with? msg :user::dead-peer-suffix)
    (:wat::string::subs msg 0
      (:wat::i64::- (:wat::string::length msg) (:wat::string::length :user::dead-peer-suffix)))
    msg))

;; Connected arm: ((ConnectOutcome::Connected binder) binder)
(:wat::core::defn :user::identity-connected-arm? [arm <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
    (:wat::core::let [ch (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 2)
        (:wat::core::let
          [pat  (:wat::core::first ch)
           body (:wat::core::nth ch 1)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "list")
            (:wat::core::let [pch (:wat::core::ast->children pat)]
              (:wat::core::if (:wat::core::= (:wat::core::length pch) 2)
                (:wat::core::if (:wat::string::contains? (:user::kw-name (:wat::core::first pch))
                                  "ConnectOutcome::Connected")
                  (:wat::core::= (:user::sym-name (:wat::core::nth pch 1))
                    (:user::sym-name body))
                  false)
                false))
            false))
        false))
    false))

;; `_` arm whose body is assertion-failed! with the dead-peer string. Returns the string or "".
(:wat::core::defn :user::dead-peer-catch-msg [arm <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
    (:wat::core::let [ch (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
        ""
        (:wat::core::if (:wat::core::= (:user::sym-name (:wat::core::first ch)) "_")
          (:wat::core::let [body (:wat::core::nth ch 1)]
            (:wat::core::if (:wat::core::= (:user::head-kw body) ":wat::kernel::assertion-failed!")
              (:wat::core::let [bch (:wat::core::ast->children body)]
                (:wat::core::if (:wat::core::< (:wat::core::length bch) 2)
                  ""
                  (:wat::core::let [arg (:wat::core::nth bch 1)]
                    (:wat::core::if (:wat::core::= (:wat::core::ast-kind arg) "string")
                      (:wat::core::let [s (:wat::core::ast-name arg)]
                        (:wat::core::if (:wat::string::contains? s "peer is dead, not a broken pipe")
                          s
                          ""))
                      ""))))
              ""))
          "")))
    ""))

;; A two-arm match: identity-Connected + dead-peer catch-all. The uniform collapsing site.
(:wat::core::defn :user::collapsing-match? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:user::head-kw node) ":wat::core::match")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 4)
        (:wat::core::let
          [a0 (:wat::core::nth ch 2)
           a1 (:wat::core::nth ch 3)]
          (:wat::core::if (:user::identity-connected-arm? a0)
            (:wat::core::not (:wat::core::= (:user::dead-peer-catch-msg a1) ""))
            (:wat::core::if (:user::identity-connected-arm? a1)
              (:wat::core::not (:wat::core::= (:user::dead-peer-catch-msg a0) ""))
              false)))
        false))
    false))

(:wat::core::defn :user::collapsing-site [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let [ch (:wat::core::ast->children node)]
    (:user::site-of
      (:wat::core::let
        [m0 (:user::dead-peer-catch-msg (:wat::core::nth ch 2))
         m1 (:user::dead-peer-catch-msg (:wat::core::nth ch 3))]
        (:wat::core::if (:wat::core::= m0 "") m1 m0)))))

(:wat::core::defn :user::replace-text
  [node <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::core::let
    [ch    (:wat::core::ast->children node)
     scrut (:wat::core::nth ch 1)
     site  (:user::collapsing-site node)]
    (:wat::string::concat
      "(:wat::service::redial-failed! \""
      (:wat::string::concat site
        (:wat::string::concat "\" "
          (:wat::string::concat (:user::node-text scrut src lines) ")"))))))

(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::collapsing-match? node)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
      (:wat::core::Tuple
        (:user::start-off node lines)
        (:user::node-text node src lines)
        (:user::replace-text node src lines)))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::seq-edits (:wat::core::ast->children node) src lines)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   src <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                    it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it src lines)))
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    items))

(:wat::core::defn :user::print-edits
  [path <- :wat::core::String
   eds <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
   i <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::if (:wat::i64::>= i (:wat::core::length eds))
    nil
    (:wat::core::do
      (:wat::kernel::println
        (:wat::core::format "CENSUS {p} off={o} -> {n}"
          :p path
          :o (:wat::i64::to-string (:wat::core::first (:wat::core::nth eds i)))
          :n (:wat::core::third (:wat::core::nth eds i))))
      (:user::print-edits path eds (:wat::i64::+ i 1)))))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     forms (:wat::core::ast->children
             (:wat::core::match (:wat::core::read-string src)
               ((:wat::core::ReadOutcome::Forms __forms) __forms)
               ((:wat::core::ReadOutcome::Malformed __cause)
                 (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause)
                   :wat::core::None :wat::core::None))))
     eds (:user::seq-edits forms src lines)
     rev (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let
      [path (:wat::core::first paths)
       src  (:wat::io::read-file path)
       lines (:wat::string::split src "\n")
       forms (:wat::core::ast->children
               (:wat::core::match (:wat::core::read-string src)
                 ((:wat::core::ReadOutcome::Forms __forms) __forms)
                 ((:wat::core::ReadOutcome::Malformed __cause)
                   (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause)
                     :wat::core::None :wat::core::None))))
       eds (:user::seq-edits forms src lines)
       _ (:user::print-edits path eds 0)
       _ (:wat::io::write-file path (:wat::fix::fix-text-apply src (:wat::core::reverse (:wat::core::sort eds))))
       _ (:wat::kernel::println
            (:wat::core::format "[redial-failed] {p} n={n}"
              :p path :n (:wat::i64::to-string (:wat::core::length eds))))]
      (:user::apply-each (:wat::core::rest paths)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
