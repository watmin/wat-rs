;; Excursus 003 strike D3, GD2a — the activation census is a standing gate.
;;
;; BRIEF-shape-strike-D3-the-activation-census-is-a-gate.md item 1. One fn per
;; `RuntimeErrorKind` whose real producer is reachable through ordinary runtime
;; dispatch (keyword/special-form dispatch, the application path, `apply_function`,
;; or the `:wat::eval-ast!`/`:wat::eval-step!`/`:wat::eval-digest-string!` eval
;; family) — driven via `call_beside_value(file!(), fn_name)` from the co-located
;; `.rs`, never hand-constructed.
;;
;; Freeze-time/registration kinds (DuplicateDefine, ReservedPrefix, UnreachableClause,
;; UnnamespacedName, DottedName, ReteDefnAxisViolation, ReteDefnRecursive,
;; UserMainMissing) live in their own separate fixtures beside this one — a kind that
;; makes FREEZING ITSELF fail cannot sit in a fixture every other probe here also
;; needs to freeze cleanly.

;; No `:user::main` — this fixture is driven entirely via `call_beside_value`'s
;; named-fn fetch, never run as a program; `startup_from_source`'s `:user::main`
;; wall is conditional on main being declared at all (`startup_bare()` passes
;; cleanly with none), so omitting it avoids the UselessMain check a trivial
;; main would otherwise trip.

;; ─── UnboundSymbol — writer 3 (symbol application path), via eval-ast! (code
;; position inside a quote is never resolved by normalize_symbol_refs, so this
;; reaches eval_list's Symbol arm genuinely unbound, not rejected at check time) ──
(:wat::core::defn :t::probe-unboundsymbol [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote (zzz-nowhere-bound 1))))

;; ─── UnknownFunction — writer 1, via :wat::core::apply's dynamic dispatch (a
;; keyword VALUE the checker cannot statically resolve) ──────────────────────────
(:wat::core::defn :t::probe-unknownfunction [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote (:wat::core::apply :my::totally-unregistered-xyz []))))

;; ─── NotValueDispatchable — writer 1 (:wat::core::apply), a BINDING-only
;; registered intrinsic with no value-level door (BRIEF-STONE-O-iv-a-the-honest-word.md) ─
(:wat::core::defn :t::probe-notvaluedispatchable [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote (:wat::core::apply :wat::f64::max-of [3.0 9.0 41.0]))))

;; ─── NotCallable — writer 3, mirrors the landed GD2b fixture exactly ───────────
(:wat::core::defn :t::probe-notcallable [] -> :wat::core::i64
  (:wat::core::let [f 5]
    (f 1)))

;; ─── TypeMismatch — writer 1 (:wat::i64::+), via eval-ast! (a literal arg-type
;; mismatch at a known call site is caught by check_program; routing through the
;; unchecked eval family reaches the RUNTIME TypeMismatch instead) ──────────────
(:wat::core::defn :t::probe-typemismatch [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote (:wat::i64::+ 1 "x"))))

;; ─── ArityMismatch — writer 1 (:wat::i64::+ is fixed 2-ary) ────────────────────
(:wat::core::defn :t::probe-aritymismatch [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote (:wat::i64::+ 1))))

;; ─── BadCondition — writer 1 (:wat::core::if), GE1's own shape (src/runtime.rs) ─
(:wat::core::defn :t::probe-badcondition [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote (:wat::core::if 5 1 0))))

;; ─── MalformedForm — writer 1 (:wat::core::if), wrong arity (missing branches) ──
(:wat::core::defn :t::probe-malformedform [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote (:wat::core::if true))))

;; ─── DivisionByZero — writer 1 (:wat::i64::/) ──────────────────────────────────
(:wat::core::defn :t::probe-divisionbyzero [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote (:wat::i64::/ 1 0))))

;; ─── IntegerOverflow — writer 1 (:wat::i64::+), i64::MAX + 1 ───────────────────
(:wat::core::defn :t::probe-integeroverflow [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote (:wat::i64::+ 9223372036854775807 1))))

