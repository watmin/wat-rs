;; wat/core.wat — :wat::core::* dispatches.
;;
;; Substrate dispatches that route polymorphic-name primitives to
;; per-Type impls. Per arc 146 DESIGN: one entity-kind (dispatch) for
;; genuinely-polymorphic primitives; per-Type impls live in Rust as
;; clean rank-1 schemes registered in `register_builtins`.
;;
;; Each declaration uses arc 146's `:wat::core::define-dispatch`
;; (slice 1). Loads BEFORE `wat/runtime.wat` so the dispatches are
;; visible to any reflection-driven macro that might reference them.

(:wat::core::define-dispatch :wat::core::length
  ((:wat::core::Vector<T>)    :wat::core::Vector/length)
  ((:wat::core::HashMap<K,V>) :wat::core::HashMap/length)
  ((:wat::core::HashSet<T>)   :wat::core::HashSet/length))
