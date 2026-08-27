;; probe-slice-one-registry-seam.wat — THE DISCONFIRMING PROBE for slice one of the rete
;; vocabulary (#55 / S3b+S4), written BEFORE the brief per examinare.
;;
;; ── THE ASSUMPTION UNDER TEST ────────────────────────────────────────────────────────────────
;;
;;   "A rete op can be DECLARED ONCE — in arc 255's builtin registry — and BOTH the dispatch
;;    AND the `where` fence read it from there."
;;
;; That is the whole justification for putting the table-driven conversion inside slice one
;; rather than inventing a fifth hand-list. If it is false, slice one's shape changes before a
;; rider is spent, not after.
;;
;; It decomposes into three claims. TWO were settled by READING the source and are asserted
;; here as CONTROLS so a future substrate change breaks this probe loudly. The THIRD is the
;; live gap and this file exists to exhibit it.
;;
;;   CLAIM 1 (read → confirmed here by a RUN).  A kwargs surface is a DEFMACRO that lowers to a
;;     POSITIONAL prime; the keyword never reaches the intrinsic. `wat/kernel/readln.wat:59` is
;;     the worked instance — it inspects `args`, recognises `:max-buffer-bytes`, and emits
;;     `(readln' N)`. CONSEQUENCE: `(:wat::rete::core::i64::+ a b :undefined -1)` does NOT require
;;     the registry to model keyword arguments, so arc 255's DESIGN open-question #3 ("is
;;     Exact|AtLeast|Range|Variadic enough, or do some builtins need keyword-arg shapes?")
;;     answers **Exact is enough** — kwargs are a macro layer ABOVE the registry.
;;     ⚠ This CORRECTS an earlier orchestrator claim that the rete op proves 255 needs richer
;;     arity. It does not. Kept visible rather than deleted.
;;
;;   CLAIM 2 (read, in `crates/wat-macros/src/wat_intrinsic.rs:53-58`, `sniff_args`). The
;;     `#[wat_intrinsic]` arity model is exactly two-valued — N leading `&WatAST` params =>
;;     `Exact(N)` with a shim arity check, or one `&[WatAST]` => `Variadic` with NO check.
;;     Combined with CLAIM 1 that is sufficient for every op in the rete vocabulary.
;;
;;   CLAIM 3 — ★ THE GAP THIS PROBE EXHIBITS. The `where` fence does NOT consult the registry.
;;     `src/rete/purity.rs` contains ZERO references to it (`IntrinsicRegistry` / `intrinsic::` /
;;     `lookup`), and decides purity from its own 133-verb hand map `intrinsic_meta`. Its module
;;     header (`purity.rs:17-20`) already names itself "the explicit v1 projection of the
;;     queryable registry that arc 255 will eventually own", and `constructor_meta`'s comment
;;     says "INTERIM … until arc 255's builtin-registry becomes the single queryable purity
;;     source and subsumes it."
;;
;;     So today the two systems answer the SAME question with OPPOSITE DEFAULTS:
;;
;;       derive_pure_deterministic (runtime.rs:24371) — the registry's reflection-site deriver
;;           pure = !is_effectful_op(name)                            DEFAULT-ALLOW
;;           (effectful = :wat::kernel:: :wat::io:: :wat::eval- :wat::load :wat::config::)
;;       intrinsic_meta (rete/purity.rs)               — the fence's table
;;           133 verbs enumerated; anything else refused                DEFAULT-DENY
;;
;;     And TOTALITY is not derivable from a namespace prefix at all — the registry has no slot
;;     for it, which is exactly why the fence had to grow its own column (#52).
;;
;; ── WHAT THIS FILE MEASURES, AND HOW TO READ IT ──────────────────────────────────────────────
;;
;;     ./target/release/wat wat-scripts/scratch-pad/probe-slice-one-registry-seam.wat
;;
;; Row A is CLAIM 1's run: the expansion of a kwargs call, printed. Read it and confirm the
;;   emitted form is a POSITIONAL call to the prime with no keyword surviving.
;; Rows B and C are the two purity oracles asked about the SAME verb, side by side. They are
;;   the fence's `pure?` (default-deny, the hand map) and `metadata-of`'s reflection answer
;;   (default-allow, the namespace deriver).
;;
;; NON-VACUITY, and it matters: row B must answer TRUE for a verb the hand map DOES classify.
;;   If every row is false the fence is simply refusing everything and the comparison measures
;;   nothing. `:wat::core::i64::+` is in the map (pure ∧ deterministic ∧ NOT total) — it is the
;;   positive control. `:wat::core::Uuid/v4` is the negative control on the determinism axis:
;;   pure but NOT deterministic, and it is the ONE entry in the deriver's NONDETERMINISTIC
;;   residual, so both oracles must agree it is non-deterministic or one of them has rotted.
;;
;; ── RESULT, RUN 2026-08-02 — exit 0, every row fired ─────────────────────────────────────────
;;
;;   A expansion ......... (:wat.kernel/readln' 4096)      <- CLAIM 1 CONFIRMED: kwarg lowered away
;;   B fence pure?  i64::+ .....  TRUE                     <- positive control, non-vacuous
;;   B fence det?   i64::+ .....  TRUE
;;   C fence det?   Uuid/v4 ....  FALSE                    <- negative control on determinism
;;   D fence pure?  Bytes::to-hex FALSE                    <- ★ THE SEAM
;;   D metadata-of  Bytes::to-hex =
;;     #wat.core.Option/Some [{:category #wat.runtime.Category/Encoding
;;                             :defined-in #wat.runtime.DefinedIn/Rust
;;                             :kind #wat.runtime.Kind/Intrinsic  :arity 1
;;                             :name :wat.core.Bytes/to-hex
;;                             :purity #wat.runtime.Purity/Pure
;;                             :determinism #wat.runtime.Determinism/Deterministic
;;                             :layer … :doc … :added "1.0.0" :ret …}]
;;
;; TWO FINDINGS, and the second is the one that shapes slice one.
;;
;; 1. THE SEAM IS REAL. One verb, two oracles, opposite answers: the fence says NOT pure; the
;;    registry says `:purity Pure, :determinism Deterministic`. Being enrolled in arc 255's
;;    registry buys a verb nothing at the `where` fence today.
;;
;; 2. ★ THE REGISTRY'S ANSWER IS A *DERIVED GUESS*, NOT A DECLARATION — so "just point the fence
;;    at the registry" would be WRONG. `:purity Pure` here is not something the author declared;
;;    it is `derive_pure_deterministic` computing `!is_effectful_op(name)` — i.e. "not under one
;;    of five effectful prefixes, therefore pure." It happens to be correct for this verb and it
;;    is DEFAULT-ALLOW for every verb, which is precisely the property a fence must not inherit.
;;    (`src/intrinsic/mod.rs`'s own accretion note says so outright: "`purity` / `determinism` →
;;    DERIVED at the reflection site … not stored on the entry.")
;;
;;    And READ THE KEY SET: `{category defined-in kind arity ret name purity layer determinism
;;    doc added}` — eleven fields and **NO `:total`**. Totality is not derivable from a namespace
;;    prefix at all, which is exactly why the fence had to grow its own column in #52.
;;
;; ⇒ THE PRESCRIPTION SLICE ONE FOLLOWS. Not "fence reads registry" but: PROMOTE the purity
;;   triple from DERIVED to STORED on the entry, declared where the op is declared — which is
;;   arc 255's OWN accretion rule ("most fields are added in the SAME strike that builds their
;;   reader"), and the rete fence is the reader that makes a namespace guess insufficient. Then
;;   `intrinsic_meta` becomes the projection its own header (`purity.rs:17-20`) already
;;   instructs, instead of a fifth parallel hand-list.
;;
;; ⛔ WHAT THIS PROBE DOES **NOT** SETTLE — do not read further than it measures.
;;   It does not prove a `#[wat_intrinsic]` handler can be enrolled under `:wat::rete::`, and it
;;   does not prove the entry can carry a STORED purity triple. Those are Rust-side and are the
;;   second half of the probe (a `tests/` unit test); this half is the wat-observable seam and
;;   it is the half that decides slice one's SHAPE.

(:wat::core::defn :seam::row [label <- :wat::core::String  v <- :wat::core::bool] -> :wat::core::String
  (:wat::string::concat label (:wat::core::if v " TRUE" " FALSE")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    ;; ── ROW A — CLAIM 1: does a kwargs surface lower to a positional prime? ───────────────
    ;; `readln` is the documented instance (a defmacro over `readln'`). Print its expansion and
    ;; read whether the `:max-buffer-bytes` keyword survives into the emitted form.
    [expanded (:wat::core::write-forms
                (:wat::core::macroexpand
                  (:wat::core::quote (:wat::kernel::readln :max-buffer-bytes 4096))))
     _  (:wat::kernel::println (:wat::string::concat "A expansion ......... " expanded))

     ;; ── ROWS B/C — the two purity oracles, same verb, side by side ────────────────────────
     ;; POSITIVE CONTROL: i64::+ is classified in the fence's hand map.
     _  (:wat::kernel::println
          (:seam::row "B fence pure?  i64::+ ..... "
                      (:wat::rete::pure? (:wat::core::quote (:wat::i64::+ 1 2)))))
     _  (:wat::kernel::println
          (:seam::row "B fence det?   i64::+ ..... "
                      (:wat::rete::deterministic? (:wat::core::quote (:wat::i64::+ 1 2)))))

     ;; NEGATIVE CONTROL on the determinism axis: pure, but the one entry in the deriver's
     ;; NONDETERMINISTIC residual. Both oracles must call it non-deterministic.
     _  (:wat::kernel::println
          (:seam::row "C fence det?   Uuid/v4 .... "
                      (:wat::rete::deterministic? (:wat::core::quote (:wat::uuid::v4)))))

     ;; ── ROW D — ★ THE SEAM, made observable ───────────────────────────────────────────────
     ;; `:wat::core::Bytes::to-hex` is the discriminator, and it is chosen by MEASUREMENT, not
     ;; by guess (a first draft used `Uuid/v5` — which IS in the map, `purity.rs:288`, so it
     ;; discriminated nothing; the fence answered TRUE for the boring reason. The map's 147
     ;; head strings were extracted and this verb confirmed ABSENT before it was written here).
     ;;
     ;; It is the perfect witness because it is BOTH:
     ;;   • ENROLLED in arc 255's registry — it is `#[wat_intrinsic]`'s own doc example, and it
     ;;     lives in `src/intrinsic/bytes.rs`, one of the 17 ops 255 has actually landed; and
     ;;   • ABSENT from the fence's hand map.
     ;;
     ;; So if the fence answers FALSE here, being in the registry buys a verb NOTHING at the
     ;; fence — the two systems are disconnected, which is the claim slice one rests on.
     _  (:wat::kernel::println
          (:seam::row "D fence pure?  Bytes::to-hex "
                      (:wat::rete::pure?
                        (:wat::core::quote (:wat::core::Bytes::to-hex (:wat::core::Vector 255 0 16))))))
     ;; …and the SAME verb through the registry's own reflection surface. The two answers side
     ;; by side ARE the finding. (Called, not quoted — a first draft printed the quoted FORM and
     ;; measured nothing.)
     _  (:wat::kernel::println "D metadata-of  Bytes::to-hex (the registry's own answer) =")
     _  (:wat::kernel::println (:wat::runtime::metadata-of :wat::core::Bytes::to-hex))]
    (:wat::kernel::println "-- A = the lowering; B/C = controls; D = the seam --")))
