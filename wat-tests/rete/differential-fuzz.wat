;; wat-tests/rete/differential-fuzz.wat — the rete differential fuzzer, in wat.
;;
;; THE PROPERTY: for every generated rule/query shape, `fire-rules` (native) and
;; `fire-rules$oracle` (the wat reference) must return the SAME number of rows —
;; after a single fire, and (the `retr` dimension, 2026-08-27) after a
;; fire → retract → RE-fire, where the second fire runs over a reduced fact set
;; on a session already carrying the first fire's memories.
;; Nothing here hardcodes a right answer — the oracle supplies every expected
;; value, which is what lets the space grow without hand-authoring per case.
;;
;; WHY QUERY ROWS AND NOT DERIVED FACTS. `production_delta` dedups derived facts
;; by value, so a rule whose `:then` binds one variable derives the SAME fact
;; however many tokens reach it. A differential over derived-fact counts reads
;; identically on a correct engine and on one that multiplies tokens — which is
;; exactly how a leading-`:not`/`:exists` defect survived the whole arc, and why
;; 37 of the 57 queries in the where-family corpus cannot see it. Every query
;; here carries the RULE'S OWN LHS, so `query` reads beta, below the dedup.
;;
;; SHAPE-SPACE, NOT FACT-VOLUME. The oracle is superlinear — measured, ~O(n²):
;; 31 facts 11.5ms, 136 facts 81ms, 556 facts 0.96s, 2236 facts 15.3s. So cases
;; stay at a handful of facts and the yield comes from shape diversity. A join or
;; negation defect shows at 3 facts exactly as at 3000. The floor is not 1 fact
;; though: telling 1 from N needs >= 2 of a class, and rounds need a chain.
;;
;; Rules and queries are assembled as `Rule` / `Query` VALUES from a pool of
;; quasiquoted forms — no source-text generation, no external tool. A case is a
;; COORDINATE, so a finding reproduces forever from its tuple.

;; `:wat::gen::` is STDLIB as of 2026-08-25 — no load-file! needed.

(:wat::core::defrecord :wat-tests::rete::fuzz::W  [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::G  [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::P1 [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::P2 [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::S1 [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::S2 [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::S3 [k <- :wat::core::i64])
(:wat::core::defrecord :wat-tests::rete::fuzz::S4 [k <- :wat::core::i64])

(:wat::core::defn :wat-tests::rete::fuzz::no-conds [] -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::PersistentVector))

;; ── the chain pool: depth D = first D links, driving D+1 fixpoint rounds ──────
;; Round depth is the highest-yield dimension we have: every pre-existing leading
;; filter test fired ONE round, where "once per fire" and "once per round" are
;; the same number, which is precisely why the defect hid.
(:wat::core::defn :wat-tests::rete::fuzz::chain [d <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::take
      (:wat::core::PersistentVector
        (:wat::rete::Rule :name "r1"
          :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::S1 (?k <- :k))))
          :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::S2 ?k))))
        (:wat::rete::Rule :name "r2"
          :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::S2 (?k <- :k))))
          :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::S3 ?k))))
        (:wat::rete::Rule :name "r3"
          :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::S3 (?k <- :k))))
          :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::S4 ?k)))))
      d)))

;; ── condition pool ───────────────────────────────────────────────────────────
(:wat::core::defn :wat-tests::rete::fuzz::prefix-conds [n <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::= n 0)
    (:wat-tests::rete::fuzz::no-conds)
    (:wat::core::if (:wat::core::= n 1)
      (:wat::core::PersistentVector (:wat::core::quasiquote (:wat-tests::rete::fuzz::P1 (?a <- :k))))
      (:wat::core::PersistentVector
        (:wat::core::quasiquote (:wat-tests::rete::fuzz::P1 (?a <- :k)))
        (:wat::core::quasiquote (:wat-tests::rete::fuzz::P2 (?b <- :k)))))))

