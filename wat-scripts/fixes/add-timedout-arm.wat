;; wat-scripts/fixes/add-timedout-arm.wat — arc 278, no client call can hang.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; `:wat::kernel::RecvOutcome` gains a nullary `TimedOut` variant. Every EXHAUSTIVE match over
;; the enum therefore needs one more arm. The checker names each miss:
;;
;;   non-exhaustive: enum :wat::kernel::RecvOutcome missing arm(s) for variant(s): TimedOut
;;
;; THE RULE (DESIGN-no-client-call-can-hang.md): for every `match` whose arm set includes a
;; `RecvOutcome::` pattern and which has NO catch-all `_` arm, insert
;;
;;   (:wat::kernel::RecvOutcome::TimedOut <mirror of the Lost arm>)
;;
;; immediately after the last existing arm. Mirror:
;;   - Lost body headed by `assertion-failed!`            → timeout-specific assertion
;;   - Lost body that IS a RecvOutcome::Lost pass-through → RecvOutcome::TimedOut
;;   - Lost body that mentions the Lost binder            → timeout-specific assertion
;;     (TimedOut is nullary; the binder has no home)
;;   - otherwise                                          → the Lost body, verbatim
;;   - no Lost arm at all                                 → timeout-specific assertion
;;
;; A match with a `_` catch-all needs nothing. A match that already carries a TimedOut
;; arm is left byte-untouched (idempotent).
;;
;; ⛔ WHY THIS CANNOT BE A GREP. A `/`-headed call (`:queue::Queue/receive`) is
;; syntactically a keyword containing `/`, same as a record accessor. The discriminator
;; is the ARM SET of a match form (RecvOutcome:: present, `_` absent, TimedOut absent),
;; which a regex cannot see. Copy of phantom-none-call-census.wat's form-context shape.
;;
;; TWO ENTRY POINTS, one rule set:
;;   `wat --grep` <this file>  -> :user::grep  (prints every Match, unapplied)
;;   `wat` <this file>         -> :user::main  (rewrites files in place)
;;
;; Usage — finder:
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat --grep ./wat-scripts/fixes/add-timedout-arm.wat
;;
;; Usage — dry-run:
;;   cp <file> /tmp/pilot.wat && printf '["/tmp/pilot.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/add-timedout-arm.wat
;;   diff <file> /tmp/pilot.wat
;;
;; Usage — apply (list EVERY path):
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/add-timedout-arm.wat

;; ── finder (rete) ────────────────────────────────────────────────────────────

(:wat::core::defrecord :to::MatchList    [id <- :wat::core::i64])
(:wat::core::defrecord :to::RecvArm      [id <- :wat::core::i64])
(:wat::core::defrecord :to::CatchAll     [id <- :wat::core::i64])
(:wat::core::defrecord :to::HasTimedOut  [id <- :wat::core::i64])

;; a list whose head (index 0) is the keyword :wat::core::match
(:wat::rete::defrule :to::match-list
  :when [(:wat::grep::Node  (?h <- :id) (?m <- :parent) (?i <- :index) (?kind <- :kind))
         (:wat::grep::Named (?h <- :id) (?n <- :name))
         (:wat::rete::where (:wat::rete::string::= ?kind "keyword"))
         (:wat::rete::where (:wat::rete::i64::= ?i 0))
         (:wat::rete::where (:wat::rete::string::= ?n ":wat::core::match"))]
  :then [(:to::MatchList ?m)])

;; tagged arm: ( (RecvOutcome::Message …) body ) — pattern is a list whose head keyword
;; contains RecvOutcome::
(:wat::rete::defrule :to::recv-arm-tagged
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
         (:wat::rete::where (:wat::rete::string::contains? ?hn "RecvOutcome::"))]
  :then [(:to::RecvArm ?m)])

;; unit arm: ( RecvOutcome::Closed body ) — pattern is the keyword itself
(:wat::rete::defrule :to::recv-arm-unit
  :when [(:to::MatchList (?m <- :id))
         (:wat::grep::Node (?arm <- :id) (?m <- :parent) (?ai <- :index))
         (:wat::rete::where (:wat::rete::i64::>= ?ai 2))
         (:wat::grep::Node  (?pat <- :id) (?arm <- :parent) (?pi <- :index) (?pk <- :kind))
         (:wat::grep::Named (?pat <- :id) (?pn <- :name))
         (:wat::rete::where (:wat::rete::i64::= ?pi 0))
         (:wat::rete::where (:wat::rete::string::= ?pk "keyword"))
         (:wat::rete::where (:wat::rete::string::contains? ?pn "RecvOutcome::"))]
  :then [(:to::RecvArm ?m)])

;; catch-all `_` arm
(:wat::rete::defrule :to::catch-all
  :when [(:to::MatchList (?m <- :id))
         (:wat::grep::Node (?arm <- :id) (?m <- :parent) (?ai <- :index))
         (:wat::rete::where (:wat::rete::i64::>= ?ai 2))
         (:wat::grep::Node  (?pat <- :id) (?arm <- :parent) (?pi <- :index) (?pk <- :kind))
         (:wat::grep::Named (?pat <- :id) (?pn <- :name))
         (:wat::rete::where (:wat::rete::i64::= ?pi 0))
         (:wat::rete::where (:wat::rete::string::= ?pk "symbol"))
         (:wat::rete::where (:wat::rete::string::= ?pn "_"))]
  :then [(:to::CatchAll ?m)])

