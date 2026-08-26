;; wat-scripts/fixes/parametrics-take-a-type-vector.wat — arc 109 Stone ②-ii.
;;
;; Rewrites every type-shaped keyword into the bracketed COLON form, keeping the rust-ish
;; `:wat::core::` head spelling (the Clojure `wat.type/` flip is a LATER, separate stone):
;;
;;   :wat::core::Vector<wat::core::i64>              -> (:wat::core::Vector [:wat::core::i64])
;;   :wat::core::HashMap<wat::core::String,…i64>     -> (:wat::core::HashMap [:wat::core::String :wat::core::i64])
;;   :wat::core::Vector<wat::core::Vector<…>>        -> nests
;;
;; Carries its OWN walk (mirrors :wat::fix::fix-text-seq-edits / fix-text-struct-edits /
;; fix-text-node-edits: structural nodes recurse via `structural?`, leaves get zero-or-one
;; span edit) but applies ONLY ONE of fix.wat's three leaf rules —
;; `:wat::fix::type-shaped-keyword?` — never `head-keyword?` (arc 300's namespace flip, out
;; of scope; this stone changes the SHAPE of the type-arg group and nothing else).
;;
;; ── WHY NOT ALSO THE "post-arrow keyword" RULE (fix.wat's rule 1) ──────────────────────
;; fix.wat's `fix-text-leaf-edits` has a first rule: ANY keyword immediately after an arrow
;; (`prev-arrow?` true) is a type annotation and gets rewritten too, type-shaped or not. The
;; BRIEF names this rule in-scope. Measured on /tmp before writing the walk this way: it is
;; **redundant** for every real site — a post-arrow keyword that IS type-shaped is already
;; caught by `type-shaped-keyword?` regardless of position, and a post-arrow keyword that is
;; a plain FQDN/user-type/type-var renders byte-identical in Colon mode (verified via the
;; `arc109-2i-colon-mode-verbatim-probe.wat` rungs) — so applying it changes nothing.
;; EXCEPT: `-> :wat::core::nil` (≈1,126 real sites in wat/+tests/+wat-scripts/ alone).
;; `:wat::core::keyword/to-type-form-colon` parses `nil` to the internal `TypeExpr::Tuple(vec![])`
;; (arc 153's canonical-unit reduction, `src/types.rs:4728`), and `type_expr_to_clojure_form`'s
;; `Tuple` arm is hard-coded to the Symbol `wat.type/Tuple` **in BOTH modes** (its own doc,
;; `src/edn/render.rs`: "the head is OUT OF SCOPE for `mode`... nothing in the acceptance
;; criteria or the contract suite exercises a Colon-mode Tuple"). So the post-arrow rule,
;; applied literally, rewrites every `-> :wat::core::nil` to `-> (wat.type/Tuple)` — a
;; Clojure-mode symbol landing in the still-Colon-mode corpus. That is exactly the "third
;; spelling that parses so nothing screams" landmine this stone exists to avoid, just via a
;; different door than the one the brief flagged (arrows). Confirmed live on
;; `tests/types/probe_arc214_lexer_primed_generic_head_primed.wat`, whose
;; `-> :wat::core::nil` became `-> (wat.type/Tuple)` under a literal rule-1+2 implementation.
;; Dropping rule 1 loses nothing (it was a no-op everywhere it wasn't corruption) and closes
;; the landmine entirely — `type-shaped-keyword?` is false for `nil` (no `<`/`>`/`(`/`)`), so
;; it is never touched. Reported to the orchestrator as a found defect in the ②-i renderer;
;; not fixed here (`src/edn/render.rs` / `src/types.rs` are out of this stone's blast radius).
;;
;; Reuses fix.wat's PUBLIC helpers only — structural?, type-shaped-keyword?,
;; fix-text-offset-of, fix-text-apply — never fix.wat's internal edit trio, and never
;; modifies fix.wat itself (that machinery serves the separate faithful-Clojure drive).
;;
;; ── A SECOND, DEEPER INSTANCE OF THE SAME LANDMINE — a rendered-output safety guard ────
;; Dropping rule 1 does not fully close the `wat.type/Tuple` hole: `type_expr_to_clojure_form`'s
;; `TypeExpr::Tuple` arm is hard-coded to the Clojure symbol IN BOTH MODES, and that arm fires
;; not only for a bare `nil`/`:( )` at top level, but for `nil` or a `:(...)` tuple type
;; NESTED as an ARGUMENT inside an otherwise perfectly legitimate, in-scope parametric — e.g.
;; `:wat::core::Result<wat::core::nil,wat::sqlite::Error>` (extremely common: a fallible verb
;; with no success payload) renders to `(:wat::core::Result [(wat.type/Tuple) :wat::sqlite::Error])`,
;; and a standalone tuple-type keyword `:(wat::core::i64,wat::core::i64,wat::core::String)`
;; (used throughout fix.wat itself, for the edit-tuple type) renders to the WRONG head+shape
;; entirely: `(wat.type/Tuple :wat::core::i64 :wat::core::i64 :wat::core::String)` — Clojure
;; head, flat (unbracketed) args, in both modes (design doc: Tuple's head is "OUT OF SCOPE for
;; `mode`"). Both are confirmed live on /tmp copies of wat/sqlite.wat and wat/fix.wat.
;;
;; These sites ARE legitimately type-shaped and squarely this stone's business, so the fix
;; can't be "never touch them" (rule-1's fix). Instead: render first, then INSPECT the
;; rendered text for the tell — any `wat.type/` substring means the renderer fell through to
;; the mode-blind Tuple arm somewhere in the tree — and refuse the edit (leave the original
;; token untouched) rather than land a mixed-mode spelling. This is the SAME discipline the
;; brief applies to printer choice (never emit what you have not verified is the right
;; spelling), applied to the renderer's *output* instead of its call site. Every skipped site
;; is a real corpus gap this codemod cannot yet close — `type_expr_to_clojure_form`'s Tuple arm
;; needs a Colon-mode rendering (a follow-up to ②-i, out of this stone's blast radius: no
;; `src/` file is touched here) before those sites can move.
;;
;; ⚠ PRINTER: renders with `:wat::core::ast->source` (VERBATIM `::`-source), never
;; `:wat::core::write-forms` (which re-spells every `::`-keyword into the EDN-dotted form —
;; a THIRD spelling, neither where the corpus is nor where it is going). Verified live via
;; wat-scripts/scratch-pad/arc109-2i-colon-mode-verbatim-probe.wat.
;;
;; The converter — :wat::core::keyword/to-type-form-colon — parses the keyword's embedded
;; `Head<args...>` / `(args...)` shape and rebuilds it as a form node with bracketed args and
;; the ORIGINAL `:wat::core::`-style head untouched; nesting and primed heads (`Peer'<I,O>`)
;; are the converter's concern, not this walk's — this walk only decides WHICH leaf to hand it.
;;
;; Comment-faithful (span-edits splice the original text) and idempotent: a keyword already in
;; the bracketed form contains no bare `<`/`>` pair, so `type-shaped-keyword?` no longer matches
;; it and a second run is a byte-identical no-op.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/source.wat"]\n' | cargo wat ./wat-scripts/fixes/parametrics-take-a-type-vector.wat
;;
;; Dry-run on a /tmp copy (MANDATORY before any corpus application — a LATER stone, not this one):
;;   cp wat/source.wat /tmp/pilot.wat
;;   printf '["/tmp/pilot.wat"]\n' | cargo wat ./wat-scripts/fixes/parametrics-take-a-type-vector.wat
;;   diff wat/source.wat /tmp/pilot.wat

;; ── the walk ──────────────────────────────────────────────────────────────────────────

;; safe-colon-rendering? — the tell for the Tuple-arm landmine: a correct Colon-mode render
;; never contains the Clojure `wat.type/` spelling anywhere (every case of the 4-way ladder
;; that IS mode-aware renders `:wat::core::…`/`:…`/a bare symbol; only the mode-BLIND Tuple
;; arm emits `wat.type/Tuple`, whether at the top or nested inside a legitimate parametric's
;; arg list). A substring check is exact here because `wat.type/` cannot appear in any correct
;; Colon-mode output.
(:wat::core::defn :user::safe-colon-rendering?
  [rendered <- :wat::core::String] -> :wat::core::bool
  (:wat::core::not (:wat::string::contains? rendered "wat.type/")))

;; declarator-head-keyword? — node is a keyword leaf whose full name (":wat::core::defn" etc.)
;; is one of the heads that open a declaration form whose OWN name sits at index 1 — a binder,
;; not a reference. The set is every head that HAS such a slot, NOT merely those carrying a
;; parametric name in today's corpus: an unlisted head is silently rendered as a reference, so a
;; short list is a latent corruption rather than a smaller change. Note
;; `:wat::rete::core::defn` (the rete-DSL `defn` variant) is deliberately NOT in this set — it
;; is a real, distinct head in the corpus, but census confirmed it never carries a parametric
;; name, so leaving it out changes nothing observable; if that ever stops being true the walk
;; below renders it as a REFERENCE (parens kept), same as any other unlisted head — never
;; silently as a binder.
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
        ;; ⚠ ADDED by the orchestrator during scoring, and the reason is the failure mode above:
        ;; an UNLISTED declarator head does not fail loudly — its name slot is rendered as a
        ;; REFERENCE and silently corrupted, which is the exact defect this stone exists to fix.
        ;; Measured: `(:wat::core::typealias :wat::cache::Lru<K,V> …)` (wat/cache.wat:68) became
        ;; `(:wat::core::typealias (:wat::cache::Lru :- [K V]) …)` before these were added.
        ;; So the set is now every head that HAS a declaration-name slot — a property of the
        ;; LANGUAGE — rather than every head that happens to carry a parametric name today — a
        ;; property of this corpus. Each destination verified to accept `name :- [T…]` (strike α)
        ;; before being listed here; the last five have zero parametric sites at present and cost
        ;; nothing to include. [[feedback_a_gate_freezes_names_never_a_count]]
        ":wat::core::typealias"
        ":wat::core::newtype"
        ":wat::core::typeunion"
        ":wat::core::recordtype"
        ":wat::core::aggregatetype"
        ":wat::core::structtype")
      (:wat::core::ast-name node))
    false))

;; strip-outer-parens — a binder is the reference form WITHOUT the application: the exact same
;; rendered text (:wat::core::keyword/to-type-form-colon + :wat::core::ast->source, the SAME
;; path the reference case uses), minus its leading `(` and trailing `)`. Never a second
;; renderer — a second renderer is a second thing to drift from the first. If the rendering
;; ever isn't application-shaped here, that is this stone's own invariant breaking, not a
;; corpus shape to paper over — STOP via assertion-failed! rather than emit a guess.
(:wat::core::defn :user::strip-outer-parens
  [rendered <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::if (:wat::core::= (:wat::string::subs rendered 0 1) "(")
                    (:wat::string::ends-with? rendered ")")
                    false)
    (:wat::string::subs rendered 1 (:wat::i64::- (:wat::string::length rendered) 1))
    (:wat::kernel::assertion-failed!
      (:wat::string::concat "parametrics-take-a-type-vector: declarator-name rendering is not application-shaped: " rendered)
      :wat::core::None :wat::core::None)))

