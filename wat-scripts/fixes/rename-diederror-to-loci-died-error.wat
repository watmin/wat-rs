;; wat-scripts/fixes/rename-diederror-to-loci-died-error.wat — arc 278 the LociDiedError stone.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; THE MIGRATION (DESIGN-loci-died-error.md): the two near-twin death-report enums
;; `:wat::kernel::ThreadDiedError` + `:wat::kernel::ProcessDiedError` are ANNIHILATED and replaced
;; by ONE loci-agnostic `:wat::kernel::LociDiedError`. And `RecvOutcome::Lost`'s cause changes from
;; a flat `:wat::kernel::Failure` to a `:wat::kernel::LociDiedError`, so a Lost arm's cause is read
;; via `LociDiedError/message`, not `Failure/message`. This codemod does three things:
;;
;;   (1) `:wat::kernel::ThreadDiedError`  -> `:wat::kernel::LociDiedError`   (prefix; type-args +
;;   (2) `:wat::kernel::ProcessDiedError` -> `:wat::kernel::LociDiedError`    accessors too)
;;   (3) SCOPED, within a `(:wat::kernel::RecvOutcome::Lost VAR)` match arm ONLY:
;;         `(:wat::kernel::Failure/message VAR)` -> `(:wat::kernel::LociDiedError/message VAR)`
;;       keyed on VAR name so a GENUINE `(Failure/message f)` (a RunResult failure, a raise
;;       receiver) and a `ServiceEvent::Lost` cause (whose type is UNCHANGED — still a Failure)
;;       are left byte-untouched.
;;
;; (1)+(2) ride `:wat::fix::rename-keyword-prefix`; (3) is the scoped edit-collector below.
;; Comment/format faithful (span edits via fix-text-apply, reverse-sorted). Idempotent
;; (re-run = 0 edits). SHIPS ALONGSIDE the Rust change that makes the old forms illegal → run under
;; the STASH-DANCE (fix.wat header): stash src/*.rs, build the OLD binary, run this, pop, rebuild.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB" …]\n' | cargo wat ./wat-scripts/fixes/rename-diederror-to-loci-died-error.wat

;; ── small helpers (mirrors eprintln-recv-arm-to-assertion-failed.wat) ──────────────
(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :user::sym-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "symbol")
    (:wat::core::ast-name n) ""))

;; lost-arm-var — if `node` is a match ARM `((:wat::kernel::RecvOutcome::Lost VAR) . body)` —
;; i.e. a list whose FIRST child is a list `(:wat::kernel::RecvOutcome::Lost <sym>)` — return
;; <sym>'s name; else "". Keys on the EXACT `RecvOutcome::Lost` keyword, so a `ServiceEvent::Lost`
;; arm (different keyword, two bindings) is NOT a match.
(:wat::core::defn :user::lost-arm-var [node <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        ""
        (:wat::core::let [pat (:wat::core::first ch)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "list")
            (:wat::core::let [pch (:wat::core::ast->children pat)]
              (:wat::core::if (:wat::core::>= (:wat::core::length pch) 2)
                (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first pch)) ":wat::kernel::RecvOutcome::Lost")
                  (:user::sym-name (:wat::core::Option/expect (:wat::core::get pch 1) "lost var"))
                  "")
                ""))
            ""))))
    ""))

;; fm-call-on? — `node` is a call `(:wat::kernel::Failure/message VAR)` whose arg symbol name
;; EQUALS `var` (and `var` is non-empty). This is the surgical discriminator: it renames ONLY the
;; Lost-cause accessor, never a genuine `(Failure/message f)`.
(:wat::core::defn :user::fm-call-on? [node <- :wat::WatAST  var <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= var "")
    false
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
      (:wat::core::let [ch (:wat::core::ast->children node)]
        (:wat::core::if (:wat::core::>= (:wat::core::length ch) 2)
          (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::kernel::Failure/message")
            (:wat::core::= (:user::sym-name (:wat::core::Option/expect (:wat::core::get ch 1) "fm arg")) var)
            false)
          false))
      false)))

;; fm-head-edit — rename the `:wat::kernel::Failure/message` HEAD keyword token of `node` to
;; `:wat::kernel::LociDiedError/message` (whole-token span replace; the arg is untouched).
(:wat::core::defn :user::fm-head-edit
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let [head (:wat::core::first (:wat::core::ast->children node))
                    h0   (:user::start-off head lines)
                    h1   (:user::end-off head lines)]
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
      (:wat::core::Tuple h0 (:wat::core::- h1 h0) ":wat::kernel::LociDiedError/message"))))

;; node-edits — scope-threading walk. `lost-var` is the RecvOutcome::Lost-bound var currently in
;; scope (""=none). A Lost arm OVERRIDES the scope for its children (nested Lost arms shadow).
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lost-var <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let
    [this-var (:user::lost-arm-var node)
     scope    (:wat::core::if (:wat::core::= this-var "") lost-var this-var)
     this     (:wat::core::if (:user::fm-call-on? node lost-var)
                (:user::fm-head-edit node lines)
                (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat this (:user::seq-edits (:wat::core::ast->children node) scope lines))
      this)))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  lost-var <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])]) it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lost-var lines)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    items))

;; ── per-file migrate ─────────────────────────────────────────────────────────
;; Pass 1+2: prefix rename Thread/ProcessDiedError -> LociDiedError (rides rename-keyword-prefix).
;; Pass 3:  scoped Failure/message -> LociDiedError/message within RecvOutcome::Lost arms.
(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [src1  (:wat::fix::rename-keyword-prefix ":wat::kernel::ThreadDiedError" ":wat::kernel::LociDiedError" src)
     src2  (:wat::fix::rename-keyword-prefix ":wat::kernel::ProcessDiedError" ":wat::kernel::LociDiedError" src1)
     lines (:wat::core::string::split src2 "\n")
     forms (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src2) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))
     eds   (:user::seq-edits forms "" lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src2 rev)))

;; ── driver ───────────────────────────────────────────────────────────────────
(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[diederror->loci-died-error] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
