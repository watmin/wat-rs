;; wat-scripts/perf/grid/where-control.wat — THE `where`-CLAUSE EXPRESSIVITY CORPUS,
;; CONTROL-FORM family, wat side.
;;
;; Sibling of where-shapes.wat / where-boolean.wat (read where-shapes.wat's header first — same
;; verdict shape, same four rules, same harness). THE QUESTION this pair asks is the one with the
;; biggest consequences for the compiled-`where` executor (task #49a): is a `where` an EXPRESSION
;; SLOT that admits the language's control forms (`if`/`let`/`cond`/`match`), or a restricted
;; predicate grammar that only accepts comparisons and boolean combinators?
;;
;; ── HOW IT RUNS ───────────────────────────────────────────────────────────────────────────────
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-control.wat   > /tmp/ours
;;     clojure -Sdeps '…'  -M  wat-scripts/perf/grid/where-control.clj > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs        # empty  ⇒  every row agrees
;;
;; `check-where-shapes.sh where-control` is that, wrapped.
;;
;; ── WHAT GROUNDED THIS CORPUS (read `src/rete/purity.rs` before disputing a row) ────────────────
;;
;; The compile-time fence a `where` expr must clear is TWO axes — `pure?` ∧ `deterministic?`
;; (`src/rete/purity.rs`, `classify_expr`). Structurally: `if`/`let`/`do` are plain entries in
;; `intrinsic_meta`'s pure∧det list (their sub-items recursed element-wise); `cond`/`match` get
;; dedicated CLAUSE-AWARE arms (every clause test/body, or every arm's scrutinee/body, must itself
;; satisfy the axis). So there is no separate "control form" deny-list structurally — only the
;; per-leaf-head deny-by-default. And `if`, `let`, and `match` ALL land in the corpus below,
;; unmodified, first try. `cond` does NOT, and for a reason the fence's own structure does not
;; predict — see STOP-1 below, the headline of this family.
;;
;; ── STOP-1 #1 — `cond` PASSES THE PURITY FENCE AND STILL CANNOT BE USED IN A `where` ─────────────
;;
;; `:wat::core::cond` is a **macro** (`wat/core.wat:1237`, `defmacro`), not a runtime primitive —
;; unlike `if`/`let`/`do`/`match`, which are dispatched directly in `runtime.rs`'s `eval_inner`
;; (`":wat::core::if" => eval_if`, `":wat::core::match" => eval_match`, etc. — no macro layer).
;; `classify_expr`'s clause-aware `cond` arm structurally APPROVES it (walked the exact form below
;; through the fence in isolation and it clears `pure?`∧`deterministic?` cleanly) — but a `where`'s
;; expr is captured as DATA via `quasiquote`, stored unevaluated in a `TestNode`, and only actually
;; evaluated later by `eval-test` -> `eval_inner` at fire time. Macro expansion never runs on that
;; stored AST. So the exact form
;;     (:wat::rete::where (:wat::core::cond ((:wat::core::= ?a 0) true) (:else false)))
;; compiles clean (both `--check` and `:wat::rete::compile`'s purity fence), then FAILS THE FIRST
;; TIME the rule actually fires, with (verbatim, from an isolated /tmp probe against the exact form
;; above):
;;     #wat.runtime/UnknownFunction {:message "unknown function: :wat::core::cond" ...}
;; That is a genuine fence/execution split, not a capability boundary the compiler can shrug off:
;; the fence says yes, the engine says no, and the "no" only shows up at first fire, not at rule
;; build time. Rows 3 and 9 below are the SAME two predicates originally written with `cond`,
;; respelled with chained/nested `if` once this was found — landed side by side with the finding so
;; the branching LOGIC is proven to compile; only `cond`'s macro-ness is the wall.
;;
;; ── STOP-1 #2 — Option/Result: FIELD ACCESS composes, CONSTRUCTION does not ─────────────────────
;;
;; `Option/expect` / `Result/expect` are explicitly, deliberately UNCLASSIFIED in `intrinsic_meta`
;; (purity.rs's own doc: "total but they raise") — confirmed with an isolated /tmp probe of exactly
;;     (:wat::rete::where (:wat::core::i64::> (:wat::core::Option/expect ?o "missing") 0))
;; which panics at RULE-COMPILE time (not `--check`, which reports clean), verbatim:
;;     compile-condition: where expr must be pure and deterministic   (wat/rete.wat:566)
;; `Result/expect` fails identically on the analogous form. Expected, and documented.
;;
;; The SURPRISE: this is not only an `expect`-shaped hole. `(:wat::core::Some x)` /
;; `:wat::core::None` / `(:wat::core::Ok x)` / `(:wat::core::Err x)` — the bare CONSTRUCTORS,
;; total, no raise, no IO — are ALSO unclassified, and fail the SAME `compile-condition` panic
;; the instant one appears anywhere reachable from a `where` (isolated /tmp probes: a `where`
;; calling a user fn `(if (> x 0) (Some x) None)`; a `where` matching a user fn that internally
;; builds `(Ok ...)`/`(Err ...)`). The reason: `constructor_meta` (purity.rs) derives a
;; constructor's purity from the frozen `TypeEnv` — it works for every user `defrecord`/`defenum`
;; because those register there with a declared `:wat::enum::Pure`/`Nature` marker. `Option` and
;; `Result` are NOT registered in the `TypeEnv` the same way (they are checker-special-cased
;; built-ins, `src/check.rs` `BARE_CONTAINER_HEADS`) — so `constructor_meta` returns `None` for
;; `:wat::core::Some`/`:wat::core::Ok`/etc., they fall through to `intrinsic_meta`, and NEITHER is
;; in that hand-list. Reading a field of Option/Result type and MATCHING it (never constructing) is
;; completely fine — row 7 below matches a bound `?o : (Option :- [i64])` with zero friction, and an
;; isolated /tmp probe confirmed the identical shape for `Result` (`match` on a bound `Result`
;; field, no `expect`, reached `n=[3]` correctly). The wall is specifically CONSTRUCTING one of the
;; two most common built-in sum types anywhere a `where` can reach — which reads as a real gap in
;; the purity registry (arc 255's eventual target), not a considered capability boundary: nobody
;; would design "you may read an Option but never build one" on purpose.
;;
;; ── THE FOUR RULES (same as where-shapes.wat; restated for this family) ─────────────────────────
;; 1. THE SHARED CONDITION BINDS EVERY FIELD (?k ?a ?b ?n ?o), identical in every rule.
;; 2. EVERY ROW MUST DISCRIMINATE A PROPER SUBSET — 0 < n < items — checked against the comment.
;; 3. SEED FROM A FORMULA OVER `i`, never a table.
;; 4. MIRROR THE OPERATION, do not idiomatise it — same arithmetic on both sides.
;;
;; ── THE FACT STREAM ───────────────────────────────────────────────────────────────────────────
;; items = 180 = 4*9*5, so mod-4/mod-9/mod-5-derived counts land on clean integers.
;;   a(i) = i mod 4              — an i64 used by `if`/`cond` arms
;;   b(i) = i mod 9               — a second i64, independent modulus, for cross-field control rows
;;   n(i) = i mod 6 == 0          — a bool field
;;   o(i) = Some(i) if i mod 3 != 0 else None   — the Option-typed field for rows 7-8

(:wat::core::defn :wsc::items [] -> :wat::core::i64 180)

(:wat::core::defn :wsc::row-count [] -> :wat::core::i64 9)

(:wat::core::defrecord :wsc::Req
  [k <- :wat::core::i64
   a <- :wat::core::i64
   b <- :wat::core::i64
   n <- :wat::core::bool
   o <- (:wat::core::Option :- [:wat::core::i64])])

(:wat::core::defrecord :wsc::Hit [k <- :wat::core::i64])

;; row 5's pure fn used inside a `let` — bump(x) := x + 1, a trivial CSE target so the row measures
;; whether a `let` binding a CALL (not a bare accessor) composes, and is used twice.
(:wat::rete::core::defn :wsc::bump [x <- :wat::core::i64] -> :wat::core::i64 (:wat::rete::core::i64::+ x 1 :undefined 0))

;; THE SHARED LEADING CONDITION, quoted once and reused by every row — only `where-c` varies.
(:wat::core::defn :wsc::conds [] -> :wat::WatAST
  (:wat::core::quasiquote (:wsc::Req (?k <- :k) (?a <- :a) (?b <- :b) (?n <- :n) (?o <- :o))))

(:wat::core::defn :wsc::ins [] -> :wat::WatAST
  (:wat::core::quasiquote (:wsc::Hit ?k)))

;; ROW 1 — `if` returning a bool, as the WHOLE predicate.
;; Hit :- Req(…) AND (if n (a > 1) (a < 2)).  n holds for i mod 6==0 (30/180); of those, a=i mod 4>1
;; means i mod 4 in {2,3} -- for i mod 6==0, i mod 4 cycles 0,2,0,2,... over the 30 multiples of 6
;; in [0,180): i=6j, j=0..29, i mod 4 = (6j) mod 4 = (2j) mod 4, which is 0 when j even, 2 when j
;; odd -> 15 have a=2 (>1, true), 15 have a=0 (false) => 15 pass on the n-branch.
;; n false (150/180): a < 2 means i mod 4 in {0,1}. Among the 150 non-multiples-of-6, i mod 4 in
;; {0,1} for 90 of the 180 total (half), minus the 15 multiples-of-6 with a=0 counted above =>
;; 90 - 15 = 75 pass on the not-n branch.  Total: 15 + 75 = 90/180.
(:wat::rete::defrule :wsc::if-whole
  :when
  [(:wsc::Req (?k <- :k) (?a <- :a) (?b <- :b) (?n <- :n) (?o <- :o)) (:wat::rete::where
                                 (:wat::rete::core::if ?n
                                   (:wat::rete::core::i64::> ?a 1)
                                   (:wat::rete::core::i64::< ?a 2)))]
  :then
  [(:wsc::Hit ?k)])

;; ROW 2 — `if` NESTED inside a comparison: the `if` returns an i64, then that i64 is compared.
;; Hit :- Req(…) AND ((if n a b) > 4).  n true (i mod 6==0, 30 facts): compares a=i mod 4, always
;; < 4, so 0 pass. n false (150 facts): compares b=i mod 9 > 4, i.e. b in {5,6,7,8} -- 4/9 of ALL
;; 180 facts (80) satisfy that, minus the ones that are also n=true (i mod 6==0 AND i mod 9 in
;; {5,6,7,8}): over one lcm(6,9)=18 period the three i mod 6==0 points are i mod 9 in {0,6,3}, of
;; which only 6 qualifies -- 1 of every 18 -- so 10 across [0,180). 80 - 10 = 70/180.
(:wat::rete::defrule :wsc::if-nested-cmp
  :when
  [(:wsc::Req (?k <- :k) (?a <- :a) (?b <- :b) (?n <- :n) (?o <- :o)) (:wat::rete::where
                                 (:wat::rete::core::i64::>
                                   (:wat::rete::core::if ?n ?a ?b)
                                   4))]
  :then
  [(:wsc::Hit ?k)])

;; ROW 3 — CHAINED `if` as the WHOLE predicate — see the STOP-1 note above the `cond` finding:
;; this is the SAME branching logic originally written with `cond` ((a==0)->true (a==1)->false
;; (a==2)->true :else false), respelled as nested `if` once `cond` itself proved unusable inside a
;; `where`. Landing both shapes side by side is the point: it isolates that the REJECTION is
;; `cond`'s macro-ness, not the branching semantics — chained `if` says the identical thing and
;; compiles clean. a = i mod 4: a==0 -> true (45/180), a==2 -> true (45/180), else -> false.
;; Total: 90/180.
(:wat::rete::defrule :wsc::if-chain-whole
  :when
  [(:wsc::Req (?k <- :k) (?a <- :a) (?b <- :b) (?n <- :n) (?o <- :o)) (:wat::rete::where
                                 (:wat::rete::core::if (:wat::rete::core::i64::= ?a 0)
                                   true
                                   (:wat::rete::core::if (:wat::rete::core::i64::= ?a 1)
                                     false
                                     (:wat::rete::core::if (:wat::rete::core::i64::= ?a 2)
                                       true
                                       false))))]
  :then
  [(:wsc::Hit ?k)])

;; ROW 4 — `let` binding a LOCAL inside the predicate, used TWICE. THE BIG ONE: if `let` is
;; admitted, a compiler must model local scope inside a where; if rejected, that is a hard
;; boundary. Hit :- Req(…) AND (let [s (+ a b)] (and (> s 4) (< s 12))).
;; s = a+b = (i mod 4) + (i mod 9), range [0,12] over the joint period lcm(4,9)=36. Enumerating all
;; 36 residues by hand: 22 of them land 5 <= s <= 11 (the strict-open interval (4,12)); 180/36 = 5
;; identical blocks => 22*5 = 110/180.
(:wat::rete::defrule :wsc::let-twice
  :when
  [(:wsc::Req (?k <- :k) (?a <- :a) (?b <- :b) (?n <- :n) (?o <- :o)) (:wat::rete::where
                                 (:wat::rete::core::let [s (:wat::rete::core::i64::+ ?a ?b :undefined 0)]
                                   (:wat::rete::core::and
                                     (:wat::rete::core::i64::> s 4)
                                     (:wat::rete::core::i64::< s 12))))]
  :then
  [(:wsc::Hit ?k)])

;; ROW 5 — a `let` whose bound value is a CALL to a pure fn, used in two places
;; (common-subexpression shape). Hit :- Req(…) AND (let [c (bump a)] (and (> c 1) (< c 5))).
;; c = a+1, a = i mod 4 in {0,1,2,3} => c in {1,2,3,4}; c>1 and c<5 means c in {2,3,4} => a in
;; {1,2,3} => 3 of every 4 residues => 135/180.
(:wat::rete::defrule :wsc::let-call-cse
  :when
  [(:wsc::Req (?k <- :k) (?a <- :a) (?b <- :b) (?n <- :n) (?o <- :o)) (:wat::rete::where
                                 (:wat::rete::core::let [c (:wsc::bump ?a)]
                                   (:wat::rete::core::and
                                     (:wat::rete::core::i64::> c 1)
                                     (:wat::rete::core::i64::< c 5))))]
  :then
  [(:wsc::Hit ?k)])

;; ROW 6 — `match` on an i64-VALUED expr (the language's structural-match, exercised over an
;; ordinary value rather than an enum — the plainest permitted shape). Hit :- Req(…) AND
;; (match a (0 false) (1 true) (2 false) (3 true)).  a=i mod 4: 1 and 3 give true => 2 of 4
;; residues => 90/180.
(:wat::rete::defrule :wsc::match-i64
  :when
  [(:wsc::Req (?k <- :k) (?a <- :a) (?b <- :b) (?n <- :n) (?o <- :o)) (:wat::rete::where
                                 (:wat::rete::core::match ?a
                                   (0 false)
                                   (1 true)
                                   (2 false)
                                   (3 true)))]
  :then
  [(:wsc::Hit ?k)])

;; ROW 7 — Option handling via `match` (no raising verb). Hit :- Req(…) AND
;; (match o ((Some v) (> v 90)) (None false)).  o = Some(i) when i mod 3 != 0 (120/180), else None.
;; Of those 120, v=i>90 holds for i in [91,179] AND i mod 3 != 0: [91,179] has 89 integers, of
;; which 29 are multiples of 3 (93..177 step 3) => 89 - 29 = 60/180.
(:wat::rete::defrule :wsc::option-match
  :when
  [(:wsc::Req (?k <- :k) (?a <- :a) (?b <- :b) (?n <- :n) (?o <- :o)) (:wat::rete::where
                                 (:wat::rete::core::match ?o
                                   ((:wat::core::Some v) (:wat::rete::core::i64::> v 90))
                                   (:wat::core::None false)))]
  :then
  [(:wsc::Hit ?k)])

;; ROW 8 — a NESTED `if` inside a `let` inside a boolean composition — the DEEP control shape.
;; Hit :- Req(…) AND (let [s (+ a b)] (and n (if (> s 6) true (< s 3)))).
;; n holds for i mod 6==0 (30/180), 6 distinct s-values per lcm(4,9,6)=36 block (i=0,6,12,18,24,30
;; give s=0,8,3,2,6,5). Per block: s>6 true at i=6 (s=8); the else-arm s<3 true at i=0 (s=0) and
;; i=18 (s=2); i=12(3),24(6),30(5) fail both => 3 of 6 per block * 5 blocks (180/36) = 15/180.
(:wat::rete::defrule :wsc::deep-nest
  :when
  [(:wsc::Req (?k <- :k) (?a <- :a) (?b <- :b) (?n <- :n) (?o <- :o)) (:wat::rete::where
                                 (:wat::rete::core::let [s (:wat::rete::core::i64::+ ?a ?b :undefined 3)]
                                   (:wat::rete::core::and
                                     ?n
                                     (:wat::rete::core::if (:wat::rete::core::i64::> s 6)
                                       true
                                       (:wat::rete::core::i64::< s 3)))))]
  :then
  [(:wsc::Hit ?k)])

;; ROW 9 — the `cond`-shaped branch-with-a-`let`-arm, respelled with `if` for the same STOP-1
;; reason as row 3: Hit :- Req(…) AND (if n (let [s (+ a b)] (> s 8)) (< b 3)).
;; n holds for i mod 6==0 (30/180) -> gate on s=a+b>8 among those 30: from row 8's 6 s-values per
;; 36-block (0,8,3,2,6,5 at i=0,6,12,18,24,30), NONE exceed 8, so this branch contributes 0.
;; !n (150/180) -> gate on b=i mod 9<3 (b in {0,1,2}, 60/180 overall); of the 30 n=true facts, 2
;; per 36-block (i=0,18) also have b in {0,1,2}, so 10 must be subtracted from the 60: 60-10=50.
;; Total: 0 + 50 = 50/180.
(:wat::rete::defrule :wsc::if-let-arm
  :when
  [(:wsc::Req (?k <- :k) (?a <- :a) (?b <- :b) (?n <- :n) (?o <- :o)) (:wat::rete::where
                                 (:wat::rete::core::if ?n
                                   (:wat::rete::core::let [s (:wat::rete::core::i64::+ ?a ?b :undefined 0)]
                                     (:wat::rete::core::i64::> s 8))
                                   (:wat::rete::core::i64::< ?b 3)))]
  :then
  [(:wsc::Hit ?k)])

(:wat::rete::defquery :wsc::q-Hit
  :params []
  :when [(?fact <- :wsc::Hit)])


;; build-rules — THE ROW DISPATCH. An unknown row is a located failure, never a silent fallback.
(:wat::core::defn :wsc::build-rules [row <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1) (:wsc::if-whole))
      ((:wat::core::= row 2) (:wsc::if-nested-cmp))
      ((:wat::core::= row 3) (:wsc::if-chain-whole))
      ((:wat::core::= row 4) (:wsc::let-twice))
      ((:wat::core::= row 5) (:wsc::let-call-cse))
      ((:wat::core::= row 6) (:wsc::match-i64))
      ((:wat::core::= row 7) (:wsc::option-match))
      ((:wat::core::= row 8) (:wsc::deep-nest))
      ((:wat::core::= row 9) (:wsc::if-let-arm))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::core::String/concat "where-control: unknown row " (:wat::core::i64::to-string row))
          :wat::core::None :wat::core::None)))))

