;; SPECIMEN — STOP-1, found while curing F-006/F-114 and driven at that cure.
;;
;; `register_variant_types` mints a singleton `TypeDef::Enum` per variant through
;; `register_variant_type`, a SIBLING door that never passes through `register_validated` — so a
;; variant singleton had no declaration span even after the cure. Both post-registration walks
;; iterate `env.iter()`, which holds `:t::Shape` and `:t::Shape.Circle` side by side, and HashMap
;; order decides which is refused first: this program reported the user's file on some runs and
;; `src/check.rs` on others, 8 runs split 4/4. A variant singleton now inherits its enum's span.
;;
;; ⛔ Do not "fix" `:t::NoSuchType` — the type not existing is the point.
(:wat::core::defenum :t::Shape :wat::enum::Pure
  :Circle [r <- :t::NoSuchType])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "unreachable — startup refuses first"))
