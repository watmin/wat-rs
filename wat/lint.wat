;; wat/lint.wat — the wat-lint framework.
;;
;; Arc 277 Stone 277.1. A pure-wat linter: a rule is `(form → Vector<Finding>)`;
;; `lint-source` runs form-level rules over every top-level form of every file;
;; `lint-stdlib` is the surface — form-level findings over the real stdlib plus
;; deporder's load-order folded in as rule-zero (report-only).
;;
;; STOP-1 in effect: the auto-fix (replacement AST → write-forms → span offset/len)
;; cannot land cleanly because :wat::core::ast-span returns ONLY the START location
;; (line/col), not the end — so computing old-len for a structural node (the whole
;; ladder form) is not possible with the current substrate primitives. The rule is
;; shipped REPORT-ONLY (fix = None). The auto-fix seam is deferred to 277.1b.
;;
;; The surface:
;;   (:wat::lint::lint-source files) — run rules over Vector<SourceFile>
;;   (:wat::lint::lint-stdlib)       — lint the real stdlib + rule-zero
;;
;; Namespace: :wat::lint::
;;
;; Worked references:
;;   wat/deporder.wat — SourceFile, stdlib-sources, structural? + recursive AST walk
;;   wat/fix.wat      — fix-text-apply + edit shape (the seam)
;;   wat/Record.wat   — :wat::Record::def for the Finding record

;; ─── Typed record: Finding (uncompilable on a wrong shape) ───────────

;; Finding — a lint result.
;; rule:     the rule name (e.g. "nested-if-=-ladder", "load-order")
;; file:     the source file path
;; line:     1-indexed line of the finding
;; col:      1-indexed column of the finding
;; severity: "error" | "warn" | "info"  (L1/L2/L3)
;; message:  human-readable description + cure
;; fix:      None (STOP-1: auto-fix deferred to 277.1b)
;;
;; fix is typed :wat::core::String where "" means no fix available.
;; The full (offset,old-len,new-text) triple seam will land when 277.1b
;; adds the ast-end-span substrate primitive.
(:wat::Record::def :wat::lint::Finding
  [rule     <- :wat::core::String
   file     <- :wat::core::String
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   severity <- :wat::core::String
   message  <- :wat::core::String
   fix      <- :wat::core::String])

;; ─── Predicate helpers ───────────────────────────────────────────────

;; lint-structural? — a node whose children we recurse into (list/vector/set/map).
;; Mirror of deporder's structural? and fix.wat's structural? (same predicate).
(:wat::core::defn :wat::lint::lint-structural?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::let [k     (:wat::core::ast-kind node)
                    kinds (:wat::core::HashSet :wat::core::String
                             "list" "vector" "map" "set")]
    (:wat::core::contains? kinds k)))

;; node-write — serialize a node to text via write-forms (used to check booleans).
(:wat::core::defn :wat::lint::node-write
  [node <- :wat::WatAST]
  -> :wat::core::String
  (:wat::core::write-forms node))

;; bool-true? — the boolean literal true.
;; ast-kind == "bool" AND write-forms renders as "true".
(:wat::core::defn :wat::lint::bool-true?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "bool")
    (:wat::core::= (:wat::lint::node-write node) "true")
    false))

;; bool-false? — the boolean literal false.
(:wat::core::defn :wat::lint::bool-false?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "bool")
    (:wat::core::= (:wat::lint::node-write node) "false")
    false))

;; ─── nested-if-=-ladder detection ────────────────────────────────────
;;
;; Detect: (if (= VAR LIT) true (if (= VAR LIT) true (if ... false)))
;; All branches returning true over the SAME VAR compared to ≥3 literals.
;;
;; Algorithm:
;;   1. if-eq-branch? — check a list is (if (= SYM LIT) true ELSE)
;;      return the symbol name (VAR), the literal, and the ELSE node.
;;   2. collect-ladder-lits — recursively walk the else-chain collecting
;;      literals, verifying the same VAR throughout; stop when ELSE is `false`.
;;      Returns Vector<String> of LIT texts, empty if the chain breaks.
;;   3. nested-if-=-ladder? — it's a ladder when ≥3 lits collected.

;; kw-or-sym? — a node we can call ast-name on (keyword or symbol).
(:wat::core::defn :wat::lint::kw-or-sym?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::if (:wat::core::= k "keyword") true
      (:wat::core::= k "symbol"))))

