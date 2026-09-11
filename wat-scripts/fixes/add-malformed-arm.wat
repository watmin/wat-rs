;; wat-scripts/fixes/add-malformed-arm.wat — excursus 001, a momentary failure is not fatal.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; `:wat::kernel::RecvOutcome` gains a TAGGED `Malformed [cause <- :wat::kernel::Failure]`
;; variant. Every EXHAUSTIVE match over the enum therefore needs one more arm. The checker
;; names each miss:
;;
;;   non-exhaustive: enum :wat::kernel::RecvOutcome missing arm(s) for variant(s): Malformed
;;
;; ⭑ COPIED FROM `add-timedout-arm.wat`, which did this job the last time a RecvOutcome
;; variant was minted. Read that file beside this one; the finder is unchanged, and the
;; differences are all in the inserted BODY and are listed here.
;;
;; THE RULE (a-momentary-failure-is-not-fatal/DESIGN.md): for every `match` whose arm set
;; includes a `RecvOutcome::` pattern and which has NO catch-all `_` arm, insert
;;
;;   ((:wat::kernel::RecvOutcome::Malformed cause) <body per the mirror table>)
;;
;; immediately after the last existing arm.
;;
;; ⛔ DIFFERENCE 1 — TAGGED, NOT UNIT. `TimedOut` is nullary, so its arm is the bare keyword
;; and that codemod had to STRIP the tagged-empty spelling it first emitted (`untag-timedout`).
;; `Malformed` carries a cause, so the tagged spelling with a `cause` binder is the correct
;; and only one. There is no untag pass here; idempotence comes from `any-malformed?` alone.
;;
;; ⛔ DIFFERENCE 2 — NEVER MIRROR A REDIAL. `Malformed` is REPORT-FINAL: a decode failure is
;; DETERMINISTIC, so retrying transmits the same bad bytes and fails identically. `Lost` is
;; the opposite — it is retried, via redial. So a `Lost` body that calls `:wat::kernel::connect`
;; must NOT be mirrored, or this codemod would wire 287 sites to retry a thing that cannot
;; succeed. That is the contract decision of the DESIGN, enforced here as a detector.
;;
;; Mirror table:
;;   - no Lost arm at all                                 → placeholder raise
;;   - Lost body headed by `assertion-failed!`            → placeholder raise
;;   - Lost body that REDIALS (mentions kernel::connect)  → placeholder raise   ← DIFFERENCE 2
;;   - Lost body that IS a RecvOutcome::Lost pass-through → RecvOutcome::Malformed pass-through
;;     (propagating the outcome upward is correct and is not a retry)
;;   - Lost body that mentions the Lost binder            → mirror VERBATIM, binding `cause`
;;     (DIFFERENCE 1: the binder HAS a home here — same payload type as Lost's — and this is
;;      what finally delivers `service.wat:1231`'s promised "carrying the cause's reason")
;;   - otherwise                                          → the Lost body, verbatim
;;
;; ⚠ THE PLACEHOLDER RAISE IS NOT THE GOAL, AND SAYS SO IN ITS OWN MESSAGE. Builder:
;; "raises are placeholders until we know better." A body must have the match's result type,
;; and `assertion-failed!` is the ONLY body that type-checks in every match because it
;; diverges — which is precisely why a mechanical insert at 287 heterogeneous sites cannot
;; choose a real disposition. Stone 2 replaces these with retry / report-final / report-gone,
;; starting at the generated bodies. What this stone buys is that the fact becomes
;; REPRESENTABLE and un-forgettable; making it SURVIVABLE is the next stone.
;;
;; A match with a `_` catch-all needs nothing. A match that already carries a Malformed
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
;;     | ./target/release/wat --grep ./wat-scripts/fixes/add-malformed-arm.wat
;;
;; Usage — dry-run:
;;   cp <file> /tmp/pilot.wat && printf '["/tmp/pilot.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/add-malformed-arm.wat
;;   diff <file> /tmp/pilot.wat
;;
;; Usage — apply (list EVERY path):
;;   printf '["pathA" "pathB" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/add-malformed-arm.wat

;; ── finder (rete) ────────────────────────────────────────────────────────────

(:wat::core::defrecord :to::MatchList    [id <- :wat::core::i64])
(:wat::core::defrecord :to::RecvArm      [id <- :wat::core::i64])
(:wat::core::defrecord :to::CatchAll     [id <- :wat::core::i64])
(:wat::core::defrecord :to::HasMalformed  [id <- :wat::core::i64])

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
(:wat::rete::defrule :to::has-malformed-tagged
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
         (:wat::rete::where (:wat::rete::string::= ?hn ":wat::kernel::RecvOutcome::Malformed"))]
  :then [(:to::HasMalformed ?m)])

