;; wat-scripts/fixes/angle-brackets-to-binder.wat — arc 109 Stone ③.
;;
;; Rewrites every angle-bracket-shaped type keyword into the `:-` binder / reference spelling:
;;
;;   :wat::cache::Lru<K,V>                 (decl-name role)  -> :wat::cache::Lru :- [K V]
;;   :wat::core::Vector<wat::core::i64>    (reference role)  -> (:wat::core::Vector :- [:wat::core::i64])
;;   :wat::kernel::Peer<Cache::Op<K,V>,Cache::Reply<K,V>>    -> (:wat::kernel::Peer :- [(:Cache::Op :- [K V]) (:Cache::Reply :- [K V])])
;;
;; WHY THIS SCRIPT EXISTS SEPARATELY FROM `parametrics-take-a-type-vector.wat` (its sibling,
;; same role for an EARLIER stone): that script's RENDER step calls
;; `:wat::core::keyword/to-type-form-colon` — the exact runtime intrinsic Arc 109 ③ walls off
;; for angle input (`src/types.rs`'s two parse doors now refuse `Head<args>` outright). The
;; CLAUDE.md STASH-DANCE (temporarily reverting the wall to run the old converter, then
;; restoring it) is the textbook answer to "the codemod's own dependency is what the Rust
;; change retired" — but it is explicitly OUT OF BOUNDS for this stone's rider brief ("Do NOT
;; commit, push, stash or amend"), and no non-git equivalent (swap files, rebuild, swap back)
;; reads as anything but the same maneuver by other means. So this script carries its OWN
;; angle-bracket → binder RENDERER, written as ordinary wat string surgery (find/split/subs/
;; trim — never a call through the walled type-parse doors), self-hosted (wat rewriting wat)
;; the same as every recorded migration, just without the shared converter dependency.
;;
;; Reuses fix.wat's structural walk + text-splice primitives (`structural?`,
;; `type-shaped-keyword?`, `fix-text-offset-of`, `fix-text-apply`) — none of which touch the
;; type parser — and `declarator-head-keyword?`'s exact head-set, copied verbatim from
;; `parametrics-take-a-type-vector.wat` (the DECL-NAME-vs-REFERENCE role split is a property of
;; the LANGUAGE, not of which stone is doing the rewriting).
;;
;; Idempotent: a keyword already in `:- [...]` form contains no `<`/`>` pair, so
;; `type-shaped-keyword?` no longer matches it and a second run is a byte-identical no-op.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB" …]\n' | cargo wat ./wat-scripts/fixes/angle-brackets-to-binder.wat
;;
;; Dry-run on a /tmp copy FIRST (R21 — mandatory before any corpus application):
;;   cp tests/foo.wat /tmp/pilot.wat
;;   printf '["/tmp/pilot.wat"]\n' | cargo wat ./wat-scripts/fixes/angle-brackets-to-binder.wat
;;   diff tests/foo.wat /tmp/pilot.wat

;; ── the angle-bracket → binder renderer (pure string surgery) ───────────────────────────

;; find-first-lt — index of the first '<' in `s` starting at `i`, or -1 if none. A keyword's
;; own parametric suffix is always the ONE `<...>` group at the top of its text (angle
;; brackets never appear elsewhere in a keyword), so the FIRST `<` is always the marker —
;; no top-level-vs-nested ambiguity at this call site (unlike `scan-for-close`/
;; `split-top-level` below, which walk INSIDE that group where nesting is real).
(:wat::core::defn :user::find-first-lt
  [s <- :wat::core::String i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::i64::> i (:wat::core::i64::- (:wat::string::length s) 1))
    -1
    (:wat::core::if (:wat::core::= (:wat::string::subs s i (:wat::core::i64::+ i 1)) "<")
      i
      (:user::find-first-lt s (:wat::core::i64::+ i 1)))))