;; already carries TimedOut — tagged spelling ((TimedOut) body)
(:wat::rete::defrule :to::has-timedout-tagged
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
         (:wat::rete::where (:wat::rete::string::= ?hn ":wat::kernel::RecvOutcome::TimedOut"))]
  :then [(:to::HasTimedOut ?m)])

;; already carries TimedOut — unit spelling (TimedOut body)
(:wat::rete::defrule :to::has-timedout-unit
  :when [(:to::MatchList (?m <- :id))
         (:wat::grep::Node (?arm <- :id) (?m <- :parent) (?ai <- :index))
         (:wat::rete::where (:wat::rete::i64::>= ?ai 2))
         (:wat::grep::Node  (?pat <- :id) (?arm <- :parent) (?pi <- :index) (?pk <- :kind))
         (:wat::grep::Named (?pat <- :id) (?pn <- :name))
         (:wat::rete::where (:wat::rete::i64::= ?pi 0))
         (:wat::rete::where (:wat::rete::string::= ?pk "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?pn ":wat::kernel::RecvOutcome::TimedOut"))]
  :then [(:to::HasTimedOut ?m)])

;; THE REPORT — RecvOutcome match, no catch-all, no TimedOut yet.
(:wat::rete::defrule :to::needs-arm
  :when [(:to::RecvArm (?m <- :id))
         (:wat::rete::not (:to::CatchAll (?m <- :id)))
         (:wat::rete::not (:to::HasTimedOut (?m <- :id)))
         (:wat::grep::Span (?m <- :id) (?ln <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?ln :col ?c :end-line ?el :end-col ?ec
           :rule "add-timedout-arm"
           :captures (:wat::rete::core::PersistentVector))])

;; CONTROL — RecvOutcome match with a catch-all; the applier leaves these byte-untouched.
(:wat::rete::defrule :to::catchall-control
  :when [(:to::RecvArm (?m <- :id))
         (:to::CatchAll (?m <- :id))
         (:wat::grep::Span (?m <- :id) (?ln <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?ln :col ?c :end-line ?el :end-col ?ec
           :rule "catchall-untouched"
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

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

;; pattern-head-name — keyword name of an arm's pattern, whether tagged (list) or unit (keyword).
(:wat::core::defn :user::pattern-head-name [pat <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "keyword")
    (:wat::core::ast-name pat)
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "list")
      (:wat::core::let [ch (:wat::core::ast->children pat)]
        (:wat::core::if (:wat::core::empty? ch) "" (:user::kw-name (:wat::core::first ch))))
      "")))

(:wat::core::defn :user::is-catchall-pat? [pat <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "symbol")
    (:wat::core::= (:wat::core::ast-name pat) "_")
    false))

(:wat::core::defn :user::recv-head? [name <- :wat::core::String] -> :wat::core::bool
  (:wat::string::contains? name "RecvOutcome::"))

(:wat::core::defn :user::timedout-head? [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::= name ":wat::kernel::RecvOutcome::TimedOut"))

(:wat::core::defn :user::lost-head? [name <- :wat::core::String] -> :wat::core::bool
  (:wat::string::contains? name "RecvOutcome::Lost"))

;; pattern-binder-name — the symbol bound by a tagged pattern `(Head binder …)`, else "".
(:wat::core::defn :user::pattern-binder-name [pat <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "list")
    (:wat::core::let [ch (:wat::core::ast->children pat)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
        ""
        (:wat::core::let [b (:wat::core::nth ch 1)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind b) "symbol")
            (:wat::core::ast-name b)
            ""))))
    ""))

(:wat::core::defn :user::mentions-symbol?
  [node <- :wat::WatAST  name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= name "")
    false
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "symbol")
      (:wat::core::= (:wat::core::ast-name node) name)
      (:wat::core::if (:wat::fix::structural? node)
        (:wat::core::foldl
          (:wat::core::fn [acc <- :wat::core::bool  c <- :wat::WatAST] -> :wat::core::bool
            (:wat::core::if acc true (:user::mentions-symbol? c name)))
          false
          (:wat::core::ast->children node))
        false))))

(:wat::core::defn :user::assertion-headed? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::kernel::assertion-failed!")))
    false))

;; Lost pass-through: body is the keyword RecvOutcome::Lost, or a list headed by it.
(:wat::core::defn :user::lost-passthrough? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
    (:user::lost-head? (:wat::core::ast-name node))
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
      (:wat::core::let [ch (:wat::core::ast->children node)]
        (:wat::core::if (:wat::core::empty? ch)
          false
          (:user::lost-head? (:user::kw-name (:wat::core::first ch)))))
      false)))

(:wat::core::def :user::timeout-assertion
  "(:wat::kernel::assertion-failed! \"recv: timed out — the peer is alive and silent\" :wat::core::None :wat::core::None)")

