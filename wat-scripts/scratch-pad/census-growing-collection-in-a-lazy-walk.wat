;; census-growing-collection-in-a-lazy-walk.wat — stone 118.B8 Part 3.
;;
;; ⛔ WHY THIS EXISTS: stone 118.B3 named a SHAPE — "a growing collection threaded through a lazy
;; walk" — called the census "Owed, tracked", and it was tracked NOWHERE. `distinct` is the known
;; instance; nobody ever looked for siblings. B8 discharges that.
;;
;; ⛔ WHY IT IS NOT A GREP. While scoping B8 I ran an awk pass that split on blank lines and it
;; returned exactly one hit. THAT NUMBER IS NOT ADMISSIBLE — splitting a nested-paren language on
;; blank lines is a boundary heuristic, and this project has been wrong counting structured source
;; with a pattern five separate times.
;; `[[feedback_three_boundary_errors_need_a_reader_not_a_fourth_pattern]]`
;;
;; ★ THE STRUCTURAL PREDICATE — the MECHANISM, not the type declaration:
;;
;;   a `defn`/`defclause` named N whose body contains a call to N, where at least one ARGUMENT of
;;   that self-call is a form whose head is a per-element GROWTH verb (`conj` / `assoc` family).
;;
;; That is exactly `distinct-walk`'s shape:
;;     (:wat::core::distinct-walk (:wat::core::conj seen value) rest)
;;
;; It deliberately does NOT read the parameter's declared type. The declared type is a claim; the
;; `conj` in the recursive argument is the mechanism. Reading types would also have required
;; parsing `<- (:wat::core::HashSet :- [T])` out of a param vector — a second boundary problem inside the
;; instrument built to avoid the first.
;;
;; ★ THE DISCRIMINATOR — an accumulator grown inside an EAGER fold is ORDINARY (one copy alive at a
;; time). The harmful version is the LAZY one: n live cells each pinning an independent copy, which
;; is what made `(into [] (distinct (range 0 8000)))` exceed 2 GB before 118.B3. So every hit also
;; reports whether its enclosing fn is a lazy walker (`stream::lazy` / `stream::next` in the body).
;; LAZY hits are the class. EAGER hits are context, and reporting them is what makes a count of one
;; falsifiable rather than merely small.
;;
;; ⚠ THE DISCRIMINATOR IS CELL CONSTRUCTION, NOT CONSUMPTION — corrected 2026-08-19 DURING this
;; census, by a hit I had not predicted. My first predicate asked whether the body mentions
;; `stream::next`, and it labelled BOTH hits LAZY: `distinct-walk` (which wraps in `stream::lazy`
;; and emits `stream::cons`, so n deferred cells each pin an independent copy of `seen` — the O(n^2)
;; that OOM'd before 118.B3) and `stream->pvec-spec` (the eager drain, tail-recursive, one copy
;; alive, whose accumulator IS its output). `stream::next` is how you CONSUME a stream and is common
;; to both. The harmful property is BUILDING a deferred cell that captures the accumulator, so the
;; predicate is `stream::lazy` / `stream::cons`.
;; `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`
;;
;; ⚠ THIS OVER-REPORTS BY DESIGN. A prefix-matched self-call (`:foo` vs `:foobar`) can score. The
;; declared name is printed on every hit so a false positive is visible on sight. For a census the
;; honest error direction is toward review, never toward a miss.
;;   `[[feedback_a_worklist_filter_is_a_claim_about_what_you_expect]]`
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/seq.wat" "wat/rete.wat"]\n' | ./target/release/wat \
;;     wat-scripts/scratch-pad/census-growing-collection-in-a-lazy-walk.wat

(:wat::core::defn :census::src
  [form <- :wat::WatAST] -> :wat::core::String
  (:wat::core::ast->source form))

(:wat::core::defn :census::kids
  [form <- :wat::WatAST] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::into [] (:wat::core::ast->children form)))

;; The head keyword of a list form, or "" for an atom / empty list.
(:wat::core::defn :census::head
  [form <- :wat::WatAST] -> :wat::core::String
  (:wat::core::let
    [ch (:census::kids form)]
    (:wat::core::if (:wat::core::empty? ch) "" (:census::src (:wat::core::first ch)))))

