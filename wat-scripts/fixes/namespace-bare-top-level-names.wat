;; wat-scripts/fixes/namespace-bare-top-level-names.wat — arc 278 DESIGN-STONE-namespacing-wall,
;; the `.wat` / `.wat.bad` corpus half: every remaining bare top-level definitional name.
;;
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; A SIBLING of `namespace-defrule-names.wat` (which closed the same gap for the `where`
;; corpus's `defrule` names): same discipline, GENERALIZED from one head (`defrule`) to
;; every top-level definitional head the wall now polices (`def`, `defn`, `defrecord`,
;; `defstruct`, `defenum`, `defsurface`, `defclause`, `defrule`, `defservice`, `deftest`,
;; `typealias`, `defmacro`), and from ONE separator (bare, prefix with `<ns>::`) to TWO
;; (bare → prefix; `/` pseudo-namespace → swap the first `/` for `::`).
;;
;; ── THE PROBLEM (DESIGN-STONE-namespacing-wall.md) ──────────────────────────────────────
;; `src/resolve/registration.rs`'s `gate` now rejects any top-level name from `Existing::
;; Absent` that isn't namespaced (`is_namespaced = name.contains("::")`). Only fn arguments
;; and `let` bindings may be bare. Two shapes of violation are live in the corpus:
;;   1. a genuinely bare name (`:arith`, `:get-config`, `:fb`) — needs a namespace prefix.
;;   2. a `/`-separated pseudo-namespace (`:rw/try`) — `/` is the ACCESSOR form
;;      (`Thread/join-result`, CONVENTIONS.md:45), so `:rw/try` claims a member of a type
;;      `rw` that doesn't exist. The file's OWN types already read `rw::Bag::Op` in the
;;      same breath — the swap (`:rw/try` → `:rw::try`) makes the fn agree with them.
;;
;; ── DISCOVERY, not a hand-kept file list ────────────────────────────────────────────────
;; `NOTE-bare-name-dispositions.md`'s file list has been wrong three times (see its own
;; header). This codemod does NOT consume that note. It is TABLE-FREE by construction: run
;; it (dry-run first) over a broad candidate superset (every non-stdlib `.wat`/`.wat.bad`),
;; and `any-needs-fix?` makes every already-clean file a byte-identical no-op — the diff
;; against the dry-run copy IS the discovery step, not a pre-computed list.
;;
;; ── PER-FILE NAMESPACE — DERIVED, never a hand-kept path→namespace table ────────────────
;; `resolve-ns`: walk the file's OWN top-level (+ splice-body, see below) definitional
;; names in order; the FIRST one that already contains "::" donates its leading segment as
;; this file's namespace (`rw::Bag::Op` ⇒ `rw`). If NONE do (the file has no namespaced name
;; at all), mint one from the file's OWN basename stem (`_`→`-`) — PER FILE, never shared:
;; several such files each define `:pi`/`:a` independently, so one shared minted namespace
;; would rebuild the very collision this pass exists to dissolve.
;;
;; ── "TOP-LEVEL" includes let/do SPLICE BODIES, not just the file's literal top form ──────
;; `wat_arc157_def.wat`'s test 7 defines `:get-config` inside a top-level `(let […] (def
;; :get-config …))` — syntactically nested, but the let is SPLICED at freeze time, so the
;; def is registered exactly like a literal top-level one (confirmed: the wall's current
;; `#wat.runtime/UnnamespacedName` fires on it). `collect-def-names` therefore recurses into
;; `:wat::core::let` / `:wat::core::do` WRAPPER bodies (never into `if`/`fn`/`defn` bodies —
;; those are runtime-only per that file's own "Gap I-B" comment, never reached at freeze,
;; so a bare name inside one is not currently part of the wall's failure set and is left
;; alone; scope stays exactly what freeze-time registration reaches).
;;
;; ── "TOP-LEVEL" ALSO includes `(:wat::core::forms …)` — a SECOND top level, not a splice ──
;; First pass missed this and shipped a real gap: `wat-tests/counter-actor-proof-process.wat`
;; passes a `(:wat::core::forms …)` block to `spawn-program'` as the CHILD process's entire
;; program; freeze bakes it as the child's own top level (proof: the child's own `:user::main`
;; lives inside one) — so `:counter/dispatch`, defined inside that block, is a top-level
;; registration in the child's world exactly as much as any name at the file's literal top,
;; and the wall caught it (`#wat.runtime/UnnamespacedName ':counter/dispatch'`,
;; `wat-tests/counter-actor-proof-process.wat:213:19`) the first time the codemod's OWN
;; touched-file set was weighed for real. `collect-def-names` recurses into `forms` bodies
;; the same way it does `do` (no bindings vector to skip); the two transform rules are
;; unchanged — a `forms`-block name is namespaced/swapped exactly like any other.
;;
;; ── PER TOP-LEVEL NAME (only when it needs fixing; already-`::` names are untouched) ─────
;;   contains "::"                → skip (left untouched)
;;   contains "/" and no "::"     → swap the FIRST "/" for "::" (`:rw/try` → `:rw::try`)
;;   otherwise                    → prefix `:<ns>::` (`:get-config` → `:<ns>::get-config`)
;;
;; `rename-keyword-exact` (shared with `namespace-defrule-names.wat`) then moves BOTH the
;; definition and every call-site spelling in one pass — same keyword token, whole-name
;; equality, so a prefix-sibling (`:needs-record2`) is never mis-matched.
;;
;; ── APPLYING THE RENAMES — a fold over a Vector of tuples, never a nested staircase ──────
;; `apply-renames` is a literal `:wat::core::foldl` threading `text` through each
;; `rename-keyword-exact` call — 24t's lesson (a hand-nested staircase of N calls stopped
;; being eyeballable and was wrong twice).
;;
;; ── IDEMPOTENCE ──────────────────────────────────────────────────────────────────────────
;; Gated on `any-needs-fix?`: once every collected name already contains "::", `migrate`
;; returns `src` UNCHANGED — no re-derivation, no re-rename, byte-identical re-run.
;;
;; ── `.wat.bad` SPECIMENS ─────────────────────────────────────────────────────────────────
;; A rename never changes VALUES or control flow, only which token spells a name (def +
;; every call site together) — so a specimen's own defect (a type mismatch, a Liskov
;; violation, a forbidden redef) survives the rename intact. The one thing to verify PER
;; SPECIMEN (not assumed): capture its `--check` failure before and after and diff the
;; *reason* — if a `.bad` file's bare name were the pinned defect itself (never observed in
;; this corpus, but the STOP the brief calls for if it ever is), renaming it would erase the
;; specimen; this codemod does not special-case that because no discovered specimen needed it.
;;
;; Dry-run on a /tmp copy + diff, THEN apply:
;;   printf '["pathA" "pathB" …]' | ./target/release/wat ./wat-scripts/fixes/namespace-bare-top-level-names.wat

