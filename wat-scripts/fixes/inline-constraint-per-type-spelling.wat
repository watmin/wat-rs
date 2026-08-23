;; wat-scripts/fixes/inline-constraint-per-type-spelling.wat — arc 278 #84, the law-A migration
;; for the INLINE ALPHA CONSTRAINT.
;;
;; ":wat::core::{=,<,>,...} are illegal — zero questions — enforce it." — the builder
;;
;; SIBLING of `rete-where-per-type-spelling.wat` (#57 S6), same machinery, DIFFERENT SCOPE.
;; That one rewrote heads inside a `(:wat::rete::where <expr>)`. This one rewrites the OTHER
;; expression surface on the LHS, the one law A never reached: a constraint clause sitting inside
;; a fact PATTERN, beside its binds —
;;
;;     (:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::core::< ?c 20))
;;                                                                  ^^^^^^^^^^^^^^ refused
;;
;; `compile-condition` (wat/rete.wat) branches on where/not/exists/accumulate and only two carry a
;; fence; a keyword-headed constraint matches none, so it fell to the fact-pattern branch, whose
;; children are classified by a SEPARATE grammar in Rust (`matcher.rs`'s `classify_rete_clause`).
;; See `DESIGN-STONE-inline-constraint-admits-non-rete.md`.
;;
;; SCOPE — why this is a form-tree codemod and not sed: `(:wat::core::= …)` occurrences OUTSIDE a
;; rule are ordinary core-level code in the same files (driver code, `defn` bodies, assertions) and
;; must not move. `defrule-list?` gates the rename table to keyword leaves reached only by
;; descending through a `(:wat::rete::defrule …)` form; `outer-edits` walks everywhere else with
;; the table absent, so nothing outside a defrule can ever match. Descending the WHOLE defrule
;; (rather than hunting the `:when` vector) is deliberate and safe: it reaches constraints nested
;; inside `(:wat::rete::and …)` / `(:wat::rete::or …)` combinators, and the `:then` RHS constructs
;; facts rather than comparing, so it has no constraint heads to move. Heads already inside a
;; `where` are rete-spelled after #57 and so are not in the table's key set.
;;
;; ⛔⛔ THE TABLE IS WORKLIST-DERIVED, NOT A GENERAL RULE — READ THIS BEFORE REUSING IT.
;; The right per-type twin depends on the OPERAND'S TYPE, which a form-tree rewrite cannot know.
;; This table is not inferred here; it is transcribed from what the CHECKER computed for THIS
;; worklist, which resolves each operand through: a `:field` reference -> its declared type; a
;; `?var` -> the `(?v <- :field)` bind naming it, ANYWHERE in the rule; a literal -> its own type.
;; Measured 2026-08-06 over all 16 files: every `<` and `>` site is i64, every `=` site is string.
;;
;; So this codemod PROPOSES and the checker DISPOSES. If it ever emits a wrong twin on a file
;; outside that worklist, `ConstraintTypeMismatch` makes it a RED BUILD, not a silent lie —
;; `(:wat::rete::core::i64::= :name "x")` on a String field is refused, by name. That gate is what
;; makes a worklist-derived table safe to run; it is NOT safe to trust the table on new input.
;;
;; ⛔ TWO FILES MUST NOT BE MIGRATED — they are the fence's own negative controls, and a rider that
;; "fixes" them has broken the thing they measure:
;;     tests/rete/probe_arc278_inline_constraint_untyped_ordering.wat
;;     tests/rete/probe_arc278_inline_constraint_untyped_equality.wat
;; They are simply absent from the path list you pass on stdin. The census counts 18 files / 38
;; sites; the migration is 16 files / 36 sites, and the difference is exactly those two.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["tests/rete/probe_arc278_northstar_cold_and_windy.wat" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/inline-constraint-per-type-spelling.wat
;; Idempotent (re-run = 0 changes): the table's `new` spellings never appear as `old` keys.

;; ── the rename table — checker-derived for THIS worklist (see the warning above) ──────────────
(:wat::core::defn :user::rename-table [] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::String])
    (:wat::core::Tuple ":wat::core::<" ":wat::rete::core::i64::<")
    (:wat::core::Tuple ":wat::core::>" ":wat::rete::core::i64::>")
    (:wat::core::Tuple ":wat::core::=" ":wat::rete::core::string::=")))

(:wat::core::defn :user::rename-lookup
  [name  <- :wat::core::String
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])]
  -> (:wat::core::Option :wat::core::String)
  (:wat::core::if (:wat::core::empty? table)
    :wat::core::None
    (:wat::core::let [pair (:wat::core::first table)
                      old  (:wat::core::first pair)
                      new  (:wat::core::second pair)]
      (:wat::core::if (:wat::core::= name old)
        (:wat::core::Some new)
        (:user::rename-lookup name (:wat::core::rest table))))))

;; ── inside a defrule: the table is LIVE ───────────────────────────────────────────────────────
(:wat::core::defn :user::inside-rule-edits
  [node  <- :wat::WatAST
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::inside-rule-edits-walk (:wat::core::ast->children node) table lines)
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
      (:wat::core::match (:user::rename-lookup (:wat::core::ast-name node) table)
        ((:wat::core::Some new)
          (:wat::core::let [off     (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                            old-len (:wat::core::string::length (:wat::core::ast-name node))]
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
              (:wat::core::Tuple off old-len new))))
        (:wat::core::None
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])))))

(:wat::core::defn :user::inside-rule-edits-walk
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    (:wat::core::concat
      (:user::inside-rule-edits (:wat::core::first items) table lines)
      (:user::inside-rule-edits-walk (:wat::core::rest items) table lines))))

;; ── the scope gate: is this form a RULE form? ─────────────────────────────────────────────────
;;
;; BOTH spellings, and the second was a real miss caught by the post-apply census, not foreseen:
;; `defrule` is the macro, but `make-rule` is the primitive it expands to — and scratch probes call
;; it DIRECTLY with quoted vectors (`(:wat::rete::make-rule "usr::hot-rule" (quote [...]) ...)`).
;; A gate that knew only `defrule` migrated one site in probe-sift-body-direct.wat and left its
;; sibling, which the re-census reported. The fire is the worklist, including the codemod's own.
(:wat::core::defn :user::defrule-list?
  [node <- :wat::WatAST
   ch   <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::if (:wat::core::empty? ch)
      false
      (:wat::core::let [head (:wat::core::first ch)]
        (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
          (:wat::core::if (:wat::core::= (:wat::core::ast-name head) ":wat::rete::defrule")
            true
            (:wat::core::= (:wat::core::ast-name head) ":wat::rete::make-rule"))
          false)))
    false))

;; ── outside a defrule: walk, but the table never applies ──────────────────────────────────────
(:wat::core::defn :user::outer-edits
  [node  <- :wat::WatAST
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:user::defrule-list? node ch)
        (:user::inside-rule-edits-walk ch table lines)
        (:user::outer-edits-walk ch table lines)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))))

(:wat::core::defn :user::outer-edits-walk
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   table <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
    (:wat::core::concat
      (:user::outer-edits (:wat::core::first items) table lines)
      (:user::outer-edits-walk (:wat::core::rest items) table lines))))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [lines     (:wat::core::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms     (:wat::core::ast->children tree)
                    table     (:user::rename-table)
                    all-edits (:user::outer-edits-walk forms table lines)
                    rev-edits (:wat::core::reverse all-edits)]
    (:wat::fix::fix-text-apply src rev-edits)))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[inline-constraint-per-type-spelling] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
