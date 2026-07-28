;; The wat source the synthetic "beta" battery pair contributes. See
;; tests/cli/synthetic_battery.rs — stands in for a SECOND downstream
;; #[wat_dispatch] extension crate, composed alongside "alpha" to prove
;; multi-battery slices don't clobber each other.
(:wat::core::defn :synthetic::beta-noop [] -> :wat::core::nil nil)