;; ── the definitional heads the wall polices — a top-level form headed by one of these has
;; its NAME at child[1] ─────────────────────────────────────────────────────────────────
(:wat::core::defn :user::def-heads [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::Vector :wat::core::String
    ":wat::core::def" ":wat::core::defn" ":wat::core::defrecord" ":wat::holon::defrecord"
    ":wat::core::defstruct" ":wat::core::defenum" ":wat::core::defsurface"
    ":wat::core::defclause" ":wat::rete::defrule" ":wat::service::defservice"
    ":wat::test::deftest" ":wat::core::typealias" ":wat::core::defmacro"))

(:wat::core::defn :user::def-head? [h <- :wat::core::String] -> :wat::core::bool
  (:wat::fix::str-in? h (:user::def-heads)))

;; splice-head? — a `let`/`do` wrapper whose BODY freeze SPLICES into the enclosing program
;; (arc157's let-splice / do-splice). `if`/`fn`/`defn` bodies are runtime-only — never
;; recursed. (`:wat::core::forms` is handled SEPARATELY, below — it is not a splice.)
(:wat::core::defn :user::splice-head? [h <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= h ":wat::core::let") true (:wat::core::= h ":wat::core::do")))

;; splice-body — the wrapper's child forms that stand in for top-level forms: `let` drops
;; its head + bindings-vector (child[0..1]); `do` drops just its head (child[0]).
(:wat::core::defn :user::splice-body [f <- :wat::WatAST h <- :wat::core::String] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::= h ":wat::core::let")
    (:wat::core::into [] (:wat::core::drop (:wat::core::ast->children f) 2))
    (:wat::core::into [] (:wat::core::drop (:wat::core::ast->children f) 1))))

