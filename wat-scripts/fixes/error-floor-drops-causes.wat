;; wat-scripts/fixes/error-floor-drops-causes.wat — excursus 003 strike B1
;; (BRIEF-shape-strike-B1-the-floor-is-message-and-location.md).
;; SCOPE: corpus
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; `:wat::core::Error` stops carrying `causes`; the floor becomes `{message location}`
;; (AUDIT-the-shape-of-an-error.md F3, RULING 2026-09-27 item 1). Every declared error
;; record's `causes <- (:wat::core::Vector :- [:wat::core::Error])` field is removed — measured
;; at 164 records across `wat/{core,runtime,check,types,load,config,resolve,stdlib,rete,macro,
;; parse}-errors.wat` (`git ls-files 'wat/*.wat' | xargs grep -cE "causes +<-"`) — with three
;; exceptions the brief rules explicitly:
;;
;;   - the three checker AGGREGATES (`:wat::check::CheckErrors`, `:wat::rete::ReteCheckErrors`,
;;     `:wat::resolve::UnresolvedReferences`) keep a vector field but RENAMED to `errors` (their
;;     items are MEMBERS, not causes — reverting step 3c's move);
;;   - the two runtime WRAPPING kinds (`:wat::runtime::EvalVerificationFailed`,
;;     `:wat::runtime::MacroExpansionFailed`) drop `causes` and gain a named
;;     `cause <- :wat::core::Error` field instead, now that the wrapped taxonomies (HashError /
;;     MacroError) are declared records (S2/S3) — the same shape `wat/macro-errors.wat`'s
;;     `ProgramBodyEvalFailed.cause` / `MacroEvalRuntimeFailed.cause` and `wat/load-errors.wat`'s
;;     `Parse.cause` already use.
;;
;; `:wat::core::Fault/of`'s quasiquote body also stops passing `:causes` to its own constructor
;; call — the one non-declaration `:causes` site in the eleven files (a keyword+value pair, not
;; a field declaration), found and deleted wherever it sits (there is exactly one).
;;
;; Deletion is a TOKEN-SPAN removal (name..arrow..type, or keyword..value): surrounding
;; whitespace/comments survive untouched — a residual blank line where a field used to sit is
;; wat-fmt's job, never this codemod's (same precedent as `drop-retired-locidiederror-arms.wat`).
;; The rename is a single-symbol whole-token replace (`fix-text-span-text` is never used as the
;; RENAME's old-text — see `wat/fix.wat`'s warning on that function — the field name's own
;; `ast-name` is the old-text instead).
;;
;; Idempotent: once a record's `causes <- (:wat::core::Vector :- [:wat::core::Error])` field is
;; gone (renamed, deleted, or replaced by `cause`), the scan finds nothing there — 0 edits on a
;; second run. Likewise the Fault/of kwarg scan finds nothing once deleted.
;;
;; Dry-run on a /tmp copy + diff, THEN apply to the corpus:
;;   printf '["wat/core.wat" "wat/runtime-errors.wat" "wat/check-errors.wat" \
;;            "wat/types-errors.wat" "wat/load-errors.wat" "wat/config-errors.wat" \
;;            "wat/resolve-errors.wat" "wat/stdlib-errors.wat" "wat/rete-errors.wat" \
;;            "wat/macro-errors.wat" "wat/parse-errors.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/error-floor-drops-causes.wat

;; ── which records get which treatment ───────────────────────────────────────────────────────

(:wat::core::defn :user::rename-target? [name <- :wat::core::String] -> :wat::core::bool
  (:wat::fix::str-in? name
    (:wat::core::Vector :- [:wat::core::String]
      ":wat::check::CheckErrors"
      ":wat::rete::ReteCheckErrors"
      ":wat::resolve::UnresolvedReferences")))

(:wat::core::defn :user::cause-add-target? [name <- :wat::core::String] -> :wat::core::bool
  (:wat::fix::str-in? name
    (:wat::core::Vector :- [:wat::core::String]
      ":wat::runtime::EvalVerificationFailed"
      ":wat::runtime::MacroExpansionFailed")))

;; decl-head? — a `defrecord` or `defsurface` top-level form — the two shapes that carry a
;; field vector we might need to edit.
(:wat::core::defn :user::decl-head? [h <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= h ":wat::core::defrecord") true
    (:wat::core::= h ":wat::core::defsurface")))

