;; wat-scripts/fixes/domain-fault-gains-location.wat — excursus 003 strike F (AUDIT-the-
;; shape-of-an-error.md F8, RULING 2026-09-27 item 5).
;; SCOPE: wat/query.wat wat/telemetry/span.wat wat/telemetry/journal.wat
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; `:wat::query::Fault` is a domain error (the `reason` every Transient/Constraint/Fatal
;; recovery-axis variant carries) that shares the floor's `Fault` name without satisfying
;; `:wat::core::Error` ({message location}) — AUDIT F8. The cure: conform it. This codemod
;; adds `:location (:wat::kernel::here)` to every `(:wat::query::Fault :message <expr>)`
;; CONSTRUCTION call across the corpus that does not already carry a `:location` kwarg —
;; `(:wat::kernel::here)` captures the span of the `(here)` form itself, so splicing it at
;; the construction call mints the span IN HAND at that exact mint site (a `defrecord`/
;; `defn` declaration has a DIFFERENT head keyword, `:wat::core::defrecord`/`:wat::core::defn`,
;; never `:wat::query::Fault`, so a declaration is never matched — only a call is).
;;
;; EXCLUDED from this run, hand-edited instead (recorded in wat/query.wat's own comment
;; beside the defrecord): `wat/query/sqlite-store.wat`'s `lift-fault`, whose `:wat::query::Fault`
;; narrows an already-located `:wat::sqlite::Fault` — the honest location is the ORIGINAL's,
;; `(:wat::sqlite::Fault/location f)`, already in hand, not a freshly minted `here`. Running
;; this codemod against that file too would be harmless (it would mint a `here` the hand-edit
;; then overwrites) but the SCOPE above omits it to keep this migration's blast radius legible.
;;
;; Insertion point: one char before the matched call's own closing paren (`ast-end-span` minus
;; one column IS the `)` itself), text ` :location (:wat::kernel::here)`.
;;
;; Usage (one EDN vector of EVERY path on stdin):
;;   printf '["wat/query.wat" "wat/telemetry/span.wat" "wat/telemetry/journal.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/domain-fault-gains-location.wat

(:wat::core::defn :user::target? [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::= name ":wat::query::Fault"))

;; has-location? — does `args` (a construction's children AFTER the head, so index 0,2,4…
;; are the kwarg KEYS) already carry a `:location` key? Scans only key positions, never values
;; (sqlite::Fault's `op` field happens to be keyword-TYPED, but that is a VALUE position here,
;; never mistaken for a key).
(:wat::core::defn :user::has-location?
  [args <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::>= i (:wat::core::length args))
    false
    (:wat::core::let [k (:wat::core::Option/expect (:wat::core::get args i) "has-location? key")]
      (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind k) "keyword")
                        (:wat::core::= (:wat::core::ast-name k) ":location") false)
        true
        (:user::has-location? args (:wat::core::+ i 2))))))

;; fault-edit — one insertion edit, right before `node`'s own closing paren.
(:wat::core::defn :user::fault-edit
  [node <- :wat::WatAST lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let [end-off   (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span node) lines)
                    paren-off (:wat::core::- end-off 1)]
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
      (:wat::core::Tuple paren-off "" " :location (:wat::kernel::here)"))))

;; edits / edits-seq — generic list/vector/map walk (mirrors positional-to-kwargs.wat's shape).
(:wat::core::defn :user::edits
  [node <- :wat::WatAST lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
        (:wat::core::let
          [head  (:wat::core::first ch)
           args  (:wat::core::into [] (:wat::core::rest ch))
           hname (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
                   (:wat::core::ast-name head) "")
           this  (:wat::core::if (:wat::core::if (:user::target? hname)
                                    (:wat::core::not (:user::has-location? args 0)) false)
                   (:user::fault-edit node lines)
                   (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))]
          (:wat::core::concat this (:user::edits-seq ch lines)))))
    (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "vector") true
                      (:wat::core::= (:wat::core::ast-kind node) "map"))
      (:user::edits-seq (:wat::core::into [] (:wat::core::ast->children node)) lines)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))))

(:wat::core::defn :user::edits-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST]) lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    (:wat::core::concat
      (:user::edits (:wat::core::first items) lines)
      (:user::edits-seq (:wat::core::into [] (:wat::core::rest items)) lines))))

;; ── per-file migrate ─────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines (:wat::string::split src "\n")
                    tree  (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
                    forms (:wat::core::ast->children tree)
                    eds   (:user::edits-seq forms lines)
                    rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

;; ── driver ───────────────────────────────────────────────────────────────────
(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[domain-fault-gains-location] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
      [:wat::kernel::ReadlnOutcome.Eof {}
        (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
