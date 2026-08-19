;; wat-scripts/fixes/rete-where-per-type-spelling.wat — arc 278 #57 S6, the law-A migration.
;;
;; "The entire rete query language may only be composed from rete primitives." — the builder
;;
;; Rewrites BUCKET A / BUCKET B head spellings — ONLY inside a `(:wat::rete::where <expr>)` form
;; — to their rete-prefixed twins, per `src/rete/vocabulary.rs`'s RETE_OPS table:
;;   BUCKET A: :wat::core::X   -> :wat::rete::core::X    (55 heads — core_name appears EXACTLY once)
;;   BUCKET B: :wat::holon::X  -> :wat::rete::holon::X   (4 heads — ditto, note holon:: not core::)
;; The table below is not invented: it is every `core_name` row in RETE_OPS whose bare string
;; occurs EXACTLY ONCE across the whole table, grep-derived (2026-08-05):
;;   grep -n 'core_name:' src/rete/vocabulary.rs | sed -E 's/.*core_name: *"([^"]+)",?/\1/' \
;;     | sort | uniq -c   →   count==1 rows split by :wat::core:: / :wat::holon:: prefix.
;; Bucket C (multi-twin — :wat::core::=, :wat::core::not=, :wat::core::first — each with 4-6/3
;; rows) and any head ABSENT from RETE_OPS entirely (:wat::core::>, :wat::core::<, :wat::core::>=,
;; :wat::core::+ — no bare/generic row exists, only the per-type i64::/f64:: rows do) are JUDGEMENT
;; sites: this codemod's rename table does not mention them, so a keyword with one of those exact
;; names is left byte-identical, by construction (the lookup returns None, no edit is emitted).
;;
;; SCOPE — why this is a form-tree codemod and not sed: `:wat::core::`/`:wat::holon::` occurrences
;; OUTSIDE a `where` form are legitimate core-level code elsewhere in the same file and must not
;; move. `where-list?` gates the rename table to keyword leaves reached only by descending through
;; a `(:wat::rete::where …)` list's own children — `outer-edits` walks everywhere else with the
;; table absent, so nothing outside a `where` can ever match.
;;
;; ACCUMULATOR FENCE — checked, not found: `wat/rete.wat`'s `compile-condition` `is-accumulate`
;; branch (arc 278 8-custom) fences the acc-form head on `pure? ∧ deterministic?` only. It never
;; calls `:wat::rete::primitive?` / raises `Axis::RetePrimitive` — that check exists at exactly one
;; call site in the whole file, the `where` branch (`wat/rete.wat:687`, `grep -n "primitive?"
;; wat/rete.wat` confirms). So there is no accumulator RetePrimitive fence to scope into today;
;; this codemod scopes to `where` only, as instructed for the case where none is found.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat-scripts/perf/grid/where-boolean.wat" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/rete-where-per-type-spelling.wat
;; Idempotent (re-run = 0 changes): the table's `new` spellings (`:wat::rete::core::…`,
;; `:wat::rete::holon::…`) never appear as `old` keys, so a second pass finds nothing to rewrite.