;; open-bracket? / close-bracket? — bracket-KIND-agnostic depth predicates. `scan-for-close`/
;; `split-top-level` track `<…>` AND `(…)` nesting TOGETHER (one depth counter, either kind
;; opens/closes it) because a tuple's inner content can carry a nested `Head<args>` element
;; (`:(Vector<i64>,String)`) and a parametric's args can, in principle, carry a nested tuple —
;; either way, the two bracket kinds are always properly nested (never crossing) in valid wat
;; type syntax, so one counter is exact.
(:wat::core::defn :user::open-bracket? [c <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= c "<") true (:wat::core::= c "(")))
(:wat::core::defn :user::close-bracket? [c <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= c ">") true (:wat::core::= c ")")))

;; scan-for-close — index of the close bracket matching the open bracket just consumed
;; (depth starts at 1, `i` is the position right after that open bracket).
(:wat::core::defn :user::scan-for-close
  [s <- :wat::core::String i <- :wat::core::i64 depth <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let [c (:wat::string::subs s i (:wat::core::i64::+ i 1))]
    (:wat::core::if (:user::open-bracket? c)
      (:user::scan-for-close s (:wat::core::i64::+ i 1) (:wat::core::i64::+ depth 1))
      (:wat::core::if (:user::close-bracket? c)
        (:wat::core::if (:wat::core::i64::= depth 1)
          i
          (:user::scan-for-close s (:wat::core::i64::+ i 1) (:wat::core::i64::- depth 1)))
        (:user::scan-for-close s (:wat::core::i64::+ i 1) depth)))))

;; split-top-level — split `s` on commas at depth 0 (nested-bracket commas are NOT split
;; points). `i` is the scan cursor, `start` is the pending segment's start, `acc` accumulates
;; trimmed segments in order.
(:wat::core::defn :user::split-top-level
  [s <- :wat::core::String i <- :wat::core::i64 depth <- :wat::core::i64 start <- :wat::core::i64
   acc <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::i64::>= i (:wat::string::length s))
    (:wat::core::conj acc (:wat::string::trim (:wat::string::subs s start i)))
    (:wat::core::let [c (:wat::string::subs s i (:wat::core::i64::+ i 1))]
      (:wat::core::if (:user::open-bracket? c)
        (:user::split-top-level s (:wat::core::i64::+ i 1) (:wat::core::i64::+ depth 1) start acc)
        (:wat::core::if (:user::close-bracket? c)
          (:user::split-top-level s (:wat::core::i64::+ i 1) (:wat::core::i64::- depth 1) start acc)
          (:wat::core::if (:wat::core::if (:wat::core::= c ",") (:wat::core::i64::= depth 0) false)
            (:user::split-top-level s (:wat::core::i64::+ i 1) depth (:wat::core::i64::+ i 1)
              (:wat::core::conj acc (:wat::string::trim (:wat::string::subs s start i))))
            (:user::split-top-level s (:wat::core::i64::+ i 1) depth start acc)))))))