;; name-node-of — a def-form's NAME node (child[1]).
(:wat::core::defn :user::name-node-of [f <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children f) 1) "name-node-of: child[1]"))

;; ── `(:wat::core::forms …)` — a SECOND top level, found by a POSITION-INDEPENDENT deep
;; search, not by splice recursion ──────────────────────────────────────────────────────
;; A `forms` block is the literal payload `spawn-program'`/`spawn-peer` ship to a CHILD
;; process; freeze bakes it as the child's own top level wherever it lexically sits — in
;; `wat-tests/counter-actor-proof-process.wat` that is inside a top-level LET's BINDING
;; VALUE (`[peer! (:wat::test::spawn-peer … (:wat::core::forms …))]`), a position
;; `splice-body` never visits (it only walks a let's BODY, past the bindings vector,
;; because only the body is what freeze splices for `let` itself). A `forms` block is not
;; splice-shaped at all — it can sit anywhere a value is written (a binding's value, a call
;; argument, …) — so it is found by an UNCONDITIONAL deep walk of every structural node,
;; independent of def-head/splice-head position. Proof it matters: the child's own
;; `:user::main` lives inside one (`wat-tests/counter-actor-proof-process.wat:209`), and
;; `:counter/dispatch` inside the same block fired `#wat.runtime/UnnamespacedName` at
;; `wat-tests/counter-actor-proof-process.wat:213:19` — the first pass's splice-only
;; recursion could not reach it.
(:wat::core::defn :user::forms-call? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::= (:wat::fix::head-name node) ":wat::core::forms"))

(:wat::core::defn :user::deep-find-forms-blocks [node <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::let [here (:wat::core::if (:user::forms-call? node)
                            (:wat::core::Vector :wat::WatAST node)
                            (:wat::core::Vector :wat::WatAST))]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat here (:user::deep-find-forms-blocks-seq (:wat::core::ast->children node)))
      here)))

(:wat::core::defn :user::deep-find-forms-blocks-seq [items <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :wat::WatAST)
    (:wat::core::concat
      (:user::deep-find-forms-blocks (:wat::core::first items))
      (:user::deep-find-forms-blocks-seq (:wat::core::rest items)))))

;; forms-block-body — a `(:wat::core::forms f1 f2 …)` node's payload forms (drop the head;
;; no bindings vector to skip, unlike `let`).
(:wat::core::defn :user::forms-block-body [fb <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::into [] (:wat::core::drop (:wat::core::ast->children fb) 1)))

;; collect-def-names-shallow — every def-form NAME node directly reachable from `items`,
;; recursing through let/do splice bodies only (never through if/fn/defn bodies, never
;; deep-searching for `forms` — that is the caller's job). Order-preserving.
(:wat::core::defn :user::collect-def-names-shallow [items <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :wat::WatAST)
    (:wat::core::let [f (:wat::core::first items) tl (:wat::core::rest items)
                      h (:wat::fix::head-name f)]
      (:wat::core::concat
        (:wat::core::if (:user::def-head? h)
          (:wat::core::let [nn (:user::name-node-of f)]
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind nn) "keyword")
              (:wat::core::Vector :wat::WatAST nn)
              (:wat::core::Vector :wat::WatAST)))
          (:wat::core::if (:user::splice-head? h)
            (:user::collect-def-names-shallow (:user::splice-body f h))
            (:wat::core::Vector :wat::WatAST)))
        (:user::collect-def-names-shallow tl)))))

