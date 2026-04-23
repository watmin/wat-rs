;; wat-tests/holon/eval_coincident.wat — tests for the
;; :wat::holon::eval-coincident? family (arc 026).
;;
;; Each variant mirrors its parent eval-*! form, applied per-side.
;; Both sides' resolved Values are atomized via value_to_atom and
;; compared via the same coincident_floor test structural
;; coincident? (arc 023) uses.
;;
;; The family:
;;   eval-coincident?         — AST (quoted forms). 2 args.
;;   eval-edn-coincident?     — EDN sources. 4 args.
;;   eval-digest-coincident?  — SHA-256 verified. 10 args. (Rust-unit-test coverage)
;;   eval-signed-coincident?  — Ed25519 verified. 14 args. (Rust-unit-test coverage)
;;
;; wat-level coverage here focuses on the AST and EDN variants —
;; the digest/signed variants need pre-computed hashes/signatures
;; whose natural test site is the Rust unit-test tier (hashes
;; computed inline via sha2, signatures via ed25519-dalek).

(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

;; ─── eval-coincident? — the book's (+ 2 2) ≡ (* 1 4) retort ──────

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-arithmetic-equivalence 1024 :error
  (:wat::core::let*
    (((r :Result<bool,wat::core::EvalError>)
      (:wat::holon::eval-coincident?
        (:wat::core::quote (:wat::core::i64::+ 2 2))
        (:wat::core::quote (:wat::core::i64::* 1 4)))))
    (:wat::test::assert-eq
      (:wat::core::match r -> :bool
        ((Ok b)  b)
        ((Err _) false))
      true)))

;; ─── Different scalars → not coincident ──────────────────────────

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-different-scalars 1024 :error
  (:wat::core::let*
    (((r :Result<bool,wat::core::EvalError>)
      (:wat::holon::eval-coincident?
        (:wat::core::quote 4)
        (:wat::core::quote 5))))
    (:wat::test::assert-eq
      (:wat::core::match r -> :bool
        ((Ok b)  b)
        ((Err _) false))
      false)))

;; ─── Same strings → coincident ───────────────────────────────────

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-same-strings 1024 :error
  (:wat::core::let*
    (((r :Result<bool,wat::core::EvalError>)
      (:wat::holon::eval-coincident?
        (:wat::core::quote "rsi")
        (:wat::core::quote "rsi"))))
    (:wat::test::assert-eq
      (:wat::core::match r -> :bool
        ((Ok b)  b)
        ((Err _) false))
      true)))

;; ─── Structurally-same holons built via quote ────────────────────

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-structurally-same-holons 1024 :error
  (:wat::core::let*
    (((r :Result<bool,wat::core::EvalError>)
      (:wat::holon::eval-coincident?
        (:wat::core::quote
          (:wat::holon::Bind (:wat::holon::Atom "k") (:wat::holon::Atom "v")))
        (:wat::core::quote
          (:wat::holon::Bind (:wat::holon::Atom "k") (:wat::holon::Atom "v"))))))
    (:wat::test::assert-eq
      (:wat::core::match r -> :bool
        ((Ok b)  b)
        ((Err _) false))
      true)))

;; ─── eval-edn-coincident? — inline EDN sources ───────────────────

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-edn-arithmetic-equivalence 1024 :error
  (:wat::core::let*
    (((r :Result<bool,wat::core::EvalError>)
      (:wat::holon::eval-edn-coincident?
        :wat::eval::string "(:wat::core::i64::+ 2 2)"
        :wat::eval::string "(:wat::core::i64::* 1 4)")))
    (:wat::test::assert-eq
      (:wat::core::match r -> :bool
        ((Ok b)  b)
        ((Err _) false))
      true)))

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-edn-different-sources 1024 :error
  (:wat::core::let*
    (((r :Result<bool,wat::core::EvalError>)
      (:wat::holon::eval-edn-coincident?
        :wat::eval::string "(:wat::core::i64::+ 2 2)"
        :wat::eval::string "(:wat::core::i64::+ 2 3)")))
    (:wat::test::assert-eq
      (:wat::core::match r -> :bool
        ((Ok b)  b)
        ((Err _) false))
      false)))

