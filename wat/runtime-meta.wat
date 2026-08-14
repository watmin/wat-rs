;; wat/runtime-meta.wat — arc 255.1b-iv-c: closed-domain enum types for
;; :wat::runtime::metadata-of reflection surface.
;;
;; Three unit-only enums: Kind / DefinedIn / Layer.
;; Capitalized variants (§5 locked-record-model decision): avoids any `:fn`
;; keyword-legality question and signals these are closed-domain type values
;; (not data keywords).
;;
;; Loading order: no eval-deps beyond :wat::core::defenum (a builtin).
;; May be placed anywhere after wat/core.wat.

;; Kind — what kind of callable is this?
;;   :Intrinsic   — implemented in Rust, exposed under a :wat:: FQDN
;;   :Fn          — a user-defined :wat::core::defn
;;   :Macro       — a user-defined :wat::core::defmacro
;;   :SpecialForm — a substrate special form (no NativeHandler; runtime-dispatched)
(:wat::core::defenum :wat::runtime::Kind :wat::enum::Pure
  :Macro
  :Fn
  :Intrinsic
  :SpecialForm)

;; DefinedIn — implementation language.
;;   :Rust — written in Rust (all intrinsics)
;;   :Wat  — written in wat (user defn / defmacro)
(:wat::core::defenum :wat::runtime::DefinedIn :wat::enum::Pure
  :Wat
  :Rust)

;; Layer — where in the system stack does this live?
;;   :Substrate — kernel/stdlib layer (all intrinsics)
;;   :Userland  — user-written code above the substrate
(:wat::core::defenum :wat::runtime::Layer :wat::enum::Pure
  :Substrate
  :Userland)

;; Purity — declared purity of an intrinsic or special form.
;;   :Pure      — produces the same output for the same input, no observable side effects
;;   :Effectful — has observable side effects (I/O, mutation, etc.)
;;   :Preserving — special forms that preserve the purity of their sub-forms
(:wat::core::defenum :wat::runtime::Purity :wat::enum::Pure
  :Pure
  :Effectful
  :Preserving)

;; Determinism — declared determinism of an intrinsic or special form.
;;   :Deterministic    — same input always produces same output
;;   :Nondeterministic — output may differ across calls (e.g. UUID, random)
;;   :Preserving       — special forms that preserve the determinism of their sub-forms
(:wat::core::defenum :wat::runtime::Determinism :wat::enum::Pure
  :Deterministic
  :Nondeterministic
  :Preserving)

;; Category — functional category.
;; Category — what kind of computation an intrinsic or special form performs.
;; ONE axis throughout: what the verb DOES. Not what it returns, not where its
;; input comes from, not which direction it crosses a type boundary — each of
;; those was proposed as a variant during arc 255 and rejected for mixing axes.
;;   :Transform   — returns the SAME value in another form (was :Encoding, renamed
;;                  2026-08-15: trim/to-lowercase/split are not encodings)
;;   :Reflection  — the program interrogating ITSELF (metadata-of, show-source)
;;   :ControlFlow — directs evaluation (if, and higher-order application)
;;   :Binding     — introduces a LOCAL, scoped name at runtime (let)
;;   :Clock       — samples the wall clock (names WHICH external source a
;;                  Nondeterministic verb draws from; entropy gets its own variant)
;;   :Arithmetic  — math on numeric domain values
;;   :Io          — input/output on a stream
;;   :Probe       — interrogates a value, derives a FACT about it (empty?, length)
;;   :Combine     — builds a larger value of the same kind (concat, conj, assoc)
;;   :Declaration — registers a program-level entity (def, defclause,
;;                  declare-acronyms). Distinct from :Binding — a declaration
;;                  registers into the program, visible to everything after it.
;;
;; ⛔ ORDER AND MEMBERSHIP ARE CHECKED against the Rust enum by
;; `intrinsic::wat_mirror_tests::every_rust_enum_matches_its_wat_defenum`.
;; The Rust side is DERIVED from the enum; this side is hand-written. Drift here
;; goes red there.
(:wat::core::defenum :wat::runtime::Category :wat::enum::Pure
  :Transform
  :Reflection
  :ControlFlow
  :Binding
  :Clock
  :Arithmetic
  :Io
  :Probe
  :Combine
  :Declaration)
