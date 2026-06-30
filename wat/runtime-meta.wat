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
;;   :Encoding    — data encoding/decoding
;;   :Reflection  — reflection and introspection
;;   :ControlFlow — control flow special forms (if, cond, etc.)
;;   :Binding     — binding special forms (let, etc.)
(:wat::core::defenum :wat::runtime::Category :wat::enum::Pure
  :Encoding
  :Reflection
  :ControlFlow
  :Binding)
