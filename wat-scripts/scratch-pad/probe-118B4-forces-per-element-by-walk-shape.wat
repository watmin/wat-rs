;; probe-118B4-forces-per-element-by-walk-shape.wat
;;
;; ⛔ THE QUESTION THIS EXISTS TO SETTLE: option 3 of the B4 fork closes `rest` ONLY, on the
;; argument that "the three-call walk needs a tail, so killing `rest` kills the walk." That is a
;; claim about what remains REPRESENTABLE after the close, and it was asserted without measurement.
;; This probe measures it.
;;
;; MECHANISM: the generator prints one "FORCED" line per cell realization. There is no memo (118.B3
;; deleted both), so every independent force of the same cell prints again. 5 elements:
;;
;;   5  = one force per element (the wall holds)
;;  >5  = the walk shape re-forces cells the user cannot see
;;
;; Three walk shapes, each on a FRESH 5-element stream:
;;   A  next-only                    — what B2b migrated the stdlib onto
;;   B  empty? + next                — representable after option 3 (rest closed, empty? open)
;;   C  empty? + first + next        — also representable after option 3 (first open too)
;;
;; ★★ MEASURED 2026-08-18, HEAD 63091aff, capped run, rc=0 — THE ANSWER, AND IT REFUTED THE CLAIM:
;;
;;      A  next-only                 6 FORCED  = n+1   1x per cell
;;      B  empty? + next            11 FORCED  = 2n+1  2x per cell
;;      C  empty? + first + next    16 FORCED  = 3n+1  3x per cell
;;
;; ⛔ WALK C USES NO `rest` AT ALL and pays the full 3x. The option-3 argument — "the three-call
;; walk needs a tail, so closing `rest` closes the walk" — is FALSE: `next` is itself a tail source,
;; so closing `rest` only changes how the user spells the same 3x walk. Option 2 (close first+rest)
;; dies on row B for the same reason. Only closing ALL THREE leaves `next` as the sole door.
;;
;; ⚠ WHAT THIS PROBE CANNOT SEE: it does not prove double-forcing becomes IMPOSSIBLE under a full
;; close — `(do (next s) (next s))` still forces twice. It proves that after a full close every
;; force is a VISIBLE `next`, countable by reading the source, where today three verbs that each
;; look free hide the cost.
;;
;; ⛔ STONE 118.B4-iii — THE WALL SHIPPED (2026-08-18): `first`/`rest`/`empty?`/`nth` no longer
;; accept a Stream. Walks B and C above are now **ILLEGAL** — `empty?` and `first` on a Stream are
;; compile-time TypeMismatch errors — so their bodies are RETIRED below rather than left in this
;; file (a `wat-scripts/` file must still LOAD under `every_wat_scripts_file_loads`, and a form
;; that no longer type-checks would break that gate). Their historical measurements stand exactly
;; as recorded above — that data does not change; only its REPRODUCIBILITY does, on purpose: the
;; wall's whole point is that walk shapes B and C can no longer be spelled. Confirmed at the wall
;; (`--check` on a two-line reproduction of each, verbatim):
;;
;;   walk B (`empty?` on Stream):
;;     :wat::core::empty?: parameter #1 expects a lazy Stream<T> has no empty? — advance it with
;;     :wat::stream::next, whose NextOutcome<T> = Item(value, rest) | Exhausted answers exactly
;;     what empty? was asked; got :wat::stream::Stream<wat::core::i64>
;;
;;   walk C (`first` on Stream, same `empty?` refusal fires first):
;;     :wat::core::first: parameter #1 expects a lazy Stream<T> has no first/second/third —
;;     advance it with :wat::stream::next (NextOutcome<T> = Item(value, rest) | Exhausted); got
;;     :wat::stream::Stream<wat::core::i64>
;;
;; Walk A is the sole survivor — it was already the only walk the migrated stdlib uses (B2b).
;;
;; RUN (capped, per the standing rule):
;;   systemd-run --user --scope -q -p MemoryMax=512M -p MemorySwapMax=0 timeout 60 \
;;     ./target/release/wat wat-scripts/scratch-pad/probe-118B4-forces-per-element-by-walk-shape.wat

(:wat::core::defn :user::gen
  [n <- :wat::core::i64] -> :wat::stream::Stream<wat::core::i64>
  (:wat::stream::lazy
    (:wat::core::do
      (:wat::kernel::println "FORCED")
      (:wat::core::if (:wat::core::<= n 0)
        (:wat::stream::empty)
        (:wat::stream::cons n (:user::gen (:wat::core::- n 1)))))))

;; A — next-only. One force per cell by construction. The ONLY walk shape THE WALL still permits.
(:wat::core::defn :user::walk-a
  [s <- :wat::stream::Stream<wat::core::i64> acc <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match (:wat::stream::next s)
    ((:wat::stream::NextOutcome::Item value rest) (:user::walk-a rest (:wat::core::+ acc value)))
    (:wat::stream::NextOutcome::Exhausted acc)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "== A next-only")
    (:wat::kernel::println (:user::walk-a (:user::gen 5) 0))))
