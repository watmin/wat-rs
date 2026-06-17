;; vigilatum: 2026-06-06T04:56:04Z — UPDATED-vigilia spec/DSL 5-spell guard
;; L1+L2=0 (cernere [CONVERGED 0+0: full vocabulary table, every expand-time
;; head verified on the pure-total allow-list] + probare [all 12 forms (16 at first earn; 4 ordering defclauses retired by Stone 245.8)
;; Expressed] + conferre [all 17 header claims verified; 6 USER-GUIDE
;; divergences fixed spec-side] + exigere [CONVERGED 0+0] + circumspicere
;; LAST [the false loads-early rationale killed at both sites; empty-step
;; behaviors documented + witnessed at their empirical failure shapes]).
;; Witness corpus: deftest-green(core-arithmetic + core-equality +
;; core-threading + core-collection-aliases + option-expect + record-def +
;; result-expect + struct-to-form + list-fold-aliases); corpus 236/0/53;
;; checker-clean. Canonical record:
;; docs/arc/2026/06/249-total-pure-macros/WARD-COREWAT-REEARN.md.
;; RE-EARNED 2026-06-06T04:56:04Z (diff-scoped, the 245 clear: Stone 245.8 retired
;; the ordering defclauses — ordering is a relational check-side intrinsic; the
;; retirement note's claims verified live [infer_ordering / dispatch arms /
;; leaves present]; corpus 236/0/53; zero defclause remnants).
;;
;; wat/core.wat — the :wat::core::* stdlib surface: short-name aliases plus the
;; polymorphic arithmetic defclauses.
;;
;; Ordering (`<`/`>`/`<=`/`>=`) is a relational check-side intrinsic (Stone 245.8),
;; the sibling of equality. The defclauses that formerly lived here are retired;
;; the per-Type leaves (`:wat::core::i64::<` etc.) remain as the type-locked tier.
;;
;; Position in the stdlib array is not load-bearing for visibility:
;; register_stdlib_defmacros (src/macros/parse.rs) walks the entire
;; concatenated stdlib in one pre-expansion pass, so every defmacro
;; (defn, concat, ->, …) is registered before any expansion runs.
;; defclause/defalias stubs are likewise pre-registered before use
;; (src/freeze.rs: preregister_stdlib_defclause_stub).

;; ─── Short-name collection aliases ──────────────────────────────────────────
;;
;; The polymorphic-name collection ops — length / empty? / contains? / get /
;; conj / assoc — are Rust ∀T intrinsics: check-side inference lives in
;; src/collection/infer.rs, eval-side in src/runtime.rs. Their per-Type leaves
;; (:Vector/length, :HashMap/get, …) remain as the backing impls.
;;
;; Single-impl ops below get a short-name alias (not a dispatch — dispatch is
;; for genuine polymorphism); both the short and long names are honest.
(:wat::core::defalias :wat::core::dissoc  :wat::core::HashMap/dissoc)
(:wat::core::defalias :wat::core::keys    :wat::core::HashMap/keys)
(:wat::core::defalias :wat::core::values  :wat::core::HashMap/values)
(:wat::core::defalias :wat::core::concat  :wat::core::Vector/concat)