;; render-ref — full REFERENCE-role rendering of an angle-bracket-CARRYING keyword's TEXT
;; (leading colon included, e.g. ":wat::cache::Lru<K,V>") -> "(:wat::cache::Lru :- [K V])".
;; Three shapes, in order:
;;   1. `fn(...)` / `wat::core::Fn(...)` — a FUNCTION type. Left UNTOUCHED (returned verbatim):
;;      rare in the corpus (arg/ret positions inside `fn(...)` are a different sub-grammar this
;;      script does not parse) and not needed for the corpus this stone chases green — a real
;;      site here is a STOP-1 candidate for a follow-up, not silently mis-rendered.
;;   2. `(A,B,...)` — the NATIVE TUPLE spelling wrapping a (possibly parametric) element. A
;;      tuple element is parsed from a raw SUBSTRING (`parse_tuple_body`/`parse_type_inner`),
;;      which has NO path to the `(Head :- [args])` FORM (that shape only parses from a real
;;      `WatAST::List` node, never from string content) — so a parametric tuple element is
;;      only expressible once the WHOLE tuple moves to the structural spelling,
;;      `(:wat::core::Tuple :- [args])` (the head `parse_type_form` special-cases to
;;      `TypeExpr::Tuple`, `src/types.rs` ~5042 — byte-identical to the string spelling).
;;   3. `Head<args>` — the ordinary parametric case -> `(Head :- [args])`.
;; render-one-arg — one element/arg TEXT: a namespaced concrete type (contains "::") renders
;; as a colon-prefixed reference (recursing for a nested parametric/tuple); a bare short
;; identifier (K, V, T, Xt — no "::") is a lexical type VARIABLE and renders AS-IS, no colon —
;; the exact "inside compounds, args are bare Rust symbols" rule `parse_type_inner` used to
;; enforce at parse time (`src/types.rs`), now a corpus-authoring convention instead.
(:wat::core::defn :user::render-ref
  [kw-text <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [stripped (:wat::string::subs kw-text 1 (:wat::string::length kw-text))]
    (:wat::core::if (:wat::core::if (:wat::string::starts-with? stripped "fn(")
                      true
                      (:wat::string::starts-with? stripped "wat::core::Fn("))
      kw-text
      (:wat::core::if (:wat::string::starts-with? stripped "(")
        (:wat::core::let [close         (:user::scan-for-close stripped 1 1)
                          inner         (:wat::string::subs stripped 1 close)
                          args          (:user::split-top-level inner 0 0 0 (:wat::core::Vector :wat::core::String))
                          rendered-args (:user::render-args args)]
          (:wat::string::interpolate "(:wat::core::Tuple :- [{a}])" :a rendered-args))
        (:wat::core::let [lt (:user::find-first-lt stripped 0)]
          (:wat::core::if (:wat::core::i64::< lt 0)
            kw-text
            (:wat::core::let [base          (:wat::string::subs stripped 0 lt)
                              close         (:user::scan-for-close stripped (:wat::core::i64::+ lt 1) 1)
                              inner         (:wat::string::subs stripped (:wat::core::i64::+ lt 1) close)
                              args          (:user::split-top-level inner 0 0 0 (:wat::core::Vector :wat::core::String))
                              rendered-args (:user::render-args args)]
              (:wat::string::interpolate "(:{b} :- [{a}])" :b base :a rendered-args))))))))

(:wat::core::defn :user::render-one-arg
  [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::string::contains? s "::")
    (:user::render-ref (:wat::string::concat ":" s))
    s))

(:wat::core::defn :user::render-args
  [args <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::String
  (:wat::string::join " "
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) a <- :wat::core::String]
        -> (:wat::core::Vector :- [:wat::core::String])
        (:wat::core::conj acc (:user::render-one-arg a)))
      (:wat::core::Vector :wat::core::String)
      args)))

;; render-decl — DECL-NAME-role rendering: the same reference text minus its outer parens
;; (siblings, no wrapping application — the exact shape `strip-outer-parens` produces in
;; `parametrics-take-a-type-vector.wat`, reimplemented here over this script's own renderer).
(:wat::core::defn :user::render-decl
  [kw-text <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [ref (:user::render-ref kw-text)]
    (:wat::core::if (:wat::string::starts-with? ref "(")
      (:wat::string::subs ref 1 (:wat::core::i64::- (:wat::string::length ref) 1))
      ref)))

;; ── the walk — byte-identical shape to `parametrics-take-a-type-vector.wat` ─────────────

;; angle-shaped-keyword? — narrower than `:wat::fix::type-shaped-keyword?` on PURPOSE: this
;; script's only business is angle brackets (Arc 109 ③), so the detector requires an ACTUAL
;; `<…>` pair. `type-shaped-keyword?` also fires on a plain paren-only tuple
;; (`:wat::core::i64,wat::core::String)` — `(` + `)`, no `<`/`>`) — which is STILL LEGAL
;; syntax this stone's wall never touched; converting it anyway would be exactly the
;; "converted something that did not scream" mistake STOP-3 warns against.
(:wat::core::defn :user::angle-shaped-keyword?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
    (:wat::core::let [name (:wat::core::ast-name node)]
      (:wat::core::if (:wat::string::contains? name "<")
        (:wat::string::contains? name ">")
        false))
    false))