(:wat::core::defn :user::timedout-body-text [lost <- (:wat::core::Option :- [:wat::WatAST])] -> :wat::core::String
  (:wat::core::match lost
    (:wat::core::None :user::timeout-assertion)
    ((:wat::core::Some arm)
      (:wat::core::let [ach (:wat::core::ast->children arm)]
        (:wat::core::if (:wat::core::< (:wat::core::length ach) 2)
          :user::timeout-assertion
          (:wat::core::let
            [pat  (:wat::core::first ach)
             body (:wat::core::nth ach 1)
             bn   (:user::pattern-binder-name pat)]
            (:wat::core::if (:user::assertion-headed? body)
              :user::timeout-assertion
              (:wat::core::if (:user::lost-passthrough? body)
                ":wat::kernel::RecvOutcome::TimedOut"
                (:wat::core::if (:user::mentions-symbol? body bn)
                  :user::timeout-assertion
                  (:wat::core::ast->source body))))))))))

(:wat::core::defn :user::find-lost
  [arms <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Option :- [:wat::WatAST])  arm <- :wat::WatAST]
      -> (:wat::core::Option :- [:wat::WatAST])
      (:wat::core::match acc
        ((:wat::core::Some v) (:wat::core::Some v))
        (:wat::core::None
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
            (:wat::core::let [ch (:wat::core::ast->children arm)]
              (:wat::core::if (:wat::core::empty? ch)
                :wat::core::None
                (:wat::core::if (:user::lost-head? (:user::pattern-head-name (:wat::core::first ch)))
                  (:wat::core::Some arm)
                  :wat::core::None)))
            :wat::core::None))))
    :wat::core::None
    arms))

(:wat::core::defn :user::arm-pat [arm <- :wat::WatAST] -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
    (:wat::core::let [ch (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::empty? ch) :wat::core::None
        (:wat::core::Some (:wat::core::first ch))))
    :wat::core::None))

(:wat::core::defn :user::any-recv-arm? [arms <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  arm <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::core::match (:user::arm-pat arm)
          (:wat::core::None false)
          ((:wat::core::Some pat) (:user::recv-head? (:user::pattern-head-name pat))))))
    false arms))

(:wat::core::defn :user::any-catchall? [arms <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  arm <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::core::match (:user::arm-pat arm)
          (:wat::core::None false)
          ((:wat::core::Some pat) (:user::is-catchall-pat? pat)))))
    false arms))

(:wat::core::defn :user::any-timedout? [arms <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  arm <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::core::match (:user::arm-pat arm)
          (:wat::core::None false)
          ((:wat::core::Some pat) (:user::timedout-head? (:user::pattern-head-name pat))))))
    false arms))

(:wat::core::defn :user::recv-match? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::core::match")
          (:user::any-recv-arm? (:wat::core::into [] (:wat::core::drop ch 2)))
          false)))
    false))

(:wat::core::defn :user::needs-timedout? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:user::recv-match? node)
    (:wat::core::let
      [ch   (:wat::core::ast->children node)
       arms (:wat::core::into [] (:wat::core::drop ch 2))]
      (:wat::core::if (:user::any-catchall? arms)
        false
        (:wat::core::not (:user::any-timedout? arms))))
    false))

(:wat::core::defn :user::match-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [ch      (:wat::core::ast->children node)
     last    (:wat::core::Option/expect (:wat::core::get ch (:wat::core::- (:wat::core::length ch) 1)) "last-arm")
     arms    (:wat::core::into [] (:wat::core::drop ch 2))
     body    (:user::timedout-body-text (:user::find-lost arms))
     insert  (:wat::string::concat " (:wat::kernel::RecvOutcome::TimedOut " (:wat::string::concat body ")"))]
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
      (:wat::core::Tuple (:user::end-off last lines) "" insert))))

(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [this (:wat::core::if (:user::needs-timedout? node)
            (:user::match-edits node lines)
            (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat this (:user::seq-edits (:wat::core::ast->children node) lines))
      this)))

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

;; TimedOut is a UNIT variant (sibling of Closed/Stopped). The first apply inserted the
;; tagged-empty spelling `((TimedOut) body)` — that is CallOutcome's shape, and the checker
;; rejects it: "TimedOut is not a tagged variant". Strip the extra parens. Idempotent: the
;; tagged token is gone after one pass.
(:wat::core::defn :user::untag-timedout [src <- :wat::core::String] -> :wat::core::String
  (:wat::string::join "(:wat::kernel::RecvOutcome::TimedOut "
    (:wat::string::split src "(:wat::kernel::RecvOutcome::TimedOut ")))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [src1  (:user::untag-timedout src)
     lines (:wat::string::split src1 "\n")
     tree  (:wat::core::match (:wat::core::read-string src1)
             ((:wat::core::ReadOutcome::Forms __forms) __forms)
             ((:wat::core::ReadOutcome::Malformed __cause)
               (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     forms (:wat::core::ast->children tree)
     eds   (:user::seq-edits forms lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src1 rev)))

(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[+timedout-arm] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