;; ─── Polymorphic arithmetic defclauses ───────────────────────────────────────
;;
;; Two layers: a per-Type binary primitive in Rust (:wat::core::<Type>::<op>,
;; always 2-ary) under a polymorphic wat defclause (:wat::core::<op>) that
;; dispatches by arity × arg-Type. Cross-type is rejected by CLAUSE ABSENCE —
;; no mixed-type clause exists, so (:wat::core::+ 1 2.0) → :NoMatchingClause,
;; no special-case Rust check needed.
;;
;; Arity (Lisp/Clojure): +/* → 0-ary identity (0 / 1), 1-ary unchanged, 2-ary
;; binary, 3+ fold. -// → no 0-ary, 1-ary identity-on-left (negate / reciprocal),
;; 2-ary binary, 3+ fold.

(:wat::core::defclause :wat::core::+
  ;; 0-ary identity: i64 0 (Lisp additive identity)
  ([] -> :wat::core::i64 0)
  ;; 1-ary: per-Type arg unchanged
  ([x <- :wat::core::i64] -> :wat::core::i64 x)
  ([x <- :wat::core::f64] -> :wat::core::f64 x)
  ;; 2-ary: direct per-Type binary call
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::+ x y))
  ;; 3+-ary: per-Type fold over rest
  ([x <- :wat::core::i64
    y <- :wat::core::i64
    & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::+ acc n))
      (:wat::core::i64::+ x y)
      rest))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::+ acc n))
      (:wat::core::f64::+ x y)
      rest)))

(:wat::core::defclause :wat::core::-
  ;; NO 0-ary clause — :NoMatchingClause fires
  ;; 1-ary per-Type: negate (identity-on-left = 0)
  ([x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::- 0 x))
  ([x <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::- 0.0 x))
  ;; 2-ary
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::- x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::- x y))
  ;; 3+-ary fold
  ([x <- :wat::core::i64
    y <- :wat::core::i64
    & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::- acc n))
      (:wat::core::i64::- x y)
      rest))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::- acc n))
      (:wat::core::f64::- x y)
      rest)))

(:wat::core::defclause :wat::core::*
  ;; 0-ary identity: i64 1 (Lisp multiplicative identity)
  ([] -> :wat::core::i64 1)
  ;; 1-ary: per-Type arg unchanged
  ([x <- :wat::core::i64] -> :wat::core::i64 x)
  ([x <- :wat::core::f64] -> :wat::core::f64 x)
  ;; 2-ary
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::* x y))
  ;; 3+-ary fold
  ([x <- :wat::core::i64
    y <- :wat::core::i64
    & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::* acc n))
      (:wat::core::i64::* x y)
      rest))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::* acc n))
      (:wat::core::f64::* x y)
      rest)))

(:wat::core::defclause :wat::core::/
  ;; NO 0-ary clause — :NoMatchingClause fires
  ;; 1-ary per-Type: reciprocal (identity-on-left = 1)
  ([x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::/ 1 x))
  ([x <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::/ 1.0 x))
  ;; 2-ary
  ([x <- :wat::core::i64
    y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::/ x y))
  ([x <- :wat::core::f64
    y <- :wat::core::f64] -> :wat::core::f64 (:wat::core::f64::/ x y))
  ;; 3+-ary fold
  ([x <- :wat::core::i64
    y <- :wat::core::i64
    & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::core::i64::/ acc n))
      (:wat::core::i64::/ x y)
      rest))
  ([x <- :wat::core::f64
    y <- :wat::core::f64
    & rest <- :wat::core::Vector<wat::core::f64>] -> :wat::core::f64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::f64
                       n <- :wat::core::f64] -> :wat::core::f64
        (:wat::core::f64::/ acc n))
      (:wat::core::f64::/ x y)
      rest)))

;; ─── Named-function binding ───────────────────────────────────────
;;
;; Arc 260.1a — defn detects a trailing `& [argspec]` kwargs section:
;;   - `& [name <- :T …]` (Vector tail)  → mints :<name>::Kwargs record, reshapes fn,
;;                                          destructures fields into the body scope.
;;   - `& sym <- :T`      (Symbol tail)  → variadic rest; pass through unchanged (no touch).
;;   - no `&` at all                     → backward-compat pass-through (UNCHANGED).
;;
;; PROGRAM-BODY path (mirrors defservice): top-level `let` evaluates at macro-expand time;
;; quasiquotes appear only at the tail of each branch. Checker skips non-quasiquote `let`/`fn`
;; binders; generated let/fn binders inside quasiquote slots use `symbol-node` (Unquote at
;; definition time → checker passes). `~reshaped-params` and `~let-binders-vec` are both
;; Unquote nodes at definition time → checker skips their contents.
;;
;; Backward-compat: NO `& [...]` kwargs section → defn behaves EXACTLY as today.
;;
;; The original: `defn` macro just binds a function value to a name:
;; (:wat::core::def :name (:wat::core::fn …)). :wat::core::fn is the one and
;; only function constructor; defn forwards the argspec/arrow/ret/body to it
;; unchanged via rest-binder splicing, and an optional metadata-map threads
;; through too — the substrate peels binding-level metadata from the fn-form,
;; so the macro template stays metadata-blind and UNCHANGED.
(:wat::core::defmacro :wat::core::defn
  [name <- :wat::WatAST
   & rest <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  ;; PROGRAM-BODY path: top-level `let`, quasiquotes only at branch tails.
  (:wat::core::let
    [params-vec   (:wat::core::Option/expect -> :wat::WatAST
                    (:wat::core::first rest)
                    "defn: rest is empty — no params vec")
     params-ch    (:wat::core::ast->children params-vec)
     params-len   (:wat::core::length params-ch)
     ;; Detect `& [...]` tail: params-len >= 2 AND second-to-last is a Symbol named "&"
     ;; AND last element is a Vector node. `& sym <- :T` (variadic rest) is excluded
     ;; because the element right after `&` is a Symbol (not a Vector).
     has-kwargs   (:wat::core::if (:wat::core::i64::>= params-len 2)
                    -> :wat::core::bool
                    (:wat::core::let
                      [stl-node  (:wat::core::Option/expect -> :wat::WatAST
                                   (:wat::core::get params-ch (:wat::core::i64::- params-len 2))
                                   "defn kwargs detect: stl index")
                       last-node (:wat::core::Option/expect -> :wat::WatAST
                                   (:wat::core::get params-ch (:wat::core::i64::- params-len 1))
                                   "defn kwargs detect: last index")]
                      (:wat::core::if (:wat::core::= (:wat::core::ast-kind stl-node) "symbol")
                        -> :wat::core::bool
                        (:wat::core::if (:wat::core::= (:wat::core::ast-name stl-node) "&")
                          -> :wat::core::bool
                          (:wat::core::= (:wat::core::ast-kind last-node) "vector")
                          false)
                        false))
                    false)]
    (:wat::core::if has-kwargs
      -> :wat::WatAST
      ;; ── KWARGS BRANCH (Arc 260.1a) ───────────────────────────────────────────
      (:wat::core::let
        [name-str        (:wat::core::keyword/to-string name)
         ;; :<name>::Kwargs — the minted record type keyword value
         kwargs-ty       (:wat::core::keyword/from-string
                           (:wat::core::string::concat name-str "::Kwargs"))
         kwargs-ty-str   (:wat::core::keyword/to-string kwargs-ty)
         ;; The inner argspec Vector node (the last element of params-ch)
         kw-argvec       (:wat::core::Option/expect -> :wat::WatAST
                            (:wat::core::last params-ch)
                            "defn kwargs: no inner argspec vector")
         kw-ch           (:wat::core::ast->children kw-argvec)
         kw-len          (:wat::core::length kw-ch)
         n-kw-fields     (:wat::core::i64::/ kw-len 3)
         ;; Validate: no nested `&` inside the kwargs section (flat, one level).
         ;; Iterates over field-name positions (0, 3, 6, …); macro-errors on `&`.
         _validate       (:wat::core::foldl
                           (:wat::core::fn [acc <- :wat::core::nil i <- :wat::core::i64]
                             -> :wat::core::nil
                             (:wat::core::let
                               [fname-node (:wat::core::Option/expect -> :wat::WatAST
                                              (:wat::core::get kw-ch (:wat::core::i64::* i 3))
                                              "defn kwargs validate: field name index")]
                               (:wat::core::if (:wat::core::= (:wat::core::ast-name fname-node) "&")
                                 -> :wat::core::nil
                                 (:wat::core::macro-error
                                   "defn kwargs section is flat: no nested & — one level")
                                 nil)))
                           nil
                           (:wat::core::range 0 n-kw-fields))
         ;; Mint record form: (:wat::Record::def :<name>::Kwargs <kw-argvec>)
         record-def      `(:wat::Record::def ~kwargs-ty ~kw-argvec)
         ;; HYGIENIC hidden kwargs binder: fresh-symbol stamps a fresh unique scope (arc 274.1) so the
         ;; binder is capture-proof BY CONSTRUCTION — it cannot collide with any caller variable, even one
         ;; literally named "kwargs". (The field binders below stay plain symbol-node — they are
         ;; INTENTIONALLY user-facing, clojure {:keys}.)
         kw-sym          (:wat::core::fresh-symbol "kwargs")
         ;; kwargs-ty as a WatAST Keyword node (needed for with-children)
         kwargs-ty-node  (:wat::core::keyword-node
                            (:wat::core::string::concat ":" kwargs-ty-str))
         ;; Build reshaped params children: drop trailing `& [...]` (last 2), append kw-sym <- kwargs-ty
         base-ch         (:wat::core::take params-ch (:wat::core::i64::- params-len 2))
         arrow-sym       (:wat::core::symbol-node "<-")
         reshaped-ch     (:wat::core::conj
                           (:wat::core::conj
                             (:wat::core::conj base-ch kw-sym)
                             arrow-sym)
                           kwargs-ty-node)
         reshaped-params (:wat::core::with-children params-vec reshaped-ch)
         ;; ret-type: rest[2] (after params-vec and ->)
         ret-type        (:wat::core::Option/expect -> :wat::WatAST
                            (:wat::core::get rest 2)
                            "defn kwargs: no return type")
         ;; body forms: rest[3..] (everything after params-vec -> ret-type)
         body-forms      (:wat::core::drop rest 3)
         ;; Build destructure let-binder items:
         ;;   [field1-sym (:<name>::Kwargs/field1 __kwargs__)  field2-sym (…) …]
         ;; field-indices: 0, 3, 6, … (name positions in kw-ch)
         field-indices   (:wat::core::map
                           (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64
                             (:wat::core::i64::* i 3))
                           (:wat::core::range 0 n-kw-fields))
         let-binder-items (:wat::core::foldl
                            (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST>
                                             i   <- :wat::core::i64]
                              -> :wat::core::Vector<wat::WatAST>
                              (:wat::core::let
                                [fname-node    (:wat::core::Option/expect -> :wat::WatAST
                                                 (:wat::core::get kw-ch i)
                                                 "defn kwargs let-binder: field name index")
                                 fname-str     (:wat::core::ast-name fname-node)
                                 ;; HYGIENIC field binder: symbol-node → Unquote at def time
                                 binder-sym    (:wat::core::symbol-node fname-str)
                                 ;; Accessor keyword: :<name>::Kwargs/<field-name>
                                 accessor-kw   (:wat::core::keyword/from-string
                                                 (:wat::core::string::concat kwargs-ty-str
                                                   (:wat::core::string::concat "/" fname-str)))
                                 ;; Accessor call: (:<name>::Kwargs/<field> __kwargs__)
                                 accessor-call `(~accessor-kw ~kw-sym)]
                                (:wat::core::conj
                                  (:wat::core::conj acc binder-sym)
                                  accessor-call)))
                            (:wat::core::Vector :wat::WatAST)
                            field-indices)
         ;; Wrap let-binder-items as a WatAST::Vector (kw-argvec is the shape template)
         let-binders-vec (:wat::core::with-children kw-argvec let-binder-items)]
        ;; Emit: (do record-def (def name (fn reshaped-params -> ret (let binders body…))))
        `(:wat::core::do
           ~record-def
           (:wat::core::def ~name
             (:wat::core::fn ~reshaped-params -> ~ret-type
               (:wat::core::let ~let-binders-vec ~@body-forms)))))
      ;; ── BACKWARD-COMPAT PASS-THROUGH (no kwargs section) ────────────────────
      `(:wat::core::def ~name (:wat::core::fn ~@rest)))))

;; Restrictions live as a :restricted-to key in the metadata-map on def/defn
;; (e.g. {:restricted-to [<prefix-kw>…]}); the substrate enforces it.

;; ─── Threading macros `->` / `->>` ───────────────────────────────
;;
;; Thread-first `->`: inject acc as the FIRST arg of each step.
;;   (-> x (f a b) g)  =>  (g (f x a b))
;; A list step `(f a…)` => `(f acc a…)`; a bare symbol/keyword step `f` => `(f acc)`.
;; Empty-list step `()`: Option/expect on (first ()) fires "-> step has no head"
;;   as a panic_any(AssertionPayload) at macro-expansion time (during startup).
(:wat::core::defmacro :wat::core::->
  [acc <- :wat::WatAST & steps <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::holon::HolonAST step <- :wat::holon::HolonAST]
       -> :wat::holon::HolonAST
       (:wat::core::if (:wat::core::List? step) -> :AST<wat::holon::HolonAST>
          `(~(:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first step) "-> step has no head") ~a ~@(:wat::core::rest step))
          `(~step ~a)))
    acc
    steps))

;; Thread-last `->>`: inject acc as the LAST arg of each step.
;;   (->> x (f a b) g)  =>  (g (f a b x))
;; A list step `(f a…)` => `(f a… acc)`; a bare symbol/keyword step `f` => `(f acc)`.
;; Empty-list step `()`: ~@() splices nothing, yielding `(acc)` — expansion succeeds
;;   but eval rejects the integer-head form with MalformedForm at runtime.
(:wat::core::defmacro :wat::core::->>
  [acc <- :wat::WatAST & steps <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::holon::HolonAST step <- :wat::holon::HolonAST]
       -> :wat::holon::HolonAST
       (:wat::core::if (:wat::core::List? step) -> :AST<wat::holon::HolonAST>
          `(~@step ~a)
          `(~step ~a)))
    acc
    steps))

;; Arc 258 Stone 258.2a — cond reborn as a wat macro over bare if.
;; (cond (test body) … (:else bodyN)) expands to nested bare (:wat::core::if …).
;; The legacy annotated form (cond -> :T arm…) is also accepted: the first
;; clause is the symbol `->` (not a List), so we strip -> and :T and
;; re-expand the remainder as a bare cond.
;;
;; cond is TOTAL: a terminal (:else body) arm is required.
;;
;; EMPTY clause list → expansion-time MacroError via keyword/from-string:
;;   (:wat::core::keyword/from-string ":else ...") rejects ':'-prefixed input with a
;;   RuntimeError (via EvalBreak::Diagnostic), which propagates as StartupError::Macro
;;   rather than panic_any, so run_err can capture it. The error message contains ":else".
;; :else arm → emit its body unconditionally (terminal).
;; test arm → (if test body (cond rest…)) and re-expand to fixpoint.
;;
;; Detecting :else: compare head structurally with (first `(:else)) — both are
;; Value::wat__WatAST, so (= head (first `(:else))) is safe for any non-List head
;; (returns false for integers/symbols, true only for the :else keyword form).
;;
;; empty? is checked FIRST (before any Option/expect) so the empty-clause case
;; goes through the RuntimeError channel, not panic_any.
(:wat::core::defmacro :wat::core::cond
  [& clauses <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  (:wat::core::if (:wat::core::empty? clauses)
    ;; empty clause list — non-exhaustive / no terminal :else. Arc 258 Stone 258.2b: use the
    ;; first-class macro-error primitive to abort with a clean diagnostic. This replaces the
    ;; old keyword-sentinel hack (keyword/from-string with a diagnostic name) which carried a
    ;; near-theoretical slip if every arm body was itself a keyword. macro-error returns Err
    ;; directly — the macro engine wraps it into a catchable MacroError without panic or noise.
    (:wat::core::macro-error "cond: non-exhaustive — needs a terminal :else arm")
    (:wat::core::if (:wat::core::List? (:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first clauses) "cond: non-exhaustive — needs a terminal :else"))
      ;; First clause is a List — bare form: (cond (test body) … (:else body))
      (:wat::core::let [arm  (:wat::core::Option/expect -> :wat::holon::HolonAST
                                (:wat::core::first clauses) "cond: empty clause list")
                        head (:wat::core::Option/expect -> :wat::holon::HolonAST
                                (:wat::core::first arm) "cond: arm has no head")]
        (:wat::core::if (:wat::core::List? head)
          ;; test arm — head is a sub-list like (= 1 2): (if head body (cond rest…))
          `(:wat::core::if
              ~head
              ~(:wat::core::Option/expect -> :wat::holon::HolonAST
                  (:wat::core::second arm) "cond: arm has no body")
              (:wat::core::cond ~@(:wat::core::rest clauses)))
          ;; non-List head — detect :else by structural comparison with the :else keyword form.
          ;; (first `(:else)) returns Option<WatAST>; Option/expect unwraps to WatAST::Keyword(":else").
          ;; = on two Value::wat__WatAST nodes uses structural PartialEq (safe for any variant pair).
          (:wat::core::if (:wat::core::= head (:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first `(:else)) "cond: internal: :else form is empty"))
            ;; :else terminal arm — emit body unconditionally
            (:wat::core::Option/expect -> :wat::holon::HolonAST
              (:wat::core::second arm) "cond: :else arm has no body")
            ;; other non-List head — treat as test arm (v1 fallback for malformed input)
            `(:wat::core::if
                ~head
                ~(:wat::core::Option/expect -> :wat::holon::HolonAST
                    (:wat::core::second arm) "cond: arm has no body")
                (:wat::core::cond ~@(:wat::core::rest clauses))))))
      ;; First clause is NOT a List (it is the -> symbol) — annotated form.
      ;; Strip -> and :T (first two elements) and re-expand as bare cond.
      `(:wat::core::cond ~@(:wat::core::rest (:wat::core::rest clauses))))))

;; ─── keyword/of — parametric keyword construction ─────────────────
;;
;; keyword/of — build the parametric keyword `:Head<arg1,arg2>` from keyword args
;; (head + args, leading colons stripped). Pure-total program over forms.
;; Arc 249 Stone 249.4a — promoted from construct_keyword_of (expand.rs).
;; Zero args: string::join "" [] = "", yielding `:Head<>` (empty angle brackets).
(:wat::core::defmacro :wat::core::keyword/of
  [head <- :wat::WatAST & args <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  (:wat::core::let [head-text (:wat::core::keyword/to-string head)
                    arg-texts (:wat::core::map
                                (:wat::core::fn [a <- :wat::holon::HolonAST] -> :wat::core::String
                                   (:wat::core::keyword/to-string a))
                                args)
                    joined (:wat::core::string::join "," arg-texts)
                    full (:wat::core::string::concat head-text
                           (:wat::core::string::concat "<"
                             (:wat::core::string::concat joined ">")))]
    `~(:wat::core::keyword/from-string full)))

;; Stone 245.8 — Polymorphic ordering defclauses RETIRED.
;; `<`/`>`/`<=`/`>=` are now a relational check-side intrinsic (`infer_ordering`
;; in src/check.rs), the sibling of `infer_equality`. The runtime dispatch arms
;; in `dispatch_keyword_head_value` (src/runtime.rs) route directly to `eval_compare`.
;; The per-Type leaves (`:wat::core::i64::<`, `:wat::core::f64::<`, etc.) remain
;; as the type-locked tier in Rust.

;; ─── Instinct-faithful ordering surface (Arc 251 Stone) ──────────────────────
;;
;; `sort'` is the Rust primitive (comparator-sort engine; fn-first `(sort' cmp xs)`).
;; `sort` and `sort-by` are Clojure-exact multi-arity defclauses over `sort'` + `<`.
;; Dispatch is purely by arity (sort: 1 vs 2; sort-by: 2 vs 3).
;; All clauses auto-generalize over bare type-vars T and K (Arc 256 / Stone 251.7).

(:wat::core::defclause :wat::core::sort
  ;; 1-ary: natural ascending — default comparator is <
  ;; T auto-generalizes (bare uppercase type-var, Arc 256 / Stone 251.7).
  ([coll <- :wat::core::Vector<T>] -> :wat::core::Vector<T>
    (:wat::core::sort'
      (:wat::core::fn [a <- :T b <- :T] -> :wat::core::bool
        (:wat::core::< a b))
      coll))
  ;; 2-ary: user-supplied boolean less-than comparator (fn-first, Clojure idiom).
  ;; Cmp is a bare type-var that unifies with the caller's Fn(T,T)->bool.
  ([cmp  <- :Cmp
    coll <- :wat::core::Vector<T>] -> :wat::core::Vector<T>
    (:wat::core::sort' cmp coll)))

(:wat::core::defclause :wat::core::sort-by
  ;; 2-ary: key function only — default comparator is < on the keys.
  ;; Keyfn is a bare type-var that unifies with the caller's Fn(T)->K.
  ([keyfn <- :Keyfn
    coll  <- :wat::core::Vector<T>] -> :wat::core::Vector<T>
    (:wat::core::sort'
      (:wat::core::fn [a <- :T b <- :T] -> :wat::core::bool
        (:wat::core::< (keyfn a) (keyfn b)))
      coll))
  ;; 3-ary: key function + comparator on keys.
  ;; Keyfn and Cmp are bare type-vars.
  ([keyfn <- :Keyfn
    cmp   <- :Cmp
    coll  <- :wat::core::Vector<T>] -> :wat::core::Vector<T>
    (:wat::core::sort'
      (:wat::core::fn [a <- :T b <- :T] -> :wat::core::bool
        (cmp (keyfn a) (keyfn b)))
      coll)))

;; ── nth — the positional, TOTAL accessor ─────────────────────────────────────
;;
;; `Vector/get` is the associative, nil-safe form (`Vec<T> × i64 -> Option<T>`,
;; None on out-of-range). `nth` is Clojure's positional idiom: the i-th element
;; returned as `T`, RAISING on out-of-range — "there IS an i-th element; give it
;; or fail." Sugar over `Option/expect (Vector/get …)`, but with the total promise.
(:wat::core::defn :wat::core::nth<T> [v <- :wat::core::Vector<T> i <- :wat::core::i64] -> :T
  (:wat::core::Option/expect -> :T (:wat::core::Vector/get v i) "nth: index out of range"))