;; ─── DeclarationInExpressionPosition — writer 1 (the declaration head itself).
;; `:wat::core::define` is HARD CUT (Stone 241.11/241.16) — it is no longer even
;; a registered keyword, so nesting it raises UnknownFunction, not this kind
;; (empirically confirmed: the first version of this fixture used `define` and
;; got exactly that). `:wat::core::defalias` IS still registered `@Purity
;; Unevaluated` with no alias, and is NOT on `is_mutation_head`'s list
;; (`src/runtime.rs`), so it survives eval-ast!'s mutation-form pre-scan and
;; reaches `dispatch_keyword_head`'s own Unevaluated-no-alias branch instead. ──
(:wat::core::defn :t::probe-declarationinexpressionposition [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote (:wat::i64::+ 1 (:wat::core::defalias :user::zzz :wat::i64::+)))))

;; ─── EvalForbidsMutationForm — writer 1 (:wat::eval-ast! itself), mirrors
;; tests/value/wat_eval_result.wat's test2 exactly ────────────────────────────────
(:wat::core::defn :t::probe-evalforbidsmutationform [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote (:wat::core::defstruct :evil::T [x <- :wat::core::i64]))))

;; ─── EvalVerificationFailed — writer 1 (:wat::eval-digest-string! itself),
;; mirrors tests/value/wat_eval_result.wat's test4 exactly ───────────────────────
(:wat::core::defn :t::probe-evalverificationfailed [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-digest-string!
    "(:wat::holon::to-holon \"x\")"
    :wat::verify::digest-sha256
    :wat::verify::string "0000000000000000000000000000000000000000000000000000000000000000"))

;; ─── PatternMatchFailed — writer 1 (:wat::core::match), GE1's own shape ────────
(:wat::core::defn :t::probe-patternmatchfailed [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote
    (:wat::core::match 5
      [:wat::core::Option.Some {:value n} n]
      [:wat::core::Option.None {} 0]))))