;; if-head? — a list whose head is a keyword/symbol with name ":wat::core::if".
;; Guards ast-name with kw-or-sym? so bool/int/list heads don't crash.
(:wat::core::defn :wat::lint::if-head?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::let [head (:wat::core::Option/expect -> :wat::WatAST
                                   (:wat::core::first ch) "if-head?: head")]
          (:wat::core::if (:wat::lint::kw-or-sym? head)
            (:wat::core::= (:wat::core::ast-name head) ":wat::core::if")
            false))))
    false))

;; eq-sym-name — a list (= SYM LIT) where head is :wat::core::=,
;; child[1] is a symbol. Returns the symbol's ast-name on success, "" on failure.
;; Guards ast-name with kw-or-sym? so non-nameable heads don't crash.
(:wat::core::defn :wat::lint::eq-sym-name
  [node <- :wat::WatAST]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::i64::< (:wat::core::length ch) 3)
        ""
        (:wat::core::let [head (:wat::core::Option/expect -> :wat::WatAST
                                   (:wat::core::first ch) "eq-sym-name: head")
                          c1   (:wat::core::Option/expect -> :wat::WatAST
                                   (:wat::core::first (:wat::core::drop ch 1)) "eq-sym-name: c1")]
          (:wat::core::if (:wat::lint::kw-or-sym? head)
            (:wat::core::if (:wat::core::= (:wat::core::ast-name head) ":wat::core::=")
              (:wat::core::if (:wat::core::= (:wat::core::ast-kind c1) "symbol")
                (:wat::core::ast-name c1)
                "")
              "")
            ""))))
    ""))

;; eq-lit-text — the text of the literal (child[2]) in an (= SYM LIT) form.
;; Returns the write-forms text of child[2], or "" if not present.
(:wat::core::defn :wat::lint::eq-lit-text
  [node <- :wat::WatAST]
  -> :wat::core::String
  (:wat::core::let [ch (:wat::core::ast->children node)]
    (:wat::core::if (:wat::core::i64::< (:wat::core::length ch) 3)
      ""
      (:wat::core::let [c2 (:wat::core::Option/expect -> :wat::WatAST
                               (:wat::core::first (:wat::core::drop ch 2)) "eq-lit-text: c2")]
        (:wat::lint::node-write c2)))))

;; collect-ladder-lits — walk an if-eq-true chain over VAR, collecting
;; the LIT texts. Returns Vector<String> of lits; empty if chain breaks.
;;
;; Arguments:
;;   form:     the current if-node we're examining
;;   var-name: the VAR name we expect (the first call passes "" = not yet set)
;;
;; The chain step:
;;   - form must be (if (= VAR LIT) true ELSE) where VAR == var-name (or var-name == "")
;;   - collect LIT
;;   - if ELSE is `false` → done, return [LIT]
;;   - if ELSE is another if node → recurse on ELSE
;;   - otherwise → chain broken, return []
(:wat::core::defn :wat::lint::collect-ladder-lits
  [form     <- :wat::WatAST
   var-name <- :wat::core::String]
  -> :wat::core::Vector<wat::core::String>
  (:wat::core::if (:wat::lint::if-head? form)
    (:wat::core::let [ch (:wat::core::ast->children form)]
      (:wat::core::if (:wat::core::i64::< (:wat::core::length ch) 4)
        (:wat::core::Vector :wat::core::String)
        (:wat::core::let [cond (:wat::core::Option/expect -> :wat::WatAST
                                   (:wat::core::first (:wat::core::drop ch 1)) "collect-ladder-lits: cond")
                          then (:wat::core::Option/expect -> :wat::WatAST
                                   (:wat::core::first (:wat::core::drop ch 2)) "collect-ladder-lits: then")
                          else-node (:wat::core::Option/expect -> :wat::WatAST
                                        (:wat::core::first (:wat::core::drop ch 3)) "collect-ladder-lits: else")]
          ;; cond must be (= VAR LIT)
          (:wat::core::let [this-var (:wat::lint::eq-sym-name cond)]
            (:wat::core::if (:wat::core::= this-var "")
              ;; cond is not (= SYM LIT) — chain broken
              (:wat::core::Vector :wat::core::String)
              ;; var must match (or be the first step)
              (:wat::core::if (:wat::core::if (:wat::core::= var-name "")
                                 true
                                 (:wat::core::= this-var var-name))
                ;; then branch must be `true`
                (:wat::core::if (:wat::lint::bool-true? then)
                  (:wat::core::let [lit (:wat::lint::eq-lit-text cond)]
                    ;; collect this LIT; check the else branch
                    (:wat::core::if (:wat::lint::bool-false? else-node)
                      ;; chain ends with false — a proper terminator
                      (:wat::core::Vector :wat::core::String lit)
                      ;; else is another node — try to recurse
                      (:wat::core::let [rest-lits (:wat::lint::collect-ladder-lits else-node this-var)]
                        (:wat::core::if (:wat::core::empty? rest-lits)
                          ;; recursion found no more ladder steps but else isn't false —
                          ;; if else is a non-false non-ladder, the chain breaks
                          (:wat::core::Vector :wat::core::String)
                          ;; prepend this LIT
                          (:wat::core::concat
                            (:wat::core::Vector :wat::core::String lit)
                            rest-lits)))))
                  ;; then is not `true` — chain broken
                  (:wat::core::Vector :wat::core::String))
                ;; var mismatch — chain broken
                (:wat::core::Vector :wat::core::String)))))))
    ;; not an if form — no lits
    (:wat::core::Vector :wat::core::String)))

