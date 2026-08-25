;; wat-scripts/fixes/rule-record-to-defrule.wat — arc 278 migration: the `where`-expressivity
;; corpus (wat-scripts/perf/grid/where-*.wat) from hand-built `Rule` records to `defrule`.
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; WHY: every rule in the corpus is built via a higher-order construction path no user would write —
;;   (:wat::core::defn :ns::rule-NAME [] -> :wat::rete::Rule
;;     (:wat::core::let [conds (:wat::core::quasiquote C) where-c (:wat::core::quasiquote (:wat::rete::where P))
;;                       ins   (:wat::core::quasiquote I)]
;;       (:wat::rete::Rule :name "NAME" :lhs (:wat::core::PersistentVector conds where-c)
;;                                      :rhs (:wat::core::PersistentVector ins))))
;; — instead of the user-facing `defrule` surface:
;;   (:wat::rete::defrule :NAME :when [C (:wat::rete::where P)] :then I)
;;
;; SURVEY (2026-08-01, all 9 pairs / 98 rows): exactly TWO shapes, discriminated by the arity of the
;; rule-defn's `let` bindings vector:
;;   Shape A (6 bindings: conds, where-c, ins ALL inline per rule) — where-shapes/nesting/multivar/numeric.
;;   Shape B (2 bindings: where-c ONLY; conds/ins are HOISTED into file-level 0-arg helpers
;;            `:ns::conds`/`:ns::ins`, called at the Rule-construction site as `(:ns::conds)`/`(:ns::ins)`)
;;            — where-boolean/string/collection/record/control.
;; Any OTHER bindings arity is a STOP (assertion-failed!), never a silent skip or a hand-fix.
;;
;; THE NAME TRAP: `run-row` prints `(:wat::rete::Rule/name rule)`, and that string is part of the
;; output line the Clojure-oracle diff compares. `defrule` derives the Rule's name from the DEFN
;; SYMBOL (colon-stripped, namespace INCLUDED) — so naively keeping the symbol `:ns::rule-NAME` would
;; print "ns::rule-NAME" instead of the corpus's short "NAME", breaking every row's diff.
;;
;; ⛔ THE "BARE IS LEGAL" PREMISE BELOW IS NO LONGER TRUE (arc 278, 2026-08-02). It was true when
;; this migration ran and is kept verbatim because it is why the migration did what it did — but a
;; bare top-level name is now a located `Registration::Unnamespaced` error at every registration
;; door, and the probe it cites has been RETIRED (its subject died with the premise; `Rule/name` is
;; covered by tests/rete/probe_arc278_5{a,b} and nine grid axes). The correction is its own recorded
;; migration: `wat-scripts/fixes/namespace-defrule-names.wat`, which namespaced these very names and
;; wrapped the `Rule/name` read in a `rule-display-name` derivation so the printed row label stayed
;; byte-identical. Read the two together; do NOT act on the paragraph below alone.
;;
;; RESOLUTION: across all 98 rows, the existing `:name "NAME"` string is EXACTLY the defn symbol's
;; suffix after "rule-" (verified by survey, not assumed) — and a BARE (non-namespaced) top-level
;; symbol is legal wat (probed: `(:wat::rete::defrule :arith ...)` derives Rule/name = "arith" exactly,
;; wat-scripts/scratch-pad/probe-bare-defrule-name.wat). So the migration names each defrule symbol
;; the BARE "NAME" (no `ns::`, no `rule-` prefix) read straight off the OLD Rule's `:name` string —
;; never off the old defn symbol — so `run-row`/`build-rules` need NOT change their printing logic at
;; all: `(:wat::rete::Rule/name rule)` comes out byte-identical. Only the CALL SITE in `build-rules`
;; (`(:ns::rule-NAME)` -> `(:NAME)`) needs updating, done via `rename-keyword-exact` in a second pass.
;;
;; Two-pass composition:
;;   pass 1 — whole-form span replacement: each rule-defn's ENTIRE top-level form is replaced by its
;;            defrule equivalent (extraction is span-text-copy from the ORIGINAL node, so any
;;            comments/formatting INSIDE a copied sub-form survive byte-identical).
;;   pass 2 — `rename-keyword-exact old-defn-symbol bare-name` for every migrated rule, over the
;;            pass-1 output — reaches the `build-rules` cond-arm call site.
;;
;; Dry-run on a /tmp copy + diff, THEN apply:
;;   printf '["wat-scripts/perf/grid/where-shapes.wat" ...]' \
;;     | ./target/release/wat ./wat-scripts/fixes/rule-record-to-defrule.wat
;; Idempotent: a re-run finds no `-> :wat::rete::Rule` rule-defn forms left (they are all now
;; `defrule` calls, a "list" headed by `:wat::rete::defrule`, not `:wat::core::defn`) — zero edits.

