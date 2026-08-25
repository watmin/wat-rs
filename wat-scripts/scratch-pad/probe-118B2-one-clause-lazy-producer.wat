;; probe-118B2-one-clause-lazy-producer.wat — the DISCONFIRMING PROBE for stone 118.B2.
;;
;; Written BEFORE B2's brief, per FM 2-bis: for a non-trivial substrate composition, grep is
;; insufficient — write the ten-line probe that attempts exactly the composition and run it. The
;; recovery doc's worked example of skipping this cost ~2 hours and a killed sweep.
;;
;; ═══ WHAT B2 CLAIMS, AND WHAT IS ACTUALLY UNPROVEN ═══════════════════════════════════════════
;;
;; B1 (488eacd0) proved a CONSUMER over `(Seqable :- [T])` works: `[s <- (Seqable :- [T])] -> i64`, called with
;; all four containers. Every verb B2 collapses is a LAZY PRODUCER, which is a different shape and
;; has never been run. Three things have to hold at once, and only the first is proven:
;;
;;   1. `Seqable/seq` resolves on all four containers                      ✅ proven, B1
;;   2. a `stream/lazy` body can `match` on `(next …)` and `stream/cons`   ❓ never run in this shape
;;   3. ★★ THE CRUX — the RECURSIVE call passes `rest`, a **Stream**, into a parameter typed
;;      `(Seqable :- [T])`, INSIDE the definition of the very fn being defined.                   ❓
;;
;; If (3) fails, the one-clause design collapses and every `<verb>-stream` TWIN comes straight
;; back — because the twin exists precisely to be the Stream-typed thing a clause can recurse into.
;; So (3) is not a detail of B2; it IS B2.
;;
;; PASS = prints all four lines below. FAIL = a TypeMismatch naming `Seqable<?N>` vs a concrete
;; container, which would be the 118.3-B defect resurfacing at a *recursive* site.
;;
;; ⚠ NOTE THE ORDER OF EVIDENCE: `--check` alone is NOT sufficient here. Task #95 (confirmed live
;; 2026-08-17) — a DOTTED call head is not type-checked at all, and `Seqable/seq` is dotted. This
;; probe must be RUN. `[[feedback_a_green_test_can_prove_nothing]]`

;; ─── (2) + (3): ONE clause, lazy producer, recursing on a Stream through a Seqable param ─────
;; This is exactly what `keep` / `map-indexed` / `dedupe` / `distinct` / `interpose` become in B2.
;; Under the old world this needs FIVE defclause arms plus a `-stream` twin.
(:wat::core::defn :probe::keep-one :- [T U]
  [f    <- [T :-> (:wat::core::Option :- [U])]
   coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [U])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next (:wat::core::Seqable/seq coll))
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::core::match (f value)
          ;; ★ (3) — `rest` is a (Stream :- [T]), handed to a (Seqable :- [T]) parameter, recursively.
          ((:wat::core::Some v) (:wat::stream::cons v (:probe::keep-one f rest)))
          (:wat::core::None (:probe::keep-one f rest))))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

;; A STATE-CARRYING producer — the harder half of the family (`keep-indexed`, `map-indexed`,
;; `dedupe`, `distinct` all thread an accumulator across the walk). Same crux, plus a threaded arg.
(:wat::core::defn :probe::index-one :- [T]
  [idx  <- :wat::core::i64
   coll <- (:wat::core::Seqable :- [T])] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next (:wat::core::Seqable/seq coll))
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::stream::cons idx (:probe::index-one (:wat::core::+ idx 1) rest)))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))

;; An unbounded source — proves the migrated shape stays LAZY (termination is the assertion).
(:wat::core::defn :probe::nat
  [i <- :wat::core::i64] -> (:wat::stream::Stream :- [:wat::core::i64])
  (:wat::stream::lazy
    (:wat::stream::cons i (:probe::nat (:wat::core::+ i 1)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [keep-even (:wat::core::fn [x <- :wat::core::i64] -> (:wat::core::Option :- [:wat::core::i64])
                 (:wat::core::if (:wat::core::= 0 (:wat::core::mod x 2))
                   (:wat::core::Some x)
                   :wat::core::None))]
    (:wat::core::do
      ;; ONE definition, FOUR container kinds at the call site — the payoff. Expect 2,4 / 2,4 / 2,4 / 2,4
      (:wat::kernel::println
        (:wat::string::join " | "
          (:wat::core::Vector :wat::core::String
            (:wat::string::join "," (:wat::core::into [] (:probe::keep-one keep-even
              (:wat::core::Vector :wat::core::i64 1 2 3 4 5))))
            (:wat::string::join "," (:wat::core::into [] (:probe::keep-one keep-even
              (:wat::core::PersistentVector 1 2 3 4 5))))
            (:wat::string::join "," (:wat::core::into [] (:probe::keep-one keep-even
              (:wat::core::List 1 2 3 4 5))))
            ;; 4th slot: a CONCRETE (Stream :- [i64]). It was `(Seqable/seq (Vector …))` while this
            ;; probe was B2's RED gate; that form is still RED, but for an UNRELATED reason —
            ;; task #95, a dotted call head is not type-checked, so a dotted surface method's
            ;; RETURN comes back as the surface's declared letter, `(Stream :- [T])`, uninstantiated.
            ;; Isolated: a concrete `(Stream :- [i64])` from an ordinary defn satisfies `(Seqable :- [i64])`
            ;; fine. Changed to the concrete form so this probe measures B1a and not #95 —
            ;; the #95 instance is recorded in MEASURED-118.B1a, not silently dropped.
            (:wat::string::join "," (:wat::core::into [] (:probe::keep-one keep-even
              (:wat::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)
                (:wat::core::Vector :wat::core::i64 1 2 3 4 5))))))))
      ;; state-carrying, over a List. Expect 0,1,2,3,4
      (:wat::kernel::println
        (:wat::string::join ","
          (:wat::core::into [] (:probe::index-one 0 (:wat::core::List 9 9 9 9 9)))))
      ;; LAZINESS over an INFINITE source through the migrated shape. Expect 0,2,4 — and it must
      ;; TERMINATE; an eager collapse here would hang rather than print.
      (:wat::kernel::println
        (:wat::string::join ","
          (:wat::core::into [] (:wat::core::take (:probe::keep-one keep-even (:probe::nat 0)) 3)))))))
