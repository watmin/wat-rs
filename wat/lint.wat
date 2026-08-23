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
;;   wat/Record.wat   — :wat::core::Record::def for the Finding record

;; ─── Typed record: FixEdit (the extent + replacement for an auto-fix) ───────

;; FixEdit — an auto-fix edit produced by a lint rule.
;; start-line / start-col: 1-indexed position of the first char of the node to replace.
;; end-line   / end-col:   1-indexed position one char PAST the last char (from ast-end-span).
;; new-text:  the replacement source text.
;; Position-based (not flat-offset-based): the rule has the spans but NOT the source;
;; the applier (apply-fixes) holds the source and flattens via fix-text-offset-of.
(:wat::core::defrecord :wat::lint::FixEdit
  [start-line <- :wat::core::i64
   start-col  <- :wat::core::i64
   end-line   <- :wat::core::i64
   end-col    <- :wat::core::i64
   new-text   <- :wat::core::String])

;; ─── Typed record: Finding (uncompilable on a wrong shape) ───────────

;; Finding — a lint result.
;; rule:     the rule name (e.g. "nested-if-=-ladder", "load-order")
;; file:     the source file path
;; line:     1-indexed line of the finding
;; col:      1-indexed column of the finding
;; severity: "error" | "warn" | "info"  (L1/L2/L3)
;; message:  human-readable description + cure
;; fix:      Some(FixEdit) = an auto-fix is available; None = report-only.
(:wat::core::defrecord :wat::lint::Finding
  [rule     <- :wat::core::String
   file     <- :wat::core::String
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   severity <- :wat::core::String
   message  <- :wat::core::String
   fix      <- (:wat::core::Option :- [:wat::lint::FixEdit])])

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
        (:wat::core::let [head (:wat::core::first ch)]
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
        (:wat::core::let [head (:wat::core::first ch)
                          c1   (:wat::core::nth ch 1)]
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
      (:wat::core::let [c2 (:wat::core::nth ch 2)]
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
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::lint::if-head? form)
    (:wat::core::let [ch (:wat::core::ast->children form)]
      (:wat::core::if (:wat::core::i64::< (:wat::core::length ch) 4)
        (:wat::core::Vector :wat::core::String)
        (:wat::core::let [cond (:wat::core::nth ch 1)
                          then (:wat::core::nth ch 2)
                          else-node (:wat::core::nth ch 3)]
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
        (:wat::core::let [cond (:wat::core::nth ch 1)]
          (:wat::lint::eq-sym-name cond))))
    ""))

;; make-ladder-finding — construct the Finding for a detected ladder.
(:wat::core::defn :wat::lint::make-ladder-finding
  [form     <- :wat::WatAST
   file     <- :wat::core::String
   var-name <- :wat::core::String
   lits     <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::lint::Finding
  (:wat::core::let [span    (:wat::core::ast-span form)
                    ep      (:wat::core::ast-end-span form)
                    ln      (:wat::core::Option/expect  
                                (:wat::core::HashMap/get span :line)
                                "make-ladder-finding: :line")
                    co      (:wat::core::Option/expect  
                                (:wat::core::HashMap/get span :col)
                                "make-ladder-finding: :col")
                    end-ln  (:wat::core::Option/expect  
                                (:wat::core::HashMap/get ep :line)
                                "make-ladder-finding: end :line")
                    end-co  (:wat::core::Option/expect  
                                (:wat::core::HashMap/get ep :col)
                                "make-ladder-finding: end :col")
                    n-lits  (:wat::core::length lits)
                    msg     (:wat::core::string::concat
                              "nested-if-=-ladder: var `"
                              var-name
                              "` compared against "
                              (:wat::core::i64::to-string n-lits)
                              " literals — use (:wat::core::contains? (:wat::core::HashSet :T lit…) var) instead")
                    new-text (:wat::core::format
                               "(:wat::core::contains? (:wat::core::HashSet :wat::type::Infer {lits}) {var})"
                               :lits (:wat::core::string::join " " lits)
                               :var var-name)
                    fe      (:wat::lint::FixEdit :start-line ln :start-col co :end-line end-ln :end-col end-co :new-text new-text)]
    (:wat::lint::Finding
      :rule "nested-if-=-ladder"
      :file file
      :line ln
      :col co
      :severity "warn"
      :message msg
      :fix (:wat::core::Some fe))))

;; rule-nested-if-=-ladder-form — run the ladder rule on ONE form (recursive walk).
;; Detects the ladder at the top level OR nested anywhere inside the form.
(:wat::core::defn :wat::lint::rule-nested-if-=-ladder-form
  [form <- :wat::WatAST
   file <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::lint::Finding])
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
          (:wat::core::fn [acc   <- (:wat::core::Vector :- [:wat::lint::Finding])
                           child <- :wat::WatAST]
            -> (:wat::core::Vector :- [:wat::lint::Finding])
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
        (:wat::core::let [head (:wat::core::first children)]
          (:wat::core::if (:wat::lint::kw-or-sym? head)
            (:wat::core::let [n (:wat::core::ast-name head)]
              (:wat::core::if (:wat::core::= n ":wat::core::string::concat")
                true
                (:wat::core::= n ":wat::core::String/concat")))
            false))
        false))
    false))