;; A per-element GROWTH verb: the container-grows-by-one family. `into`/`concat` are BULK and
;; deliberately excluded — they are not the per-element accumulation this class is about.
(:wat::core::defn :census::is-growth-verb?
  [name <- :wat::core::String] -> :wat::core::bool
  (:wat::core::or
    (:wat::core::or
      (:wat::core::= name ":wat::core::conj")
      (:wat::core::= name ":wat::core::assoc"))
    (:wat::core::or
      (:wat::core::or
        (:wat::core::= name ":wat::core::HashSet/conj")
        (:wat::core::= name ":wat::core::PersistentVector/conj"))
      (:wat::core::or
        (:wat::core::= name ":wat::core::HashMap/assoc")
        (:wat::core::= name ":wat::core::PersistentMap/assoc")))))

;; Does this form have ANY argument that is a growth-verb call?
(:wat::core::defn :census::has-growth-arg?
  [form <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool child <- :wat::WatAST] -> :wat::core::bool
      (:wat::core::or acc (:census::is-growth-verb? (:census::head child))))
    false
    (:wat::core::into [] (:wat::core::drop (:census::kids form) 1))))

;; Is `call-head` a self-call of the fn declared as `declared`? `declared` may carry type params
;; (`:wat::core::distinct-walk :- [T]`) while the call site never does.
(:wat::core::defn :census::is-self-call?
  [declared <- :wat::core::String call-head <- :wat::core::String] -> :wat::core::bool
  (:wat::core::and
    (:wat::core::not (:wat::core::= call-head ""))
    (:wat::core::String/starts-with? declared call-head)))

;; Walk a fn body hunting self-calls that carry a grown accumulator.
(:wat::core::defn :census::hunt
  [path <- :wat::core::String declared <- :wat::core::String form <- :wat::WatAST]
  -> :wat::core::nil
  (:wat::core::do
    (:wat::core::if
      (:wat::core::and
        (:census::is-self-call? declared (:census::head form))
        (:census::has-growth-arg? form))
      (:wat::kernel::println
        (:wat::string::concat "  HIT  "
          (:wat::string::concat path
            (:wat::string::concat "  ::  "
              (:wat::string::concat declared
                (:wat::string::concat "  ::  " (:census::src form)))))))
      nil)
    (:wat::core::run!
      (:wat::core::fn [c <- :wat::WatAST] -> :wat::core::nil (:census::hunt path declared c))
      (:census::kids form))))

;; Does this fn body walk lazily? `stream::lazy` wraps a deferred cell; `stream::next` pulls one.
;; Either makes the enclosing fn a walker whose accumulator can be pinned per-cell.
(:wat::core::defn :census::is-lazy-body?
  [form <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let
    [s (:census::src form)]
    (:wat::core::or
      (:wat::core::String/contains? s ":wat::stream::lazy")
      (:wat::core::String/contains? s ":wat::stream::cons"))))

;; At every form: if it declares a fn, hunt its body for the shape; then descend regardless.
(:wat::core::defn :census::walk
  [path <- :wat::core::String form <- :wat::WatAST] -> :wat::core::nil
  (:wat::core::let
    [h  (:census::head form)
     ch (:census::kids form)]
    (:wat::core::do
      (:wat::core::if
        (:wat::core::and
          (:wat::core::or
            (:wat::core::= h ":wat::core::defn")
            (:wat::core::= h ":wat::core::defclause"))
          (:wat::core::> (:wat::core::length ch) 2))
        (:wat::core::let
          [declared (:census::src (:wat::core::nth ch 1))]
          (:wat::core::do
            (:wat::kernel::println
              (:wat::string::concat
                (:wat::core::if (:census::is-lazy-body? form) "  [LAZY] " "  [eager] ")
                declared))
            (:census::hunt path declared form)))
        nil)
      (:wat::core::run!
        (:wat::core::fn [c <- :wat::WatAST] -> :wat::core::nil (:census::walk path c))
        ch))))

(:wat::core::defn :census::file
  [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::string::concat "== " path))
    (:census::walk path
      (:wat::core::match (:wat::core::read-string (:wat::io::read-file path))
        ((:wat::core::ReadOutcome::Forms __forms) __forms)
        ((:wat::core::ReadOutcome::Malformed __cause)
          (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause)
            :wat::core::None :wat::core::None))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::run!
    (:wat::core::fn [p <- :wat::core::String] -> :wat::core::nil (:census::file p))
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