;; ─── eval-digest-coincident? — SHA-256-verified per side ─────────
;;
;; Pre-computed digests (run `printf '%s' '<src>' | sha256sum`):
;;   "(:wat::core::i64::+ 2 2)" ->
;;     844049a88ac83175756184fd59e9b7746b3e8bbe745ba8afe8fa5f1ec5fb274e
;;   "(:wat::core::i64::* 1 4)" ->
;;     3571299726bb0f014a3cea5e91cd1623a94fffb7ac1641525ff1ca56c7140e45
;;
;; If a source string changes, regenerate; the load.rs digest-load
;; tests follow the same pattern for a runnable template.

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-digest-arithmetic-equivalence 1024 :error
  (:wat::core::let*
    (((r :Result<bool,wat::core::EvalError>)
      (:wat::holon::eval-digest-coincident?
        :wat::eval::string "(:wat::core::i64::+ 2 2)"
        :wat::verify::digest-sha256
        :wat::verify::string "844049a88ac83175756184fd59e9b7746b3e8bbe745ba8afe8fa5f1ec5fb274e"
        :wat::eval::string "(:wat::core::i64::* 1 4)"
        :wat::verify::digest-sha256
        :wat::verify::string "3571299726bb0f014a3cea5e91cd1623a94fffb7ac1641525ff1ca56c7140e45")))
    (:wat::test::assert-eq
      (:wat::core::match r -> :bool
        ((Ok b)  b)
        ((Err _) false))
      true)))

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-digest-bad-hex-errs 1024 :error
  ;; Side A carries a zero-hex digest that doesn't match the source;
  ;; verify fires before parse → Err(EvalError{kind=verification-failed}).
  (:wat::core::let*
    (((r :Result<bool,wat::core::EvalError>)
      (:wat::holon::eval-digest-coincident?
        :wat::eval::string "(:wat::core::i64::+ 2 2)"
        :wat::verify::digest-sha256
        :wat::verify::string "0000000000000000000000000000000000000000000000000000000000000000"
        :wat::eval::string "(:wat::core::i64::* 1 4)"
        :wat::verify::digest-sha256
        :wat::verify::string "3571299726bb0f014a3cea5e91cd1623a94fffb7ac1641525ff1ca56c7140e45")))
    (:wat::test::assert-eq
      (:wat::core::match r -> :bool
        ((Ok _)  true)     ;; unexpected — verify should have failed
        ((Err _) false))
      false)))

;; ─── eval-signed-coincident? — Ed25519-verified per side ─────────
;;
;; Fixed signing key: SigningKey::from_bytes(&[7u8; 32]).
;; Pubkey (same for both sides — one signer):
;;   6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw=
;; Signatures (run the ignored runtime::tests::print_fixed_signatures
;; helper to regenerate if a source changes):
;;   src-a sig = ZR3nyIPpRSKItQKfFH46p96UbwYpr2TlaysNbnnxZvpA6QiuXftuzmA3xUDfaZ+qWMNCk3m51XzXzXGguo6XCA==
;;   src-b sig = PrDdUtimBlhGDD7atAdR9lHJc01Efok8VtsgX3/qHGjuGgkf+3GlbFE1ZGxf/uEA6VYkcd7tCWc4ipKr1AcCCw==

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-signed-arithmetic-equivalence 1024 :error
  (:wat::core::let*
    (((r :Result<bool,wat::core::EvalError>)
      (:wat::holon::eval-signed-coincident?
        :wat::eval::string "(:wat::core::i64::+ 2 2)"
        :wat::verify::signed-ed25519
        :wat::verify::string "ZR3nyIPpRSKItQKfFH46p96UbwYpr2TlaysNbnnxZvpA6QiuXftuzmA3xUDfaZ+qWMNCk3m51XzXzXGguo6XCA=="
        :wat::verify::string "6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw="
        :wat::eval::string "(:wat::core::i64::* 1 4)"
        :wat::verify::signed-ed25519
        :wat::verify::string "PrDdUtimBlhGDD7atAdR9lHJc01Efok8VtsgX3/qHGjuGgkf+3GlbFE1ZGxf/uEA6VYkcd7tCWc4ipKr1AcCCw=="
        :wat::verify::string "6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw=")))
    (:wat::test::assert-eq
      (:wat::core::match r -> :bool
        ((Ok b)  b)
        ((Err _) false))
      true)))

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-signed-wrong-sig-errs 1024 :error
  ;; Side A carries src-B's sig against src-A; verify fails →
  ;; Err(EvalError{kind=verification-failed}).
  (:wat::core::let*
    (((r :Result<bool,wat::core::EvalError>)
      (:wat::holon::eval-signed-coincident?
        :wat::eval::string "(:wat::core::i64::+ 2 2)"
        :wat::verify::signed-ed25519
        :wat::verify::string "PrDdUtimBlhGDD7atAdR9lHJc01Efok8VtsgX3/qHGjuGgkf+3GlbFE1ZGxf/uEA6VYkcd7tCWc4ipKr1AcCCw=="
        :wat::verify::string "6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw="
        :wat::eval::string "(:wat::core::i64::* 1 4)"
        :wat::verify::signed-ed25519
        :wat::verify::string "PrDdUtimBlhGDD7atAdR9lHJc01Efok8VtsgX3/qHGjuGgkf+3GlbFE1ZGxf/uEA6VYkcd7tCWc4ipKr1AcCCw=="
        :wat::verify::string "6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw=")))
    (:wat::test::assert-eq
      (:wat::core::match r -> :bool
        ((Ok _)  true)     ;; unexpected — verify should have failed
        ((Err _) false))
      false)))