;; THE FILTER POOL — each entry is a VECTOR of conditions, because some shapes are
;; more than one LHS element (accumulate carries its own threshold `where`).
;;
;;   0 none · 1 exists · 2 not · 3 accumulate(count|max)+threshold · 4 intra-condition :or
;;   5 intra-condition :not-of-a-constraint · 6 top-level :or ACROSS conditions
;;   7 :not over a DERIVED class (stratified negation)
;;
;; 3 and 4 are the widening. The accumulate threshold is `>= 2` deliberately, so
;; the answer VARIES with the `dups` dimension instead of being vacuously true —
;; a gate that cannot change its mind measures nothing. And 4 is the THIRD `or`
;; engine in rete (top-level-across-conditions, where-expression, and
;; intra-condition); the intra-condition one had no corpus at all until
;; `where-or-inline`, and reaching `Op::Or`/`Op::Not` is exactly what that found.
;; ── family 3, the ACCUMULATE family — TWO KINDS, and the split is the point ──
;;
;; `fp` carries both parameters: kind = fp/3, threshold = (fp mod 3) + 1. Packing them into the
;; existing dependent parameter costs no new dimension, and `param-space` already varies per shape
;; — this is what `bind` is for.
;;
;; WHY THESE TWO AND NOT ALL NINE. The accumulator surface is nine verbs, but by RETURN TYPE it is
;; only two classes (`wat/rete/acc.wat`): `count`/`sum` return a bare `i64` and always produce a
;; token; `max`/`min`/`mean` return `(Option i64)` and produce NO TOKEN when the `:from` set is
;; empty. Until 2026-08-27 the fuzzer generated `count` only, so the ENTIRE Option class — and with
;; it the empty-set arm — was never differentially tested. `count` + `max` covers both classes;
;; `sum` would be a second `count` and `min`/`mean` a second `max`, at 1.5x the budget each.
;;
;; THE THRESHOLD RANGE IS 0,1,2 AND THE ZERO IS LOAD-BEARING — measured, not assumed. The first
;; cut of this used 1,2,3 (carried over from when `count` was the only kind) and it could NOT
;; reach the class it was added for. On an EMPTY `:from` set, `count` returns 0 and FAILS a
;; threshold of 1, while `max` returns None and emits no token — both produce zero rows, by
;; different routes, indistinguishable. Only at threshold 0 do they split: `count` fires (0 >= 0)
;; and `max` still does not. Measured directly, dups=1:
;;
;;     thr1  count non-empty 1 | max non-empty 0     <- the kinds differ (non-vacuous either way)
;;     thr1  count EMPTIED   0 | max EMPTIED   0     <- SAME answer, different routes
;;     thr0  count EMPTIED   1 | max EMPTIED   0     <- the None arm, only visible here
;;
;; And the empty set is only REACHABLE because of the retraction dimension: `dups=1` with
;; `retr=1` retracts the one W and leaves `:from` empty. Two widenings that only pay together —
;; neither alone reaches this row.
(:wat::core::defn :wat-tests::rete::fuzz::acc-cond [fp <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::let [kind (:wat::core::i64::quot fp 3)
                    ;; thresholds 0,1,2 — NOT 1,2,3. Zero is the load-bearing one and it was
                    ;; missing: see the header note. It is not a vacuous gate, it is the ONLY
                    ;; spelling that separates `max`'s None arm from `count`'s zero.
                    thr  (:wat::core::i64::rem fp 3)]
    (:wat::core::if (:wat::core::= kind 0)
      (:wat::core::PersistentVector
        (:wat::core::quasiquote (?n <- (:wat::rete::acc::count) :from (:wat-tests::rete::fuzz::W)))
        ;; PARAMETERIZED: the threshold is generated, not hardcoded, so the
        ;; gate genuinely changes its mind across the space.
        (:wat::core::quasiquote
          (:wat::rete::where (:wat::rete::core::i64::>= ?n (:wat::core::unquote thr)))))
      (:wat::core::PersistentVector
        (:wat::core::quasiquote
          (?n <- (:wat::rete::acc::max ?v) :from (:wat-tests::rete::fuzz::W (?v <- :k))))
        (:wat::core::quasiquote
          (:wat::rete::where (:wat::rete::core::i64::>= ?n (:wat::core::unquote thr))))))))

