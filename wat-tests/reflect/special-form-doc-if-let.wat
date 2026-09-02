;; wat-tests/reflect/special-form-doc-if-let.wat — Stone 255.SF probe (wat-direct).
;;
;; The special-form half of the intrinsic doc-contract, proven on `if` (the
;; @arg-fits shape) and `let` (the @syntax-carries shape) — the two exemplars
;; that freeze the `wat_special_form!` contract, the way `bytes` froze the
;; value-intrinsic one.
;;
;; RED at HEAD: `:wat::core::if` and `:wat::core::let` are NOT in the registry
;; (special_forms.rs carries an UNENFORCED `signature: HolonAST` sketch +
;; `doc_string: None`). `render-doc` routes through `registry().lookup_entry`
;; → None → raises "no registered intrinsic found for FQDN" → these tests fail.
;;
;; GREEN after 255.SF: both register as `Kind::SpecialForm` (handler: None);
;; `render-doc` returns the `@syntax` grammar + prose.

;; if — the @arg-fits special form. The @syntax grammar + prose must render.
(:wat::test::deftest :wat-tests::reflect::render-doc-of-if
  
  (:wat::core::let
    [rendered (:wat::core::render-doc :wat::core::if)]
    (:wat::core::do
      ;; @syntax grammar is rendered end-to-end
      (:wat::test::assert-contains rendered "(:wat::core::if <cond> <then> <else>)")
      ;; prose proves the BODY is rendered (not just the name) — "branch" is
      ;; load-bearing in if's description.
      (:wat::test::assert-contains rendered "branch")
      ;; Purity is Preserving — rendered as the enum variant string.
      (:wat::test::assert-contains rendered "Preserving"))))

;; let — the @syntax-carries / no-@arg special form. The grammar must render.
(:wat::test::deftest :wat-tests::reflect::render-doc-of-let
  
  (:wat::core::let
    [rendered (:wat::core::render-doc :wat::core::let)]
    (:wat::core::do
      (:wat::test::assert-contains rendered "(:wat::core::let [<binder> <expr> ...] <body>+)")
      ;; prose proves the body renders — "scope" is load-bearing in let's description.
      (:wat::test::assert-contains rendered "scope"))))
