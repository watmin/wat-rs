;; arc 255.67 — the SAME canonical :user::main contract, the cutover's new spelling:
;; `[] -> wat.type/nil` must be recognized as identical to `[] -> :wat::core::nil`
;; (the denotation door), not refused as BareLegacyMainSignature. Found via this
;; stone's own stdlib conversion: `wat/test.wat`'s `deftest` macro emits exactly this
;; shape, and hundreds of `wat-tests/**/*.wat` deftests failed to freeze until
;; `check_legacy_user_main_signature` (src/check.rs) learned to denote before comparing.
(wat.core/defn user/main [] :- wat.type/nil (wat.kernel/println "new-spelling main ran"))