(:wat::core::defn :wat-tests::rete::fuzz::filt-cond [f <- :wat::core::i64  fp <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::cond
    ((:wat::core::= f 0) (:wat-tests::rete::fuzz::no-conds))
    ((:wat::core::= f 1)
      (:wat::core::PersistentVector
        (:wat::core::quasiquote (:wat::rete::exists (:wat-tests::rete::fuzz::W (?w <- :k))))))
    ((:wat::core::= f 2)
      (:wat::core::PersistentVector
        (:wat::core::quasiquote (:wat::rete::not (:wat-tests::rete::fuzz::G (?g <- :k))))))
    ((:wat::core::= f 3) (:wat-tests::rete::fuzz::acc-cond fp))
    ;; 4 — intra-condition `:or`, the SECOND of rete's three `or` engines.
    ((:wat::core::= f 4)
      (:wat::core::PersistentVector
        (:wat::core::quasiquote
          (:wat-tests::rete::fuzz::W (?w <- :k)
            (:wat::rete::or (:wat::rete::core::i64::> ?w (:wat::core::unquote fp))
                            (:wat::rete::core::i64::< ?w 3))))))
    ;; 5 — intra-condition `:not` of a CONSTRAINT (not of a condition).
    ((:wat::core::= f 5)
      (:wat::core::PersistentVector
        (:wat::core::quasiquote
          (:wat-tests::rete::fuzz::W (?w <- :k)
            (:wat::rete::not (:wat::rete::core::i64::> ?w 100))))))
    ;; 6 — top-level `:or` ACROSS conditions: network branches, the first of rete's three `or`
    ;; engines, and the only one that binds a DIFFERENT variable per branch.
    ((:wat::core::= f 6)
      (:wat::core::PersistentVector
        (:wat::core::quasiquote
          (:wat::rete::or (:wat-tests::rete::fuzz::P1 (?a <- :k))
                          (:wat-tests::rete::fuzz::W (?w <- :k))))))
    ;; 7 — `:not` over a DERIVED class. STRATIFICATION: S2 exists only because the chain derives
    ;; it, so the answer must depend on the depth dimension. This is where family C lived.
    (:else
      (:wat::core::PersistentVector
        (:wat::core::quasiquote
          (:wat::rete::not (:wat-tests::rete::fuzz::S2)))))))

;; A CONSTANT predicate: POSITION is the variable under test here. A `where`
;; naming a variable bound LATER is a compile-time question, not a differential
;; one, and is deliberately NOT generated — a case that fails to compile would
;; take the whole batch with it. That gap is real and stated, not hidden.
(:wat::core::defn :wat-tests::rete::fuzz::where-cond [] -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::PersistentVector
    (:wat::core::quasiquote (:wat::rete::where (:wat::rete::core::i64::> 1 0)))))

;; 0 none · 1 first · 2 last
(:wat::core::defn :wat-tests::rete::fuzz::build-lhs
  [prefix <- :wat::core::i64  f <- :wat::core::i64  fp <- :wat::core::i64  wpos <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::let [head (:wat::core::if (:wat::core::= wpos 1) (:wat-tests::rete::fuzz::where-cond) (:wat-tests::rete::fuzz::no-conds))
                    tail (:wat::core::if (:wat::core::= wpos 2) (:wat-tests::rete::fuzz::where-cond) (:wat-tests::rete::fuzz::no-conds))
                    body (:wat::core::PersistentVector/concat
                           (:wat-tests::rete::fuzz::prefix-conds prefix) (:wat-tests::rete::fuzz::filt-cond f fp))]
    (:wat::core::PersistentVector/concat
      (:wat::core::PersistentVector/concat head body) tail)))

;; THE CASE IS A RECORD, not a bare coordinate — built by `gen-record` over five
;; `gen-ints`. The property then reads `(:wat-tests::rete::fuzz::Case/depth c)` instead of
;; `(i64s c 4)`, so a dimension cannot be silently transposed by a reader, and
;; adding a dimension is a field rather than an index everyone must re-count.
(:wat::core::defrecord :wat-tests::rete::fuzz::Case
  [dups   <- :wat::core::i64
   wpos   <- :wat::core::i64
   prefix <- :wat::core::i64
   filt   <- :wat::core::i64
   fparam <- :wat::core::i64
   depth  <- :wat::core::i64
   retr   <- :wat::core::i64])

