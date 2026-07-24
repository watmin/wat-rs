;; wat-tests/reflect/reflection-surface.wat — Stone 255.1b-v RED probes (wat-direct).
;;
;; The reflection surface over the intrinsic registry — show-source + render-doc —
;; proven on the `core::Bytes` pilot, dogfooded in wat.
;;
;; RED at HEAD: `:wat::core::show-source` and `:wat::core::render-doc` are unregistered
;; → the calls raise "unknown function" → the test fails.
;; GREEN after 255.1b-v: both verbs return Strings; the substrings are present.

;; show-source returns the captured Rust handler source of an intrinsic.
;; Uniform Pry lens: an intrinsic shows its Rust source (a user form would show its
;; wat source via write-forms). The handler fn name is the load-bearing substring.
(:wat::test::deftest' :wat-tests::reflect::show-source-of-bytes-to-hex
  
  (:wat::test::assert-contains
    (:wat::core::show-source :wat::core::Bytes::to-hex)
    "eval_bytes_to_hex"))

;; render-doc renders metadata-of into a human String (with newlines); the caller
;; prints it. A String IS a clean EDN value — the newlines render on println.
;; "lowercase" proves the PROSE is rendered (not just the name); "Bytes::to-hex"
;; proves the name/signature line is rendered.
(:wat::test::deftest' :wat-tests::reflect::render-doc-of-bytes-to-hex
  
  (:wat::core::let
    [rendered (:wat::core::render-doc :wat::core::Bytes::to-hex)]
    (:wat::core::do
      (:wat::test::assert-contains rendered "lowercase")
      (:wat::test::assert-contains rendered "Bytes::to-hex")
      ;; @see is rendered end-to-end: to-hex's "See also" points at its inverse,
      ;; from-hex — proving @see is declared (corpus), rendered (render-doc), and
      ;; checked (the dangling-ref test) on the pilot, not carried-but-dark.
      (:wat::test::assert-contains rendered "from-hex"))))

;; Spec-complete end-to-end (weigh): the variadic witness counts its args.
(:wat::test::deftest' :wat-tests::reflect::variadic-witness-counts-args
  
  (:wat::test::assert-eq (:wat::intrinsic::variadic-args-measurement 1 2 3) 3))
