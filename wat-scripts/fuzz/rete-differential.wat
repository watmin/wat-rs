;; wat-scripts/fuzz/rete-differential.wat — the rete differential fuzzer.
;;
;; THE PROPERTY: for every generated rule/query shape, `fire-rules` (native) and
;; `fire-rules$oracle` (the wat reference) must return the SAME number of rows.
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

(:wat::load-file! "../lib/gen.wat")

(:wat::core::defrecord :user::W  [k <- :wat::core::i64])
(:wat::core::defrecord :user::G  [k <- :wat::core::i64])
(:wat::core::defrecord :user::P1 [k <- :wat::core::i64])
(:wat::core::defrecord :user::P2 [k <- :wat::core::i64])
(:wat::core::defrecord :user::S1 [k <- :wat::core::i64])
(:wat::core::defrecord :user::S2 [k <- :wat::core::i64])
(:wat::core::defrecord :user::S3 [k <- :wat::core::i64])
(:wat::core::defrecord :user::S4 [k <- :wat::core::i64])

(:wat::core::defn :user::no-conds [] -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::PersistentVector))

;; ── the chain pool: depth D = first D links, driving D+1 fixpoint rounds ──────
;; Round depth is the highest-yield dimension we have: every pre-existing leading
;; filter test fired ONE round, where "once per fire" and "once per round" are
;; the same number, which is precisely why the defect hid.
(:wat::core::defn :user::chain [d <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::take
      (:wat::core::PersistentVector
        (:wat::rete::Rule :name "r1"
          :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:user::S1 (?k <- :k))))
          :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:user::S2 ?k))))
        (:wat::rete::Rule :name "r2"
          :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:user::S2 (?k <- :k))))
          :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:user::S3 ?k))))
        (:wat::rete::Rule :name "r3"
          :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:user::S3 (?k <- :k))))
          :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:user::S4 ?k)))))
      d)))

;; ── condition pool ───────────────────────────────────────────────────────────
(:wat::core::defn :user::prefix-conds [n <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::= n 0)
    (:user::no-conds)
    (:wat::core::if (:wat::core::= n 1)
      (:wat::core::PersistentVector (:wat::core::quasiquote (:user::P1 (?a <- :k))))
      (:wat::core::PersistentVector
        (:wat::core::quasiquote (:user::P1 (?a <- :k)))
        (:wat::core::quasiquote (:user::P2 (?b <- :k)))))))

;; 0 none · 1 exists · 2 not
(:wat::core::defn :user::filt-cond [f <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::= f 0)
    (:user::no-conds)
    (:wat::core::if (:wat::core::= f 1)
      (:wat::core::PersistentVector (:wat::core::quasiquote (:wat::rete::exists (:user::W (?w <- :k)))))
      (:wat::core::PersistentVector (:wat::core::quasiquote (:wat::rete::not (:user::G (?g <- :k))))))))

;; A CONSTANT predicate: POSITION is the variable under test here. A `where`
;; naming a variable bound LATER is a compile-time question, not a differential
;; one, and is deliberately NOT generated — a case that fails to compile would
;; take the whole batch with it. That gap is real and stated, not hidden.
(:wat::core::defn :user::where-cond [] -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::PersistentVector
    (:wat::core::quasiquote (:wat::rete::where (:wat::rete::core::i64::> 1 0)))))

;; 0 none · 1 first · 2 last
(:wat::core::defn :user::build-lhs
  [prefix <- :wat::core::i64  f <- :wat::core::i64  wpos <- :wat::core::i64]
  -> (:wat::core::PersistentVector :- [:wat::WatAST])
  (:wat::core::let [head (:wat::core::if (:wat::core::= wpos 1) (:user::where-cond) (:user::no-conds))
                    tail (:wat::core::if (:wat::core::= wpos 2) (:user::where-cond) (:user::no-conds))
                    body (:wat::core::PersistentVector/concat
                           (:user::prefix-conds prefix) (:user::filt-cond f))]
    (:wat::core::PersistentVector/concat
      (:wat::core::PersistentVector/concat head body) tail)))

