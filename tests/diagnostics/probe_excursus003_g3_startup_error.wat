;; tests/diagnostics/probe_excursus003_g3_startup_error.wat — excursus 003 step 3b, gate G3's
;; StartupError producer. The retired colon spelling of `Option::Some` (arc 255's dot-flip) —
;; a `:wat::*` head the resolve pass checks. Same trigger as
;; `probe_arc255_the_blanket_hides_a_phantom_head__colon_spelling_is_refused.wat`. Loaded via
;; `include_str!` and run through a direct `wat` binary invocation (exec) — driven that way
;; because measured: through `:wat::test::spawn-peer`'s process locus (fork), this exact
;; trigger lands as a runtime Panic instead of StartupError.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [_ (:wat::core::Option::Some {:value 7})] nil))
