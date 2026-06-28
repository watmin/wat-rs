;; tests/wat_lang/probe_def_not_special.wat — co-located fixture.
;; Arc 170 slice 3 Gap I-B: probe 2 (def at expression position) +
;; probe 3 (top-level def still works). Probe 4 uses bad file.
;; Probes 1 and 5 use separate spawn fixtures.

;; Probe 2: def at expression position — startup succeeds; calling :my::bad
;; emits DeclarationInExpressionPosition at runtime.
(:wat::core::defn :my::bad [] -> :wat::core::nil (:wat::core::def :x 1))

;; Probe 3: def at top-level still works.
(:wat::core::def :my-answer 42)
(:wat::core::defn :my::compute [] -> :wat::core::i64 :my-answer)

;; Required for startup (child processes still need :user::main below them).
