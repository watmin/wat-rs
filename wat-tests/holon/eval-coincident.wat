;; wat-tests/holon/eval-coincident.wat — tests for the
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


;; ─── eval-coincident? — the book's (+ 2 2) ≡ (* 1 4) retort ──────

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-arithmetic-equivalence
  ()
  (:wat::core::let*
    (((r :wat::core::Result<wat::core::bool,wat::core::EvalError>)
      (:wat::holon::eval-coincident?
        (:wat::core::quote (:wat::core::i64::+,2 2 2))
        (:wat::core::quote (:wat::core::i64::*,2 1 4)))))
    (:wat::test::assert-eq
      (:wat::core::match r -> :wat::core::bool
        ((:wat::core::Ok b)  b)
        ((:wat::core::Err _) false))
      true)))

;; ─── Different scalars → not coincident ──────────────────────────

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-different-scalars
  ()
  (:wat::core::let*
    (((r :wat::core::Result<wat::core::bool,wat::core::EvalError>)
      (:wat::holon::eval-coincident?
        (:wat::core::quote 4)
        (:wat::core::quote 5))))
    (:wat::test::assert-eq
      (:wat::core::match r -> :wat::core::bool
        ((:wat::core::Ok b)  b)
        ((:wat::core::Err _) false))
      false)))

;; ─── Same strings → coincident ───────────────────────────────────

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-same-strings
  ()
  (:wat::core::let*
    (((r :wat::core::Result<wat::core::bool,wat::core::EvalError>)
      (:wat::holon::eval-coincident?
        (:wat::core::quote "rsi")
        (:wat::core::quote "rsi"))))
    (:wat::test::assert-eq
      (:wat::core::match r -> :wat::core::bool
        ((:wat::core::Ok b)  b)
        ((:wat::core::Err _) false))
      true)))

;; ─── Structurally-same holons built via quote ────────────────────

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-structurally-same-holons
  ()
  (:wat::core::let*
    (((r :wat::core::Result<wat::core::bool,wat::core::EvalError>)
      (:wat::holon::eval-coincident?
        (:wat::core::quote
          (:wat::holon::Bind (:wat::holon::Atom "k") (:wat::holon::Atom "v")))
        (:wat::core::quote
          (:wat::holon::Bind (:wat::holon::Atom "k") (:wat::holon::Atom "v"))))))
    (:wat::test::assert-eq
      (:wat::core::match r -> :wat::core::bool
        ((:wat::core::Ok b)  b)
        ((:wat::core::Err _) false))
      true)))

;; ─── eval-edn-coincident? — inline EDN sources ───────────────────

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-edn-arithmetic-equivalence
  ()
  (:wat::core::let*
    (((r :wat::core::Result<wat::core::bool,wat::core::EvalError>)
      (:wat::holon::eval-edn-coincident?
 "(:wat::core::i64::+,2 2 2)"
 "(:wat::core::i64::*,2 1 4)")))
    (:wat::test::assert-eq
      (:wat::core::match r -> :wat::core::bool
        ((:wat::core::Ok b)  b)
        ((:wat::core::Err _) false))
      true)))

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-edn-different-sources
  ()
  (:wat::core::let*
    (((r :wat::core::Result<wat::core::bool,wat::core::EvalError>)
      (:wat::holon::eval-edn-coincident?
 "(:wat::core::i64::+,2 2 2)"
 "(:wat::core::i64::+,2 2 3)")))
    (:wat::test::assert-eq
      (:wat::core::match r -> :wat::core::bool
        ((:wat::core::Ok b)  b)
        ((:wat::core::Err _) false))
      false)))