;; leaf-edits — a keyword leaf gets ONE edit iff it is structurally type-shaped
;; (:wat::fix::type-shaped-keyword?) AND its rendering is safe (:user::safe-colon-rendering?);
;; every other leaf (symbols incl. arrows, head keywords, bare data keywords, literals) is
;; left alone, and a type-shaped keyword whose rendering the converter cannot yet do
;; correctly in Colon mode is left untouched rather than landing a mixed-mode spelling.
;; `prev-decl-head?` (threaded by seq-edits) says the immediately preceding sibling was an
;; index-0 declarator-head keyword — i.e. THIS node is the declaration's own name, a binder,
;; not a reference — so its rendering gets the outer parens stripped (:user::strip-outer-parens)
;; instead of the reference form's wrapping `(...)`.
(:wat::core::defn :user::leaf-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   prev-decl-head? <- :wat::core::bool]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
                    (:wat::fix::type-shaped-keyword? node)
                    false)
    (:wat::core::let [rendered (:wat::core::ast->source (:wat::keyword::to-type-form-colon node))]
      (:wat::core::if (:user::safe-colon-rendering? rendered)
        (:wat::core::let [span    (:wat::core::ast-span node)
                          off     (:wat::fix::fix-text-offset-of span lines)
                          nm      (:wat::core::ast-name node)
                          old-len nm
                          text    (:wat::core::if prev-decl-head?
                                    (:user::strip-outer-parens rendered)
                                    rendered)]
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
            (:wat::core::Tuple off old-len text)))
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))))