;; ── the rename table — the PURE-RENAME subset only, exact whole-name match only ───────────────
;;
;; ⛔ FALLBACK-CLASS OPS ARE DELIBERATELY ABSENT, AND MUST STAY ABSENT. 17 rows were removed from
;; this table after they were measured, not reasoned about:
;;
;;     :wat::core::{i64,f64}::{+ - * /}  ·  i64::{mod quot rem}  ·  string::subs
;;     {PersistentVector,Vector,List}/get   ·   holon::{cosine dot}
;;
;; Every one of these is `OpClass::Fallback` in `src/rete/vocabulary.rs`. Their rete surface takes
;; FOUR arguments — the two real operands plus a MANDATORY `:undefined <value>` kwarg — because
;; that declared fallback is what makes a partial op total. So they are NOT a rename at all; they
;; are a CALL-SHAPE change, and the fallback value is a per-call-site decision about what the
;; expression should mean when its domain hole is hit. A machine must not pick that.
;;
;; PROVEN BY RUN, both directions (2026-08-05):
;;     (:wat::rete::core::i64::+ 1 2)                -> ArityMismatch: expected 4 argument(s); got 2
;;     (:wat::rete::core::i64::+ 1 2 :undefined -1)  -> 3
;;
;; A spelling-only rename of these compiles past the fence and then dies at dispatch. An earlier
;; run of this codemod DID rename them — 77 sites across the corpus, every one an ArityMismatch —
;; and the whole application was reverted. The brief that authorised it called all 55 heads "pure
;; textual"; it was wrong, and the design stone's own "The call shape — RULED" section already said
;; so. Re-adding these rows re-creates 77 broken sites.
;;
;; Also absent, for a different reason — MULTI-TWIN heads, where the operand TYPE picks the module:
;;     :wat::core::=  /  :wat::core::not=   (6 twins each: i64 f64 string bool keyword enum)
;;     :wat::core::first                    (3 twins: PersistentVector Vector List)
;; and heads with NO twin at all, which are per-type by RULING and never get a generic form:
;;     :wat::core::{> < >= +}
;; The lookup simply returns None for all of these, so they are left byte-identical for hand work.
(:wat::core::defn :user::rename-table [] -> :wat::core::Vector<(wat::core::String,wat::core::String)>
  (:wat::core::Vector :(wat::core::String,wat::core::String)
    (:wat::core::Tuple ":wat::core::and" ":wat::rete::core::and")
    (:wat::core::Tuple ":wat::core::bool::to-string" ":wat::rete::core::bool::to-string")
    (:wat::core::Tuple ":wat::core::cond" ":wat::rete::core::cond")
    (:wat::core::Tuple ":wat::core::f64::<" ":wat::rete::core::f64::<")
    (:wat::core::Tuple ":wat::core::f64::<=" ":wat::rete::core::f64::<=")
    (:wat::core::Tuple ":wat::core::f64::=" ":wat::rete::core::f64::=")
    (:wat::core::Tuple ":wat::core::f64::>" ":wat::rete::core::f64::>")
    (:wat::core::Tuple ":wat::core::f64::>=" ":wat::rete::core::f64::>=")
    (:wat::core::Tuple ":wat::core::f64::not=" ":wat::rete::core::f64::not=")
    (:wat::core::Tuple ":wat::core::f64::to-string" ":wat::rete::core::f64::to-string")
    (:wat::core::Tuple ":wat::core::filter" ":wat::rete::core::filter")
    (:wat::core::Tuple ":wat::core::fn" ":wat::rete::core::fn")
    (:wat::core::Tuple ":wat::core::foldl" ":wat::rete::core::foldl")
    ;; Arc 118.B6b: `:wat::core::foldr`/`:wat::rete::core::foldr` retired — the pair that used to
    ;; live here is gone from both sides (core's verb AND its rete vocabulary row), so this
    ;; migration table no longer names it. See DESIGN-STONE-118.B6b-retire-foldr.md.
    (:wat::core::Tuple ":wat::core::i64::<" ":wat::rete::core::i64::<")
    (:wat::core::Tuple ":wat::core::i64::<=" ":wat::rete::core::i64::<=")
    (:wat::core::Tuple ":wat::core::i64::=" ":wat::rete::core::i64::=")
    (:wat::core::Tuple ":wat::core::i64::>" ":wat::rete::core::i64::>")
    (:wat::core::Tuple ":wat::core::i64::>=" ":wat::rete::core::i64::>=")
    (:wat::core::Tuple ":wat::core::i64::not=" ":wat::rete::core::i64::not=")
    (:wat::core::Tuple ":wat::core::i64::to-f64" ":wat::rete::core::i64::to-f64")
    (:wat::core::Tuple ":wat::core::i64::to-string" ":wat::rete::core::i64::to-string")
    (:wat::core::Tuple ":wat::core::if" ":wat::rete::core::if")
    (:wat::core::Tuple ":wat::core::let" ":wat::rete::core::let")
    (:wat::core::Tuple ":wat::core::map" ":wat::rete::core::map")
    (:wat::core::Tuple ":wat::core::match" ":wat::rete::core::match")
    (:wat::core::Tuple ":wat::core::not" ":wat::rete::core::not")
    (:wat::core::Tuple ":wat::core::or" ":wat::rete::core::or")
    (:wat::core::Tuple ":wat::core::PersistentMap/contains-key?" ":wat::rete::core::PersistentMap/contains-key?")
    (:wat::core::Tuple ":wat::core::PersistentVector/contains?" ":wat::rete::core::PersistentVector/contains?")
    (:wat::core::Tuple ":wat::core::PersistentVector/length" ":wat::rete::core::PersistentVector/length")
    (:wat::core::Tuple ":wat::core::reduce" ":wat::rete::core::reduce")
    (:wat::core::Tuple ":wat::core::String/concat" ":wat::rete::core::String/concat")
    (:wat::core::Tuple ":wat::core::String/contains?" ":wat::rete::core::String/contains?")
    (:wat::core::Tuple ":wat::core::String/empty?" ":wat::rete::core::String/empty?")
    (:wat::core::Tuple ":wat::core::String/ends-with?" ":wat::rete::core::String/ends-with?")
    (:wat::core::Tuple ":wat::core::string::length" ":wat::rete::core::string::length")
    (:wat::core::Tuple ":wat::core::String/starts-with?" ":wat::rete::core::String/starts-with?")
    (:wat::core::Tuple ":wat::core::string::to-lowercase" ":wat::rete::core::string::to-lowercase")
    (:wat::core::Tuple ":wat::core::string::trim" ":wat::rete::core::string::trim")
    (:wat::core::Tuple ":wat::holon::coincident?" ":wat::rete::holon::coincident?")
    (:wat::core::Tuple ":wat::holon::presence?" ":wat::rete::holon::presence?")))

;; rename-lookup — Some(new) if name exactly equals some pair's old; else None. No prefix/boundary
;; logic (unlike :wat::fix::rename-keyword-prefix) — every table entry is a WHOLE keyword name.
(:wat::core::defn :user::rename-lookup
  [name  <- :wat::core::String
   table <- :wat::core::Vector<(wat::core::String,wat::core::String)>]
  -> (:wat::core::Option :wat::core::String)
  (:wat::core::if (:wat::core::empty? table)
    :wat::core::None
    (:wat::core::let [pair (:wat::core::first table)
                      old  (:wat::core::first pair)
                      new  (:wat::core::second pair)]
      (:wat::core::if (:wat::core::= name old)
        (:wat::core::Some new)
        (:user::rename-lookup name (:wat::core::rest table))))))

;; ── inside a `where` subtree: recurse everywhere, rename any matching keyword leaf ────────────
(:wat::core::defn :user::inside-where-edits
  [node  <- :wat::WatAST
   table <- :wat::core::Vector<(wat::core::String,wat::core::String)>
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::fix::structural? node)
    (:user::inside-where-edits-walk (:wat::core::ast->children node) table lines)
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
      (:wat::core::match (:user::rename-lookup (:wat::core::ast-name node) table)
        ((:wat::core::Some new)
          (:wat::core::let [off     (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
                            old-len (:wat::core::string::length (:wat::core::ast-name node))]
            (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)
              (:wat::core::Tuple off old-len new))))
        (:wat::core::None (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))))
      (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String)))))