;; names-in-forms-blocks — every def-form NAME node inside every `forms`-block found ANYWHERE
;; (any depth, any position) in `forms`, applying the same shallow (+ let/do splice) walk to
;; each block's own body. A `forms` block nested inside another is reached too: the deep
;; search already recurses through the outer block's children before returning.
(:wat::core::defn :user::names-in-forms-blocks [forms <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::WatAST]) fb <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::WatAST])
      (:wat::core::concat acc (:user::collect-def-names-shallow (:user::forms-block-body fb))))
    (:wat::core::Vector :wat::WatAST)
    (:user::deep-find-forms-blocks-seq forms)))

;; collect-def-names — the file-level entry point: names at the literal top level (+ let/do
;; splice bodies) PLUS names inside every `forms` block anywhere in the file (position-
;; independent — see above). Order between the two groups does not matter for `resolve-ns`
;; today (no discovered file's first namespaced name lives only inside a `forms` block), but
;; keeping the shallow walk first keeps the common case's derivation reading top-to-bottom.
(:wat::core::defn :user::collect-def-names [forms <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::concat
    (:user::collect-def-names-shallow forms)
    (:user::names-in-forms-blocks forms)))

;; ── classifying a collected name ─────────────────────────────────────────────────────────

(:wat::core::defn :user::already-ns? [nn <- :wat::WatAST] -> :wat::core::bool
  (:wat::string::contains? (:wat::core::ast-name nn) "::"))

(:wat::core::defn :user::needs-fix? [nn <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::not (:user::already-ns? nn)))

(:wat::core::defn :user::any-needs-fix? [names <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool n <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::if acc true (:user::needs-fix? n)))
    false
    names))

;; ── deriving / minting the file's namespace ──────────────────────────────────────────────

;; find-ns — the FIRST already-namespaced collected name donates its leading segment.
(:wat::core::defn :user::find-ns [names <- (:wat::core::Vector :- [:wat::WatAST])] -> (:wat::core::Option :wat::core::String)
  (:wat::core::if (:wat::core::empty? names)
    :wat::core::None
    (:wat::core::let [n (:wat::core::first names) tl (:wat::core::rest names)]
      (:wat::core::if (:user::already-ns? n)
        (:wat::core::let [nm   (:wat::core::ast-name n)
                          seg0 (:wat::core::first (:wat::string::split nm "::"))]
          (:wat::core::Some (:wat::string::strip-leading-colon seg0)))
        (:user::find-ns tl)))))

;; basename — the path's final "/"-segment (the filename with extension).
(:wat::core::defn :user::basename [path <- :wat::core::String] -> :wat::core::String
  (:wat::core::Option/expect (:wat::core::last (:wat::string::split path "/")) "basename: split always >= 1"))

;; strip-wat-ext — drop a trailing ".wat.bad" or ".wat" (checked longest-first).
(:wat::core::defn :user::strip-wat-ext [base <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::string::ends-with? base ".wat.bad")
    (:wat::string::subs base 0 (:wat::core::i64::- (:wat::string::length base) 8))
    (:wat::core::if (:wat::string::ends-with? base ".wat")
      (:wat::string::subs base 0 (:wat::core::i64::- (:wat::string::length base) 4))
      base)))

;; mint-ns — PER FILE, from the file's OWN basename stem, "_" -> "-". Never shared: a file
;; with no namespaced name at all gets a namespace that belongs to it alone.
(:wat::core::defn :user::mint-ns [path <- :wat::core::String] -> :wat::core::String
  (:wat::string::join "-" (:wat::string::split (:user::strip-wat-ext (:user::basename path)) "_")))

;; resolve-ns — derive from the file's own first namespaced name; mint from its basename
;; only when the file has none at all.
(:wat::core::defn :user::resolve-ns [names <- (:wat::core::Vector :- [:wat::WatAST]) path <- :wat::core::String] -> :wat::core::String
  (:wat::core::match (:user::find-ns names)
    ((:wat::core::Some ns) ns)
    (:wat::core::None (:user::mint-ns path))))

