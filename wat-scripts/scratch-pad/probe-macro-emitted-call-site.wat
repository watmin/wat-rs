;; probe-macro-emitted-call-site.wat — does a MACRO-EMITTED call inherit the invocation span?
;;
;; ⚠ THIS DOES NOT MEASURE BRACKETS, and an earlier version of this header claimed it did.
;; It is a REPLICA of what bracket's shape was believed to be — it never calls
;; `:wat::bracket::map`, `map-worker`, or a pool, so it proves nothing about production.
;; The real bracket measurement is probe-bracket-label-real.wat (runs an actual pool, reads
;; each worker's own /proc/self/cmdline). Kept because the property it DOES measure is real
;; and worth a standing witness: a call emitted by a macro template reports the CALLER.
;;
;; THE SHAPE (bracket.wat's arrangement, MEASURED for real in probe-bracket-label-real.wat):
;;
;;     user writes   (bracket-map …)          <- a MACRO
;;       expands to  (map-worker …)           <- a call EMITTED BY THE TEMPLATE
;;         inside map-worker: (call-site)     <- what does IT see?
;;
;; `map-worker` is positional, so there is no kwargs adapter frame — that much was checked.
;; But the call to map-worker is EMITTED BY A MACRO, so before the restamp (c8d002fa) it
;; carried wat/bracket.wat's TEMPLATE span — which would make every Bracket label in `ps`
;; name bracket.wat, CONSTANT across every pool and every caller. The restamp should now
;; give it the user's invocation instead. "Should" is the word that has been wrong all day,
;; so this measures it.
;;
;; Distinct from probe-call-site-through-macro.wat: there, the macro generated a DEFN and the
;; USER called it directly (so the call was user-written). Here the CALL ITSELF is emitted by
;; the template — the bracket arrangement, and the case that was never probed.
;;
;;   PASS: both report THEIR OWN caller line in main (30 and 31), and they DIFFER.
;;   FAIL: both report this file's macro-definition line, identical — the constant label.

(:wat::core::defn :probe::inner [] -> :wat::kernel::Frame
  (:wat::core::let
    [origin (:wat::kernel::call-site)]
    origin))

;; the macro EMITS the call to :probe::inner — mirroring bracket-map emitting (map-worker …)
(:wat::core::defmacro :probe::emit-call [] -> :wat::WatAST
  `(:probe::inner))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:probe::emit-call))
    (:wat::kernel::println (:probe::emit-call))))
