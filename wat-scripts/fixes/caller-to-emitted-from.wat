;; wat-scripts/fixes/caller-to-emitted-from.wat — arc 278 caller.2 migration codemod.
;;
;; Self-hosted, comment-faithful fix-wat codemod — NO hand-editing of .wat files, use the tool.
;;
;; Flips the telemetry `caller` field (a forgeable hand-typed keyword) to `emitted-from <-
;; :wat::kernel::Frame` (the real captured call-site, via `(:wat::kernel::call-site)`). The two
;; field DECLS in wat/telemetry.wat are hand-edited (a single-line rename, out of scope for a
;; structural tool); THIS codemod migrates every WRITER/READER call site:
;;
;;   (a) DIRECT WRITE   `:caller <literal-keyword>` inside a `:wat::telemetry::Log` or
;;       `:wat::telemetry::Span::LogRequest` construction
;;         -> `:emitted-from (:wat::kernel::call-site)`   (rename key, REPLACE value)
;;   (b) PRODUCER FORWARD `:caller (:T/caller req)` inside the same two constructions
;;         -> `:emitted-from (:T/emitted-from req)`         (rename key, rename the accessor
;;                                                             head in the value — via rule (c))
;;   (c) ACCESSOR READ  `:wat::telemetry::Log/caller` / `:wat::telemetry::Span::LogRequest/caller`
;;         -> `.../emitted-from`                             (exact-name keyword leaf rewrite,
;;                                                             fires wherever the accessor appears,
;;                                                             including nested inside (b)'s value)
;;
;; ⚠ EXACT-NAME MATCH ONLY — never a prefix/boundary rule. `:caller'` (trailing apostrophe) is a
;; COMPLETELY DIFFERENT keyword (the arena's `caller'` DEFSERVICE — `:caller'::Record`,
;; `:caller'::State`, `:caller'/start`, `:caller'::Handle/addr`, 14+ occurrences). Every match
;; below is a full `(:wat::core::= name ":caller")` / exact accessor-name string equality — NOT
;; `:wat::fix::rename-keyword-prefix` (that rule's right-boundary check treats `'` as a valid
;; boundary char, i.e. it WOULD shred `:caller'::Record` -> `:emitted-from'::Record`). Do not
;; swap in the prefix-rename helper here.
;;
;; The `:caller` KEY-rename only fires inside a construction whose head is EXACTLY
;; `:wat::telemetry::Log` or `:wat::telemetry::Span::LogRequest` (never a bare `:caller` kwarg
;; elsewhere — those don't exist in this corpus; if one shows up the codemod silently leaves it
;; alone, by construction, since it only edits inside a matched ctor's child list).
;;
;; Comment/format faithful (span edits via fix-text-apply, wat/fix.wat). Idempotent (re-run = 0
;; edits: after migration the ctor children no longer contain a `:caller` keyword and the
;; accessor keywords no longer end in `/caller`).
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/telemetry/span.wat" "tests/services/probe_arc278_journal_logs_on_process.wat" ...]\n' \
;;     | cargo wat ./wat-scripts/fixes/caller-to-emitted-from.wat

;; ── small helpers ────────────────────────────────────────────────────────────
(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::ast-name n) ""))

;; caller-kw? — EXACT match on the bare `:caller` keyword (never a prefix; `:caller'`, `:t::caller`,
;; `:my::kernel::caller` etc. all have a DIFFERENT ast-name and fail this equality).
(:wat::core::defn :user::caller-kw? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::= (:wat::core::ast-name n) ":caller")
    false))

;; accessor-new-name — EXACT-name map for the two field-accessor keywords; identity (no edit) for
;; every other keyword, including `:caller'`-family names (never touched — different string).
(:wat::core::defn :user::accessor-new-name [name <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= name ":wat::telemetry::Log/caller")
    ":wat::telemetry::Log/emitted-from"
    (:wat::core::if (:wat::core::= name ":wat::telemetry::Span::LogRequest/caller")
      ":wat::telemetry::Span::LogRequest/emitted-from"
      name)))

;; accessor-edit — zero-or-one whole-token replace edit for a keyword LEAF whose exact name is one
;; of the two accessors above.
(:wat::core::defn :user::accessor-edit
  [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword")
    (:wat::core::let
      [name     (:wat::core::ast-name n)
       new-name (:user::accessor-new-name name)]
      (:wat::core::if (:wat::core::= new-name name)
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
          (:wat::core::Tuple (:user::start-off n lines) name new-name))))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))))

;; ── ctor :caller key/value edits — only ever called with a matched Log/LogRequest ctor's
;; children (head + flat kwargs). Scans every index; a `:caller` KEY gets renamed, and — iff its
;; value is itself a bare keyword LITERAL (the direct-write case) — the value is replaced whole
;; with `(:wat::kernel::call-site)`. The producer-forward case (value is a list) is left for the
;; general recursive walk to rewrite via accessor-edit (its head keyword matches rule (c) above).
(:wat::core::defn :user::ctor-caller-edits
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]) i <- :wat::core::i64]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::let
        [k (:wat::core::Option/expect (:wat::core::get ch i) "ctor-caller-edits: k")]
        (:wat::core::if (:user::caller-kw? k)
          ;; key-edit's old-text = (ast-name k) — already verified by caller-kw? to equal
          ;; ":caller"; val-edit's old-text = (ast-name v) — the literal keyword value being
          ;; replaced. NEVER span text (both rename a keyword leaf, STOP-1 territory).
          (:wat::core::let
            [ks       (:user::start-off k lines)
             key-edit (:wat::core::Tuple ks (:wat::core::ast-name k) ":emitted-from")
             nxt      (:wat::core::get ch (:wat::core::+ i 1))]
            (:wat::core::match nxt
              (:wat::core::None
                (:wat::core::concat acc (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]) key-edit)))
              ((:wat::core::Some v)
                (:wat::core::if (:wat::core::= (:wat::core::ast-kind v) "keyword")
                  (:wat::core::let
                    [vs       (:user::start-off v lines)
                     val-edit (:wat::core::Tuple vs (:wat::core::ast-name v) "(:wat::kernel::call-site)")]
                    (:wat::core::concat acc
                      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]) key-edit val-edit)))
                  (:wat::core::concat acc
                    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]) key-edit))))))
          acc)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
    (:wat::core::range 0 (:wat::core::length ch))))

;; ── general recursive walk ───────────────────────────────────────────────────
(:wat::core::defn :user::node-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
        (:wat::core::let
          [hname (:user::kw-name (:wat::core::first ch))
           ctor? (:wat::core::if (:wat::core::= hname ":wat::telemetry::Log") true
                   (:wat::core::= hname ":wat::telemetry::Span::LogRequest"))
           ctor-edits (:wat::core::if ctor?
                        (:user::ctor-caller-edits ch lines)
                        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])))]
          (:wat::core::concat ctor-edits (:user::seq-edits ch lines)))))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::seq-edits (:wat::core::ast->children node) lines)
      (:user::accessor-edit node lines))))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]) it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it lines)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
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
        (:wat::kernel::println (:wat::string::concat "[caller->emitted-from] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
