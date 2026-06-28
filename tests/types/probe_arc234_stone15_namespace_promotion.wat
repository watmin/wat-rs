;; tests/types/probe_arc234_stone15_namespace_promotion.wat
;; Co-located fixture for probe_arc234_stone15_namespace_promotion.rs (arc 234 Stone 234.1.5).
;; Only probe 4 uses WAT; probes 1-3 and 5 are pure Rust substrate tests.

;; Probe 4: :wat::Record type annotation is accepted by the type checker.
(:wat::core::defn :user::accept-record [_v <- :wat::Record] -> :wat::core::nil nil)
(:wat::core::defn :user::probe-4 [] -> :wat::core::nil nil)
