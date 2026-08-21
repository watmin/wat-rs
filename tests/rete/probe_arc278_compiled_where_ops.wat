;; Fixture for probe_arc278_compiled_where_ops.rs — DESIGN-STONE-compiled-where's disconfirming
;; probe (FM 2-bis: drawn BEFORE the brief).
;;
;; The corpus's most common non-arithmetic `where` shape is a record accessor over a bound ?var —
;; `(:arena::Route/status ?route)`. The compiled executor must reach that field's value with no
;; `Environment` and no head dispatch. This fixture supplies the two halves the probe compares:
;; the value AS THE INTERPRETER PRODUCES IT (through the keyword-as-accessor fall-through, the LAST
;; arm of eval_inner's head dispatch — keyword_accessor_record), and the record itself, so the probe
;; can read field 0 directly the way a compiled `Op::Field` would.
;;
;; Kept deliberately minimal: this proves a MECHANISM is reachable, not a behaviour.

(:wat::core::defrecord :p::Route
  [status <- :wat::core::i64
   method <- :wat::core::String])

;; The record under test — one constructor, used by both entry points below so the probe is
;; comparing two readings of the SAME value, never two different constructions.
(:wat::core::defn :p::the-record [] -> :p::Route
  (:p::Route :status 200 :method "POST"))

;; Half one: the field read the way the interpreter does it today — through the accessor form.
(:wat::core::defn :p::status-via-accessor [] -> :wat::core::i64
  (:p::Route/status (:p::the-record)))

;; wat-scripts/tests fixtures load under the every_wat_scripts_file_loads gate; a :user::main keeps
;; this file a complete program rather than a fragment.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:p::status-via-accessor)))