;; ladder-var-name — get the VAR name from a chain (the symbol in child[1] of cond).
(:wat::core::defn :wat::lint::ladder-var-name
  [form <- :wat::WatAST]
  -> :wat::core::String
  (:wat::core::if (:wat::lint::if-head? form)
    (:wat::core::let [ch (:wat::core::ast->children form)]
      (:wat::core::if (:wat::core::i64::< (:wat::core::length ch) 2)
        ""
        (:wat::core::let [cond (:wat::core::Option/expect -> :wat::WatAST
                                   (:wat::core::first (:wat::core::drop ch 1)) "ladder-var-name: cond")]
          (:wat::lint::eq-sym-name cond))))
    ""))

;; make-ladder-finding — construct the Finding for a detected ladder.
(:wat::core::defn :wat::lint::make-ladder-finding
  [form     <- :wat::WatAST
   file     <- :wat::core::String
   var-name <- :wat::core::String
   lits     <- :wat::core::Vector<wat::core::String>]
  -> :wat::lint::Finding
  (:wat::core::let [span    (:wat::core::ast-span form)
                    ln      (:wat::core::Option/expect -> :wat::core::i64
                                (:wat::core::HashMap/get span :line)
                                "make-ladder-finding: :line")
                    co      (:wat::core::Option/expect -> :wat::core::i64
                                (:wat::core::HashMap/get span :col)
                                "make-ladder-finding: :col")
                    n-lits  (:wat::core::length lits)
                    msg     (:wat::core::string::concat
                              "nested-if-=-ladder: var `"
                              var-name
                              "` compared against "
                              (:wat::core::i64::to-string n-lits)
                              " literals — use (:wat::core::contains? (:wat::core::HashSet :T lit…) var) instead")]
    (:wat::lint::Finding
      "nested-if-=-ladder"
      file
      ln
      co
      "warn"
      msg
      "")))

;; rule-nested-if-=-ladder-form — run the ladder rule on ONE form (recursive walk).
;; Detects the ladder at the top level OR nested anywhere inside the form.
(:wat::core::defn :wat::lint::rule-nested-if-=-ladder-form
  [form <- :wat::WatAST
   file <- :wat::core::String]
  -> :wat::core::Vector<wat::lint::Finding>
  ;; Check if THIS form is the root of a ladder
  (:wat::core::let [lits (:wat::lint::collect-ladder-lits form "")]
    (:wat::core::if (:wat::core::i64::>= (:wat::core::length lits) 3)
      ;; This form IS a ladder — report it (don't recurse into it)
      (:wat::core::Vector :wat::lint::Finding
        (:wat::lint::make-ladder-finding form file
          (:wat::lint::ladder-var-name form) lits))
      ;; Not a top-level ladder — recurse into children (if structural)
      (:wat::core::if (:wat::lint::lint-structural? form)
        (:wat::core::foldl
          (:wat::core::fn [acc   <- :wat::core::Vector<wat::lint::Finding>
                           child <- :wat::WatAST]
            -> :wat::core::Vector<wat::lint::Finding>
            (:wat::core::concat acc
              (:wat::lint::rule-nested-if-=-ladder-form child file)))
          (:wat::core::Vector :wat::lint::Finding)
          (:wat::core::ast->children form))
        (:wat::core::Vector :wat::lint::Finding)))))

;; ─── concat-abuse detection ──────────────────────────────────────────
;;
;; Detect: (string::concat <mix of string-literal args and non-literal args>)
;; A hand-rolled template — the cure is (:wat::core::format "…{name}…" :name v …).
;;
;; All-literal (concat "a" "b") → not abuse (nothing to interpolate).
;; All-value   (concat a b)     → not abuse (no literal scaffolding).
;; Only the mix triggers the rule.

;; concat-head? — a list whose head is a keyword/symbol with name
;; ":wat::core::string::concat" OR ":wat::core::String/concat".
(:wat::core::defn :wat::lint::concat-head?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [children (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::i64::>= (:wat::core::length children) 1)
        (:wat::core::let [head (:wat::core::Option/expect -> :wat::WatAST
                                  (:wat::core::first children)
                                  "concat-head?: first child")]
          (:wat::core::if (:wat::lint::kw-or-sym? head)
            (:wat::core::let [n (:wat::core::ast-name head)]
              (:wat::core::if (:wat::core::= n ":wat::core::string::concat")
                true
                (:wat::core::= n ":wat::core::String/concat")))
            false))
        false))
    false))