;; node-edits — structural nodes recurse (via seq-edits, fresh at index 0 of THEIR OWN
;; children); leaves go to leaf-edits, carrying whatever prev-decl-head? seq-edits computed
;; for this position in the PARENT's child list.
(:wat::core::defn :user::node-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])
   prev-decl-head? <- :wat::core::bool]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::seq-edits (:wat::core::ast->children node) lines true false)
    (:user::leaf-edits node lines prev-decl-head?)))

;; seq-edits — left-to-right walk over a child vector, collecting edits in ascending offset
;; order. Position-AWARE, copying the shape of :wat::fix::fix-seq (wat/fix.wat:123), which
;; already proves context-threading works over exactly this recursive sibling shape.
;; `type-shaped-keyword?` alone fires the same regardless of where the keyword sits — but
;; WHERE it sits still decides HOW it renders: index 0 of a structural node names the
;; declaration being made (one of :user::declarator-head-keyword?'s heads), and that
;; declaration's own name at index 1 is a BINDER, not a reference. So two flags thread through:
;; `is-first?` — this call's head item sits at index 0 of its node — and `prev-decl-head?` —
;; the immediately preceding sibling WAS an index-0 declarator-head keyword. leaf-edits reads
;; `prev-decl-head?` to choose the binder rendering over the reference one.
(:wat::core::defn :user::seq-edits
  [items           <- (:wat::core::Vector :- [:wat::WatAST])
   lines           <- (:wat::core::Vector :- [:wat::core::String])
   is-first?       <- :wat::core::bool
   prev-decl-head? <- :wat::core::bool]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
    (:wat::core::let [h               (:wat::core::first items)
                      this-decl-head? (:wat::core::if is-first? (:user::declarator-head-keyword? h) false)]
      (:wat::core::concat
        (:user::node-edits h lines prev-decl-head?)
        (:user::seq-edits (:wat::core::rest items) lines false this-decl-head?)))))

;; convert — src string -> migrated-src string. Parses, walks top-level forms for edits
;; (ascending offset), reverses to right-to-left, splices the ORIGINAL text via
;; :wat::fix::fix-text-apply so comments and formatting between edited tokens survive.
(:wat::core::defn :user::convert
  [src <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms     (:wat::core::ast->children tree)
                    all-edits (:user::seq-edits forms lines true false)
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
        (:wat::kernel::println (:wat::string::concat "[parametrics-take-a-type-vector] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