;; ── computing each rename ────────────────────────────────────────────────────────────────

;; swap-slash — ":rw/try" -> ":rw::try" (swap the FIRST "/" for "::"; a name with more than
;; one "/" keeps the rest joined by "/", so only the first separator is reinterpreted).
(:wat::core::defn :user::swap-slash [nm <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [segs (:wat::string::split nm "/")
                    seg0 (:wat::core::first segs)
                    tail (:wat::core::rest segs)]
    (:wat::core::String/concat seg0 (:wat::core::String/concat "::" (:wat::string::join "/" tail)))))

;; new-name-for — the fixed spelling for a name already known to need-fix.
(:wat::core::defn :user::new-name-for [nn <- :wat::WatAST ns <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [nm (:wat::core::ast-name nn)]
    (:wat::core::if (:wat::string::contains? nm "/")
      (:user::swap-slash nm)
      (:wat::core::String/concat ":" (:wat::core::String/concat ns
        (:wat::core::String/concat "::" (:wat::string::strip-leading-colon nm)))))))

;; collect-renames — (old,new) pairs for every collected name that needs fixing.
(:wat::core::defn :user::collect-renames
  [names <- (:wat::core::Vector :- [:wat::WatAST]) ns <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? names)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::String]))
    (:wat::core::let [n (:wat::core::first names) tl (:wat::core::rest names)]
      (:wat::core::if (:user::needs-fix? n)
        (:wat::core::let [old (:wat::core::ast-name n)
                          new (:user::new-name-for n ns)]
          (:wat::core::concat
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::String :wat::core::String]) (:wat::core::Tuple old new))
            (:user::collect-renames tl ns)))
        (:user::collect-renames tl ns)))))

;; apply-renames — a fold over the (old,new) Vector, never a nested staircase (24t's lesson).
(:wat::core::defn :user::apply-renames
  [text    <- :wat::core::String
   renames <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])]
  -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String p <- (:wat::core::Tuple :- [:wat::core::String :wat::core::String])] -> :wat::core::String
      (:wat::fix::rename-keyword-exact (:wat::core::first p) (:wat::core::second p) acc))
    text
    renames))

;; ── per-file migrate ─────────────────────────────────────────────────────────────────────

;; A handful of `.wat.bad` specimens are deliberately malformed at the LEX/PARSE layer
;; (e.g. whitespace inside `<>` — `wat_arc072_letstar_parametric_whitespace.wat.bad`, a
;; "clean lex-layer error" fixture). `read-string` on those returns `ReadOutcome::Malformed`
;; — there is no form tree to walk, hence vacuously no top-level name to rename. Leave the
;; file byte-identical rather than raising: it is out of this codemod's scope by
;; construction (a parse failure predates any question of namespacing), and forcing it
;; through would either crash or, worse, "fix" a file that must stay unparseable.
(:wat::core::defn :user::migrate [src <- :wat::core::String path <- :wat::core::String] -> :wat::core::String
  (:wat::core::match (:wat::core::read-string src)
    ((:wat::core::ReadOutcome::Malformed __cause) src)
    ((:wat::core::ReadOutcome::Forms tree0)
      (:wat::core::let
        [forms0 (:wat::core::ast->children tree0)
         names0 (:user::collect-def-names forms0)]
        (:wat::core::if (:wat::core::not (:user::any-needs-fix? names0))
          src ;; idempotent no-op — every collected name is already namespaced
          (:wat::core::let
            [ns      (:user::resolve-ns names0 path)
             renames (:user::collect-renames names0 ns)]
            (:user::apply-renames src renames)))))))

;; ── driver: rewrite each path given on stdin (a JSON array of strings) ──────────────────
(:wat::core::defn :user::rewrite-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [p (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file p (:user::migrate (:wat::io::read-file p) p))
        (:wat::kernel::println (:wat::core::String/concat "[namespace-bare-top-level-names] " p))
        (:user::rewrite-each (:wat::core::into [] (:wat::core::rest paths)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [paths (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
    (:user::rewrite-each paths)))