;; concat-arg-counts — count literal and non-literal args in a concat call.
;; Returns Tuple(n-lits, n-vals) where n-lits = count of "string" ast-kind args,
;; n-vals = count of all other arg kinds.
(:wat::core::defn :wat::lint::concat-arg-counts
  [node <- :wat::WatAST]
  -> :(wat::core::i64,wat::core::i64)
  (:wat::core::let [children (:wat::core::ast->children node)
                    args     (:wat::core::drop children 1)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :(wat::core::i64,wat::core::i64)
                       arg <- :wat::WatAST]
        -> :(wat::core::i64,wat::core::i64)
        (:wat::core::let [lits (:wat::core::first acc)
                          vals (:wat::core::second acc)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind arg) "string")
            (:wat::core::Tuple (:wat::core::i64::+ lits 1) vals)
            (:wat::core::Tuple lits (:wat::core::i64::+ vals 1)))))
      (:wat::core::Tuple 0 0)
      args)))

;; concat-abuse? — true when the concat call mixes string literals with non-literals.
(:wat::core::defn :wat::lint::concat-abuse?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::lint::concat-head? node)
    (:wat::core::let [counts (:wat::lint::concat-arg-counts node)
                      n-lits (:wat::core::first counts)
                      n-vals (:wat::core::second counts)]
      (:wat::core::if (:wat::core::i64::>= n-lits 1)
        (:wat::core::i64::>= n-vals 1)
        false))
    false))

;; make-concat-finding — construct the Finding for a detected concat-abuse.
(:wat::core::defn :wat::lint::make-concat-finding
  [form   <- :wat::WatAST
   file   <- :wat::core::String
   n-lits <- :wat::core::i64
   n-vals <- :wat::core::i64]
  -> :wat::lint::Finding
  (:wat::core::let [span (:wat::core::ast-span form)
                    ln   (:wat::core::Option/expect -> :wat::core::i64
                             (:wat::core::HashMap/get span :line)
                             "make-concat-finding: :line")
                    co   (:wat::core::Option/expect -> :wat::core::i64
                             (:wat::core::HashMap/get span :col)
                             "make-concat-finding: :col")
                    msg  (:wat::core::string::concat
                            "concat-abuse: string::concat interleaves "
                            (:wat::core::i64::to-string n-lits)
                            " literal(s) with "
                            (:wat::core::i64::to-string n-vals)
                            " value(s) — use (:wat::core::format \"…{name}…\" :name v …) instead")]
    (:wat::lint::Finding
      "concat-abuse"
      file
      ln
      co
      "warn"
      msg
      "")))

