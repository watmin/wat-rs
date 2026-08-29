;; probe-bracket-label-real.wat — does a REAL bracket worker's ps label name the CALLER?
;;
;; ⚠ THIS REPLACES A BAD PROBE. `probe-macro-emitted-call-site.wat` measured a REPLICA I built
;; of what I believed bracket's shape to be — a macro emitting a call to a defn. It never
;; touched `:wat::bracket::map`, `map-worker`, or a pool, so it proved something about my model
;; and nothing about production. A probe that does not walk the exact substrate path production
;; walks proves nothing about production. This one runs an actual pool.
;;
;; WHAT IS UNDER TEST: `:wat::bracket::map` is a MACRO whose template emits
;; `(:wat::bracket::map-worker ~locus ~items …)` (wat/bracket.wat:795). Inside `map-worker`,
;; `(:wat::kernel::call-site)` supplies the `file`/`line` for each runner's
;; `#wat.process/Bracket {:id N :file … :line …}` ps label. If that emitted call carries the
;; TEMPLATE's span, every Bracket label in `ps` names wat/bracket.wat — CONSTANT across every
;; pool and every caller, useless for telling two concurrent pools apart.
;;
;; MEASUREMENT: run the pool on a PROCESS locus, and have each worker read its OWN
;; /proc/self/cmdline — the real OS argv the label was written into, not an in-process claim.
;;
;;   PASS: each worker's cmdline carries this file's path and the `<<map-call>>` line below.
;;   FAIL: it carries wat/bracket.wat (the template) — the constant label.

(:wat::core::defn :probe::own-cmdline [_i <- :wat::core::i64] -> :wat::core::String
  (:wat::io::read-file "/proc/self/cmdline"))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [out (:wat::bracket::map (:wat::spawn::process) (:wat::core::Vector :- [:wat::core::i64] 1 2)
           :probe::own-cmdline)] ;; <<map-call>>
    (:wat::kernel::println out)))