;; seed — stage Req(i) for i in [0, items) via the BATCH verb (one rebuild). Every field is a
;; FORMULA over i, independently computable on the Clara side so nothing rots as a hand-kept table.
(:wat::core::defn :wsc::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::let [a       (:wat::core::i64::- i (:wat::core::i64::* (:wat::core::i64::/ i 4) 4))
                          b       (:wat::core::i64::- i (:wat::core::i64::* (:wat::core::i64::/ i 9) 9))
                          n       (:wat::core::= 0 (:wat::core::i64::- i (:wat::core::i64::* (:wat::core::i64::/ i 6) 6)))
                          is-mult3 (:wat::core::= 0 (:wat::core::i64::- i (:wat::core::i64::* (:wat::core::i64::/ i 3) 3)))
                          o       (:wat::core::if is-mult3 :wat::core::None (:wat::core::Some i))]
          (:wat::core::PersistentVector/conj acc
            (:wsc::Req :k i :a a :b b :n n :o o))))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

;; derived-ints fired — every derived Hit's key k, sorted ascending. THE accuracy witness.
(:wat::core::defn :wsc::derived-ints
  [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort
    (:wat::core::into (:wat::core::Vector :wat::core::i64)
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact")] (:wsc::Hit/k f)))
        (:wat::rete::query fired (:wsc::q-Hit))))))