;; rule-concat-abuse-form — run the concat-abuse rule on ONE form (recursive walk).
;; Detects concat-abuse at the top level OR nested anywhere inside the form.
(:wat::core::defn :wat::lint::rule-concat-abuse-form
  [form <- :wat::WatAST
   file <- :wat::core::String]
  -> :wat::core::Vector<wat::lint::Finding>
  ;; Check if THIS form is a concat-abuse
  (:wat::core::if (:wat::lint::concat-abuse? form)
    ;; This form IS a concat-abuse — report it (don't recurse into it)
    (:wat::core::let [counts (:wat::lint::concat-arg-counts form)
                      n-lits (:wat::core::first counts)
                      n-vals (:wat::core::second counts)]
      (:wat::core::Vector :wat::lint::Finding
        (:wat::lint::make-concat-finding form file n-lits n-vals)))
    ;; Not a concat-abuse — recurse into children (if structural)
    (:wat::core::if (:wat::lint::lint-structural? form)
      (:wat::core::foldl
        (:wat::core::fn [acc   <- :wat::core::Vector<wat::lint::Finding>
                         child <- :wat::WatAST]
          -> :wat::core::Vector<wat::lint::Finding>
          (:wat::core::concat acc
            (:wat::lint::rule-concat-abuse-form child file)))
        (:wat::core::Vector :wat::lint::Finding)
        (:wat::core::ast->children form))
      (:wat::core::Vector :wat::lint::Finding))))

;; ─── lint-source: run all rules over a Vector<SourceFile> ────────────

;; lint-file — run all form-level rules over one SourceFile.
(:wat::core::defn :wat::lint::lint-file
  [sf <- :wat::deporder::SourceFile]
  -> :wat::core::Vector<wat::lint::Finding>
  (:wat::core::let [path   (:wat::deporder::SourceFile/path sf)
                    source (:wat::deporder::SourceFile/source sf)
                    tree   (:wat::core::read-string source)
                    forms  (:wat::core::ast->children tree)]
    (:wat::core::foldl
      (:wat::core::fn [acc  <- :wat::core::Vector<wat::lint::Finding>
                       form <- :wat::WatAST]
        -> :wat::core::Vector<wat::lint::Finding>
        (:wat::core::concat acc
          (:wat::core::concat
            (:wat::lint::rule-nested-if-=-ladder-form form path)
            (:wat::lint::rule-concat-abuse-form form path))))
      (:wat::core::Vector :wat::lint::Finding)
      forms)))

;; lint-source — run form-level rules over every file in Vector<SourceFile>.
;; The primary pure entry point for the linter.
(:wat::core::defn :wat::lint::lint-source
  [files <- :wat::core::Vector<wat::deporder::SourceFile>]
  -> :wat::core::Vector<wat::lint::Finding>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::Vector<wat::lint::Finding>
                     sf  <- :wat::deporder::SourceFile]
      -> :wat::core::Vector<wat::lint::Finding>
      (:wat::core::concat acc (:wat::lint::lint-file sf)))
    (:wat::core::Vector :wat::lint::Finding)
    files))

;; ─── rule-zero: deporder load-order as Findings ──────────────────────

;; violation->finding — convert a deporder Violation into a rule-zero Finding.
;; Violations have no span (deporder doesn't walk for positions); line and col = 0.
;; The fix is always "" (no mechanical fix — load-order is a human decision).
(:wat::core::defn :wat::lint::violation->finding
  [v <- :wat::deporder::Violation]
  -> :wat::lint::Finding
  (:wat::lint::Finding
    "load-order"
    (:wat::deporder::Violation/referencer v)
    0
    0
    "error"
    (:wat::core::string::concat
      "load-order violation: "
      (:wat::deporder::Violation/referencer v)
      " (pos "
      (:wat::core::i64::to-string (:wat::deporder::Violation/referencer-pos v))
      ") eval-depends on "
      (:wat::deporder::Violation/definer v)
      " (pos "
      (:wat::core::i64::to-string (:wat::deporder::Violation/definer-pos v))
      ") which loads later — symbol: "
      (:wat::deporder::Violation/symbol v))
    ""))

;; violations->findings — map Violations to rule-zero Findings.
(:wat::core::defn :wat::lint::violations->findings
  [viols <- :wat::core::Vector<wat::deporder::Violation>]
  -> :wat::core::Vector<wat::lint::Finding>
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::Vector<wat::lint::Finding>
                     v   <- :wat::deporder::Violation]
      -> :wat::core::Vector<wat::lint::Finding>
      (:wat::core::concat acc
        (:wat::core::Vector :wat::lint::Finding
          (:wat::lint::violation->finding v))))
    (:wat::core::Vector :wat::lint::Finding)
    viols))

;; ─── lint-stdlib: the surface ─────────────────────────────────────────

;; lint-stdlib — the top-level surface:
;;   form-level findings over the real stdlib (lint-source on stdlib-sources)
;;   PLUS deporder's load-order check folded in as rule-zero (report-only).
;;
;; Currently 0 rule-zero violations (arc 275 fixed them all).
;; Any future load-order regression will surface immediately here.
(:wat::core::defn :wat::lint::lint-stdlib
  []
  -> :wat::core::Vector<wat::lint::Finding>
  (:wat::core::let [srcs   (:wat::deporder::stdlib-sources)
                    form-findings (:wat::lint::lint-source srcs)
                    viols  (:wat::deporder::verify srcs)
                    rule-zero-findings (:wat::lint::violations->findings viols)]
    (:wat::core::concat form-findings rule-zero-findings)))
