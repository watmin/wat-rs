;; wat-scripts/fixes/mandatory-typed-quasiquote-residual.wat — arc 109 Stone 3 (THE WALL) finding.
;;
;; `wat-scripts/fixes/one-param-spec.wat` (Stone 1) required every would-be type-arg slot to be
;; TYPE-SHAPED (`:user::type-shaped-elem?`: a literal keyword, or a compound list whose OWN head
;; names a recognised parametric) before rewriting a bare head into `:- [...]` — deliberately
;; conservative, because a handful of heads (PersistentVector / PersistentMap / Option / Result)
;; legitimately allow "no type spec at all, values only, T inferred". That gate has a blind spot
;; it was never built to see: a QUASIQUOTE TEMPLATE's unquoted reference (`~name`) desugars to
;; `(:wat::core::unquote name)` — a List node whose head keyword is `:wat::core::unquote`, which
;; names neither arity source, so `type-shaped-elem?` calls it "not type-shaped" and `classify`
;; lands on kind 0 (not-a-match) — SILENTLY, not even a "SKIP" report (kinds 3/5/6/7 report;
;; kind 0 does not, by design — it means "this list isn't a param-spec call at all", which is
;; wrong here, not merely unreported).
;;
;; Found by Stone 3's wall (`docs/arc/2026/04/109-kill-std/BRIEF-STONE-one-param-spec-the-wall.md`):
;; the first post-wall floor's cascade traced to exactly THREE source sites, all in
;; `wat/service.wat`'s `defservice` macro template (each an "empty vector of a declared element
;; type" — `(:wat::core::Vector ~ty)`, zero values, one unquote-wrapped type arg):
;;
;;   peers-only-expr   (line ~1307)  (:wat::core::Vector ~selectable-peer-ty)
;;   arm-fn's conj     (line ~1576)  (:wat::core::Vector ~selectable-peer-ty)
;;   apply's 0-arg call(line ~2475)  (:wat::core::Vector ~selectable-entry-ty)
;;
;; Each expands once per `defservice` consumer, so the SAME three template bugs cascaded into
;; 118 checker errors across every `defservice`-based file in the bundled stdlib (journal.wat,
;; span.wat, mem.wat, sqlite-store.wat, cache.wat, stdio.wat) — the "large fail count is the
;; progress meter, not a crisis" the brief predicts, traced to its actual (tiny) root.
;;
;; ★ WHY THIS IS SAFE WITHOUT `type-shaped-elem?`'S GATE: unlike PersistentVector / Option /
;; PersistentMap / Result, `:wat::core::Vector` / `:wat::core::HashMap` / `:wat::core::HashSet`
;; are the substrate's three MANDATORY-TYPED constructors — `src/check.rs`'s
;; `infer_list_constructor` / `infer_hashmap_constructor` / `infer_hashset_constructor` require
;; the leading param-spec unconditionally; there is no "no spec, values only" reading for them at
;; ANY arity. So for exactly these three heads, `args.length == declared-arity` is unambiguous
;; evidence of an attempted param-spec regardless of whether the slot is keyword-shaped — the
;; general codemod's type-shape gate was always more conservative than these three heads need.
;; This codemod does NOT replace `one-param-spec.wat` (which still owns the general, type-shape-
;; gated case for every other parametric); it is the narrow follow-up for the shape that gate
;; structurally cannot see: an unquote-wrapped (`(:wat::core::unquote name)` /
;; `(:wat::core::unquote-splicing name)`) reference in the type-arg slot(s).
;;
;; ── mechanics — PURE INSERTION, identical discipline to `one-param-spec.wat` ────────────────
;; Two insertions per matched site: `":- ["` immediately before the first type-arg's own span,
;; `"]"` immediately after the last type-arg's own span. Recursion is total (independent of
;; whether a node itself matched), so a nested match inside a matched or non-matched node is
;; still found in the same pass.
;;
;; ── usage — R21: dry-run on a /tmp copy FIRST, diff it, THEN apply to the corpus ────────────
;; One EDN vector of paths on stdin (this codemod needs no cross-file arity table — the three
;; heads' arities are substrate constants, not corpus-declared):
;;
;;   printf '["/tmp/pilot.wat"]\n' | cargo wat ./wat-scripts/fixes/mandatory-typed-quasiquote-residual.wat
;;   diff original.wat /tmp/pilot.wat
;;
;; Idempotent: a site already `:-`-marked has `args[0]` as the literal `:-` keyword — a
;; `WatAST::Keyword`, never unquote-wrapped — so `residual-match?`'s prefix check
;; (`:user::type-prefix-unquoted?`) fails on the very first element regardless of arity or
;; trailing-value count. See `:user::residual-arity` / `:user::residual-match?`.

;; ── span/offset helpers (identical shape to `one-param-spec.wat`) ───────────────────────────
(:wat::core::defn :user::start-off
  [n <- :wat::WatAST lines <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))
(:wat::core::defn :user::end-off
  [n <- :wat::WatAST lines <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))

;; residual-arity — declared arity for exactly the three MANDATORY-TYPED heads; -1 for anything
;; else (never guessed).
(:wat::core::defn :user::residual-arity [hn <- :wat::core::String] -> :wat::core::i64
  (:wat::core::if (:wat::core::= hn ":wat::core::Vector") 1
    (:wat::core::if (:wat::core::= hn ":wat::core::HashSet") 1
      (:wat::core::if (:wat::core::= hn ":wat::core::HashMap") 2
        -1))))

;; unquote-wrapped? — true only for `(:wat::core::unquote X)` / `(:wat::core::unquote-splicing X)`
;; — the exact shape `type-shaped-elem?` cannot classify. Deliberately NOT "any List" — a
;; genuine nested type reference (`(:pilot::Entry ...)`) is `one-param-spec.wat`'s business, not
;; this codemod's; this one only closes the unquote gap.
(:wat::core::defn :user::unquote-wrapped?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::let [h (:wat::core::first ch)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
            (:wat::core::if (:wat::core::= (:wat::core::ast-name h) ":wat::core::unquote") true
              (:wat::core::= (:wat::core::ast-name h) ":wat::core::unquote-splicing"))
            false))))
    false))