;; ── node-text: verbatim source substring for a node's span ─────────────────────────────────
(:wat::core::defn :user::node-text
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> :wat::core::String
  (:wat::string::subs src
    (:wat::fix::node-start-offset node lines)
    (:wat::fix::node-end-offset node lines)))

;; quasi-text — node is `(:wat::core::quasiquote FORM)`; returns FORM's verbatim source text.
(:wat::core::defn :user::quasi-text
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   src   <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [ch   (:wat::core::ast->children node)
                    form (:wat::core::Option/expect (:wat::core::get ch 1) "quasi-text: form")]
    (:user::node-text form lines src)))

;; ends-with? — s ends with suf (bare string compare; suf shorter-or-equal-length required).
(:wat::core::defn :user::ends-with? [s <- :wat::core::String suf <- :wat::core::String] -> :wat::core::bool
  (:wat::core::let [ls   (:wat::string::length s)
                    lsuf (:wat::string::length suf)]
    (:wat::core::if (:wat::core::< ls lsuf)
      false
      (:wat::core::= (:wat::string::subs s (:wat::core::i64::- ls lsuf) ls) suf))))

;; ── rule-defn? — a top-level `(:wat::core::defn NAME [] -> :wat::rete::Rule BODY)` form.
;; Gated on rettype ONLY (":wat::rete::Rule"); the conds/ins helpers return :wat::WatAST and are
;; never mistaken for a rule-defn.
(:wat::core::defn :user::rule-defn? [f <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind f) "list")
    (:wat::core::let [ch (:wat::core::ast->children f)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 6)
        (:wat::core::let [head    (:wat::core::first ch)
                          rettype (:wat::core::Option/expect (:wat::core::get ch 4) "rule-defn?: rettype")]
          (:wat::core::if (:wat::core::= (:wat::core::ast-name head) ":wat::core::defn")
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind rettype) "keyword")
              (:wat::core::= (:wat::core::ast-name rettype) ":wat::rete::Rule")
              false)
            false))
        false))
    false))

;; helper-defn? — a top-level `(:wat::core::defn NAME [] -> :wat::WatAST BODY)` whose NAME ends
;; with `suffix` (e.g. "::conds" / "::ins") — the Shape-B hoisted-condition/insert helper.
(:wat::core::defn :user::helper-defn? [f <- :wat::WatAST suffix <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind f) "list")
    (:wat::core::let [ch (:wat::core::ast->children f)]
      (:wat::core::if (:wat::core::= (:wat::core::length ch) 6)
        (:wat::core::let [head    (:wat::core::first ch)
                          namekw  (:wat::core::Option/expect (:wat::core::get ch 1) "helper-defn?: name")
                          rettype (:wat::core::Option/expect (:wat::core::get ch 4) "helper-defn?: rettype")]
          (:wat::core::if (:wat::core::= (:wat::core::ast-name head) ":wat::core::defn")
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind namekw) "keyword")
              (:wat::core::if (:user::ends-with? (:wat::core::ast-name namekw) suffix)
                (:wat::core::if (:wat::core::= (:wat::core::ast-kind rettype) "keyword")
                  (:wat::core::= (:wat::core::ast-name rettype) ":wat::WatAST")
                  false)
                false)
              false)
            false))
        false))
    false))

;; find-helper — first top-level form matching helper-defn? for `suffix`; None if absent
;; (Shape-A-only files have neither `::conds` nor `::ins` helpers).
(:wat::core::defn :user::find-helper
  [forms  <- (:wat::core::Vector :- [:wat::WatAST])
   suffix <- :wat::core::String]
  -> (:wat::core::Option :wat::WatAST)
  (:wat::core::if (:wat::core::empty? forms)
    :wat::core::None
    (:wat::core::let [f (:wat::core::first forms) tl (:wat::core::rest forms)]
      (:wat::core::if (:user::helper-defn? f suffix)
        (:wat::core::Some f)
        (:user::find-helper tl suffix)))))

