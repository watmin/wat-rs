;; probe-call-site-through-macro.wat — WHERE does `call-site` land for MACRO-GENERATED code?
;;
;; The #6 origin lift binds `(:wat::kernel::call-site)` inside a `defn` that a macro emitted
;; (defservice's start/resume). The builder reports the resulting Service labels all name
;; wat/core.wat rather than the caller. Two candidate explanations, and this file separates
;; them instead of guessing:
;;
;;   A. the enclosing `(:wat::core::let …)` pushes a frame, so `.first()` is the `let`, not
;;      the caller  -> case 1 below would report THIS file's let-line, not main's call-line.
;;   B. a MACRO-GENERATED body carries the macro template's spans, so `call-site` inside one
;;      reports wherever the template lives -> case 2 reports this file's macro body.
;;
;; Expected if NEITHER holds (the lift is correct as written): both cases report the CALL
;; line in `:user::main`, and the two differ from each other.

;; ── case 1: call-site inside a `let`, hand-written (the map-worker shape) ─────────────
(:wat::core::defn :probe::via-let [] -> :wat::kernel::Frame
  (:wat::core::let
    [origin (:wat::kernel::call-site)]
    origin))

;; ── case 2: call-site inside a MACRO-GENERATED defn (the defservice start shape) ──────
(:wat::core::defmacro :probe::gen-spawner [name <- :wat::WatAST] -> :wat::WatAST
  `(:wat::core::defn ~name [] -> :wat::kernel::Frame
     (:wat::core::let
       [origin (:wat::kernel::call-site)]
       origin)))

(:probe::gen-spawner :probe::via-macro)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:probe::via-let))     ;; <- expect THIS line
    (:wat::kernel::println (:probe::via-macro)))) ;; <- expect THIS line (a different one)
