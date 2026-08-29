;; wat/grep.wat — wat-grep's vocabulary: the fact base it inserts, per file, and the ONE query.
;;
;; DESIGN: docs/arc/2026/06/278-rules-engine/DESIGN-STONE-wat-grep-is-a-feature.md — the contract,
;; the file's shape, and why each name is what it is. THE CONTRACT, verbatim:
;;
;;   wat-grep DECLARES   Node · Named · Span      the fact base it inserts, per file
;;                       Match · Capture          what a rule asserts
;;                       q-match                  the one query — never written by a user
;;   the USER DECLARES   rules over those, asserting Match
;;   wat-grep OWNS       compile · the lease · the loop · the reset · the query · the print
;;
;; This is a MOVE of proven code from
;; wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat (shipped numbers: `5d650b807`,
;; wat/fix.wat Node=4316 Span=4316, neg-consumer 435, probe_do_splice 33). The migration is
;; `:fx::` → `:wat::grep::`, `:fx::Acc` becomes `:wat::grep::Facts`, and the four inline
;; `Option/expect` + `HashMap/get` unwraps of an `ast-span`/`ast-end-span` HashMap collapse into
;; one door, `extent-of` — after this file, no other site anywhere unwraps a span.
;;
;; ─── THE ONE DESIGN DECISION (carried from corpus-03, unchanged) ─────────────
;; Node identity is assigned by a PRE-ORDER walk with a threaded counter. Pre-order is chosen
;; because it makes `parent` always ALREADY ASSIGNED when a child is numbered — a post-order walk
;; would force a second pass to back-fill parents.
;;
;; ─── AND THE GUARD (corpus 01's, now load-bearing for real) ──────────────────
;; `ast-name` is PARTIAL — it raises on any node that is not Symbol/Keyword/StringLit. The guard
;; is structural instead: a `Named` fact is emitted ONLY for a nameable kind, so an unnameable
;; node simply HAS NO NAME FACT and every downstream rule that joins `Named` cannot see it. The
;; absence IS the guard.
;;
;; ★ NON-VACUITY: `Node` must exceed `Named` on any real file. `Span` is emitted UNCONDITIONALLY
;; beside `Node` (unlike `Named`) because `ast-span`/`ast-end-span` are TOTAL — Span == Node is
;; the non-vacuity control for that half.

;; ── the fact base wat-grep inserts, one set per file ────────────────────────────────

(:wat::core::defrecord :wat::grep::Node
  [id     <- :wat::core::i64
   parent <- :wat::core::i64
   index  <- :wat::core::i64
   kind   <- :wat::core::String])

;; ONLY for a nameable kind — the absence IS the guard.
(:wat::core::defrecord :wat::grep::Named
  [id   <- :wat::core::i64
   name <- :wat::core::String])

;; EVERY node — Span == Node is the non-vacuity control. Flat, not nested, and NOT
;; :wat::core::Span: `ast-span` returns keyword->i64 so it cannot carry :file; the file is a
;; property of the RUN, not of a node; and a rule binds FIELDS, not sub-records.
;; ⚠ CROSS-REFERENCE: this field list (line/col/end-line/end-col) must stay in lockstep with
;; :wat::grep::Extent's four fields below — nothing pins them together, so a rename of one must
;; be made in both by hand.
(:wat::core::defrecord :wat::grep::Span
  [id       <- :wat::core::i64
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64])

;; ONLY when the span holds exactly this node's own name — the fact a REWRITING rule joins.
;; `Named` says WHAT a node is called; `Written` says AND IT IS SPELLED HERE. A reader-
;; synthesized node (`~` -> unquote, `` ` `` -> quasiquote, `\c` -> char/of, …) gets a `Named`
;; fact (its name is real) but NOT a `Written` fact (the span it carries is the literal token's,
;; not its own name's) — `ast-name` returns verbatim token TEXT, so for a single-line named node
;; `end-col - col == length(name)` iff the name is actually spelled at that span; not a heuristic.
;; It carries coordinates, not just `{id}`: a rewriting rule joins ONE fact and never touches
;; `Span` at all. See DESIGN-STONE-wat-grep-never-lies.md F2.
(:wat::core::defrecord :wat::grep::Written
  [id       <- :wat::core::i64
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64])

;; ── what a rule asserts ─────────────────────────────────────────────────────────────

(:wat::core::defrecord :wat::grep::Capture
  [name  <- :wat::core::String
   value <- :wat::core::String])

;; FLAT — no nested Extent — because a rule binds FIELDS, not sub-records.
(:wat::core::defrecord :wat::grep::Match
  [file     <- :wat::core::String
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64
   rule     <- :wat::core::String
   captures <- (:wat::core::PersistentVector :- [:wat::grep::Capture])])

;; ── the in-process coordinate, and THE ONE DOOR ─────────────────────────────────────

(:wat::core::defrecord :wat::grep::Extent
  [line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64])

;; extent-of — the ONLY site that unwraps an ast-span/ast-end-span HashMap. Mirrors
;; wat/fix.wat's fix-text-offset-of shape. After this fn exists, nothing else anywhere unwraps a
;; span — that is what the name promises.
(:wat::core::defn :wat::grep::extent-of
  [node <- :wat::WatAST]
  -> :wat::grep::Extent
  (:wat::core::let [sp (:wat::core::ast-span node)
                    ep (:wat::core::ast-end-span node)]
    (:wat::grep::Extent
      :line     (:wat::core::Option/expect (:wat::hashmap::get sp :line) "extent-of: :line")
      :col      (:wat::core::Option/expect (:wat::hashmap::get sp :col)  "extent-of: :col")
      :end-line (:wat::core::Option/expect (:wat::hashmap::get ep :line) "extent-of: :end-line")
      :end-col  (:wat::core::Option/expect (:wat::hashmap::get ep :col)  "extent-of: :end-col"))))

;; ── source → facts ──────────────────────────────────────────────────────────────────

;; Source — ONE fact per file, naming where the other facts came from.
;;
;; ⛔ WHY THIS EXISTS, and it is a correction. The DESIGN argued — correctly — that "the file is a
;; property of the RUN, not of a node; repeating it on every fact in a 4316-node file is 4316
;; copies of one string." That is why `file` is not a field on Node or Span. But the design then
;; gave the run's property NO route to a rule at all, and `facts-of` took the source's CONTENTS
;; without its IDENTITY — so the knowledge died at a parameter list while `run-one` was holding
;; the path one expression away. Every Match a rule could assert carried a filename the rule
;; author had typed by hand.
;;
;; A rule that wants the filename joins this ONE fact; a rule that does not, ignores it. One
;; string per file, not one per node — the design's own argument, finally with a destination.
(:wat::core::defrecord :wat::grep::Source
  [file <- :wat::core::String])

;; Unreadable — "I could not read this file", the fact F1 was missing. A rule can join it and
;; reason about coverage; `run-one` ALSO prints it to stderr unconditionally, because an opt-in
;; fact does nothing for a consumer who does not know to opt in — the exact way today's silence
;; works. `reason`/`line`/`col` come straight off the parser's own `:wat::core::Error` cause
;; (`Error/message`, `Error/location` -> `:wat::kernel::Location/line`+`/col`); nothing is
;; invented here. See DESIGN-STONE-wat-grep-never-lies.md F1.
(:wat::core::defrecord :wat::grep::Unreadable
  [file   <- :wat::core::String
   reason <- :wat::core::String
   line   <- :wat::core::i64
   col    <- :wat::core::i64])

(:wat::core::defrecord :wat::grep::Facts
  [source     <- :wat::grep::Source
   nodes      <- (:wat::core::PersistentVector :- [:wat::grep::Node])
   named      <- (:wat::core::PersistentVector :- [:wat::grep::Named])
   spans      <- (:wat::core::PersistentVector :- [:wat::grep::Span])
   written    <- (:wat::core::PersistentVector :- [:wat::grep::Written])
   unreadable <- (:wat::core::PersistentVector :- [:wat::grep::Unreadable])])

;; ── internal walk plumbing (not part of the wat-grep contract; the walk's threading) ────
;; Moved verbatim from corpus-03's :fx::Acc / :fx::ChildAcc, renamed.

(:wat::core::defrecord :wat::grep::Acc
  [next-id <- :wat::core::i64
   nodes   <- (:wat::core::PersistentVector :- [:wat::grep::Node])
   named   <- (:wat::core::PersistentVector :- [:wat::grep::Named])
   spans   <- (:wat::core::PersistentVector :- [:wat::grep::Span])
   written <- (:wat::core::PersistentVector :- [:wat::grep::Written])])

;; per-level child accumulator: the walk's Acc plus this level's running index
(:wat::core::defrecord :wat::grep::ChildAcc
  [acc <- :wat::grep::Acc
   idx <- :wat::core::i64])

;; nameable? — the TOTAL guard in front of the partial `ast-name`.
(:wat::core::defn :wat::grep::nameable?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::if (:wat::core::= k "symbol") true
      (:wat::core::if (:wat::core::= k "keyword") true
        (:wat::core::= k "string")))))

;; structural? — does this node HAVE children to descend into?
(:wat::core::defn :wat::grep::structural?
  [node <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::contains?
      (:wat::core::HashSet :- [:wat::type::Infer] "list" "vector" "map" "set") k)))

;; walk — assign this node an id, emit its facts, then descend. Pre-order, so `parent` is always
;; already numbered when a child is reached.
(:wat::core::defn :wat::grep::walk
  [acc    <- :wat::grep::Acc
   node   <- :wat::WatAST
   parent <- :wat::core::i64
   index  <- :wat::core::i64]
  -> :wat::grep::Acc
  (:wat::core::let
    [id    (:wat::grep::Acc/next-id acc)
     kind  (:wat::core::ast-kind node)
     nodes (:wat::vector::conj (:wat::grep::Acc/nodes acc)
             (:wat::grep::Node :id id :parent parent :index index :kind kind))
     ;; THE GUARD: no name fact for an unnameable node. `ast-name` is never reached for one.
     named (:wat::core::if (:wat::grep::nameable? node)
             (:wat::vector::conj (:wat::grep::Acc/named acc)
               (:wat::grep::Named :id id :name (:wat::core::ast-name node)))
             (:wat::grep::Acc/named acc))
     ;; NO GUARD: extent-of is total (ast-span / ast-end-span are total). Every node gets a Span.
     ex    (:wat::grep::extent-of node)
     spans (:wat::vector::conj (:wat::grep::Acc/spans acc)
             (:wat::grep::Span :id id
                        :line     (:wat::grep::Extent/line ex)
                        :col      (:wat::grep::Extent/col ex)
                        :end-line (:wat::grep::Extent/end-line ex)
                        :end-col  (:wat::grep::Extent/end-col ex)))
     ;; THE GUARD, again: `written?` is exact — nameable AND single-line AND the span's width
     ;; equals the written name's length. `ast-name` is only called under the `nameable?` guard,
     ;; the same discipline `named` above already uses.
     written? (:wat::core::if (:wat::grep::nameable? node)
                (:wat::core::if (:wat::core::= (:wat::grep::Extent/line ex) (:wat::grep::Extent/end-line ex))
                  (:wat::core::=
                    (:wat::i64::- (:wat::grep::Extent/end-col ex) (:wat::grep::Extent/col ex))
                    (:wat::string::length (:wat::core::ast-name node)))
                  false)
                false)
     written (:wat::core::if written?
               (:wat::vector::conj (:wat::grep::Acc/written acc)
                 (:wat::grep::Written :id id
                            :line     (:wat::grep::Extent/line ex)
                            :col      (:wat::grep::Extent/col ex)
                            :end-line (:wat::grep::Extent/end-line ex)
                            :end-col  (:wat::grep::Extent/end-col ex)))
               (:wat::grep::Acc/written acc))
     acc'  (:wat::grep::Acc :next-id (:wat::i64::+ id 1) :nodes nodes :named named :spans spans :written written)]
    (:wat::core::if (:wat::grep::structural? node)
      (:wat::grep::ChildAcc/acc
        (:wat::core::foldl
          (:wat::core::fn [ca <- :wat::grep::ChildAcc  child <- :wat::WatAST] -> :wat::grep::ChildAcc
            (:wat::grep::ChildAcc
              :acc (:wat::grep::walk (:wat::grep::ChildAcc/acc ca) child id (:wat::grep::ChildAcc/idx ca))
              :idx (:wat::i64::+ (:wat::grep::ChildAcc/idx ca) 1)))
          (:wat::grep::ChildAcc :acc acc' :idx 0)
          (:wat::core::ast->children node)))
      acc')))

(:wat::core::defn :wat::grep::empty-acc [] -> :wat::grep::Acc
  (:wat::grep::Acc :next-id 1
            :nodes   (:wat::core::PersistentVector)
            :named   (:wat::core::PersistentVector)
            :spans   (:wat::core::PersistentVector)
            :written (:wat::core::PersistentVector)))

;; the pair `facts-of` pulls out of the ONE match on `read-string` — a Forms/Malformed match
;; must decide `acc` AND `unreadable` together, or the parse runs twice.
(:wat::core::defrecord :wat::grep::FactsOfResult
  [acc        <- :wat::grep::Acc
   unreadable <- (:wat::core::PersistentVector :- [:wat::grep::Unreadable])])

;; facts-of — every top-level form of one source string, walked into one fact base.
;;
;; `read-string` returns a FACED OUTCOME, not a bare vector — the no-hidden-failures law. A
;; string that will not parse is a RESULT the extractor carries, never a crash: Malformed yields
;; an EMPTY fact base (as before) PLUS the `Unreadable` fact that says so out loud — F1. The
;; cause is already in hand at the match arm; `Error/message` is the reason, `Error/location`'s
;; `:wat::kernel::Location` is the line/col, straight off the parser's own diagnostic (mirrors
;; `wat/fix.wat:351`'s ONE difference: fix.wat raises immediately, an APPLIER's contract; grep
;; reports and keeps going, a FINDER's contract — see run-one/run for where that fires).
;; ⚠ `path` is not decoration: it is the file's IDENTITY, and a signature that took only `src`
;; is exactly where the filename used to be lost. It is used for nothing but the Source fact and
;; the Unreadable fact's own `file`.
(:wat::core::defn :wat::grep::facts-of
  [path <- :wat::core::String
   src  <- :wat::core::String]
  -> :wat::grep::Facts
  (:wat::core::let
    [result (:wat::core::match (:wat::core::read-string src)
              ((:wat::core::ReadOutcome::Forms forms)
                (:wat::grep::FactsOfResult
                  :acc (:wat::grep::ChildAcc/acc
                         (:wat::core::foldl
                           (:wat::core::fn [ca <- :wat::grep::ChildAcc  form <- :wat::WatAST] -> :wat::grep::ChildAcc
                             (:wat::grep::ChildAcc
                               :acc (:wat::grep::walk (:wat::grep::ChildAcc/acc ca) form 0 (:wat::grep::ChildAcc/idx ca))
                               :idx (:wat::i64::+ (:wat::grep::ChildAcc/idx ca) 1)))
                           (:wat::grep::ChildAcc :acc (:wat::grep::empty-acc) :idx 0)
                           (:wat::core::ast->children forms)))
                  :unreadable (:wat::core::PersistentVector :- [:wat::grep::Unreadable])))
              ((:wat::core::ReadOutcome::Malformed __cause)
                (:wat::grep::FactsOfResult
                  :acc (:wat::grep::empty-acc)
                  :unreadable
                    (:wat::core::PersistentVector :- [:wat::grep::Unreadable]
                      (:wat::grep::Unreadable
                        :file   path
                        :reason (:wat::core::Error/message __cause)
                        :line   (:wat::kernel::Location/line (:wat::core::Error/location __cause))
                        :col    (:wat::kernel::Location/col  (:wat::core::Error/location __cause)))))))
     acc (:wat::grep::FactsOfResult/acc result)]
    (:wat::grep::Facts
      :source     (:wat::grep::Source :file path)
      :nodes      (:wat::grep::Acc/nodes acc)
      :named      (:wat::grep::Acc/named acc)
      :spans      (:wat::grep::Acc/spans acc)
      :written    (:wat::grep::Acc/written acc)
      :unreadable (:wat::grep::FactsOfResult/unreadable result))))

;; ── the ONE query — never written by a user; wat-grep owns exactly one query so the printer
;; is TOTAL, rendering exactly one type it fully knows. ───────────────────────────────────────

(:wat::rete::defquery :wat::grep::q-match
  :params []
  :when [(?fact <- :wat::grep::Match)])

;; ── the driver — :wat::grep::run ────────────────────────────────────────────────────
;; DESIGN: docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-grep-mode.md
;; BRIEF:  docs/arc/2026/06/278-rules-engine/BRIEF-STONE-the-grep-driver.md
;;
;; facts-as-records — Facts holds several DIFFERENTLY-TYPED vectors (Node/Named/Span/Written/
;; Unreadable); the single `with-overlay` call each file gets takes ONE `(PersistentVector :-
;; [:wat::core::Record])`, so all of them must be merged before that call, not after (a rule's
;; Node×Span join needs both present in the same insert). `PersistentVector/conj`'s element
;; position accepts any defrecord as a :wat::core::Record — the same coercion the DESIGN's own
;; probe uses to build a Record vector by hand (wat-grep-with-network-shape.wat's
;; `grep-one-file`) — so a fold of `conj` per sub-vector is the general shape for N sub-vectors
;; of differing concrete record types.
(:wat::core::defn :wat::grep::facts-as-records
  [facts <- :wat::grep::Facts]
  -> (:wat::core::PersistentVector :- [:wat::core::Record])
  (:wat::core::let
    [acc0 (:wat::core::PersistentVector :- [:wat::core::Record])
     acc1 (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])
                             n   <- :wat::grep::Node]
              -> (:wat::core::PersistentVector :- [:wat::core::Record])
              (:wat::vector::conj acc n))
            acc0
            (:wat::grep::Facts/nodes facts))
     acc2 (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])
                             nm  <- :wat::grep::Named]
              -> (:wat::core::PersistentVector :- [:wat::core::Record])
              (:wat::vector::conj acc nm))
            acc1
            (:wat::grep::Facts/named facts))
     acc3 (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])
                             sp  <- :wat::grep::Span]
              -> (:wat::core::PersistentVector :- [:wat::core::Record])
              (:wat::vector::conj acc sp))
            acc2
            (:wat::grep::Facts/spans facts))
     acc4 (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])
                             w   <- :wat::grep::Written]
              -> (:wat::core::PersistentVector :- [:wat::core::Record])
              (:wat::vector::conj acc w))
            acc3
            (:wat::grep::Facts/written facts))
     acc5 (:wat::core::foldl
            (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])
                             u   <- :wat::grep::Unreadable]
              -> (:wat::core::PersistentVector :- [:wat::core::Record])
              (:wat::vector::conj acc u))
            acc4
            (:wat::grep::Facts/unreadable facts))]
    ;; the ONE Source fact, last — a rule joins it to name the file it matched in.
    (:wat::vector::conj acc5 (:wat::grep::Facts/source facts))))

;; print-match — the ONE printer. It knows exactly one type because wat-grep owns exactly one
;; query; nothing here ranks, filters, or counts. `query-read`'s binding maps key a query's
;; params by "?name" (rules-corpus-03's own read of q-match: `(PersistentMap/get m "?fact")`).
(:wat::core::defn :wat::grep::print-match
  [binding <- :wat::core::PersistentMap]
  -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::str
      (:wat::core::Option/expect
        (:wat::map::get binding "?fact")
        "wat::grep::print-match: q-match binding has no ?fact"))))

;; run-one — one file, through the ALREADY-COMPILED network via `overlay`. `overlay` always
;; re-seeds from the network's compiled base (with-overlay's own contract), so this function
;; never has a prior file's session in scope to thread forward — the isolation the DESIGN calls
;; structural, not disciplined.
;;
;; Returns THIS file's `Unreadable` facts (0 or 1) rather than printing them directly — see
;; `run`'s header comment for why. `run-each`/`run` fold every file's answer into the pinned
;; contract (report every bad file, then exit non-zero at the END), never stopping early: a
;; finder that halted at the first bad file in a 1567-file corpus would hide the other 1566
;; answers, which is `fix.wat`'s (an APPLIER's) contract, not this one's.
(:wat::core::defn :wat::grep::run-one
  [overlay <- :wat::rete::Overlay
   path    <- :wat::core::String]
  -> (:wat::core::PersistentVector :- [:wat::grep::Unreadable])
  (:wat::core::let
    [facts       (:wat::grep::facts-of path (:wat::io::read-file path))
     records     (:wat::grep::facts-as-records facts)
     fired       (overlay records)
     matches     (:wat::rete::query fired (:wat::grep::q-match))
     ran-matches (:wat::core::run! :wat::grep::print-match matches)]
    (:wat::grep::Facts/unreadable facts)))

;; run-each — the loop. Identical recursive shape to every recorded stdin-harness codemod
;; (`wat-scripts/fixes/angle-brackets-to-binder.wat`'s `apply-each`): first path, recurse on
;; rest. No count, no header, no separator — silence for a file whose rules assert nothing is
;; the honest answer, not an error. Returns the CONCATENATION of every file's `Unreadable`
;; facts — the recursion always runs every path (never short-circuits on a bad one), so all of
;; them are collected before `run` decides whether to raise.
(:wat::core::defn :wat::grep::run-each
  [overlay <- :wat::rete::Overlay
   paths   <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::PersistentVector :- [:wat::grep::Unreadable])
  (:wat::core::if (:wat::core::empty? paths)
    (:wat::core::PersistentVector :- [:wat::grep::Unreadable])
    (:wat::vector::concat
      (:wat::grep::run-one overlay (:wat::core::first paths))
      (:wat::grep::run-each overlay (:wat::core::rest paths)))))

;; run — the driver. Reads ONE EDN vector of paths from stdin, compiles `rules` + the single
;; query `:wat::grep::q-match` ONCE (the driver compiles, so the driver holds the lease, in one
;; scope — `with-overlay` releases it when this call returns), then threads every file through
;; that one compiled network via `overlay`, each file re-seeded from the compiled base.
;;
;; ⛔ THE PINNED CONTRACT — F1: every bad file is collected (via `run-each`, above), and if ANY
;; file along the way was unreadable, THIS reports every one of them AND exits non-zero, once,
;; at the end. A run that skipped files silently did not fulfil its contract, and a zero exit on
;; an incomplete census is exactly the lie this stone kills.
;;
;; ⚠ ONE call to `eprintln`, not a print-per-file loop: `:wat::kernel::eprintln` is wat's PANIC
;; channel (`wat/kernel/diagnostics.wat:52`, `src/check.rs`'s "TERMINATING form" registration —
;; it emits to stderr THEN TERMINATES non-zero at runtime; there is no benign, non-terminating
;; stderr-write primitive in the substrate). A per-file `eprintln` inside `run-one` would die on
;; the FIRST bad file and hide every other answer — exactly `fix.wat`'s (an APPLIER's) contract,
;; and exactly what the pinned contract above forbids. So every file's `Unreadable` fact is
;; collected first (never printed early), and the ONE terminating call at the very end both
;; names every bad file (its payload is the whole vector) and produces the non-zero exit — the
;; two contractual requirements collapse into the one primitive built for exactly this.
(:wat::core::defn :wat::grep::run
  [rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])]
  -> :wat::core::nil
  (:wat::core::let
    [paths (:wat::core::match (:wat::kernel::readln)
             ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
             (:wat::kernel::ReadlnOutcome::Eof
               (:wat::kernel::assertion-failed! "wat::grep::run: readln: end of input"
                 :wat::core::None :wat::core::None))
             (:wat::kernel::ReadlnOutcome::Stopped
               (:wat::kernel::assertion-failed! "wat::grep::run: readln: stop requested"
                 :wat::core::None :wat::core::None)))
     bad
       (:wat::rete::with-overlay rules
         (:wat::core::PersistentVector :- [:wat::rete::Query] (:wat::grep::q-match))
         (:wat::core::fn [overlay <- :wat::rete::Overlay]
           -> (:wat::core::PersistentVector :- [:wat::grep::Unreadable])
           (:wat::grep::run-each overlay paths)))]
    (:wat::core::if (:wat::core::empty? bad)
      nil
      (:wat::kernel::eprintln bad))))