;; helper-text-opt — the verbatim FORM text inside a helper's `(quasiquote FORM)` body, if the
;; helper exists in this file.
(:wat::core::defn :user::helper-text-opt
  [forms  <- (:wat::core::Vector :- [:wat::WatAST])
   lines  <- (:wat::core::Vector :- [:wat::core::String])
   src    <- :wat::core::String
   suffix <- :wat::core::String]
  -> (:wat::core::Option :wat::core::String)
  (:wat::core::match (:user::find-helper forms suffix)
    ((:wat::core::Some h)
      (:wat::core::let [ch   (:wat::core::ast->children h)
                        body (:wat::core::Option/expect (:wat::core::get ch 5) "helper-text-opt: body")]
        (:wat::core::Some (:user::quasi-text body lines src))))
    (:wat::core::None :wat::core::None)))

;; build-defrule-text — the replacement source text for one migrated rule.
(:wat::core::defn :user::build-defrule-text
  [name-str  <- :wat::core::String
   cond-text <- :wat::core::String
   where-text <- :wat::core::String
   ins-text  <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::String/concat
    (:wat::core::String/concat "(:wat::rete::defrule :" name-str)
    (:wat::core::String/concat "\n  :when\n  ["
      (:wat::core::String/concat cond-text
        (:wat::core::String/concat " "
          (:wat::core::String/concat where-text
            (:wat::core::String/concat "]\n  :then\n  "
              (:wat::core::String/concat ins-text ")"))))))))

;; rule-edit — for a rule-defn form, the one whole-span replacement edit; [] for anything else.
;; STOPS (assertion-failed!) on a bindings arity other than 2 (Shape B) / 6 (Shape A) — a shape the
;; survey did not find, never silently skipped and never hand-fixed.
(:wat::core::defn :user::rule-edit
  [f              <- :wat::WatAST
   lines          <- (:wat::core::Vector :- [:wat::core::String])
   src            <- :wat::core::String
   conds-text-opt <- (:wat::core::Option :wat::core::String)
   ins-text-opt   <- (:wat::core::Option :wat::core::String)]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:user::rule-defn? f)
    (:wat::core::let
      [ch        (:wat::core::ast->children f)
       body      (:wat::core::Option/expect (:wat::core::get ch 5) "rule-edit: body")
       bch       (:wat::core::ast->children body)
       bindings  (:wat::core::Option/expect (:wat::core::get bch 1) "rule-edit: bindings")
       rule-call (:wat::core::Option/expect (:wat::core::get bch 2) "rule-edit: rule-call")
       rcch      (:wat::core::ast->children rule-call)
       name-node (:wat::core::Option/expect (:wat::core::get rcch 2) "rule-edit: name-node")
       name-str  (:wat::core::ast-name name-node)
       bindch    (:wat::core::ast->children bindings)
       n         (:wat::core::length bindch)
       off       (:wat::fix::node-start-offset f lines)
       len       (:wat::core::i64::- (:wat::fix::node-end-offset f lines) off)]
      (:wat::core::if (:wat::core::= n 6)
        ;; Shape A — conds/where-c/ins all bound inline in this rule's own let.
        (:wat::core::let
          [conds-val  (:wat::core::Option/expect (:wat::core::get bindch 1) "rule-edit: conds-val")
           wherec-val (:wat::core::Option/expect (:wat::core::get bindch 3) "rule-edit: wherec-val")
           ins-val    (:wat::core::Option/expect (:wat::core::get bindch 5) "rule-edit: ins-val")
           new-text   (:user::build-defrule-text name-str
                        (:user::quasi-text conds-val lines src)
                        (:user::quasi-text wherec-val lines src)
                        (:user::quasi-text ins-val lines src))]
          (:wat::core::Vector :wat::fix::Edit (:wat::core::Tuple off len new-text)))
        (:wat::core::if (:wat::core::= n 2)
          ;; Shape B — only where-c is local; conds/ins come from the file-level helpers.
          (:wat::core::let
            [wherec-val (:wat::core::Option/expect (:wat::core::get bindch 1) "rule-edit: wherec-val")
             cond-text  (:wat::core::Option/expect conds-text-opt
                          (:wat::core::String/concat "rule-record-to-defrule: shape-B rule "
                            (:wat::core::String/concat name-str
                              " needs a file-level `::conds` helper but none was found")))
             ins-text   (:wat::core::Option/expect ins-text-opt
                          (:wat::core::String/concat "rule-record-to-defrule: shape-B rule "
                            (:wat::core::String/concat name-str
                              " needs a file-level `::ins` helper but none was found")))
             new-text   (:user::build-defrule-text name-str cond-text
                          (:user::quasi-text wherec-val lines src) ins-text)]
            (:wat::core::Vector :wat::fix::Edit (:wat::core::Tuple off len new-text)))
          ;; Neither shape — STOP. Never a silent skip, never a hand-fix.
          (:wat::kernel::assertion-failed!
            (:wat::core::String/concat "rule-record-to-defrule: unrecognized let-bindings arity "
              (:wat::core::String/concat (:wat::core::i64::to-string n)
                (:wat::core::String/concat " in rule " name-str)))
            :wat::core::None :wat::core::None))))
    (:wat::core::Vector :wat::fix::Edit)))

