;; wat-scripts/perf/grid/where-nesting.wat — the NESTING-DEPTH family of the `where`-expressivity
;; corpus, wat side. Twin of where-nesting.clj. Copied in SHAPE from where-shapes.wat/.clj (read
;; BOTH before touching this file) per docs/arc/2026/06/278-rules-engine/BRIEF-where-corpus-families.md.
;;
;; FAMILY: nesting depth — USER FORMS CALLING USER FORMS. Where where-shapes.wat's row 5 (`big?`)
;; established that a single user-defined pure fn can sit in a `where` clause, this family asks how
;; far that goes: does purity-checking a `where` predicate survive when the predicate's own fn calls
;; another user fn, which calls another, N deep? Every user `defn` in a `where` clause is checked by
;; `src/rete/purity.rs`'s `classify_fn`, which recurses into the callee's body TRANSITIVELY — so a
;; chain of pure fns is exactly the shape task #49a's compiler must be able to walk, not just the
;; single-call case row 5 already covers.
;;
;; ── WHAT WAS FOUND (the headline) ─────────────────────────────────────────────────────────────
;;
;; NO DEPTH BOUNDARY EXISTS. `classify_fn` (purity.rs:485) threads one `seen: HashSet<String>` of
;; fqdns through the whole walk; a back-edge to an fqdn already in `seen` returns `true` (a cycle
;; contributes no NEW violation — purity.rs:27-31), so mutual/chained recursion is handled by
;; construction, not by a depth cap. There is no `MAX_DEPTH` anywhere in `src/rete/` or `src/check.rs`
;; (grepped, none found) — depth is bounded only by native Rust call-stack depth during the
;; interpreter's own (non-tail) recursive walk. Measured OUTSIDE this corpus (a throwaway /tmp chain,
;; never committed, so it cannot rot the `every_wat_scripts_file_loads` gate): straight-line chains of
;; pure fns (c1 -> c2 -> ... -> cN, each `cN(k) = cN-1(k) + 3`) used directly as a compiled `where`
;; predicate were tried at N = 10, 50, 200, 1000, 5000, 20000 — every one compiled (`--check` and a
;; full run) and fired correctly. This is a genuine capability finding, not a gap: STOP-1 never
;; triggered for pure-fn nesting depth, at any depth tried. Row 5 below keeps a depth-10 chain IN the
;; gated corpus itself as the standing witness, rather than resting the claim on a deleted probe.
;;
;; ── THE SHARED CONDITION ──────────────────────────────────────────────────────────────────────
;;
;; One record, `Req [k m]`, shared verbatim by every row's leading `[Req …]` pattern (?k ?m bound in
;; every rule even where a row's own `where` ignores one). Only the trailing predicate differs per row
;; — rule 1 of the four (see the brief). `k` drives the depth chains; `m` exists solely so the
;; two-bound-var row (row 7) has a second, independently-seeded field to compare against `k`.
;;
;; ── ROWS (8-15 required; 11 landed) ───────────────────────────────────────────────────────────
;;   1  depth-2  chain  — c2(k) = c1(k)+3, c1(k) = k mod 13.               c2 > 10  -> 75 of 200
;;   2  depth-3  chain  — c3(k) = c2(k)+3.                                  c3 > 15  -> 45 of 200
;;   3  depth-4  chain  — c4(k) = c3(k)+3.                                  c4 > 15  -> 90 of 200
;;   4  depth-5  chain  — c5(k) = c4(k)+3.                                  c5 > 20  -> 60 of 200
;;   5  depth-10 chain  — c10(k) = c1(k) + 9*3 = c1(k)+27 (the "how far     c10 > 32 -> 105 of 200
;;      does it keep going" row, kept live in the gate rather than left as a deleted /tmp probe).
;;   6  inline arithmetic tree, 6 levels, ZERO fn calls — pure structural depth, the reference
;;      the compiler needs SEPARATE from call depth (a nested AST the compiler could specialise
;;      without a call at all):
;;        g1=k+10  g2=g1*2  g3=g2-15  g4=g3/3  g5=g4+7  g6=g5*2            g6 > 157 -> 94 of 200
;;   7  TWO bound vars — twoarg(a,b) = (a+b) > 113, called as (twoarg ?k ?m).        -> 108 of 200
;;   8  argument IS a call — (wrap (c2 ?k)): the where-clause's OWN top-level call takes, as its
;;      argument, a call to a DIFFERENT pure fn — nesting at the call-site's argument position,
;;      not inside a callee's body. wrap(v) = (v mod 4) == 0.                        -> 46 of 200
;;   9  mutual/chained helpers — f calls g AND h; g and h BOTH call the shared hub. Diamond shape:
;;      hub(k)=k mod 17, g=hub+2, h=hub*2, f = (g>5) and (h<25).                     -> 108 of 200
;;  10  non-bool return THEN compared — score(k) = (3k) mod 11 (returns i64); where-c wraps it in
;;      `>` from outside.                                            score(?k) > 6  -> 72 of 200
;;  11  bool returned DIRECTLY, contrasted with row 10 — is-good(k) calls score(k) internally and
;;      returns bool itself; the where-clause is the bare call, no external comparison.
;;      is-good(k) = score(k) is even.                                (is-good ?k)  -> 109 of 200
;;
;; ── SEED (rule 3: formula over i, never a table) ──────────────────────────────────────────────
;;   k(i) = i
;;   m(i) = (7i + 11) mod 40
;;
;; ── MIRRORING (rule 4) ────────────────────────────────────────────────────────────────────────
;; Every `/` on a wat side is `quot` on the Clara side (Clojure `/` on two ints yields a RATIO); every
;; mod is spelled out as `x - (x/n)*n`, matching where-shapes' convention, so a row measures the
;; constraint and not a translation choice.

(:wat::core::defn :wnst::items [] -> :wat::core::i64 200)

(:wat::core::defn :wnst::row-count [] -> :wat::core::i64 11)

(:wat::core::defrecord :wnst::Req [k <- :wat::core::i64  m <- :wat::core::i64])
(:wat::core::defrecord :wnst::Hit [k <- :wat::core::i64])

;; ── depth chain c1..c10 — each level's ONLY new work is "+3", so the chain measures purity-check
;; nesting depth and nothing else. c1 is the leaf (no call); cN calls c(N-1).
(:wat::rete::core::defn :wnst::c1  [k <- :wat::core::i64] -> :wat::core::i64
  (:wat::rete::core::i64::- k (:wat::rete::core::i64::* (:wat::rete::core::i64::/ k 13 :undefined 0) 13 :undefined 0) :undefined 0))
(:wat::rete::core::defn :wnst::c2  [k <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ (:wnst::c1 k) 3 :undefined 0))
(:wat::rete::core::defn :wnst::c3  [k <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ (:wnst::c2 k) 3 :undefined 0))
(:wat::rete::core::defn :wnst::c4  [k <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ (:wnst::c3 k) 3 :undefined 0))
(:wat::rete::core::defn :wnst::c5  [k <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ (:wnst::c4 k) 3 :undefined 0))
(:wat::rete::core::defn :wnst::c6  [k <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ (:wnst::c5 k) 3 :undefined 0))
(:wat::rete::core::defn :wnst::c7  [k <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ (:wnst::c6 k) 3 :undefined 0))
(:wat::rete::core::defn :wnst::c8  [k <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ (:wnst::c7 k) 3 :undefined 0))
(:wat::rete::core::defn :wnst::c9  [k <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ (:wnst::c8 k) 3 :undefined 0))
(:wat::rete::core::defn :wnst::c10 [k <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ (:wnst::c9 k) 3 :undefined 0))

;; row 7's two-bound-var predicate.
(:wat::rete::core::defn :wnst::twoarg [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::core::i64::> (:wat::rete::core::i64::+ a b :undefined 0) 113))

;; row 8's outer fn — its ARGUMENT (not its body) is where the nested call lives, at the call site.
(:wat::rete::core::defn :wnst::wrap [v <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::core::i64::= 0 (:wat::rete::core::i64::- v (:wat::rete::core::i64::* (:wat::rete::core::i64::/ v 4 :undefined 0) 4 :undefined 0) :undefined 0)))

;; row 9's diamond — f calls g and h; g and h BOTH call hub. Not a straight chain: TWO fns share one
;; callee, exercising `classify_fn`'s `seen`-set on a shared dependency rather than a linear back-edge.
(:wat::rete::core::defn :wnst::hub [k <- :wat::core::i64] -> :wat::core::i64
  (:wat::rete::core::i64::- k (:wat::rete::core::i64::* (:wat::rete::core::i64::/ k 17 :undefined 0) 17 :undefined 0) :undefined 0))
(:wat::rete::core::defn :wnst::g [k <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ (:wnst::hub k) 2 :undefined 0))
(:wat::rete::core::defn :wnst::h [k <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::* (:wnst::hub k) 2 :undefined 1000000))
(:wat::rete::core::defn :wnst::f [k <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::core::and
    (:wat::rete::core::i64::> (:wnst::g k) 5)
    (:wat::rete::core::i64::< (:wnst::h k) 25)))

;; rows 10/11's shared int-returning helper, and the bool wrapper that calls it.
(:wat::rete::core::defn :wnst::score [k <- :wat::core::i64] -> :wat::core::i64
  (:wat::rete::core::let [v (:wat::rete::core::i64::* k 3 :undefined 0)]
    (:wat::rete::core::i64::- v (:wat::rete::core::i64::* (:wat::rete::core::i64::/ v 11 :undefined 0) 11 :undefined 0) :undefined 0)))
(:wat::rete::core::defn :wnst::is-good [k <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::core::let [sc (:wnst::score k)]
    (:wat::rete::core::i64::= 0 (:wat::rete::core::i64::- sc (:wat::rete::core::i64::* (:wat::rete::core::i64::/ sc 2 :undefined 0) 2 :undefined 0) :undefined 1))))

;; ROW 1 — depth-2 chain. c2(k) = c1(k)+3, c1(k) = k mod 13. c2 > 10 <=> c1 in {8..12} -> 75 of 200.
(:wat::rete::defrule :wnst::depth2
  :when
  [(:wnst::Req (?k <- :k) (?m <- :m)) (:wat::rete::where (:wat::rete::core::i64::> (:wnst::c2 ?k) 10))]
  :then
  [(:wnst::Hit ?k)])

;; ROW 2 — depth-3 chain. c3 > 15 <=> c1 in {10,11,12} -> 45 of 200.
(:wat::rete::defrule :wnst::depth3
  :when
  [(:wnst::Req (?k <- :k) (?m <- :m)) (:wat::rete::where (:wat::rete::core::i64::> (:wnst::c3 ?k) 15))]
  :then
  [(:wnst::Hit ?k)])

;; ROW 3 — depth-4 chain. c4 > 15 <=> c1 in {7..12} -> 90 of 200.
(:wat::rete::defrule :wnst::depth4
  :when
  [(:wnst::Req (?k <- :k) (?m <- :m)) (:wat::rete::where (:wat::rete::core::i64::> (:wnst::c4 ?k) 15))]
  :then
  [(:wnst::Hit ?k)])

;; ROW 4 — depth-5 chain. c5 > 20 <=> c1 in {9,10,11,12} -> 60 of 200.
(:wat::rete::defrule :wnst::depth5
  :when
  [(:wnst::Req (?k <- :k) (?m <- :m)) (:wat::rete::where (:wat::rete::core::i64::> (:wnst::c5 ?k) 20))]
  :then
  [(:wnst::Hit ?k)])

;; ROW 5 — depth-10 chain, the "keeps going past 5" witness kept live in the gate.
;; c10 = c1 + 27. c10 > 32 <=> c1 in {6..12} -> 105 of 200.
(:wat::rete::defrule :wnst::depth10
  :when
  [(:wnst::Req (?k <- :k) (?m <- :m)) (:wat::rete::where (:wat::rete::core::i64::> (:wnst::c10 ?k) 32))]
  :then
  [(:wnst::Hit ?k)])

;; ROW 6 — deep INLINE expression tree, 6 arithmetic levels, ZERO fn calls: the pure-structural-depth
;; reference, separate from call depth. g1=k+10 g2=g1*2 g3=g2-15 g4=g3/3 g5=g4+7 g6=g5*2 -> g6>157
;; selects 94 of 200 (all intermediate values stay non-negative by construction, so wat's truncating
;; `i64::/` and Clojure's `quot` agree without a sign subtlety to reason about).
(:wat::rete::defrule :wnst::inline-tree
  :when
  [(:wnst::Req (?k <- :k) (?m <- :m)) (:wat::rete::where
                                (:wat::rete::core::i64::>
                                  (:wat::rete::core::i64::*
                                    (:wat::rete::core::i64::+
                                      (:wat::rete::core::i64::/
                                        (:wat::rete::core::i64::-
                                          (:wat::rete::core::i64::* (:wat::rete::core::i64::+ ?k 10 :undefined 0) 2 :undefined 0)
                                          15 :undefined 0)
                                        3 :undefined 0)
                                      7 :undefined 0)
                                    2 :undefined 0)
                                  157))]
  :then
  [(:wnst::Hit ?k)])

;; ROW 7 — TWO bound vars, both live at test time. twoarg(?k, ?m) = (?k+?m) > 113 -> 108 of 200.
(:wat::rete::defrule :wnst::two-arg
  :when
  [(:wnst::Req (?k <- :k) (?m <- :m)) (:wat::rete::where (:wnst::twoarg ?k ?m))]
  :then
  [(:wnst::Hit ?k)])

;; ROW 8 — the where-clause's OWN top-level call takes a call to a DIFFERENT pure fn as its argument:
;; (wrap (c2 ?k)). wrap(v) = v mod 4 == 0. c2(k) = c1(k)+3 in {4,8,12} <=> c1 in {1,5,9} -> 46 of 200.
(:wat::rete::defrule :wnst::arg-is-call
  :when
  [(:wnst::Req (?k <- :k) (?m <- :m)) (:wat::rete::where (:wnst::wrap (:wnst::c2 ?k)))]
  :then
  [(:wnst::Hit ?k)])

;; ROW 9 — mutual/chained helpers: f calls g AND h, both of which call the shared hub.
;; hub in {4..12} (mod 17) -> 108 of 200.
(:wat::rete::defrule :wnst::diamond
  :when
  [(:wnst::Req (?k <- :k) (?m <- :m)) (:wat::rete::where (:wnst::f ?k))]
  :then
  [(:wnst::Hit ?k)])

;; ROW 10 — a user fn returning a NON-bool (i64), compared from OUTSIDE the call.
;; score(k) = (3k) mod 11; score > 6 -> 72 of 200.
(:wat::rete::defrule :wnst::int-then-compare
  :when
  [(:wnst::Req (?k <- :k) (?m <- :m)) (:wat::rete::where (:wat::rete::core::i64::> (:wnst::score ?k) 6))]
  :then
  [(:wnst::Hit ?k)])

;; ROW 11 — the CONTRAST with row 10: a user fn returning bool DIRECTLY (internally calling score),
;; used bare in the where clause with no external comparison. is-good(k) = score(k) even -> 109 of 200.
(:wat::rete::defrule :wnst::bool-direct
  :when
  [(:wnst::Req (?k <- :k) (?m <- :m)) (:wat::rete::where (:wnst::is-good ?k))]
  :then
  [(:wnst::Hit ?k)])

(:wat::rete::defquery :wnst::q-Hit
  :params []
  :when [(?fact <- :wnst::Hit)])


;; build-rules row — THE ROW DISPATCH. An unknown row is a located failure, never a silent fallback.
(:wat::core::defn :wnst::build-rules [row <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1)  (:wnst::depth2))
      ((:wat::core::= row 2)  (:wnst::depth3))
      ((:wat::core::= row 3)  (:wnst::depth4))
      ((:wat::core::= row 4)  (:wnst::depth5))
      ((:wat::core::= row 5)  (:wnst::depth10))
      ((:wat::core::= row 6)  (:wnst::inline-tree))
      ((:wat::core::= row 7)  (:wnst::two-arg))
      ((:wat::core::= row 8)  (:wnst::arg-is-call))
      ((:wat::core::= row 9)  (:wnst::diamond))
      ((:wat::core::= row 10) (:wnst::int-then-compare))
      ((:wat::core::= row 11) (:wnst::bool-direct))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::core::String/concat "where-nesting: unknown row " (:wat::core::i64::to-string row))
          :wat::core::None :wat::core::None)))))