;; is-defmacro-form? — a list whose head is a keyword/symbol with name ":wat::core::defmacro".
;; Guards ast-name with kw-or-sym? so non-nameable heads don't crash.
(:wat::core::defn :wat::lint::is-defmacro-form?
  [form <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind form) "list")
    (:wat::core::let [ch (:wat::core::ast->children form)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::let [head (:wat::core::first ch)]
          (:wat::core::if (:wat::lint::kw-or-sym? head)
            (:wat::core::= (:wat::core::ast-name head) ":wat::core::defmacro")
            false))))
    false))

;; concat-arg-counts — count literal and non-literal args in a concat call.
;; Returns Tuple(n-lits, n-vals) where n-lits = count of "string" ast-kind args,
;; n-vals = count of all other arg kinds.
(:wat::core::defn :wat::lint::concat-arg-counts
  [node <- :wat::WatAST]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let [children (:wat::core::ast->children node)
                    ;; Arc 118.2a — `drop` flipped LAZY (returns Stream); `foldl` below is
                    ;; unchanged (Vector/List/PersistentVector only) and consumes `args` fully,
                    ;; so force it eager here.
                    args     (:wat::core::into [] (:wat::core::drop children 1))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
                       arg <- :wat::WatAST]
        -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
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

;; concat-format-fix — compute an auto-fix for a concat-abuse form, if eligible.
;;
;; Eligible when:
;;   - every non-string-literal arg is ast-kind "symbol" (bare name → honest placeholder)
;;   - no string-literal arg's inner text contains `"`, `{`, or `}` (keeps template simple)
;; Ineligible (compound value slot, or literal with special chars) → None (report-only).
;;
;; When eligible: fold args in order building:
;;   - template  (String): literal args → their inner text; symbol args → "{name}"
;;   - kwarg-names (Vector<String>): symbol names, deduped first-seen-order
;; head-str = if in-defmacro? ":wat::core::string::interpolate" else ":wat::core::format"
;; Emit: new-text = "(<head-str> \"<template>\" :a a :b b …)"
;; Return: Some(FixEdit start-line start-col end-line end-col new-text)
;; extent = ast-span..ast-end-span of the whole concat form (same as ladder fix).
(:wat::core::defn :wat::lint::concat-format-fix
  [form        <- :wat::WatAST
   in-defmacro? <- :wat::core::bool]
  -> (:wat::core::Option :- [:wat::lint::FixEdit])
  (:wat::core::let [;; Arc 118.2a — `drop` flipped LAZY; `args` feeds two `foldl` calls below
                    ;; (Vector/List/PersistentVector-only, unchanged) — force eager here.
                    args     (:wat::core::into [] (:wat::core::drop (:wat::core::ast->children form) 1))
                    ;; ── Step 1: eligibility fold ─────────────────────────────
                    ;; acc = bool (still-eligible). Fold args; if any arg fails,
                    ;; propagate false (no early exit — fold goes to the end).
                    eligible (:wat::core::foldl
                               (:wat::core::fn [ok  <- :wat::core::bool
                                                arg <- :wat::WatAST]
                                 -> :wat::core::bool
                                 (:wat::core::if ok
                                   (:wat::core::if (:wat::core::= (:wat::core::ast-kind arg) "string")
                                     ;; literal: inner text must contain NONE of " { }
                                     ;; (a boolean test, not a nested-if ladder — the very smell
                                     ;; this tool exists to abolish; intueri caught the author's hand)
                                     (:wat::core::let [inner (:wat::core::ast-name arg)]
                                       (:wat::core::not
                                         (:wat::core::or (:wat::core::string::contains? inner "\"")
                                           (:wat::core::or (:wat::core::string::contains? inner "{")
                                             (:wat::core::string::contains? inner "}")))))
                                     ;; non-literal: must be a bare symbol
                                     (:wat::core::= (:wat::core::ast-kind arg) "symbol"))
                                   false))
                               true
                               args)]
    (:wat::core::if eligible
      ;; ── Step 2: build template + kwarg-names ────────────────────────
      ;; acc = Tuple(template, kwarg-names) : :(String, Vector<String>)
      (:wat::core::let [build-result
                         (:wat::core::foldl
                           (:wat::core::fn [acc <- (:wat::core::Tuple :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
                                            arg <- :wat::WatAST]
                             -> (:wat::core::Tuple :- [:wat::core::String (:wat::core::Vector :- [:wat::core::String])])
                             (:wat::core::let [tmpl  (:wat::core::first acc)
                                               names (:wat::core::second acc)]
                               (:wat::core::if (:wat::core::= (:wat::core::ast-kind arg) "string")
                                 ;; literal → append inner text to template
                                 (:wat::core::Tuple
                                   (:wat::core::string::concat tmpl (:wat::core::ast-name arg))
                                   names)
                                 ;; symbol → append {name} to template; dedup-add to names
                                 (:wat::core::let [nm (:wat::core::ast-name arg)]
                                   (:wat::core::Tuple
                                     (:wat::core::string::concat tmpl
                                       (:wat::core::string::concat "{"
                                         (:wat::core::string::concat nm "}")))
                                     (:wat::core::if (:wat::core::contains? names nm)
                                       names
                                       (:wat::core::conj names nm)))))))
                           (:wat::core::Tuple "" (:wat::core::Vector :wat::core::String))
                           args)
                        template   (:wat::core::first build-result)
                        kwarg-names (:wat::core::second build-result)
                        ;; ── Step 3: emit new-text ────────────────────────────
                        ;; head-str: interpolate inside a defmacro, format elsewhere
                        head-str   (:wat::core::if in-defmacro?
                                     ":wat::core::string::interpolate"
                                     ":wat::core::format")
                        ;; "(<head-str> \"<template>\"" + " :nm nm" … + ")"
                        kwargs-text (:wat::core::foldl
                                      (:wat::core::fn [acc <- :wat::core::String
                                                       nm  <- :wat::core::String]
                                        -> :wat::core::String
                                        (:wat::core::string::concat acc
                                          (:wat::core::string::concat " :"
                                            (:wat::core::string::concat nm
                                              (:wat::core::string::concat " " nm)))))
                                      ""
                                      kwarg-names)
                        new-text   (:wat::core::string::concat
                                     (:wat::core::format "({head-str}" :head-str head-str)
                                     (:wat::core::string::concat " \""
                                       (:wat::core::string::concat template
                                         (:wat::core::string::concat "\""
                                           (:wat::core::string::concat kwargs-text ")")))))
                        ;; ── Step 4: span from ast-span + ast-end-span of form ─
                        span    (:wat::core::ast-span form)
                        ep      (:wat::core::ast-end-span form)
                        ln      (:wat::core::Option/expect  
                                    (:wat::core::HashMap/get span :line)
                                    "concat-format-fix: :line")
                        co      (:wat::core::Option/expect  
                                    (:wat::core::HashMap/get span :col)
                                    "concat-format-fix: :col")
                        end-ln  (:wat::core::Option/expect  
                                    (:wat::core::HashMap/get ep :line)
                                    "concat-format-fix: end :line")
                        end-co  (:wat::core::Option/expect  
                                    (:wat::core::HashMap/get ep :col)
                                    "concat-format-fix: end :col")
                        fe      (:wat::lint::FixEdit :start-line ln :start-col co :end-line end-ln :end-col end-co :new-text new-text)]
        (:wat::core::Some fe))
      ;; ineligible (compound slot or special-char literal) — report-only
      :wat::core::None)))

;; make-concat-finding — construct the Finding for a detected concat-abuse.
(:wat::core::defn :wat::lint::make-concat-finding
  [form         <- :wat::WatAST
   file         <- :wat::core::String
   n-lits       <- :wat::core::i64
   n-vals       <- :wat::core::i64
   in-defmacro? <- :wat::core::bool]
  -> :wat::lint::Finding
  (:wat::core::let [span (:wat::core::ast-span form)
                    ln   (:wat::core::Option/expect  
                             (:wat::core::HashMap/get span :line)
                             "make-concat-finding: :line")
                    co   (:wat::core::Option/expect  
                             (:wat::core::HashMap/get span :col)
                             "make-concat-finding: :col")
                    msg  (:wat::core::string::concat
                            "concat-abuse: string::concat interleaves "
                            (:wat::core::i64::to-string n-lits)
                            " literal(s) with "
                            (:wat::core::i64::to-string n-vals)
                            " value(s) — use (:wat::core::format \"…{name}…\" :name v …) instead")]
    (:wat::lint::Finding
      :rule "concat-abuse"
      :file file
      :line ln
      :col co
      :severity "warn"
      :message msg
      :fix (:wat::lint::concat-format-fix form in-defmacro?))))

;; rule-concat-abuse-form — run the concat-abuse rule on ONE form (recursive walk).
;; Detects concat-abuse at the top level OR nested anywhere inside the form.
;; in-defmacro? tracks whether the current form is nested inside a defmacro body.
(:wat::core::defn :wat::lint::rule-concat-abuse-form
  [form         <- :wat::WatAST
   file         <- :wat::core::String
   in-defmacro? <- :wat::core::bool]
  -> (:wat::core::Vector :- [:wat::lint::Finding])
  ;; Check if THIS form is a concat-abuse
  (:wat::core::if (:wat::lint::concat-abuse? form)
    ;; This form IS a concat-abuse — report it (don't recurse into it)
    (:wat::core::let [counts (:wat::lint::concat-arg-counts form)
                      n-lits (:wat::core::first counts)
                      n-vals (:wat::core::second counts)]
      (:wat::core::Vector :wat::lint::Finding
        (:wat::lint::make-concat-finding form file n-lits n-vals in-defmacro?)))
    ;; Not a concat-abuse — recurse into children (if structural)
    ;; child's in-defmacro? = current in-defmacro? OR (is this form a defmacro?)
    (:wat::core::if (:wat::lint::lint-structural? form)
      (:wat::core::let [child-in-defmacro? (:wat::core::or in-defmacro? (:wat::lint::is-defmacro-form? form))]
        (:wat::core::foldl
          (:wat::core::fn [acc   <- (:wat::core::Vector :- [:wat::lint::Finding])
                           child <- :wat::WatAST]
            -> (:wat::core::Vector :- [:wat::lint::Finding])
            (:wat::core::concat acc
              (:wat::lint::rule-concat-abuse-form child file child-in-defmacro?)))
          (:wat::core::Vector :wat::lint::Finding)
          (:wat::core::ast->children form)))
      (:wat::core::Vector :wat::lint::Finding))))

;; ─── lint-source: run all rules over a Vector<SourceFile> ────────────

;; lint-file — run all form-level rules over one SourceFile.
(:wat::core::defn :wat::lint::lint-file
  [sf <- :wat::source::File]
  -> (:wat::core::Vector :- [:wat::lint::Finding])
  (:wat::core::let [path   (:wat::source::File/path sf)
                    source (:wat::source::File/source sf)
                    tree   (:wat::core::match (:wat::core::read-string source) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms  (:wat::core::ast->children tree)]
    (:wat::core::foldl
      (:wat::core::fn [acc  <- (:wat::core::Vector :- [:wat::lint::Finding])
                       form <- :wat::WatAST]
        -> (:wat::core::Vector :- [:wat::lint::Finding])
        (:wat::core::concat acc
          (:wat::core::concat
            (:wat::lint::rule-nested-if-=-ladder-form form path)
            (:wat::lint::rule-concat-abuse-form form path false))))
      (:wat::core::Vector :wat::lint::Finding)
      forms)))

;; lint-source — run form-level rules over every file in Vector<SourceFile>.
;; The primary pure entry point for the linter.
(:wat::core::defn :wat::lint::lint-source
  [files <- (:wat::core::Vector :- [:wat::source::File])]
  -> (:wat::core::Vector :- [:wat::lint::Finding])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::lint::Finding])
                     sf  <- :wat::source::File]
      -> (:wat::core::Vector :- [:wat::lint::Finding])
      (:wat::core::concat acc (:wat::lint::lint-file sf)))
    (:wat::core::Vector :wat::lint::Finding)
    files))