;; ─── EffectfulInStep — writer 1 (:wat::eval-step! itself), GE1's own shape ─────
(:wat::core::defn :t::probe-effectfulinstep [] -> (:wat::core::Result :- [:wat::eval::StepResult :wat::kernel::Failure])
  (:wat::eval-step! (:wat::core::quote
    (:wat::kernel::assertion-failed!' "x" :wat::core::Option.None :wat::core::Option.None))))

;; ─── NoStepRule — writer 1 (:wat::eval-step! itself), GE1's own shape ──────────
(:wat::core::defn :t::probe-nosteprule [] -> (:wat::core::Result :- [:wat::eval::StepResult :wat::kernel::Failure])
  (:wat::eval-step! (:wat::core::quote (:wat::holon::from-wat x))))

;; ─── MacroExpansionFailed — writer 1 (:wat::core::macroexpand), a self-
;; referential macro that never converges (ExpansionDepthExceeded).
;;
;; Two things empirically disproved before this shape:
;; - A BARE self-call (`(:t::self-loop)` -> `(:t::self-loop)`) reproduces the
;;   SAME bytes every step, so `src/reflect/expand.rs`'s runtime fixpoint
;;   walker sees "stopped changing" and returns success immediately.
;; - Wrapping the OUTER form once (`(:wat::core::do (:t::self-loop))`) changes
;;   the TOP-LEVEL head away from the macro's own name after one step, so the
;;   walker — which only re-checks whether the CURRENT top-level head is still
;;   a macro, not whether a macro call survives somewhere nested — stops
;;   there too, leaving the inner `:t::self-loop` unexpanded forever.
;; The real shape needs the TOP-LEVEL head to stay `:t::self-loop` (so the
;; walker keeps re-expanding) while the tree still changes every step (so it
;; never reaches a byte-identical fixpoint): grow the carried argument instead
;; of the outer form.
(:wat::core::defmacro :t::self-loop [n <- :wat::WatAST] -> :wat::WatAST
  `(:t::self-loop (:wat::core::list ~n)))

(:wat::core::defn :t::probe-macroexpansionfailed [] -> :wat::WatAST
  (:wat::core::macroexpand (:wat::core::quote (:t::self-loop 1))))

;; ─── ServiceNotRunning — writer 1 (:wat::kernel::println), the main thread's
;; ThreadIO is never primed outside bootstrap_wat_vm_process ─────────────────────
(:wat::core::defn :t::probe-servicenotrunning [] -> :wat::core::nil
  (:wat::kernel::println "x"))

;; ─── UnknownField — writer 1 (the unknown accessor keyword itself) ─────────────
(:wat::core::defstruct :t::R [a <- :wat::core::i64])

(:wat::core::defn :t::probe-unknownfield [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote (:bogus-field (:t::R :a 1)))))

;; ─── NoMatchingClause — writer 1 (the defclause's own FQDN) ────────────────────
(:wat::core::defclause :t::pick02
  ([x <- :wat::core::i64] :guard (:wat::i64::> x 100) -> :wat::core::i64 x))

(:wat::core::defn :t::probe-nomatchingclause [] -> :wat::core::i64
  (:t::pick02 1))

;; ─── PostconditionFailed — writer 1 (the defclause's own FQDN) ─────────────────
(:wat::core::defclause :t::pick07
  ([x <- :wat::core::i64] -> :wat::core::i64
    :ensure (:wat::core::fn [result <- :wat::core::i64] -> :wat::core::bool
              (:wat::i64::> result 0))
    x))

(:wat::core::defn :t::probe-postconditionfailed [] -> :wat::core::i64
  (:t::pick07 -5))

;; ─── ArityMismatch, writer-4 row — GD2a's mutation gate needs at least one row
;; actually named by `apply_function`'s OWN guard (writer 4), where that naming is
;; OBSERVABLY different from every other writer's. Empirically disproved first
;; shape: a KEYWORD-headed call to a user fn (`(:t::probe-helper 1)`) has writer 1
;; (`dispatch_keyword_head`) already enter the SAME name before `apply_function`
;; ever runs — mutating writer 4's guard away changed nothing (the activation was
;; already correct, coincidentally, from writer 1). The shape that actually
;; isolates writer 4: a SYMBOL-headed call to a local bound to an ANONYMOUS fn
;; value. Writer 3 enters the symbol `f` AS WRITTEN; once `f` resolves to a real
;; function, `apply_function`'s own guard (writer 4) SUPERSEDES it with the
;; callee's own display name — for an anonymous fn, the FQDN of its TYPE
;; (`:wat::core::Fn`, `ANON_FN_SYMBOL`, `src/value/frame.rs`), never `"f"`. With
;; writer 4 intact: `:wat::core::Fn`. With writer 4 mutated away: stuck at `"f"`
;; (writer 3's name, never overwritten) — a genuinely different, observable string.
(:wat::core::defn :t::probe-aritymismatch-via-apply [] -> (:wat::core::Result :- [:wat::holon::HolonAST :wat::kernel::Failure])
  (:wat::eval-ast! (:wat::core::quote
    (:wat::core::let [f (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ a b))]
      (f 1)))))

;; MacroAbort deliberately has no fn here — measured, not driven. Its ONLY real
;; producer is `(:wat::core::macro-error "msg")` inside a `defmacro` body during
;; EXPANSION (`:wat::core::macro-error` is check-time refused anywhere else,
;; `#wat.macro/ExpandOnlyOutsideMacro` — confirmed empirically: an ordinary
;; top-level call, as `tests/diagnostics/probe_excursus003_d3_gd2a_census.rs`'s
;; strike report describes, breaks freeze for the WHOLE fixture rather than
;; raising at runtime). That one path is intercepted by `macro_eval_pre_validated`
;; (`src/macros/eval.rs`), which FLATTENS the raised `RuntimeErrorKind::MacroAbort`
;; into `MacroErrorKind::MalformedTemplate{reason}` — discarding the frame
;; entirely, before any observer. See `tests/macros/probe_arc258_stone2b_macro_error_c03.wat`
;; / `.rs`'s `contract_03`, which proves exactly this flattening. No strike has
;; fixed it (strike E fixed the analogous eval-ast!/eval-step! flattening, not
;; this one), so there is no path through which MacroAbort's own frame is
;; observable today — the census lists it, it does not fake a producer here.
