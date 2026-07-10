;; Arc 293 — surface-splice CONFLICT probe (must be REJECTED once splice ships).
;;
;; The merge rule (builder, 2026-07-04): a field name appearing across splices (or splice+own)
;; must carry an IDENTICAL type — same-name-same-type dedupes to one field; same-name-DIFFERENT-type
;; is unrepresentable and MUST NOT compile. Here `foobar` is installed as :i64 by A and :String by B.
;;
;; NEGATIVE arm: startup must FAIL with a conflict/malformed-decl error. (At HEAD it also fails —
;; splice is unbuilt — so this arm becomes meaningful only after splice ships; the positive probe is
;; the RED gate. This fixture is the build's own correctness check that the conflict is caught.)

(:wat::core::defsurface :probe::HasIntFoobar :nature :wat::core::Record
  :features [foobar <- :wat::core::i64])

(:wat::core::defsurface :probe::HasStrFoobar :nature :wat::core::Record
  :features [foobar <- :wat::core::String])

;; splices two surfaces that install `foobar` at CONFLICTING types → MUST be rejected.
(:wat::core::defrecord :probe::Conflict
  [~@:probe::HasIntFoobar
   ~@:probe::HasStrFoobar
   own <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "should never reach here — Conflict must fail to declare"))
