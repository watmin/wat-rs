;; wat-scripts/fixes/mandate-request-malformed.wat — arc 278 Stone 2 (ANNIHILATE the knob) migration.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; Stone 1 built the request-SHAPE wall and defaulted it OFF behind an opt-in clause. Builder
;; ruling: a knob whose off-position is "crash on malformed input" is a non-option surfaced as a
;; choice. Stone 2 deletes the clause and generates the guard UNCONDITIONALLY — which makes
;; `:RequestMalformed` MANDATORY on every serviceable op-Response enum, exactly the standing
;; `:RequestTooLarge` has under ruling A (`synthesize_surface_protocol`, src/types.rs).
;;
;; This migrates, per file:
;;
;;   (a) the op-Response DECL — a `defenum` carrying `:RequestTooLarge [bytes cap]` gains its
;;       SHAPE sibling immediately after it:
;;         :RequestMalformed [path     <- :wat::core::Vector<wat::core::String>
;;                            expected <- :wat::core::String
;;                            got      <- :wat::core::String]
;;
;;   (b) every CALLER match — a `match` carrying an arm `((:T::RequestTooLarge b c) BODY)` gains
;;       the sibling arm `((:T::RequestMalformed mpath mexpected mgot) BODY')`, because enum
;;       matches are exhaustive with no wildcard arm (arc 109). Two BODY' families, decided
;;       STRUCTURALLY from the RequestTooLarge arm's own body:
;;         • PROPAGATION — body is `(:U::RequestTooLarge b c)` (an s2s consumer re-raising the
;;           peer's breach as its OWN op's refusal) → `(:U::RequestMalformed mpath mexpected mgot)`.
;;           A shape refusal from a downstream service is the same class of fact as a size
;;           refusal, and swallowing it would re-open the very DoS this stone closes.
;;         • TERMINAL — anything else (a test caller, a stdio prime) → the ruling-A precedent
;;           this codemod's sibling `response-record-to-enum.wat` set for its own new arms:
;;           `(:wat::kernel::assertion-failed! "unexpected RequestMalformed" ...)`. A terminal
;;           caller that constructs its own typed request CANNOT be malformed — if it ever is,
;;           that is a bug and must be loud, never swallowed.
;;
;; WHICH enums are op-Responses is DISCOVERED, not hardcoded, and the gate is EXACT: an enum is a
;; ruling-A op-Response iff it carries `:RequestTooLarge` — that variant is checker-forced on
;; precisely the serviceable-op-Response set and on nothing else. Same for the arms: a match that
;; needs the new arm is a match that already faces `::RequestTooLarge`. No grepping, no guessing.
;;
;; Because the rewrite keys on the AST, `RequestTooLarge` occurring inside a STRING (wat/service.wat's
;; `"::{variant-pascal}Response::RequestTooLarge"` codegen template, this directory's own
;; `response-record-to-enum.wat`) is byte-untouched. That is the whole reason it is a codemod.
;;
;; Comment/format faithful (span edits via fix-text-apply; indentation is read off the source column
;; of the node being followed). Idempotent: an enum that already declares `:RequestMalformed` and a
;; match that already has the `::RequestMalformed` arm are both skipped, so re-run = 0 edits.
;;
;; NO STASH-DANCE NEEDED: this migration is purely ADDITIVE and legal under the OLD checker (a new
;; enum variant, a new match arm). Run it FIRST, with the pre-Stone-2 binary; THEN land the
;; src/types.rs lock + wat/service.wat's unconditional guard. Migration first, contract lock second
;; — the exact order ruling A used for `:RequestTooLarge`.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/telemetry/journal.wat" …]\n' \
;;     | cargo wat ./wat-scripts/fixes/mandate-request-malformed.wat

;; ── small helpers ────────────────────────────────────────────────────────────
(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

;; col-of — the 1-indexed source column a node starts at. `(- col 1)` is exactly the node's
;; indentation when it starts its line, which is how every site in this corpus is written; the
;; inserted sibling therefore lands at the same indentation as the node it follows.
(:wat::core::defn :user::col-of [n <- :wat::WatAST] -> :wat::core::i64
  (:wat::core::Option/expect
    (:wat::core::HashMap/get (:wat::core::ast-span n) :col)
    "col-of: :col"))

(:wat::core::defn :user::spaces [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::if (:wat::core::<= n 0)
    ""
    (:wat::core::string::concat " " (:user::spaces (:wat::core::- n 1)))))

(:wat::core::defn :user::ends-with? [s <- :wat::core::String  suf <- :wat::core::String]
  -> :wat::core::bool
  (:wat::core::let [ls (:wat::core::string::length s)
                    lf (:wat::core::string::length suf)]
    (:wat::core::if (:wat::core::< ls lf)
      false
      (:wat::core::= (:wat::core::string::subs s (:wat::core::- ls lf) ls) suf))))

;; rtl->rm — `:T::RequestTooLarge` → `:T::RequestMalformed` (suffix swap; caller has already
;; established the suffix via ends-with?).
(:wat::core::defn :user::rtl->rm [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::concat
    (:wat::core::string::subs s 0
      (:wat::core::- (:wat::core::string::length s)
        (:wat::core::string::length "::RequestTooLarge")))
    "::RequestMalformed"))

(:wat::core::defn :user::no-edits []
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))

;; find-kw-index — index of the first child that is EXACTLY the keyword `name`; -1 if absent.
(:wat::core::defn :user::find-kw-index
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  name <- :wat::core::String] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
      (:wat::core::if (:wat::core::>= acc 0)
        acc
        (:wat::core::if
          (:wat::core::= (:user::kw-name (:wat::core::Option/expect (:wat::core::get ch i) "fki")) name)
          i
          acc)))
    (:wat::core::- 0 1)
    (:wat::core::range 0 (:wat::core::length ch))))

