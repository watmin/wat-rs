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
;;   :Intrinsic — implemented in Rust, exposed under a :wat:: FQDN
;;   :Fn        — a user-defined :wat::core::defn
;;   :Macro     — a user-defined :wat::core::defmacro
(:wat::core::defenum :wat::runtime::Kind
  :Macro
  :Fn
  :Intrinsic)

;; DefinedIn — implementation language.
;;   :Rust — written in Rust (all intrinsics)
;;   :Wat  — written in wat (user defn / defmacro)
(:wat::core::defenum :wat::runtime::DefinedIn
  :Wat
  :Rust)

;; Layer — where in the system stack does this live?
;;   :Substrate — kernel/stdlib layer (all intrinsics)
;;   :Userland  — user-written code above the substrate
(:wat::core::defenum :wat::runtime::Layer
  :Substrate
  :Userland)