;; ─── eval-digest-coincident? — SHA-256-verified per side ─────────
;;
;; Pre-computed digests (run `printf '%s' '<src>' | sha256sum`):
;;   "(:wat::core::i64::+,2 2 2)" ->
;;     d4e368d75d1972482ae02398a37cef9fed68d2cb572f2354e31930b07ebb37cc
;;   "(:wat::core::i64::*,2 1 4)" ->
;;     03e5d2e5386ae6a04a279ad2c3bef2d2c6b6bca0bac25e3f902b68764a5a0484
;;
;; If a source string changes, regenerate; the load.rs digest-load
;; tests follow the same pattern for a runnable template.

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-digest-arithmetic-equivalence
  ()
  (:wat::core::let*
    (((r :wat::core::Result<wat::core::bool,wat::core::EvalError>)
      (:wat::holon::eval-digest-string-coincident?
 "(:wat::core::i64::+,2 2 2)"
        :wat::verify::digest-sha256
        :wat::verify::string "d4e368d75d1972482ae02398a37cef9fed68d2cb572f2354e31930b07ebb37cc"
 "(:wat::core::i64::*,2 1 4)"
        :wat::verify::digest-sha256
        :wat::verify::string "03e5d2e5386ae6a04a279ad2c3bef2d2c6b6bca0bac25e3f902b68764a5a0484")))
    (:wat::test::assert-eq
      (:wat::core::match r -> :wat::core::bool
        ((:wat::core::Ok b)  b)
        ((:wat::core::Err _) false))
      true)))

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-digest-bad-hex-errs
  ()
  ;; Side A carries a zero-hex digest that doesn't match the source;
  ;; verify fires before parse → Err(EvalError{kind=verification-failed}).
  (:wat::core::let*
    (((r :wat::core::Result<wat::core::bool,wat::core::EvalError>)
      (:wat::holon::eval-digest-string-coincident?
 "(:wat::core::i64::+,2 2 2)"
        :wat::verify::digest-sha256
        :wat::verify::string "0000000000000000000000000000000000000000000000000000000000000000"
 "(:wat::core::i64::*,2 1 4)"
        :wat::verify::digest-sha256
        :wat::verify::string "03e5d2e5386ae6a04a279ad2c3bef2d2c6b6bca0bac25e3f902b68764a5a0484")))
    (:wat::test::assert-eq
      (:wat::core::match r -> :wat::core::bool
        ((:wat::core::Ok _)  true)     ;; unexpected — verify should have failed
        ((:wat::core::Err _) false))
      false)))

;; ─── eval-signed-coincident? — Ed25519-verified per side ─────────
;;
;; Fixed signing key: SigningKey::from_bytes(&[7u8; 32]).
;; Pubkey (same for both sides — one signer):
;;   6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw=
;; Signatures are guarded by the `wat_test_embedded_signatures_verify`
;; Rust unit test — if a source string changes here, that test fails
;; with "SRC_X signature drifted." Regenerate by adding a temporary
;; eprintln to sign_src_ed25519, OR via a scratch binary that calls
;; the helper:
;;   src-a sig = 3bQjvWistCp2jyK0AU6+9ZQZp/yMk2gB/ycbjIOGpFd3FBIwGaa/TqsHV4Elb4P0HxBo6eSr0q3qwZ8xaKOgBw==
;;   src-b sig = OrYNwvRnWgytoHL77zLAB8EQItkav/KnUTpmacu9AuxS8LKu4Fjda9dvgc5ruNq5Fc8GB52v+/BGew7rxxiXCw==

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-signed-arithmetic-equivalence
  ()
  (:wat::core::let*
    (((r :wat::core::Result<wat::core::bool,wat::core::EvalError>)
      (:wat::holon::eval-signed-string-coincident?
 "(:wat::core::i64::+,2 2 2)"
        :wat::verify::signed-ed25519
        :wat::verify::string "3bQjvWistCp2jyK0AU6+9ZQZp/yMk2gB/ycbjIOGpFd3FBIwGaa/TqsHV4Elb4P0HxBo6eSr0q3qwZ8xaKOgBw=="
        :wat::verify::string "6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw="
 "(:wat::core::i64::*,2 1 4)"
        :wat::verify::signed-ed25519
        :wat::verify::string "OrYNwvRnWgytoHL77zLAB8EQItkav/KnUTpmacu9AuxS8LKu4Fjda9dvgc5ruNq5Fc8GB52v+/BGew7rxxiXCw=="
        :wat::verify::string "6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw=")))
    (:wat::test::assert-eq
      (:wat::core::match r -> :wat::core::bool
        ((:wat::core::Ok b)  b)
        ((:wat::core::Err _) false))
      true)))

(:wat::test::deftest :wat-tests::holon::eval-coincident::test-signed-wrong-sig-errs
  ()
  ;; Side A carries src-B's sig against src-A; verify fails →
  ;; Err(EvalError{kind=verification-failed}).
  (:wat::core::let*
    (((r :wat::core::Result<wat::core::bool,wat::core::EvalError>)
      (:wat::holon::eval-signed-string-coincident?
 "(:wat::core::i64::+,2 2 2)"
        :wat::verify::signed-ed25519
        :wat::verify::string "OrYNwvRnWgytoHL77zLAB8EQItkav/KnUTpmacu9AuxS8LKu4Fjda9dvgc5ruNq5Fc8GB52v+/BGew7rxxiXCw=="
        :wat::verify::string "6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw="
 "(:wat::core::i64::*,2 1 4)"
        :wat::verify::signed-ed25519
        :wat::verify::string "OrYNwvRnWgytoHL77zLAB8EQItkav/KnUTpmacu9AuxS8LKu4Fjda9dvgc5ruNq5Fc8GB52v+/BGew7rxxiXCw=="
        :wat::verify::string "6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw=")))
    (:wat::test::assert-eq
      (:wat::core::match r -> :wat::core::bool
        ((:wat::core::Ok _)  true)     ;; unexpected — verify should have failed
        ((:wat::core::Err _) false))
      false)))