;; already carries TimedOut — unit spelling (TimedOut body)
(:wat::rete::defrule :to::has-malformed-unit
  :when [(:to::MatchList (?m <- :id))
         (:wat::grep::Node (?arm <- :id) (?m <- :parent) (?ai <- :index))
         (:wat::rete::where (:wat::rete::i64::>= ?ai 2))
         (:wat::grep::Node  (?pat <- :id) (?arm <- :parent) (?pi <- :index) (?pk <- :kind))
         (:wat::grep::Named (?pat <- :id) (?pn <- :name))
         (:wat::rete::where (:wat::rete::i64::= ?pi 0))
         (:wat::rete::where (:wat::rete::string::= ?pk "keyword"))
         (:wat::rete::where (:wat::rete::string::= ?pn ":wat::kernel::RecvOutcome::Malformed"))]
  :then [(:to::HasMalformed ?m)])

;; THE REPORT — RecvOutcome match, no catch-all, no TimedOut yet.
(:wat::rete::defrule :to::needs-arm
  :when [(:to::RecvArm (?m <- :id))
         (:wat::rete::not (:to::CatchAll (?m <- :id)))
         (:wat::rete::not (:to::HasMalformed (?m <- :id)))
         (:wat::grep::Span (?m <- :id) (?ln <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source (?f <- :file))]
  :then [(:wat::grep::Match :file ?f :line ?ln :col ?c :end-line ?el :end-col ?ec
           :rule "add-malformed-arm"
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

(:wat::core::defn :user::malformed-head? [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::= name ":wat::kernel::RecvOutcome::Malformed"))

;; ⛔ DIFFERENCE 2's detector — a Lost body that REDIALS must not be mirrored onto Malformed.
;; `Lost` is retried by reconnecting; `Malformed` is deterministic and retrying it re-sends the
;; same bad bytes. Mirroring a redial here would wire hundreds of sites to retry a thing that
;; cannot succeed, which is the contract decision this codemod must not break.
(:wat::core::defn :user::redials? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
    (:wat::string::contains? (:wat::core::ast-name node) ":wat::kernel::connect")
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::bool  c <- :wat::WatAST] -> :wat::core::bool
          (:wat::core::if acc true (:user::redials? c)))
        false
        (:wat::core::ast->children node))
      false)))

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

;; ⚠ THE PLACEHOLDER, AND IT NAMES ITSELF AS ONE. Builder: "raises are placeholders until we
;; know better." A match arm's body must have the match's result type, and `assertion-failed!`
;; is the only body that type-checks in EVERY match because it diverges — so a mechanical
;; insert at hundreds of heterogeneous sites cannot pick a real disposition. Stone 2 replaces
;; these with retry / report-final / report-gone. The message says so, so that a reader who
;; hits one at runtime knows it is unfinished rather than intended.
(:wat::core::def :user::malformed-placeholder
  "(:wat::kernel::assertion-failed! \"recv: malformed frame — the peer could not decode our message; this arm is an UNMIGRATED PLACEHOLDER (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)\" :wat::core::None :wat::core::None)")