;; close-target — the node whose END SPAN correctly bounds the text to wrap. Measured live
;; (`wat/service.wat`, all three sites): `(:wat::core::unquote name)`'s OWN span covers only the
;; `~` reader-macro CHARACTER itself (one column wide — `~sel` at col 77, ending col 78) — the
;; wrapped symbol `name` carries its own, separately-tracked span for the REST of the token
;; (`sele…er-ty` at col 78, ending col 96, right where the enclosing call's own closing paren
;; begins). So the unquote node's start is the right OPEN-bracket anchor (`~` IS where the
;; type-arg region begins) but its end is NOT the right CLOSE-bracket anchor — using it verbatim
;; put the closing `]` one character after the `~`, before the symbol text, corrupting
;; `~selectable-peer-ty` into `~]selectable-peer-ty` on the dry-run pilot. The wrapped child's
;; own end-span is the one that reaches the true end of the reference.
(:wat::core::defn :user::close-target [node <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::if (:user::unquote-wrapped? node)
    (:wat::core::nth (:wat::core::ast->children node) 1)
    node))

;; type-prefix-unquoted? — the first `n` elements of `args` are ALL unquote-wrapped. Checks only
;; the PREFIX (the type slots), never the trailing value args — found live (`wat/bracket.wat`):
;; `(:wat::core::Vector ~coords-ty-kw ~coords-sym)` is Vector's type slot (`~coords-ty-kw`)
;; followed by exactly ONE trailing VALUE (`~coords-sym`, itself unquote-wrapped too, but as a
;; VALUE, not evidence of a second type slot) — `all-unquote-wrapped?`'s original all-or-nothing
;; check demanded `args.length == n` and missed this two-arg shape entirely.
(:wat::core::defn :user::take-n
  [items <- (:wat::core::Vector :- [:wat::WatAST]) n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::if (:wat::i64::<= n 0) true (:wat::core::empty? items))
    (:wat::core::Vector :- [:wat::WatAST])
    (:wat::core::concat (:wat::core::Vector :- [:wat::WatAST] (:wat::core::first items))
      (:user::take-n (:wat::core::rest items) (:wat::i64::- n 1)))))

(:wat::core::defn :user::type-prefix-unquoted?
  [args <- (:wat::core::Vector :- [:wat::WatAST]) n <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::if (:wat::i64::<= n 0) true
    (:wat::core::if (:wat::core::empty? args) false
      (:wat::core::if (:user::unquote-wrapped? (:wat::core::first args))
        (:user::type-prefix-unquoted? (:wat::core::rest args) (:wat::i64::- n 1))
        false))))

