;; PROBE — room 4 judgment call, site `src/runtime.rs:1715` (value-position `:None`/
;; `:wat::core::None` keyword evaluation). Does the qualified spelling `:wat::core::Option::None`
;; reach this site at VALUE position (as opposed to pattern position, covered by the main probe
;; `probe-runtime-qualified-builtin-variant.wat`)?
;;
;; ⭐ MEASURED FINDING: yes, and extending it was load-bearing, not cosmetic. Before the fix,
;; `:wat::core::Option::None` at value position was NOT intercepted at :1715 (exact-string guard),
;; so it fell through to the generic user-enum unit-variant door (`sym.unit_variant`) — which
;; ALSO answers for it, because `wat::core::Option` is registered as a genuine `TypeDef::Enum`
;; (`types.rs:1239`, part of this same arc). That generic door constructs `Value::Enum`, NOT the
;; native `Value::Option` — a real cross-representation split. Measured live, pre-fix: matching
;; a value built this way against the EXISTING bare `:None` pattern raised `PatternMatchFailed`
;; with `value-type "wat::core::Enum"` rather than matching. Extending :1715 with the qualified
;; string, so it runs BEFORE the generic fallback, closes that split — the qualified None value
;; now stays on the native `Value::Option` representation like its bare/FQDN siblings.
;;
;; This probe demonstrates the fixed state: a value constructed with the qualified spelling at
;; value position, matched against the pre-existing BARE `:None` pattern (not the qualified
;; pattern — that cross-spelling round trip is the point).

(:wat::core::defn :user::give-none [] -> (:wat::core::Option :- [:wat::core::i64])
  :wat::core::Option::None)

(:wat::core::defn :user::check [] -> :wat::core::i64
  (:wat::core::match (:user::give-none)
    ((:wat::core::Some x) x)
    (:wat::core::None -1)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::check)))