;; ── (a) the op-Response DECL ─────────────────────────────────────────────────
;; A `defenum` whose variant keywords include `:RequestTooLarge` (and not yet `:RequestMalformed`)
;; is a ruling-A op-Response. Insert the shape sibling right after the size variant's field vector.
(:wat::core::defn :user::defenum-edits
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let
    [irtl (:user::find-kw-index ch ":RequestTooLarge")
     irm  (:user::find-kw-index ch ":RequestMalformed")]
    (:wat::core::if
      (:wat::core::if (:wat::core::>= irtl 0) (:wat::core::< irm 0) false)
      (:wat::core::let
        [kw (:wat::core::Option/expect (:wat::core::get ch irtl) "rtl kw")
         fv (:wat::core::Option/expect (:wat::core::get ch (:wat::core::+ irtl 1)) "rtl fields")]
        (:wat::core::if (:wat::core::= (:wat::core::ast-kind fv) "vector")
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
            (:wat::core::Tuple (:user::end-off fv lines) 0
              (:wat::core::string::concat "\n"
                (:wat::core::string::concat (:user::spaces (:wat::core::- (:user::col-of kw) 1))
                  ":RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String]"))))
          (:user::no-edits)))
      (:user::no-edits))))

;; ── (b) the CALLER match arm ─────────────────────────────────────────────────
;; arm-head-kw — for an arm node `((:T::Variant a b) BODY)`, the pattern's head keyword name
;; (""  when the arm is not that shape: a bare-keyword pattern, a `_` wildcard, a body-less arm).
(:wat::core::defn :user::arm-head-kw [arm <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
    (:wat::core::let [ach (:wat::core::ast->children arm)]
      (:wat::core::if (:wat::core::< (:wat::core::length ach) 2)
        ""
        (:wat::core::let [pat (:wat::core::first ach)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "list")
            (:wat::core::let [pch (:wat::core::ast->children pat)]
              (:wat::core::if (:wat::core::empty? pch)
                ""
                (:user::kw-name (:wat::core::first pch))))
            ""))))
    ""))

;; rm-arm-body — BODY' for the synthesized `::RequestMalformed` arm, decided from the
;; `::RequestTooLarge` arm's own body (see the header's two families).
(:wat::core::defn :user::rm-arm-body [arm <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let
    [ach  (:wat::core::ast->children arm)
     body (:wat::core::Option/expect (:wat::core::get ach 1) "rtl arm body")
     bhd  (:wat::core::if (:wat::core::= (:wat::core::ast-kind body) "list")
            (:wat::core::let [bch (:wat::core::ast->children body)]
              (:wat::core::if (:wat::core::empty? bch) "" (:user::kw-name (:wat::core::first bch))))
            "")]
    (:wat::core::if (:user::ends-with? bhd "::RequestTooLarge")
      (:wat::core::string::concat "(" (:wat::core::string::concat (:user::rtl->rm bhd)
        " mpath mexpected mgot)"))
      "(:wat::kernel::assertion-failed! \"unexpected RequestMalformed\" :wat::core::None :wat::core::None)")))

;; find-rtl-arm — index of the first arm whose pattern head ends in `::RequestTooLarge`; -1 if none.
;; has-rm-arm? — whether some arm already faces `::RequestMalformed` (the idempotency gate).
(:wat::core::defn :user::find-arm-suffix
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  suf <- :wat::core::String] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
      (:wat::core::if (:wat::core::>= acc 0)
        acc
        (:wat::core::if
          (:user::ends-with? (:user::arm-head-kw (:wat::core::Option/expect (:wat::core::get ch i) "arm")) suf)
          i
          acc)))
    (:wat::core::- 0 1)
    (:wat::core::range 0 (:wat::core::length ch))))

(:wat::core::defn :user::match-edits
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::let
    [irtl (:user::find-arm-suffix ch "::RequestTooLarge")
     irm  (:user::find-arm-suffix ch "::RequestMalformed")]
    (:wat::core::if
      (:wat::core::if (:wat::core::>= irtl 0) (:wat::core::< irm 0) false)
      (:wat::core::let
        [arm  (:wat::core::Option/expect (:wat::core::get ch irtl) "rtl arm")
         head (:user::arm-head-kw arm)
         ind  (:wat::core::- (:user::col-of arm) 1)]
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
          (:wat::core::Tuple (:user::end-off arm lines) 0
            (:wat::core::string::concat "\n"
              (:wat::core::string::concat (:user::spaces ind)
                (:wat::core::string::concat "((" (:wat::core::string::concat (:user::rtl->rm head)
                  (:wat::core::string::concat " mpath mexpected mgot)\n"
                    (:wat::core::string::concat (:user::spaces (:wat::core::+ ind 2))
                      (:wat::core::string::concat (:user::rm-arm-body arm) ")"))))))))))
      (:user::no-edits))))

;; ── walk ─────────────────────────────────────────────────────────────────────
(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:user::no-edits)
        (:wat::core::let
          [hname (:user::kw-name (:wat::core::first ch))
           this  (:wat::core::if (:wat::core::= hname ":wat::core::defenum")
                   (:user::defenum-edits ch lines)
                   (:wat::core::if (:wat::core::= hname ":wat::core::match")
                     (:user::match-edits ch lines)
                     (:user::no-edits)))]
          (:wat::core::concat this (:user::seq-edits ch lines)))))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::seq-edits (:wat::core::ast->children node) lines)
      (:user::no-edits))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
                     it  <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lines)))
    (:user::no-edits)
    items))

;; ── per-file migrate ─────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::core::string::split src "\n")
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
        (:wat::kernel::println (:wat::core::string::concat "[mandate-request-malformed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
