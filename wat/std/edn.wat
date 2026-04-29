;; :wat::edn::Tagged + :wat::edn::NoTag — newtype wrappers around
;; HolonAST that signal which EDN write strategy the substrate should
;; use when binding to a TEXT column (sqlite auto-dispatch / arc 085).
;;
;; A field declared `:wat::edn::Tagged` carries a HolonAST that the
;; substrate writes via `:wat::edn::write` — round-trip-safe; readable
;; back via `:wat::edn::read`. Use for log message data, anything that
;; the consumer wants to parse back into HolonAST.
;;
;; A field declared `:wat::edn::NoTag` carries a HolonAST that the
;; substrate writes via `:wat::edn::write-notag` — drops `#namespace/Type`
;; tags from struct + enum-variant renders, producing the natural form
;; humans (and SQL queries) read directly. Use for indexed columns
;; (namespace, metric_name, dimensions, etc.) where the natural form is
;; what queries match against.
;;
;; Both newtypes are the same shape at runtime — `Value::Struct`
;; arity 1 carrying the inner HolonAST at field index 0 (per arc 049's
;; tuple-struct compilation). The auto-dispatch shim looks at the field's
;; declared type-name (`:wat::edn::Tagged` vs `:wat::edn::NoTag`) to
;; pick the write strategy. The Value's runtime payload is the same
;; HolonAST either way.
;;
;; Constructors auto-derived per arc 049:
;;   `:wat::edn::Tagged/new` — `(:fn(:wat::holon::HolonAST) -> :wat::edn::Tagged)`
;;   `:wat::edn::NoTag/new`  — `(:fn(:wat::holon::HolonAST) -> :wat::edn::NoTag)`
;; Plus accessors `:wat::edn::Tagged/0` and `:wat::edn::NoTag/0` for the
;; inner HolonAST (mirrors Rust tuple-struct `.0`).

(:wat::core::newtype :wat::edn::Tagged :wat::holon::HolonAST)
(:wat::core::newtype :wat::edn::NoTag  :wat::holon::HolonAST)