(:wat::core::defn :user::collect-edits
  [forms          <- (:wat::core::Vector :- [:wat::WatAST])
   lines          <- (:wat::core::Vector :- [:wat::core::String])
   src            <- :wat::core::String
   conds-text-opt <- (:wat::core::Option :wat::core::String)
   ins-text-opt   <- (:wat::core::Option :wat::core::String)]
  -> (:wat::core::Vector :- [:wat::fix::Edit])
  (:wat::core::if (:wat::core::empty? forms)
    (:wat::core::Vector :wat::fix::Edit)
    (:wat::core::concat
      (:user::rule-edit (:wat::core::first forms) lines src conds-text-opt ins-text-opt)
      (:user::collect-edits (:wat::core::rest forms) lines src conds-text-opt ins-text-opt))))

;; ── renames: build-rules' `(:ns::rule-NAME)` call site -> `(:NAME)` ─────────────────────────
(:wat::core::defn :user::rule-rename
  [f <- :wat::WatAST]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::if (:user::rule-defn? f)
    (:wat::core::let
      [ch        (:wat::core::ast->children f)
       name-kw   (:wat::core::Option/expect (:wat::core::get ch 1) "rule-rename: name-kw")
       old       (:wat::core::ast-name name-kw)
       body      (:wat::core::Option/expect (:wat::core::get ch 5) "rule-rename: body")
       bch       (:wat::core::ast->children body)
       rule-call (:wat::core::Option/expect (:wat::core::get bch 2) "rule-rename: rule-call")
       rcch      (:wat::core::ast->children rule-call)
       name-node (:wat::core::Option/expect (:wat::core::get rcch 2) "rule-rename: name-node")
       new       (:wat::core::String/concat ":" (:wat::core::ast-name name-node))]
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::String]) (:wat::core::Tuple old new)))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::String]))))

(:wat::core::defn :user::collect-renames
  [forms <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? forms)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::String]))
    (:wat::core::concat
      (:user::rule-rename (:wat::core::first forms))
      (:user::collect-renames (:wat::core::rest forms)))))

(:wat::core::defn :user::apply-renames
  [text    <- :wat::core::String
   renames <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])]
  -> :wat::core::String
  (:wat::core::if (:wat::core::empty? renames)
    text
    (:wat::core::let [p   (:wat::core::first renames)
                      old (:wat::core::first p)
                      new (:wat::core::second p)]
      (:user::apply-renames (:wat::fix::rename-keyword-exact old new text) (:wat::core::rest renames)))))

;; ── per-file migrate ─────────────────────────────────────────────────────────────────────────
(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines          (:wat::string::split src "\n")
     tree           (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
     forms          (:wat::core::ast->children tree)
     conds-text-opt (:user::helper-text-opt forms lines src "::conds")
     ins-text-opt   (:user::helper-text-opt forms lines src "::ins")
     edits          (:user::collect-edits forms lines src conds-text-opt ins-text-opt)
     renames        (:user::collect-renames forms)
     text1          (:wat::fix::fix-text-apply src (:wat::core::reverse (:wat::core::sort edits)))
     text2          (:user::apply-renames text1 renames)]
    text2))

;; ── driver: rewrite each path given on stdin (a JSON array of strings) ──────────────────────
(:wat::core::defn :user::rewrite-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [p (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file p (:user::migrate (:wat::io::read-file p)))
        (:wat::kernel::println (:wat::core::String/concat "[defrule] " p))
        (:user::rewrite-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [paths (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
    (:user::rewrite-each paths)))