;; decl-name-keyword? — child[1] (the declared name position) is a literal keyword. Guards
;; against a QUASIQUOTED `defrecord`/`defsurface` FORM-BUILDING macro (e.g. `wat/core.wat:1195`'s
;; `` `(:wat::core::defrecord ~coords-kw ~swapped-argvec)`` inside a `defmacro` body) whose name
;; position is an unquote splice, not a real declared name — `ast-name` refuses those nodes, and
;; there is no `causes` field to migrate on a form that isn't a literal declaration anyway.
(:wat::core::defn :user::decl-name-keyword? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [ch (:wat::core::ast->children node)]
    (:wat::core::if (:wat::core::< (:wat::core::length ch) 2)
      false
      (:wat::core::= (:wat::core::ast-kind (:wat::core::Option/expect (:wat::core::get ch 1) "decl-name-keyword?: child[1]")) "keyword"))))

;; find-after-keyword — the child immediately following the FIRST keyword named `kw`.
;; `:wat::core::Error`'s `defsurface` carries its field vector after `:features`.
(:wat::core::defn :user::find-after-keyword
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  kw <- :wat::core::String] -> :wat::WatAST
  (:wat::core::let [h  (:wat::core::Option/expect (:wat::core::get ch 0) "find-after-keyword: ran out of children")
                    tl (:wat::core::into [] (:wat::core::rest ch))]
    (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
                      (:wat::core::= (:wat::core::ast-name h) kw)
                      false)
      (:wat::core::Option/expect (:wat::core::get tl 0) "find-after-keyword: nothing after the keyword")
      (:user::find-after-keyword tl kw))))

;; field-vector-of — the declared record/surface's field vector node.
;; `defrecord` puts it at child[2] (head name vector); `defsurface` puts it after `:features`.
(:wat::core::defn :user::field-vector-of
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  head <- :wat::core::String] -> :wat::WatAST
  (:wat::core::if (:wat::core::= head ":wat::core::defsurface")
    (:user::find-after-keyword ch ":features")
    (:wat::core::Option/expect (:wat::core::get ch 2) "field-vector-of: defrecord child[2]")))

;; causes-index — the index of the "causes" field-NAME symbol in a field vector's children.
(:wat::core::defn :user::causes-index
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  i <- :wat::core::i64] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::if (:wat::core::>= i (:wat::core::length ch))
    :wat::core::Option.None
    (:wat::core::let [c (:wat::core::Option/expect (:wat::core::get ch i) "causes-index: in-bounds by the guard above")]
      (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind c) "symbol")
                        (:wat::core::= (:wat::core::ast-name c) "causes")
                        false)
        (:wat::core::Option.Some {:value i})
        (:user::causes-index ch (:wat::core::+ i 1))))))

;; deletion-edit — removes the NAME..ARROW..TYPE span (3 children starting at idx).
(:wat::core::defn :user::deletion-edit
  [ch    <- (:wat::core::Vector :- [:wat::WatAST])
   idx   <- :wat::core::i64
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> :wat::fix::Edit
  (:wat::core::let [name-node (:wat::core::Option/expect (:wat::core::get ch idx) "deletion-edit: name")
                    type-node (:wat::core::Option/expect (:wat::core::get ch (:wat::core::+ idx 2)) "deletion-edit: type")
                    off       (:wat::fix::node-start-offset name-node lines)
                    old       (:wat::fix::fix-text-span-text (:wat::core::ast-span name-node) (:wat::core::ast-end-span type-node) lines src)]
    (:wat::core::Tuple off old "")))

;; record-edits — 0+ edits for one defrecord/defsurface form's `causes` field.
(:wat::core::defn :user::record-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::let [head (:wat::fix::head-name node)]
    (:wat::core::if (:wat::core::if (:user::decl-head? head) (:user::decl-name-keyword? node) false)
      (:wat::core::let [ch0  (:wat::core::ast->children node)
                        name (:wat::core::ast-name (:wat::core::Option/expect (:wat::core::get ch0 1) "record-edits: decl name"))
                        fv   (:user::field-vector-of ch0 head)
                        fch  (:wat::core::ast->children fv)]
        (:wat::core::match (:user::causes-index fch 0)
          [:wat::core::Option.None {} (:wat::core::Vector :- [:wat::fix::Edit])]
          [:wat::core::Option.Some {:value idx}
            (:wat::core::if (:user::rename-target? name)
              (:wat::core::let [name-node (:wat::core::Option/expect (:wat::core::get fch idx) "record-edits: causes node")
                                off       (:wat::fix::node-start-offset name-node lines)]
                (:wat::core::Vector :- [:wat::fix::Edit] (:wat::core::Tuple off "causes" "errors")))
              (:wat::core::if (:user::cause-add-target? name)
                (:wat::core::let [del      (:user::deletion-edit fch idx lines src)
                                  last-idx (:wat::core::- (:wat::core::length fch) 1)
                                  last     (:wat::core::Option/expect (:wat::core::get fch last-idx) "record-edits: last field")
                                  ins-off  (:wat::fix::node-end-offset last lines)
                                  ins      (:wat::core::Tuple ins-off "" "\n   cause <- :wat::core::Error")]
                  (:wat::core::Vector :- [:wat::fix::Edit] del ins))
                (:wat::core::Vector :- [:wat::fix::Edit] (:user::deletion-edit fch idx lines src))))]))
      (:wat::core::Vector :- [:wat::fix::Edit]))))

;; ── Fault/of's `:causes (:wat::core::Vector :- [:wat::core::Error])` kwarg pair ─────────────
;; The ONE non-declaration site. Deleted wherever a list carries this exact keyword+value pair
;; (measured: exactly one occurrence in the eleven files, inside Fault/of's quasiquote body).

(:wat::core::defn :user::kwarg-causes-index
  [ch    <- (:wat::core::Vector :- [:wat::WatAST])
   i     <- :wat::core::i64
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::if (:wat::core::>= (:wat::core::+ i 1) (:wat::core::length ch))
    :wat::core::Option.None
    (:wat::core::let [k (:wat::core::Option/expect (:wat::core::get ch i) "kwarg-causes-index: k")
                      v (:wat::core::Option/expect (:wat::core::get ch (:wat::core::+ i 1)) "kwarg-causes-index: v")]
      (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind k) "keyword")
                        (:wat::core::= (:wat::core::ast-name k) ":causes")
                        false)
        (:wat::core::if (:wat::core::= (:wat::fix::fix-text-span-text (:wat::core::ast-span v) (:wat::core::ast-end-span v) lines src)
                          "(:wat::core::Vector :- [:wat::core::Error])")
          (:wat::core::Option.Some {:value i})
          (:user::kwarg-causes-index ch (:wat::core::+ i 1) lines src))
        (:user::kwarg-causes-index ch (:wat::core::+ i 1) lines src)))))

(:wat::core::defn :user::fault-of-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::match (:user::kwarg-causes-index ch 0 lines src)
        [:wat::core::Option.None {} (:wat::core::Vector :- [:wat::fix::Edit])]
        [:wat::core::Option.Some {:value idx}
          (:wat::core::let [kn  (:wat::core::Option/expect (:wat::core::get ch idx) "fault-of-edits: k")
                            vn  (:wat::core::Option/expect (:wat::core::get ch (:wat::core::+ idx 1)) "fault-of-edits: v")
                            off (:wat::fix::node-start-offset kn lines)
                            old (:wat::fix::fix-text-span-text (:wat::core::ast-span kn) (:wat::core::ast-end-span vn) lines src)]
            (:wat::core::Vector :- [:wat::fix::Edit] (:wat::core::Tuple off old "")))]))
    (:wat::core::Vector :- [:wat::fix::Edit])))

;; walk / walk-children — full-tree recursive scan (every node, not just top-level forms).
(:wat::core::defn :user::walk
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::concat
    (:wat::core::concat (:user::record-edits node lines src) (:user::fault-of-edits node lines src))
    (:user::walk-children (:wat::core::ast->children node) lines src)))

(:wat::core::defn :user::walk-children
  [nodes <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? nodes)
    (:wat::core::Vector :- [:wat::fix::Edit])
    (:wat::core::concat
      (:user::walk (:wat::core::Option/expect (:wat::core::get nodes 0) "walk-children: head") lines src)
      (:user::walk-children (:wat::core::into [] (:wat::core::rest nodes)) lines src))))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines (:wat::string::split src "\n")
                    tree  (:wat::core::match (:wat::core::read-string src)
                            [:wat::core::ReadOutcome.Forms {:forms __forms} __forms]
                            [:wat::core::ReadOutcome.Malformed {:cause __cause}
                              (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
                    forms (:wat::core::ast->children tree)
                    eds   (:user::walk-children forms lines src)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse (:wat::core::sort eds)))))

;; ── driver: rewrite each path given on stdin (a JSON/EDN array of strings) ─────────────────
(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::Option/expect (:wat::core::get paths 0) "apply-each: path")]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[error-floor-drops-causes] " path))
        (:user::apply-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum]
      [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