;; ── RETRACTION: fire, remove a fact, fire AGAIN ──────────────────────────────
;;
;; Added 2026-08-27. Until then every case only ever INSERTED, and that left the whole
;; non-monotonic direction untested: a fact LEAVING must un-derive everything it supported.
;; `retract` is stage-only — it removes from `Session/facts` by VALUE and the caller re-fires
;; (`wat/rete/oracle/insert.wat`) — so this dimension is really "does a SECOND fire, over a
;; reduced fact set, on a session that already carries memories from the first, agree between the
;; engines". Memories accumulating across fires is precisely the shape that produced families A
;; and C (a query harvested from a beta that was never cleared), which is why this is the widening
;; worth having rather than another condition family.
;;
;; THE TARGET DEPENDS ON THE SHAPE, and costs no cardinality because it is a FUNCTION of `depth`
;; rather than a generated dimension:
;;   depth > 0  → retract the chain SEED. S2/S3/S4 exist only by derivation, so this tests
;;                TRANSITIVE un-derivation — and for the `:not`-over-a-derived-class family it
;;                flips the answer, which is exactly where family C lived.
;;   depth = 0  → nothing to cascade, so retract `(W 0)`, which every case has. Since the W's are
;;                DISTINCT this removes exactly ONE, so the accumulate's count drops by one rather
;;                than to zero — and at `dups=1` it empties the set, which is the only way this
;;                space reaches an `Option`-returning accumulate's None arm.
;;
;; A shape where the retraction touches nothing the query reads is NOT wasted: "an unrelated fact
;; leaving must not perturb this answer" is the same class of property as "an inert cascade's
;; round count must not leak into this answer", and that one was a live defect for three days.
(:wat::core::defn :wat-tests::rete::fuzz::refire-native
  [d <- :wat::core::i64  st <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::if (:wat::core::> d 0)
    (:wat::core::match (:wat::rete::fire-rules
      (:wat::rete::retract (:wat::core::match (:wat::rete::fire-rules st) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:wat-tests::rete::fuzz::S1 1))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
    (:wat::core::match (:wat::rete::fire-rules
      (:wat::rete::retract (:wat::core::match (:wat::rete::fire-rules st) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:wat-tests::rete::fuzz::W 0))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))

(:wat::core::defn :wat-tests::rete::fuzz::refire-oracle
  [d <- :wat::core::i64  st <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::if (:wat::core::> d 0)
    (:wat::core::match (:wat::rete::fire-rules$oracle
      (:wat::rete::retract (:wat::core::match (:wat::rete::fire-rules$oracle st) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:wat-tests::rete::fuzz::S1 1))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
    (:wat::core::match (:wat::rete::fire-rules$oracle
      (:wat::rete::retract (:wat::core::match (:wat::rete::fire-rules$oracle st) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:wat-tests::rete::fuzz::W 0))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))))

;; ── the property ─────────────────────────────────────────────────────────────
(:wat::core::defn :wat-tests::rete::fuzz::prop [c <- :wat-tests::rete::fuzz::Case] -> :wat::core::bool
  (:wat::core::let [dups   (:wat::core::i64::+ (:wat-tests::rete::fuzz::Case/dups c) 1)
                    wpos   (:wat-tests::rete::fuzz::Case/wpos c)
                    prefix (:wat-tests::rete::fuzz::Case/prefix c)
                    f      (:wat-tests::rete::fuzz::Case/filt c)
                    fp     (:wat-tests::rete::fuzz::Case/fparam c)
                    d      (:wat-tests::rete::fuzz::Case/depth c)
                    retr   (:wat-tests::rete::fuzz::Case/retr c)
                    q  (:wat::rete::Query :name "q" :params (:wat::core::PersistentVector)
                         :lhs (:wat-tests::rete::fuzz::build-lhs prefix f fp wpos))
                    s0 (:wat::rete::compile-all (:wat-tests::rete::fuzz::chain d) (:wat::core::PersistentVector q))
                    ;; W_i = (W i), DISTINCT — not `dups` copies of one value. Until 2026-08-27
                    ;; every W was `(W 7)`, and that quietly made two dimensions vacuous: an
                    ;; accumulate's `max`/`min` cannot vary when every value is equal, and family
                    ;; 4's `(?w > fp)` was TRUE for every generated `fp` (0..2) because w was
                    ;; always 7 — so its `:or` never once took the second arm. A gate that cannot
                    ;; change its mind measures nothing; this makes count, sum, max and the inline
                    ;; `:or` all vary with `dups`.
                    ws (:wat::core::into (:wat::core::PersistentVector)
                         (:wat::core::mapv
                           (:wat::core::fn [i <- :wat::core::i64] -> :wat-tests::rete::fuzz::W (:wat-tests::rete::fuzz::W i))
                           (:wat::core::range 0 dups)))
                    s1 (:wat::core::match (:wat::rete::insert-all s0 ws) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
                    s2 (:wat::core::match (:wat::rete::insert-all s1 (:wat::core::PersistentVector (:wat-tests::rete::fuzz::P1 1))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
                    s3 (:wat::core::match (:wat::rete::insert-all s2 (:wat::core::PersistentVector (:wat-tests::rete::fuzz::P2 1))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
                    st (:wat::core::match (:wat::rete::insert-all s3 (:wat::core::PersistentVector (:wat-tests::rete::fuzz::S1 1))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
                    nf (:wat::core::if (:wat::core::= retr 0)
                         (:wat::core::match (:wat::rete::fire-rules st) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                         (:wat-tests::rete::fuzz::refire-native d st))
                    of (:wat::core::if (:wat::core::= retr 0)
                         (:wat::core::match (:wat::rete::fire-rules$oracle st) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                         (:wat-tests::rete::fuzz::refire-oracle d st))
                    n  (:wat::core::length (:wat::rete::query nf q))
                    o  (:wat::core::length (:wat::rete::query of q))]
    ;; NO println here, deliberately: a deftest body runs before stdio services
    ;; are up, and more importantly a test should report through its ASSERTION, not
    ;; a side channel. The count is what the gate pins; WHICH coordinates diverge is
    ;; recoverable at any time because the space is finite, ordered and
    ;; deterministic — re-running with a printing `prop` is a one-line change, and
    ;; a coordinate is a permanent case name rather than a seed.
    (:wat::core::= n o)))


;; ── the SPACE excludes unmatchable shapes; the property no longer has to ─────
;; A case with no filter AND no fact condition leaves nothing to match on. That
;; used to be an `if` guarding the property body, plus a second pure enumeration
;; counting how many coordinates were real. Both are gone: `gen-such-that` makes
;; the exclusion part of the GENERATOR, so `Gen/card` IS the case count.
;;
;; This is the shape `test.check`'s `such-that` cannot have. There, filtering an
;; opaque random source means retry-and-discard — it can give up after N tries and
;; it skews what survives. Here the survivors are computed once and exactly.
(:wat::core::defn :wat-tests::rete::fuzz::shape-is-matchable [c <- :wat-tests::rete::fuzz::Case] -> :wat::core::bool
  (:wat::core::not (:wat::core::and (:wat::core::= (:wat-tests::rete::fuzz::Case/filt c) 0)
                                    (:wat::core::= (:wat-tests::rete::fuzz::Case/prefix c) 0))))

;; ── THE SPACE IS DEPENDENT — `bind`, not a fixed product ────────────────────
;;
;; Before `bind` existed this was a flat `record` over five fixed `ints`, so every
;; shape's parameters had to be HARDCODED — the accumulate threshold always 2, the
;; inline `:or` bound always 5. That was not a modelling choice; it is what a
;; fixed-shape coordinate space can express.
;;
;; `bind` generates the SHAPE first, then a parameter space that DEPENDS on it, so
;; branch cardinalities differ per shape — the thing a product cannot do.
;;
;; ⚠ AND THIS IS WHY `frequency` IS ABSENT AND STAYS ABSENT. In `test.check` it
;; biases a random DRAW. Here every point is visited exactly once, so a weight
;; cannot change what enumeration sees at all — and where it WOULD matter, in a
;; prefix of a sampled order, the bias is already expressible because CARDINALITY
;; IS THE WEIGHT: a shape with a 3-point parameter space occupies three times the
;; indices of a parameterless one, and does so here without anyone asking.
;; `one-of [a a b]` is a 2:1 mix. The combinator would add no expressive power,
;; only a second way to say the same thing.
(:wat::core::defn :wat-tests::rete::fuzz::param-space [f <- :wat::core::i64] -> (:wat::gen::Gen :- [:wat::core::i64])
  (:wat::core::cond
    ;; 6 = 2 accumulator kinds x 3 thresholds, decoded by `acc-cond`.
    ((:wat::core::= f 3) (:wat::gen::ints 0 6))
    ((:wat::core::= f 4) (:wat::gen::ints 0 3))
    (:else               (:wat::gen::ints 0 1))))

(:wat::core::defn :wat-tests::rete::fuzz::for-shape [f <- :wat::core::i64] -> (:wat::gen::Gen :- [:wat-tests::rete::fuzz::Case])
  (:wat::gen::record :wat-tests::rete::fuzz::Case
    (:wat::gen::ints 0 3)
    (:wat::gen::ints 0 3)
    (:wat::gen::ints 0 3)
    (:wat::gen::ints f (:wat::core::i64::+ f 1))
    (:wat-tests::rete::fuzz::param-space f)
    (:wat::gen::ints 0 4)
    (:wat::gen::ints 0 2)))

(:wat::core::defn :wat-tests::rete::fuzz::space [] -> (:wat::gen::Gen :- [:wat-tests::rete::fuzz::Case])
  (:wat::gen::such-that :wat-tests::rete::fuzz::shape-is-matchable
    (:wat::gen::bind (:wat::gen::ints 0 8) :wat-tests::rete::fuzz::for-shape)))

;; ── THE GATE — NOW A ZERO, AND IT WAS EARNED RATHER THAN ASSERTED ───────────
;;
;; This was a RATCHET for three days because a zero it could not honestly hold would have been a
;; lie: 120 real divergences of 1260 shapes, then 72, each family reproduced standalone in
;; tests/rete/probe_arc278_fuzzer_found_divergences.{rs,wat} and listed in
;; docs/arc/2026/06/278-rules-engine/RETE-FIX-LIST.md. Every one of them is now closed, so the
;; count is 0 and this is an AGREEMENT GATE: native and the `$oracle` agree bit-for-bit on the
;; whole generated space, and ANY nonzero is a regression with a coordinate attached.
;;
;; A ratchet is the right shape while defects are known-and-open — asserting 0 then would have
;; reddened the floor and blocked unrelated work, and deleting the shape that found them to keep
;; a gate green is the trade this codebase refuses. It is the WRONG shape once the count reaches
;; 0: a pinned 72 would have gone on passing after the fix and said nothing.
;;
;; ── FAMILIES A AND C CLOSED 2026-08-26: 72 -> 0. ONE ROOT, NOT TWO. ─────────
;;    18  f=3  accumulate  A leading accum in a QUERY: native = round count, oracle = 1
;;    54  f=7  :not over a DERIVED class, in a QUERY
;;                         C native = 1, oracle = 0 — ALL at depth >= 1, never depth 0
;;
;; They read as two defects and are one sentence: A QUERY'S NON-MONOTONIC CONDITION WAS EVALUATED
;; INSIDE THE FIXPOINT INSTEAD OF ONCE AGAINST THE CLOSED WORLD. A constrained query is harvested
;; from `wm.beta`, which by the semi-naive contract accumulates across rounds and is never
;; cleared — so it holds tokens that were only ever true of the round that produced them. A
;; leading accumulate's parent gains one per round (A); a `:not` propagated in round 0 is never
;; retracted when a later round derives the fact (C). Non-monotonic is exactly the class a later
;; round can invalidate, which is why all 72 were `:not` and accumulate and nothing else.
;;
;; The STRATIFIED driver never had it — it fires the query slice once against the closure — and
;; that is what `fire_unstratified` (src/rete/kernel/fire/rules.rs) now gives the fast path, at
;; ONE door both `max_s == 0` exits go through, so the requery cannot be skipped by taking the
;; other one. Held by grid axes `where-accum-lead-cascade` (A) and `where-not-derived-in-query`
;; (C), where native, the `$oracle` AND Clara 0.24.0 all print identical rows.
;;
;; Clara also settled the question that had blocked C: a `defquery`'s negation IS stratified the
;; way a `defrule`'s is. The oracle was right; native was wrong; the arc's standing rule held.
;;
;; ── FAMILY B CLOSED 2026-08-26: 120 -> 72, and the 48 it removed are named. ──
;; A `:where` binding NOTHING sorts into `sort-lhs`'s INDEPENDENT partition and lands ABOVE the
;; accumulate, so the graph is RootJoin -> Test -> Accumulate -> Test -> Production. The
;; accumulate pass is 3.25 and the filter pass is 3.5, so that leading Test had never fired when
;; the accumulate read its parent delta: it saw nothing and the rule matched ZERO, while the SAME
;; rule without the bindless `:where` matched fine. Fixed by pulling a Test parent forward in
;; `src/rete/kernel/fire/pass/accumulate.rs`. Held by grid axis `where-accum-where-chain`, where
;; native, the `$oracle` AND Clara 0.24.0 all now print the same two rows.
;;
;; An EMPTY space fails outright — a shape-space filtered to nothing means the run
;; tested NOTHING, which must never read as "no divergences found".
;; THE BUDGET, AND THE RUNNER THAT HAS TO GRANT IT. The default deftest budget is
;; 5000ms. This run takes ~21.9s alone — 2520 shapes, each firing BOTH engines, and the half with
;; `retr=1` firing each engine TWICE (fire, retract, re-fire). Measured 2026-08-27 by driving
;; `(:wat::gen::check (space) prop)` from a scratch `:user::main` on the ALREADY-BUILT binary,
;; which is the right loop for a wat-only change: `card=2520 points=2520 violations=0` in 21.9s
;; with ZERO compile. The loaded-floor cost is ~1.9x the isolated cost (9.2s -> 17.175s at the
;; previous width), so budget against ~41s, not against 21.9s. The
;; budget is raised rather than the space shrunk: cutting shapes to fit a timer
;; trades coverage for a green clock, and every shape here has either found a
;; defect or proved one absent.
;;
;; ⚠ RAISING IT HERE IS ONLY HALF. Until 2026-08-26 this annotation was
;; UNREACHABLE and nothing said so. `scripts/floor.sh` passes no `--profile`, so
;; nextest's `[profile.default]` SIGTERMed at 30s — half the budget argued for
;; above — and the rete cohort's 60s/120s override missed this test because it
;; filters `binary_id(wat::rete)` while a wat deftest compiles into `wat::kernel`.
;; `.config/nextest.toml` now names `test(deftest_wat_tests_rete_fuzz)` in all
;; three profile mirrors. If this test is ever renamed, THAT FILTER MUST MOVE WITH
;; IT or the 30s kill returns silently.
;;
;; ⛔ NO ORACLE RATIO IS QUOTED, DELIBERATELY. This comment used to say the
;; `$oracle` is "~300x the cost of everything else here (measured: ~7ms/case
;; against the library's ~23us/point)". The library figure was a `coords`
;; measurement; this file's space is `such-that o bind o record`, measured
;; 2026-08-26 at ~287us/point (mean of 6) — ~12x dearer. What is true and stays true: the
;; generator is a SMALL fraction of a case (~0.29ms of the floor's 13.6ms/case,
;; ~2%) and the engines-plus-oracle are the rest. The RATIO itself is not worth
;; pinning: the `$oracle` is slow-but-correct by design, carries no perf
;; requirement, and gets passively faster as wat stops being interpreted — so any
;; multiple against it shrinks on its own, and the generator's SHARE grows.
;; 90s, not 60s: the measured loaded cost is ~41s, and 60s left only 1.46x of margin once
;; retraction doubled the space. 90s keeps ~2.2x AND stays BELOW nextest's 120s kill — the
;; ordering matters, because a wat-side time-limit failure names the test and the budget, while a
;; SIGTERM at the harness level names neither.
(:wat::test::time-limit "90s")
(:wat::test::deftest :wat-tests::rete::fuzz::test-native-matches-oracle
  (:wat::core::match
    (:wat::gen::check (:wat-tests::rete::fuzz::space) :wat-tests::rete::fuzz::prop)
    ((:wat::gen::CheckOutcome::Checked cases bad _first)
      (:wat::core::let [_ (:wat::test::assert-true (:wat::core::> cases 0))]
        (:wat::test::assert-eq bad 0)))
    (:wat::gen::CheckOutcome::EmptySpace (:wat::test::assert-true false))))


;; ── THE SPACE'S OWN NON-VACUITY GATE ─────────────────────────────────────────
;;
;; `acc-cond`'s header states a measured table: which accumulator kind × threshold combinations
;; actually distinguish `count` from `max`. That table is the JUSTIFICATION for generating 648
;; extra shapes, and a justification behind no gate is exactly the drift this codebase keeps
;; removing — the same reason `wat_scripts_grid_axes_live` refuses a new sized axis without a
;; deliberately non-vacuous size.
;;
;; So the table is asserted, not merely written down. If any row moves, either the engine changed
;; or the widening stopped reaching what it was added for; both must be loud. The first cut of
;; `acc-cond` DID stop reaching it — thresholds were 1,2,3 and the `Option`/None arm needs 0 —
;; and this gate is what that mistake bought.
(:wat::rete::defquery :wat-tests::rete::fuzz::nv-count-1 :params []
  :when [(?n <- (:wat::rete::acc::count) :from (:wat-tests::rete::fuzz::W))
         (:wat::rete::where (:wat::rete::core::i64::>= ?n 1))])

(:wat::rete::defquery :wat-tests::rete::fuzz::nv-count-0 :params []
  :when [(?n <- (:wat::rete::acc::count) :from (:wat-tests::rete::fuzz::W))
         (:wat::rete::where (:wat::rete::core::i64::>= ?n 0))])

(:wat::rete::defquery :wat-tests::rete::fuzz::nv-max-1 :params []
  :when [(?n <- (:wat::rete::acc::max ?v) :from (:wat-tests::rete::fuzz::W (?v <- :k)))
         (:wat::rete::where (:wat::rete::core::i64::>= ?n 1))])

(:wat::rete::defquery :wat-tests::rete::fuzz::nv-max-0 :params []
  :when [(?n <- (:wat::rete::acc::max ?v) :from (:wat-tests::rete::fuzz::W (?v <- :k)))
         (:wat::rete::where (:wat::rete::core::i64::>= ?n 0))])

;; dups=1 — a single `(W 0)`, so count=1 and max=0; retracting it empties the `:from` set.
(:wat::core::defn :wat-tests::rete::fuzz::nv-rows
  [emptied <- :wat::core::bool  q <- :wat::rete::Query] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::core::match (:wat::rete::insert
                         (:wat::rete::compile-all
                           (:wat::core::PersistentVector)
                           (:wat::core::PersistentVector
                             (:wat-tests::rete::fuzz::nv-count-1) (:wat-tests::rete::fuzz::nv-count-0)
                             (:wat-tests::rete::fuzz::nv-max-1)   (:wat-tests::rete::fuzz::nv-max-0)))
                         (:wat-tests::rete::fuzz::W 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
                    fired (:wat::core::if emptied
                            (:wat::core::match (:wat::rete::fire-rules
                              (:wat::rete::retract (:wat::core::match (:wat::rete::fire-rules s0) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:wat-tests::rete::fuzz::W 0))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                            (:wat::core::match (:wat::rete::fire-rules s0) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))]
    (:wat::core::length (:wat::rete::query fired q))))

;; `deftest` takes a name and ONE body form, so the six rows are sequenced in a `let` —
;; the same idiom the differential test above uses for its own two-step body.
(:wat::test::deftest :wat-tests::rete::fuzz::test-accumulate-kinds-are-not-vacuous
  (:wat::core::let
    ;; A — the KINDS differ at a threshold the space generates. Without this row the `max`
    ;; shapes would be 648 restatements of `count`.
    [a1 (:wat-tests::rete::fuzz::nv-rows false (:wat-tests::rete::fuzz::nv-count-1))
     a2 (:wat-tests::rete::fuzz::nv-rows false (:wat-tests::rete::fuzz::nv-max-1))
     ;; B — on an EMPTY set at threshold 1 they agree, by different routes: count fails the
     ;; threshold, max emits no token. This row is WHY threshold 0 had to be generated.
     b1 (:wat-tests::rete::fuzz::nv-rows true (:wat-tests::rete::fuzz::nv-count-1))
     b2 (:wat-tests::rete::fuzz::nv-rows true (:wat-tests::rete::fuzz::nv-max-1))
     ;; C — threshold 0 on an empty set is the ONLY place the Option/None arm is observable.
     c1 (:wat-tests::rete::fuzz::nv-rows true (:wat-tests::rete::fuzz::nv-count-0))
     c2 (:wat-tests::rete::fuzz::nv-rows true (:wat-tests::rete::fuzz::nv-max-0))
     _  (:wat::test::assert-eq a1 1)
     _  (:wat::test::assert-eq a2 0)
     _  (:wat::test::assert-eq b1 0)
     _  (:wat::test::assert-eq b2 0)
     _  (:wat::test::assert-eq c1 1)]
    (:wat::test::assert-eq c2 0)))