;; render-ints — " 3 13 23 …". A plain space-joined rendering, NOT the EDN printer — see
;; where-shapes.wat's identical helper for why this must not be `:wat::edn::write`.
(:wat::core::defn :wsc::render-ints [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  x <- :wat::core::i64] -> :wat::core::String
      (:wat::core::String/concat acc
        (:wat::core::String/concat " " (:wat::core::i64::to-string x))))
    ""
    v))

;; run-row row -> the corpus line for ONE shape, in its OWN session.
;; rule-display-name — TOTAL derivation of the printed row label from a Rule/name that may
;; now carry this file's namespace prefix (e.g. "NS::arith") after the namespacing wall.
;; `string::split` on "::" always returns >= 1 segment (the whole string, unsplit, when
;; "::" is absent); folding with SEED = full while always overwriting the accumulator
;; with the current segment lands on the LAST segment without ever calling a partial
;; verb (`first`/`nth`/`Option/expect`) — the seed also makes the no-"::" case return
;; the input UNCHANGED, and even an impossible empty split falls back to the seed
;; instead of raising.
(:wat::core::defn :wsc::rule-display-name
  [full <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  seg <- :wat::core::String] -> :wat::core::String seg)
    full
    (:wat::core::string::split full "::")))

(:wat::core::defn :wsc::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let [rules   (:wsc::build-rules row)
                    rule    (:wat::core::first rules)
                    staged  (:wsc::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wsc::q-Hit))) (:wsc::items))
                    fired   (:wat::rete::fire-rules staged)
                    derived (:wsc::derived-ints fired)
                    n       (:wat::core::Vector/length derived)]
    (:wat::core::String/concat
      (:wat::core::String/concat
        (:wat::core::String/concat "row " (:wat::core::i64::to-string row))
        (:wat::core::String/concat " " (:wsc::rule-display-name (:wat::rete::Rule/name rule))))
      (:wat::core::String/concat
        (:wat::core::String/concat " n=" (:wat::core::i64::to-string n))
        (:wat::core::String/concat " ->" (:wsc::render-ints derived))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wsc::run-row row)))
    nil
    (:wat::core::range 1 (:wat::core::i64::+ (:wsc::row-count) 1))))
