;; 255 — the-registry-answers-first-wave-2: the seventeen NAMED-GUARD facts moved into their own
;; registrations, plus the two pure-duplicate guards deleted (`:wat::uuid::v4`, `:wat::stream::next`)
;; (DESIGN-STONE-the-registry-answers-first-wave-2.md). Extends wave 1's probe shape
;; (`255-the-registry-answers-first.wat`) rather than replacing it.
;;
;; ── WHY THIS PROBE USES `:wat::rete::total?`, NOT A `where`/`then` FENCE ────────────────────────
;;
;; Wave 1 already measured (and its DESIGN's REFUTED section records) that NONE of its eleven
;; `:wat::string::`/`:wat::edn::` verbs ever reach a `where`/`then` fence — Law A
;; (`:wat::rete::primitive?`, `compile-condition`'s `is-rete` conjunct) refuses any head that is
;; not a row in `RETE_OPS`, independent of purity/determinism/totality. This wave's seventeen
;; include NINE `:wat::rete::`-namespaced verbs (`pure?`, `deterministic?`, `total?`, `primitive?`,
;; `vocabulary-admitted?`, `cond-has-deferred-constraint?`, `alpha-match`, `alpha-match-local`,
;; `alpha-match-under`) — STOP-5 asked whether THEIR namespace lets them clear Law A where wave 1's
;; string verbs could not. **Measured, out-of-tree, against the pre-stone binary** (`target/
;; release/wat`, before any Rust change in this stone) — NOT embedded in this committed script,
;; because a committed `.wat` must LOAD and this is a RUNTIME panic, not a load failure:
;;
;;   (:wat::rete::where (:wat::rete::vocabulary-admitted? (:wat::core::quote :wat::rete::core::cond)))
;;     => #wat.kernel/AssertionFailure "compile-condition: where expr is not a rete primitive —
;;        ':wat::rete::vocabulary-admitted?' is not a rete primitive; a where admits only
;;        :wat::rete:: ops"
;;   (:wat::rete::where (:wat::rete::total? (:wat::core::quote (:wat::core::= 1 1))))
;;     => #wat.kernel/AssertionFailure "compile-condition: where expr is not a rete primitive —
;;        ':wat::rete::total?' is not a rete primitive; a where admits only :wat::rete:: ops"
;;
;; ★ STOP-5 ANSWERED: NO, the `:wat::rete::` namespace does NOT let these nine clear Law A. Law A
;; is gated on `rete_op_for(head)` — an exact-match lookup against the `RETE_OPS` TABLE
;; (`src/rete/vocabulary.rs`), never a bare namespace-prefix test — and none of these nine is a row
;; in that table (confirmed by reading `intrinsic_meta`, `src/rete/purity.rs:251`: these nine are
;; only REACHED because `rete_op_for` already returned `None` for them, falling through to the
;; hand-written guards this stone retires). So exactly like wave 1's string verbs, ALL SEVENTEEN of
;; this wave's verbs are refused by a `where`/`then` fence, unconditionally, both BEFORE and AFTER
;; this stone — Law A is untouched, and `RETE_OPS` membership is untouched. The fence surface
;; cannot demonstrate this stone's fact-moves either, for the same reason wave 1's couldn't.
;;
;; The consumer that DOES read `intrinsic_meta`'s totality axis in isolation (no Law A conjunct) is
;; `:wat::rete::total?` — the same standalone introspection predicate wave 1 used, and (per STOP-1)
;; itself one of this wave's seventeen. Re-deriving it did NOT change its own self-report (see
;; `rete::total? total?` below) — the reporter stays trustworthy.
;;
;; ── BEFORE THIS STONE (measured, pre-existing `target/release/wat`, 2026-08-31) ─────────────────
;;
;;   hashmap::keys total?                        true
;;   map::keys total?                            true
;;   type-params-used-in total?                  true      <- the guard's claim
;;   type-equal? total?                          true      <- the guard's claim
;;   stream::empty total?                        true
;;   rete::pure? total?                          true
;;   rete::total? total?                         true      <- the reporter, reporting on itself
;;   rete::vocabulary-admitted? total?           true      <- the guard's claim
;;   rete::cond-has-deferred-constraint? total?  true
;;   rete::alpha-match total?                    true
;;   uuid::v4 total? (duplicate, must not move)  false
;;   stream::next total? (duplicate, must not move) false
;;   string::split total? (control, stays Unreviewed) false
;;
;; ── AFTER THIS STONE (expected, once the registry answers) ──────────────────────────────────────
;;
;; Fourteen of the seventeen move IN UNCHANGED (their `@Totality` is now `Total` at their own
;; registration, re-derived from each body — see `src/intrinsic/{hashmap,map,stream,rete}.rs` and
;; `src/rete/purity.rs`'s retirement comments): `hashmap::keys/values`, `map::keys/values`,
;; `stream::empty/cons`, `rete::pure?/deterministic?/total?/primitive?/cond-has-deferred-
;; constraint?`, `rete::alpha-match/-local/-under`.
;;
;; ⛔ THREE of the seventeen do NOT move in unchanged — re-reading their own bodies overturned the
;; guard's `total: true`, exactly as wave 1's `concat` overturned its guard:
;;
;;   `:wat::core::type-params-used-in` — `param_name_of` (`src/intrinsic/reflect.rs`) raises
;;   `TypeMismatch` for a well-typed but non-Symbol/Keyword `params` element (this verb carries no
;;   checker TypeScheme at all — `intrinsic/mod.rs`'s `FROZEN_CHECKER_DEBT_LEDGER` — so nothing
;;   stops that shape reaching a well-typed call). Now `@Totality Partial`.
;;   `:wat::core::type-equal?` — its own doc already said outright "given a node that does not
;;   parse as a type at all, this RAISES rather than returning `false`"; confirmed by reading
;;   `parse_type_node`'s call site. Now `@Totality Partial`.
;;   `:wat::rete::vocabulary-admitted?` — after the checker-guaranteed `Value::wat__WatAST` unwrap
;;   (same as the other rete predicates), the body destructures a SECOND level, requiring
;;   specifically `WatAST::Keyword`; nothing in the declared `:wat::WatAST` type rules out a quoted
;;   List/Symbol/number reaching the `other =>` `TypeMismatch` arm. Now `@Totality Partial`.
;;
;; All three were confirmed empirically against the pre-stone binary: each call passes `--check`
;; (exit 0, well-typed) and raises `TypeMismatch` at run — the same "passes check, raises at run"
;; signature wave 1's `concat` had.
;;
;;   hashmap::keys total?                        true      (unchanged)
;;   map::keys total?                            true      (unchanged)
;;   type-params-used-in total?                  FALSE     <- CHANGES
;;   type-equal? total?                          FALSE     <- CHANGES
;;   stream::empty total?                        true      (unchanged)
;;   rete::pure? total?                          true      (unchanged)
;;   rete::total? total?                         true      (unchanged — the reporter is unaffected)
;;   rete::vocabulary-admitted? total?           FALSE     <- CHANGES
;;   rete::cond-has-deferred-constraint? total?  true      (unchanged)
;;   rete::alpha-match total?                    true      (unchanged)
;;   uuid::v4 total? (duplicate, must not move)  false     (unchanged — free proof: registry already agreed)
;;   stream::next total? (duplicate, must not move) false  (unchanged — free proof: registry already agreed)
;;   string::split total? (control, stays Unreviewed) false (unchanged — a different stone's work)
;;
;; No `where`/`then` fence in the corpus is affected either way (STOP-5, above): all seventeen are
;; refused by Law A regardless of this stone, before and after.
;;
;; Run: target/release/wat wat-scripts/scratch-pad/255-the-registry-answers-first-wave-2.wat

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [hm-keys-total   (:wat::rete::total? (:wat::core::quote (:wat::hashmap::keys m)))
     map-keys-total  (:wat::rete::total? (:wat::core::quote (:wat::map::keys m)))
     tpui-total      (:wat::rete::total? (:wat::core::quote (:wat::core::type-params-used-in p n)))
     teq-total       (:wat::rete::total? (:wat::core::quote (:wat::core::type-equal? a b)))
     stream-empty-total (:wat::rete::total? (:wat::core::quote (:wat::stream::empty)))
     rete-pure-total (:wat::rete::total? (:wat::core::quote (:wat::rete::pure? e)))
     rete-total-total (:wat::rete::total? (:wat::core::quote (:wat::rete::total? e)))
     rete-vocab-total (:wat::rete::total? (:wat::core::quote (:wat::rete::vocabulary-admitted? h)))
     rete-cond-total (:wat::rete::total? (:wat::core::quote (:wat::rete::cond-has-deferred-constraint? c)))
     alpha-match-total (:wat::rete::total? (:wat::core::quote (:wat::rete::alpha-match c f)))
     uuid-v4-total   (:wat::rete::total? (:wat::core::quote (:wat::uuid::v4)))
     stream-next-total (:wat::rete::total? (:wat::core::quote (:wat::stream::next s)))
     split-total     (:wat::rete::total? (:wat::core::quote (:wat::string::split a b)))]
    (:wat::kernel::println (:wat::string::concat "hashmap::keys total?                        " (:wat::core::bool::to-string hm-keys-total)))
    (:wat::kernel::println (:wat::string::concat "map::keys total?                            " (:wat::core::bool::to-string map-keys-total)))
    (:wat::kernel::println (:wat::string::concat "type-params-used-in total?                  " (:wat::core::bool::to-string tpui-total)))
    (:wat::kernel::println (:wat::string::concat "type-equal? total?                          " (:wat::core::bool::to-string teq-total)))
    (:wat::kernel::println (:wat::string::concat "stream::empty total?                        " (:wat::core::bool::to-string stream-empty-total)))
    (:wat::kernel::println (:wat::string::concat "rete::pure? total?                          " (:wat::core::bool::to-string rete-pure-total)))
    (:wat::kernel::println (:wat::string::concat "rete::total? total?                         " (:wat::core::bool::to-string rete-total-total)))
    (:wat::kernel::println (:wat::string::concat "rete::vocabulary-admitted? total?           " (:wat::core::bool::to-string rete-vocab-total)))
    (:wat::kernel::println (:wat::string::concat "rete::cond-has-deferred-constraint? total?  " (:wat::core::bool::to-string rete-cond-total)))
    (:wat::kernel::println (:wat::string::concat "rete::alpha-match total?                    " (:wat::core::bool::to-string alpha-match-total)))
    (:wat::kernel::println (:wat::string::concat "uuid::v4 total? (dup, must not move)        " (:wat::core::bool::to-string uuid-v4-total)))
    (:wat::kernel::println (:wat::string::concat "stream::next total? (dup, must not move)    " (:wat::core::bool::to-string stream-next-total)))
    (:wat::kernel::println (:wat::string::concat "string::split total? (control)              " (:wat::core::bool::to-string split-total)))))