;; ─── rule-zero: deporder load-order as Findings ──────────────────────

;; violation->finding — convert a deporder Violation into a rule-zero Finding.
;; Violations have no span (deporder doesn't walk for positions); line and col = 0.
;; The fix is always None (no mechanical fix — load-order is a human decision).
(:wat::core::defn :wat::lint::violation->finding
  [v <- :wat::deporder::Violation]
  -> :wat::lint::Finding
  (:wat::lint::Finding
    :rule "load-order"
    :file (:wat::deporder::Violation/referencer v)
    :line 0
    :col 0
    :severity "error"
    :message (:wat::core::string::concat
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
    :fix :wat::core::None))

;; violations->findings — map Violations to rule-zero Findings.
(:wat::core::defn :wat::lint::violations->findings
  [viols <- (:wat::core::Vector :- [:wat::deporder::Violation])]
  -> (:wat::core::Vector :- [:wat::lint::Finding])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::lint::Finding])
                     v   <- :wat::deporder::Violation]
      -> (:wat::core::Vector :- [:wat::lint::Finding])
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
  -> (:wat::core::Vector :- [:wat::lint::Finding])
  (:wat::core::let [srcs   (:wat::deporder::stdlib-sources)
                    form-findings (:wat::lint::lint-source srcs)
                    viols  (:wat::deporder::verify srcs)
                    rule-zero-findings (:wat::lint::violations->findings viols)]
    (:wat::core::concat form-findings rule-zero-findings)))