(:wat::core::defn :user::i64s [v <- (:wat::core::PersistentVector :- [:wat::core::i64])  i <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::Option/expect (:wat::core::get v i) "coordinate digit"))

;; ── the property ─────────────────────────────────────────────────────────────
;; Coordinate digit order matches BASES below: [dups wpos prefix f d].
;; SKIP: no filter and no fact condition leaves nothing to match on.
(:wat::core::defn :user::prop [c <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::let [dups   (:wat::core::i64::+ (:user::i64s c 0) 1)
                    wpos   (:user::i64s c 1)
                    prefix (:user::i64s c 2)
                    f      (:user::i64s c 3)
                    d      (:user::i64s c 4)]
    (:wat::core::let [q  (:wat::rete::Query :name "q" :params (:wat::core::PersistentVector)
                             :lhs (:user::build-lhs prefix f wpos))
                        s0 (:wat::rete::compile-all (:user::chain d) (:wat::core::PersistentVector q))
                        ws (:wat::core::into (:wat::core::PersistentVector)
                             (:wat::core::mapv
                               (:wat::core::fn [i <- :wat::core::i64] -> :user::W (:user::W 7))
                               (:wat::core::range 0 dups)))
                        s1 (:wat::rete::insert-all s0 ws)
                        s2 (:wat::rete::insert-all s1 (:wat::core::PersistentVector (:user::P1 1)))
                        s3 (:wat::rete::insert-all s2 (:wat::core::PersistentVector (:user::P2 1)))
                        st (:wat::rete::insert-all s3 (:wat::core::PersistentVector (:user::S1 1)))
                        nf (:wat::rete::fire-rules st)
                        of (:wat::rete::fire-rules$oracle st)
                        n  (:wat::core::length (:wat::rete::query nf q))
                        o  (:wat::core::length (:wat::rete::query of q))]
        (:wat::core::if (:wat::core::= n o)
          0
          (:wat::core::let [_ (:wat::kernel::println
                                (:wat::core::String/concat
                                  (:wat::core::String/concat "MISMATCH coord=" (:wat::core::edn::to-string c))
                                  (:wat::core::String/concat
                                    (:wat::core::String/concat " native=" (:wat::core::i64::to-string n))
                                    (:wat::core::String/concat " oracle=" (:wat::core::i64::to-string o)))))]
            1)))))

;; ── the SPACE excludes invalid shapes; the property no longer has to ─────────
;; A coordinate with no filter AND no fact condition leaves nothing to match on.
;; That used to be an `if` guarding the body of `prop`, with a second pure pass
;; counting how many coordinates were real cases. Both are gone: `gen-such-that`
;; makes the exclusion part of the GENERATOR, so `card` is already the true case
;; count and the property does one thing.
;;
;; This is the shape `test.check`'s `such-that` cannot have. There, filtering an
;; opaque random source means retry-and-discard — it can give up after N tries and
;; it skews what survives. Here the survivors are computed once and exactly, so
;; `Gen/card` IS the denominator, measured rather than asserted.
(:wat::core::defn :user::shape-is-matchable
  [c <- (:wat::core::PersistentVector :- [:wat::core::i64])] -> :wat::core::bool
  (:wat::core::not (:wat::core::and (:wat::core::= (:user::i64s c 3) 0)
                                    (:wat::core::= (:user::i64s c 2) 0))))

;; bases: dups 3 · wpos 3 · prefix 3 · filter 3 · depth 4
(:wat::core::defn :user::space [] -> (:user::Gen :- [(:wat::core::PersistentVector :- [:wat::core::i64])])
  (:user::gen-such-that :user::shape-is-matchable
    (:user::gen-coords (:wat::core::PersistentVector 3 3 3 3 4))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [g   (:user::space)
                    ran (:user::Gen/card g)
                    bad (:user::gen-check g :user::prop)]
    (:wat::kernel::println
      (:wat::core::String/concat
        (:wat::core::String/concat "space=324 cases=" (:wat::core::i64::to-string (:user::Gen/card g)))
        (:wat::core::String/concat " mismatches=" (:wat::core::i64::to-string bad))))))