;; ⭑ Returns the WHOLE arm text, pattern included, because the BINDER depends on the body:
;; a verbatim mirror of a Lost body that names its binder only compiles if this arm binds the
;; SAME name. So the pattern is minted with `bn` when mirroring such a body, `cause` for the
;; pass-through, and `_cause` where the payload is unused.
(:wat::core::defn :user::malformed-arm-text [lost <- (:wat::core::Option :- [:wat::WatAST])] -> :wat::core::String
  (:wat::core::let
    [arm-with (:wat::core::fn [binder <- :wat::core::String  body <- :wat::core::String]
                -> :wat::core::String
                (:wat::string::concat
                  (:wat::string::concat " ((:wat::kernel::RecvOutcome::Malformed " binder)
                  (:wat::string::concat ") " (:wat::string::concat body ")"))))]
    (:wat::core::match lost
      (:wat::core::None (:wat::core::apply arm-with ["_cause" :user::malformed-placeholder]))
      ((:wat::core::Some arm)
        (:wat::core::let [ach (:wat::core::ast->children arm)]
          (:wat::core::if (:wat::core::< (:wat::core::length ach) 2)
            (:wat::core::apply arm-with ["_cause" :user::malformed-placeholder])
            (:wat::core::let
              [pat  (:wat::core::first ach)
               body (:wat::core::nth ach 1)
               bn   (:user::pattern-binder-name pat)]
              ;; ⛔⛔ ALLOW-LIST, NOT DENY-LIST — and the pilot diff is why.
              ;; This branch first read: mirror the Lost body verbatim unless it is
              ;; assertion-headed or calls `:wat::kernel::connect`. The pilot on
              ;; `wat-scripts/queue/sqs.wat` then produced
              ;;   ((RecvOutcome::Malformed _cause) (Tuple (dial-store) empty-envs both))
              ;; — `(dial-store)` IS a redial, wrapped in a local helper, so the
              ;; `connect`-keyword detector never saw it. 2 of 5 mirrors in ONE file wired a
              ;; RETRY into the variant whose contract is REPORT-FINAL. A deny-list cannot
              ;; enumerate the ways a body can redial; an allow-list does not have to.
              ;;
              ;; So: mirror ONLY shapes positively recognized as safe — a Lost pass-through
              ;; (propagating an outcome upward is not a retry) and a literal `nil` (inert).
              ;; Everything else takes the placeholder, which is the honest answer to "we do
              ;; not yet know this site's disposition." `:user::redials?` is kept as a second
              ;; belt, and `bn`/`mentions-symbol?` are no longer consulted for the body.
              (:wat::core::if (:user::lost-passthrough? body)
                (:wat::core::apply arm-with
                  ["cause" "(:wat::kernel::RecvOutcome::Malformed cause)"])
                (:wat::core::if (:wat::core::and
                                  (:wat::core::= (:wat::core::ast-kind body) "symbol")
                                  (:wat::core::= (:wat::core::ast-name body) "nil"))
                  (:wat::core::apply arm-with ["_cause" "nil"])
                  (:wat::core::apply arm-with ["_cause" :user::malformed-placeholder]))))))))))

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

(:wat::core::defn :user::any-malformed? [arms <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool  arm <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc true
        (:wat::core::match (:user::arm-pat arm)
          (:wat::core::None false)
          ((:wat::core::Some pat) (:user::malformed-head? (:user::pattern-head-name pat))))))
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

(:wat::core::defn :user::needs-malformed? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:user::recv-match? node)
    (:wat::core::let
      [ch   (:wat::core::ast->children node)
       arms (:wat::core::into [] (:wat::core::drop ch 2))]
      (:wat::core::if (:user::any-catchall? arms)
        false
        (:wat::core::not (:user::any-malformed? arms))))
    false))

(:wat::core::defn :user::match-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [ch      (:wat::core::ast->children node)
     last    (:wat::core::Option/expect (:wat::core::get ch (:wat::core::- (:wat::core::length ch) 1)) "last-arm")
     arms    (:wat::core::into [] (:wat::core::drop ch 2))
     ;; The whole arm, pattern included — the binder is chosen with the body (see
     ;; :user::malformed-arm-text), so a verbatim mirror still resolves its symbol.
     insert  (:user::malformed-arm-text (:user::find-lost arms))]
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
      (:wat::core::Tuple (:user::end-off last lines) "" insert))))

(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [this (:wat::core::if (:user::needs-malformed? node)
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

;; ⛔ NO UNTAG PASS HERE, DELIBERATELY. `add-timedout-arm.wat` carries a `untag-timedout`
;; step because `TimedOut` is a UNIT variant and its first apply emitted the tagged-empty
;; spelling `((TimedOut) body)`, which the checker rejects. `Malformed` IS tagged — it carries
;; `cause <- :wat::kernel::Failure` — so the tagged spelling with a binder is the correct and
;; only form, and stripping parens here would produce exactly the error that pass exists to
;; undo. Idempotence comes from `:user::any-malformed?` alone: a match that already carries a
;; Malformed arm is never edited, so re-running is a no-op.
(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [src1  src
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
        (:wat::kernel::println (:wat::string::concat "[+malformed-arm] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
