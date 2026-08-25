;; wat-scripts/fixes/eprintln-recv-arm-to-assertion-failed.wat — arc 278 the eprintln annihilation.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; THE ABUSE (R53's own wall's mechanism, R51 TYPO TANGO): the recv' OUTCOME WALL surfaces
;; `::Lost`/`::Closed` via `eprintln` — but `eprintln` is wat's PANIC (panic_any, terminal) AND the
;; ONLY raise-face that writes stdio. In a no-stdio context it → ServiceNotRunning, which MASKS the
;; real failure (the 39-test weigh cascade). The DEATH channel misused as the recv'-read surfacing.
;;
;; THE FIX (DESIGN-eprintln-annihilation.md, the pinned contract): move the recv'-wall surfacing off
;; `eprintln` onto `assertion-failed!` — a stdio-free SIBLING of the same panic_any mechanism
;; (catchable, structured, no ServiceNotRunning mask). Per matched node:
;;
;;   (:wat::kernel::eprintln ARG)  ->  (:wat::kernel::assertion-failed! ARG :wat::core::None :wat::core::None)
;;
;; WHICH eprintln is a recv'-wall arm is DISCRIMINATED by its ARG (grounded: 102 `Failure/message`
;; + 90 `"recv':…"` = the ~194 wall arms; the ~20 legitimate death-channel eprintlns take entirely
;; different args — "dying…", "oops", 42, CircuitBreak, "STOP0-FAIL:" — and are left byte-untouched):
;;
;;   ::Lost arm   ARG is a call headed by  :wat::kernel::Failure/message   (a recv-cause accessor;
;;                                                                          no legit eprintln uses it)
;;   ::Closed arm ARG is a string literal whose first `:`-segment is  recv'  ("recv': … closed …")
;;
;; Comment/format faithful (span edits via fix-text-apply, reverse-sorted). Idempotent (re-run = 0
;; edits: an `assertion-failed!` head is no longer `eprintln`). NO Rust change ships with this — both
;; verbs already exist, so no stash-dance: `cargo build --release` once, then run.
;;
;; NOTE (the stream-loop `::Closed` exception): a small hand-set (bracket_runner_stream_of_messages,
;; bracket_runner_large_stream, probe-m1-phantom-d, w3-n-dial-runner) whose clean EOF must terminate,
;; not raise — this codemod converts their `::Closed` to assertion-failed! too; they are HAND-FIXED to
;; a clean terminal AFTER, per the DESIGN execution plan.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB" …]\n' | cargo wat ./wat-scripts/fixes/eprintln-recv-arm-to-assertion-failed.wat

;; ── small helpers (mirrors response-record-to-enum.wat) ──────────────────────────
(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

;; recv-cause-arg? — ARG is a call `(:wat::kernel::Failure/message …)` (the ::Lost arm).
(:wat::core::defn :user::recv-cause-arg? [arg <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arg) "list")
    (:wat::core::let [ch (:wat::core::ast->children arg)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::kernel::Failure/message")))
    false))

;; recv-closed-arg? — ARG is a string literal whose first `:`-segment is `recv'` (the ::Closed arm,
;; "recv': … closed …"). ast-name on a StringLit returns the unquoted content.
(:wat::core::defn :user::recv-closed-arg? [arg <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arg) "string")
    (:wat::core::= (:wat::core::first (:wat::string::split (:wat::core::ast-name arg) ":")) "recv'")
    false))

;; recv-arm-eprintln? — a list `(:wat::kernel::eprintln ARG)` (exactly 2 children) whose ARG is a
;; recv-cause call OR a recv-closed string. Nothing else is touched.
(:wat::core::defn :user::recv-arm-eprintln? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 2)
        (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::kernel::eprintln")
          (:wat::core::let [arg (:wat::core::Option/expect (:wat::core::get ch 1) "arg")]
            (:wat::core::if (:user::recv-cause-arg? arg) true (:user::recv-closed-arg? arg)))
          false)
        false))
    false))

;; ── EDITS: head rename + append ` :None :None` after ARG ─────────────────────────
(:wat::core::defn :user::eprintln-edits
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let
    [head (:wat::core::Option/expect (:wat::core::get ch 0) "ep head")
     arg  (:wat::core::Option/expect (:wat::core::get ch 1) "ep arg")
     h0   (:user::start-off head lines)
     h1   (:user::end-off head lines)]
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
      (:wat::core::Tuple h0 (:wat::core::- h1 h0) ":wat::kernel::assertion-failed!")
      (:wat::core::Tuple (:user::end-off arg lines) 0 " :wat::core::None :wat::core::None"))))

;; walk one node → its edits + descendants'.
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let
    [this (:wat::core::if (:user::recv-arm-eprintln? node)
            (:user::eprintln-edits (:wat::core::ast->children node) lines)
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat this (:user::seq-edits (:wat::core::ast->children node) lines))
      this)))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])]) it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lines)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    items))

;; ── per-file migrate ─────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     forms (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))
     eds   (:user::seq-edits forms lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

;; ── driver ───────────────────────────────────────────────────────────────────
(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[eprintln->assertion-failed!] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
