;; probe-arc278-fnforms-walks-into-quoted-data.wat — THE ISOLATION, with no DSL in sight.
;;
;; THE CLAIM UNDER TEST (builder-ruled a bug): `fn-forms`'s closure walker descends into QUOTED
;; data and demands that the symbols inside it resolve. Quoted forms are DATA — nothing in them is
;; a reference, and nothing in them will ever be evaluated in the position it sits.
;;
;; WHY THIS FILE HAS NO RETE IN IT. The failure was found through `sift-rules`, whose `defrule`
;; expands to a defn that quotes its `:when`/`:then` (wat/rete.wat:2385-2400) — so the pattern
;; variable `?c` is quoted data. If the defect is real it has NOTHING to do with rete, with
;; pattern variables, or with `?`-prefixes: it is "the walker reads data as code." This file
;; proves that with plain `quote` and ordinary symbols, so no one can read the fix as a
;; rete-specific accommodation.
;;
;; ⚠ NON-VACUITY — the two arms differ in ONE thing. Both defns are shipped through the SAME
;; `fn-forms` call shape. `:probe::clean` has no quote; `:probe::quoted-junk` has the same body
;; wrapped in `quote`. If BOTH raise, the quote is not the discriminator and this instrument is
;; measuring something else. If NEITHER raises, the defect is not here and the claim is refuted.
;; Only clean-passes + quoted-raises isolates it.

;; ── the CONTROL: no quote, and every symbol it names is real ────────────────────────────────
(:wat::core::defn :probe::clean [] -> :wat::core::i64
  42)

;; ── the SUBJECT: identical shipping, but the body is QUOTED DATA naming things that do not
;; exist and are not meant to. This is legal, ordinary wat: `quote` is how you build a form.
;; `mystery-symbol` is a bare Symbol — deliberately NOT `?`-prefixed, so the result cannot be
;; read as being about rete's pattern-variable spelling.
(:wat::core::defn :probe::quoted-junk [] -> :wat::WatAST
  (:wat::core::quote (mystery-symbol another-nonexistent-name)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; ARM 1 — the control. Expect a closure to come back.
     clean-forms (:wat::kernel::fn-forms :probe::clean
                   (:wat::keyword::from-string "user::root-clean"))
     _c (:wat::kernel::println
          (:wat::string::concat "CONTROL closure forms="
            (:wat::i64::to-string (:wat::core::length clean-forms))))
     ;; ARM 2 — the subject. If the walker reads quoted data as code, THIS raises, and the raise
     ;; names `mystery-symbol` — a symbol that appears nowhere except inside a quote.
     junk-forms (:wat::kernel::fn-forms :probe::quoted-junk
                  (:wat::keyword::from-string "user::root-junk"))
     _j (:wat::kernel::println
          (:wat::string::concat "SUBJECT closure forms="
            (:wat::i64::to-string (:wat::core::length junk-forms))))]
    ;; ★ THIS FILE CHANGED ROLE WHEN THE FIX LANDED. It was written as a RED gate: reaching
    ;; this line at all was the DISCONFIRMATION, because the subject arm raised before it.
    ;; The walker now routes through `resolve::boundary::quote_boundary`, so both arms return
    ;; and this line is the PASS. Left in place as the standing REGRESSION gate — if the
    ;; subject arm ever raises again, the walker has gone back to reading data as code.
    (:wat::kernel::println
      "PASS — both arms extracted; the walker treats quoted data as DATA, not as references")))