;; seed session items — stage Req(i) for i in [0, items). Both fields are FORMULAS over i (rule 3):
;;   k(i) = i
;;   m(i) = (7i + 11) mod 40
(:wat::core::defn :wnst::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::let [mraw (:wat::core::i64::+ (:wat::core::i64::* 7 i) 11)
                          m    (:wat::core::i64::- mraw (:wat::core::i64::* (:wat::core::i64::/ mraw 40) 40))]
          (:wat::core::PersistentVector/conj acc (:wnst::Req :k i :m m))))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

;; derived-ints fired — every derived Hit's key k, sorted ascending. THE accuracy witness.
(:wat::core::defn :wnst::derived-ints
  [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:wnst::Hit/k f)))
        (:wat::rete::query fired (:wnst::q-Hit))))))

;; render-ints — " 3 13 23 …". A plain space-joined rendering, NOT the EDN printer — see
;; where-shapes.wat's render-ints for why (byte-identical rendering across engines).
(:wat::core::defn :wnst::render-ints [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  x <- :wat::core::i64] -> :wat::core::String
      (:wat::core::String/concat acc
        (:wat::core::String/concat " " (:wat::core::i64::to-string x))))
    ""
    v))

;; run-row row -> the corpus line for ONE shape, in its OWN session (mirrors where-shapes.wat exactly:
;; per-row isolation, so a divergence names the row that caused it).
;; rule-display-name — TOTAL derivation of the printed row label from a Rule/name that may
;; now carry this file's namespace prefix (e.g. "NS::arith") after the namespacing wall.
;; `string::split` on "::" always returns >= 1 segment (the whole string, unsplit, when
;; "::" is absent); folding with SEED = full while always overwriting the accumulator
;; with the current segment lands on the LAST segment without ever calling a partial
;; verb (`first`/`nth`/`Option/expect`) — the seed also makes the no-"::" case return
;; the input UNCHANGED, and even an impossible empty split falls back to the seed
;; instead of raising.
(:wat::core::defn :wnst::rule-display-name
  [full <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  seg <- :wat::core::String] -> :wat::core::String seg)
    full
    (:wat::core::string::split full "::")))

(:wat::core::defn :wnst::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let [rules   (:wnst::build-rules row)
                    rule    (:wat::core::first rules)
                    staged  (:wnst::seed (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wnst::q-Hit))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))) (:wnst::items))
                    fired   (:wat::core::match (:wat::rete::fire-rules staged) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                    derived (:wnst::derived-ints fired)
                    n       (:wat::core::Vector/length derived)]
    (:wat::core::String/concat
      (:wat::core::String/concat
        (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
        (:wat::core::String/concat " " (:wnst::rule-display-name (:wat::rete::Rule/name rule))))
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))
        (:wat::core::String/concat " ->" (:wnst::render-ints derived))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wnst::run-row row)))
    nil
    (:wat::core::range 1 (:wat::core::i64::+ (:wnst::row-count) 1))))