;; residual-match? — head is one of the three mandatory-typed heads, args.length >= its declared
;; arity, and the FIRST `n` args (the type slots only — trailing values are untouched and may be
;; anything, including further unquote-wrapped symbols) are ALL unquote-wrapped. Excludes an
;; already-`:-`-marked call for free: its args[0] is the literal `:-` keyword, never
;; unquote-wrapped, so `type-prefix-unquoted?` fails on the first element.
(:wat::core::defn :user::residual-match?
  [hn <- :wat::core::String args <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::let [n (:user::residual-arity hn)]
    (:wat::core::if (:wat::i64::< n 0) false
      (:wat::core::if (:wat::i64::< (:wat::core::length args) n) false
        (:user::type-prefix-unquoted? args n)))))

;; ── edit collection — same open/insert-per-arg/close discipline as `one-param-spec.wat`'s
;; `args-edits-split`, called ONLY over the type-arg prefix (`take-n args n`, never the trailing
;; values) — there is nothing nested to find inside an unquote-wrapped type slot for THIS
;; codemod; recursion into trailing value args (which CAN nest a real match) is handled
;; separately by `collect-edits`'s own unconditional `collect-edits-seq ch lines` walk below. ──
(:wat::core::defn :user::args-edits
  [args <- (:wat::core::Vector :- [:wat::WatAST])
   idx  <- :wat::core::i64
   n    <- :wat::core::i64
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? args)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    (:wat::core::let [h      (:wat::core::first args)
                      tl     (:wat::core::into [] (:wat::core::rest args))
                      open-e (:wat::core::if (:wat::core::= idx 0)
                               (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                                 (:wat::core::Tuple (:user::start-off h lines) "" ":- ["))
                               (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))
                      close-e (:wat::core::if (:wat::core::= idx (:wat::i64::- n 1))
                                (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
                                  (:wat::core::Tuple (:user::end-off (:user::close-target h) lines) "" "]"))
                                (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))
                      rest-e (:user::args-edits tl (:wat::i64::+ idx 1) n lines)]
      (:wat::core::concat open-e (:wat::core::concat close-e rest-e)))))

;; collect-edits — walk every List node; residual-match? fires an edit AND still recurses into
;; the head/args (mirrors `one-param-spec.wat`: a matched node can still contain a nested match).
(:wat::core::defn :user::collect-edits
  [node <- :wat::WatAST lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch   (:wat::core::ast->children node)
                       here (:wat::core::if (:wat::core::empty? ch)
                              false
                              (:wat::core::if (:wat::core::= (:wat::core::ast-kind (:wat::core::first ch)) "keyword")
                                (:user::residual-match?
                                  (:wat::core::ast-name (:wat::core::first ch))
                                  (:wat::core::into [] (:wat::core::rest ch)))
                                false))]
      (:wat::core::concat
        (:wat::core::if here
          (:wat::core::let [n (:user::residual-arity (:wat::core::ast-name (:wat::core::first ch)))]
            (:user::args-edits (:user::take-n (:wat::core::into [] (:wat::core::rest ch)) n) 0 n lines))
          (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))
        (:user::collect-edits-seq ch lines)))
    (:wat::core::if (:wat::fix::structural? node)
      (:user::collect-edits-seq (:wat::core::ast->children node) lines)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))))

(:wat::core::defn :user::collect-edits-seq
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    (:wat::core::concat (:user::collect-edits (:wat::core::first items) lines)
                        (:user::collect-edits-seq (:wat::core::rest items) lines))))

;; ── per-file pass ────────────────────────────────────────────────────────────────────────────
(:wat::core::defn :user::parse-forms [src <- :wat::core::String] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::ast->children
    (:wat::core::match (:wat::core::read-string src)
      ((:wat::core::ReadOutcome::Forms __forms) __forms)
      ((:wat::core::ReadOutcome::Malformed __cause)
        (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))

(:wat::core::defn :user::convert [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    forms     (:user::parse-forms src)
                    all-edits (:user::collect-edits-seq forms lines)
                    rev-edits (:wat::core::reverse all-edits)]
    (:wat::fix::fix-text-apply src rev-edits)))

;; ── driver — ONE EDN vector of paths on stdin: read+write targets. ──────────────────────────
(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::convert (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[mandatory-typed-quasiquote-residual] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::read-path-vector [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::match (:wat::kernel::readln)
    ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
    (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
    (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:user::read-path-vector)))