(:wat::core::defn :user::inside-where-edits-walk
  [items <- :wat::core::Vector<wat::WatAST>
   table <- :wat::core::Vector<(wat::core::String,wat::core::String)>
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
    (:wat::core::concat
      (:user::inside-where-edits (:wat::core::first items) table lines)
      (:user::inside-where-edits-walk (:wat::core::rest items) table lines))))

;; where-list? — true iff node is a List whose head keyword is exactly ":wat::rete::where".
(:wat::core::defn :user::where-list?
  [node <- :wat::WatAST
   ch   <- :wat::core::Vector<wat::WatAST>]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::if (:wat::core::empty? ch)
      false
      (:wat::core::let [head (:wat::core::first ch)]
        (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
          (:wat::core::= (:wat::core::ast-name head) ":wat::rete::where")
          false)))
    false))

;; ── outer walk: everywhere OUTSIDE a where, no renames — only switches into inside-where-edits
;; when a `(:wat::rete::where …)` list is found, and only for THAT list's own children ───────────
(:wat::core::defn :user::outer-edits
  [node  <- :wat::WatAST
   table <- :wat::core::Vector<(wat::core::String,wat::core::String)>
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:user::where-list? node ch)
        (:user::inside-where-edits-walk ch table lines)
        (:user::outer-edits-walk ch table lines)))
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))))

(:wat::core::defn :user::outer-edits-walk
  [items <- :wat::core::Vector<wat::WatAST>
   table <- :wat::core::Vector<(wat::core::String,wat::core::String)>
   lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::Vector<(wat::core::i64,wat::core::i64,wat::core::String)>
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :(wat::core::i64,wat::core::i64,wat::core::String))
    (:wat::core::concat
      (:user::outer-edits (:wat::core::first items) table lines)
      (:user::outer-edits-walk (:wat::core::rest items) table lines))))

;; ── per-file migrate: parse → collect scoped edits → splice ORIGINAL text (comment-faithful) ──
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

;; ── driver: read the EDN path vector from stdin, rewrite each file in place ───────────────────
(:wat::core::defn :user::apply-each
  [paths <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[rete-where-per-type-spelling] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
