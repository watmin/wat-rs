;; The wat source the synthetic "alpha" battery pair contributes. See
;; tests/cli/synthetic_battery.rs — stands in for one downstream
;; #[wat_dispatch] extension crate's `wat/` directory. A trivial no-op:
;; the fixture's claim is about the Battery PLUMBING (does the tuple's
;; wat_sources() fn get called, and does its content survive the round
;; trip), not about what the wat itself does.
(:wat::core::defn :synthetic::alpha-noop [] -> :wat::core::nil ())