;; declarator-head-keyword? — copied verbatim (the DECL-NAME-vs-REFERENCE role split is a
;; property of the language, not of which stone is doing the rewriting).
(:wat::core::defn :user::declarator-head-keyword?
  [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
    (:wat::core::contains?
      (:wat::core::HashSet :wat::type::Infer
        ":wat::core::defn"
        ":wat::core::defenum"
        ":wat::core::defsurface"
        ":wat::core::defrecord"
        ":wat::holon::defrecord"
        ":wat::core::defstruct"
        ":wat::service::defservice"
        ":wat::core::typealias"
        ":wat::core::newtype"
        ":wat::core::typeunion"
        ":wat::core::recordtype"
        ":wat::core::aggregatetype"
        ":wat::core::structtype")
      (:wat::core::ast-name node))
    false))

;; leaf-edits — a keyword leaf gets ONE edit iff `:user::angle-shaped-keyword?` (structural,
;; string-based — unaffected by the wall). `prev-decl-head?` (threaded by seq-edits) picks
;; render-decl over render-ref.
;;
;; call-head? (Arc 109 fence): true iff this node sits at index 0 of a `list` container —
;; that is a CALL HEAD, e.g. `(:test::make-3tuple<T> args)` (class D) or a method-member
;; name `(make<T> [self] -> :T)` (class C). Neither role is a REFERENCE, and this codemod
;; only knows DECL-NAME and REFERENCE — measured (DESIGN-STONE-annihilate-the-angle-bracket.md)
;; to double-colon the arg and emit a form standing where a callable head goes, which then
;; fails ArityMismatch. Rather than silently emit that illegal shape again on every future
;; run, REFUSE loudly and point at the hand-fix step. Class D/C sites are hand-fixed per
;; BRIEF-STONE-annihilate-the-angle-bracket.md STEP 2, never routed through this codemod.
(:wat::core::defn :user::leaf-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   prev-decl-head? <- :wat::core::bool
   call-head? <- :wat::core::bool]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::angle-shaped-keyword? node)
    (:wat::core::if call-head?
      (:wat::kernel::assertion-failed!
        (:wat::string::interpolate
          "angle-brackets-to-binder: refusing call-head site `{n}` — a call-site type application (class D) or method-member name (class C) has no REFERENCE-role render (it is not a callable/name, a form is not a name). Hand-fix it per BRIEF-STONE-annihilate-the-angle-bracket.md STEP 2 instead of running this codemod over it."
          :n (:wat::core::ast-name node))
        :wat::core::None :wat::core::None)
      (:wat::core::let [nm      (:wat::core::ast-name node)
                         text    (:wat::core::if prev-decl-head?
                                   (:user::render-decl nm)
                                   (:user::render-ref nm))
                         span    (:wat::core::ast-span node)
                         off     (:wat::fix::fix-text-offset-of span lines)
                         old-len nm]
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
          (:wat::core::Tuple off old-len text))))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))))

(:wat::core::defn :user::node-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   prev-decl-head? <- :wat::core::bool
   is-first? <- :wat::core::bool
   parent-kind <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::seq-edits (:wat::core::ast->children node) lines true false (:wat::core::ast-kind node))
    (:user::leaf-edits node lines prev-decl-head?
      (:wat::core::if is-first? (:wat::core::= parent-kind "list") false))))

(:wat::core::defn :user::seq-edits
  [items           <- (:wat::core::Vector :- [:wat::WatAST])
   lines           <- (:wat::core::Vector :- [:wat::core::String])
   is-first?       <- :wat::core::bool
   prev-decl-head? <- :wat::core::bool
   parent-kind     <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
    (:wat::core::let [h               (:wat::core::first items)
                       this-decl-head? (:wat::core::if is-first? (:user::declarator-head-keyword? h) false)]
      (:wat::core::concat
        (:user::node-edits h lines prev-decl-head? is-first? parent-kind)
        (:user::seq-edits (:wat::core::rest items) lines false this-decl-head? parent-kind)))))

(:wat::core::defn :user::convert
  [src <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src)
                                 ((:wat::core::ReadOutcome::Forms __forms) __forms)
                                 ((:wat::core::ReadOutcome::Malformed __cause)
                                   (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms     (:wat::core::ast->children tree)
                    all-edits (:user::seq-edits forms lines true false "top")
                    rev-edits (:wat::core::reverse all-edits)]
    (:wat::fix::fix-text-apply src rev-edits)))

;; ── file/stdin harness — identical shape to every recorded migration ────────────────────
(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::convert (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[angle-brackets-to-binder] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