;; ─── apply-fixes + lint-fix-file: the auto-fix applier ───────────────

;; apply-fixes — apply all Some fixes from findings to the source in sf.
;; For each finding with a Some(FixEdit): converts the span positions to flat offsets
;; using fix-text-offset-of (via {:line,:col} HashMaps), computes old-len via
;; fix-text-span-len, collects Tuple(off, old-len, new-text) in ascending order,
;; reverses to right-to-left, then splices via fix-text-apply.
(:wat::core::defn :wat::lint::apply-fixes
  [sf       <- :wat::source::File
   findings <- (:wat::core::Vector :- [:wat::lint::Finding])]
  -> :wat::core::String
  (:wat::core::let [src   (:wat::source::File/source sf)
                    lines (:wat::core::string::split src "\n")
                    edits (:wat::core::foldl
                            (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
                                             f   <- :wat::lint::Finding]
                              -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])])
                              (:wat::core::match (:wat::lint::Finding/fix f)  
                                (:wat::core::None acc)
                                ((:wat::core::Some fe)
                                 (:wat::core::let [start-map (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                                               :line (:wat::lint::FixEdit/start-line fe)
                                                               :col  (:wat::lint::FixEdit/start-col fe))
                                                   end-map   (:wat::core::HashMap :wat::core::keyword :wat::core::i64
                                                               :line (:wat::lint::FixEdit/end-line fe)
                                                               :col  (:wat::lint::FixEdit/end-col fe))
                                                   off       (:wat::fix::fix-text-offset-of start-map lines)
                                                   old-len   (:wat::fix::fix-text-span-len start-map end-map lines)
                                                   new-text  (:wat::lint::FixEdit/new-text fe)]
                                   (:wat::core::concat acc
                                     (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
                                       (:wat::core::Tuple off old-len new-text)))))))
                            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String]))
                            findings)
                    rev-edits (:wat::core::reverse edits)]
    (:wat::fix::fix-text-apply src rev-edits)))

;; lint-fix-file — lint a SourceFile and apply all auto-fixes, returning the fixed source.
;; Convenience entry called by probes and the sweep: lint-file → apply-fixes.
(:wat::core::defn :wat::lint::lint-fix-file
  [sf <- :wat::source::File]
  -> :wat::core::String
  (:wat::lint::apply-fixes sf (:wat::lint::lint-file sf)))
