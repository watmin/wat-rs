;; wat-scripts/fixes/tuple-parens-to-binder.wat — arc 109 "the comma dies in the reader" cascade.
;;
;; Rewrites the retired `:(A,B,C)` tuple-literal keyword shape into the `:-` binder / reference
;; spelling already live in the stdlib (`wat/bracket.wat`, `wat/spawn.wat` use
;; `(:wat::core::Tuple :- [:wat::core::i64 O])` throughout):
;;
;;   :(wat::core::i64,wat::core::bool)   ->   (:wat::core::Tuple :- [:wat::core::i64 :wat::core::bool])
;;   :(wat::core::i64,T,wat::core::String) -> (:wat::core::Tuple :- [:wat::core::i64 T :wat::core::String])
;;
;; WHY THIS SCRIPT CANNOT USE `fix.wat`'s STRUCTURAL WALK (`read-string` + `with-children`), unlike
;; most recorded migrations: the Rust change THIS SCRIPT'S OWN STONE SHIPPED ALONGSIDE
;; (`crates/wat-reader/src/lexer.rs` — a comma can no longer enter a keyword body, at any bracket
;; depth) is exactly what makes `:(A,B,C)` illegal. `read-string` on a file that STILL CONTAINS the
;; retired shape now fails to lex the WHOLE FILE before this script's logic ever runs — the same
;; "converter renders through a walled door" trap `angle-brackets-to-binder.wat`'s header documents
;; for its OWN stone, except here the wall blocks the READ side too, not just the render side (that
;; script's `render-ref`/`render-args`/`split-top-level`/`scan-for-close` never touch the type
;; parser, so they are REUSED VERBATIM below — only the WALK is new). The STASH-DANCE (revert the
;; Rust wall, rebuild, run the codemod against the old permissive reader, restore the wall) is the
;; textbook answer, but it is explicitly OUT OF BOUNDS for this stone's rider brief ("Do NOT commit,
;; push, stash or amend"). So this script walks the RAW FILE TEXT directly — a small hand-rolled
;; scanner that tracks string-literal and line-comment state (so it never mistakes prose like
;; `;; :(A,B,C) tuple type` for code) and, outside those, bracket depth (so a comma nested inside a
;; tuple element's own `<...>`/`(...)` is not mistaken for a top-level separator) — never a call
;; through `read-string` or any other walled door.
;;
;; SCOPE: only `:(` keyword bodies (colon DIRECTLY followed by an open paren) with a TOP-LEVEL
;; comma inside are rewritten. `:()` (unit/empty tuple) and `:(T)` (single-element, no comma) are
;; UNTOUCHED — they never used the retired comma permission, so the wall never touched them
;; (additive-refusal only, brief STOP-2). `:fn(...)`/`:wat::core::Fn(...)` function-type keywords
;; are a DIFFERENT shape (`:f`, not `:(` — the colon is followed by `f`, never `(`) and are outside
;; this scanner's match set entirely; the corpus census for this stone's gated directories
;; (`wat-scripts/`, `wat-tests/`, `tests/`) found zero comma-carrying `:fn(...)->...` sites, so no
;; renderer for that shape is included here (R21 — build exactly what the census requires; invent
;; nothing extra).
;;
;; Idempotent: after rewriting, the site is a `(:wat::core::Tuple :- […])` LIST, not a `:(`-leading
;; keyword — `find-first-open-paren-keyword` no longer matches it, so a second pass is a
;; byte-identical no-op.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB" …]\n' | cargo wat ./wat-scripts/fixes/tuple-parens-to-binder.wat
;;
;; Dry-run on a /tmp copy FIRST (R21 — mandatory before any corpus application):
;;   cp tests/foo.wat /tmp/pilot.wat
;;   printf '["/tmp/pilot.wat"]\n' | cargo wat ./wat-scripts/fixes/tuple-parens-to-binder.wat
;;   diff tests/foo.wat /tmp/pilot.wat

;; ── the tuple -> binder renderer (pure string surgery, reused verbatim from
;;    angle-brackets-to-binder.wat — none of it touches the type parser) ────────────────────────

(:wat::core::defn :user::open-bracket? [c <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= c "<") true (:wat::core::= c "(")))
(:wat::core::defn :user::close-bracket? [c <- :wat::core::String] -> :wat::core::bool
  (:wat::core::if (:wat::core::= c ">") true (:wat::core::= c ")")))

;; scan-for-close — index of the close bracket matching the open bracket just consumed
;; (depth starts at 1, `i` is the position right after that open bracket).
(:wat::core::defn :user::scan-for-close
  [s <- :wat::core::String i <- :wat::core::i64 depth <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let [c (:wat::string::subs s i (:wat::i64::+ i 1))]
    (:wat::core::if (:user::open-bracket? c)
      (:user::scan-for-close s (:wat::i64::+ i 1) (:wat::i64::+ depth 1))
      (:wat::core::if (:user::close-bracket? c)
        (:wat::core::if (:wat::i64::= depth 1)
          i
          (:user::scan-for-close s (:wat::i64::+ i 1) (:wat::i64::- depth 1)))
        (:user::scan-for-close s (:wat::i64::+ i 1) depth)))))

;; split-top-level — split `s` on commas at depth 0 (nested-bracket commas are NOT split points).
(:wat::core::defn :user::split-top-level
  [s <- :wat::core::String i <- :wat::core::i64 depth <- :wat::core::i64 start <- :wat::core::i64
   acc <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::i64::>= i (:wat::string::length s))
    (:wat::core::conj acc (:wat::string::trim (:wat::string::subs s start i)))
    (:wat::core::let [c (:wat::string::subs s i (:wat::i64::+ i 1))]
      (:wat::core::if (:user::open-bracket? c)
        (:user::split-top-level s (:wat::i64::+ i 1) (:wat::i64::+ depth 1) start acc)
        (:wat::core::if (:user::close-bracket? c)
          (:user::split-top-level s (:wat::i64::+ i 1) (:wat::i64::- depth 1) start acc)
          (:wat::core::if (:wat::core::if (:wat::core::= c ",") (:wat::i64::= depth 0) false)
            (:user::split-top-level s (:wat::i64::+ i 1) depth (:wat::i64::+ i 1)
              (:wat::core::conj acc (:wat::string::trim (:wat::string::subs s start i))))
            (:user::split-top-level s (:wat::i64::+ i 1) depth start acc)))))))

;; render-ref — full REFERENCE-role rendering of a `:(`-leading keyword's TEXT (leading colon
;; included, e.g. ":(wat::core::i64,wat::core::String)") -> "(:wat::core::Tuple :- [args])". A
;; tuple ALWAYS appears in reference position (never as a `defrecord`-style decl name — you cannot
;; name a declaration `:(A,B)`), so there is no decl/ref role split to make here (unlike the angle-
;; bracket sibling, which has one).
;; render-one-arg — one split-top-level segment. Three shapes:
;;   - starts with "(" — a NESTED tuple that split-top-level peeled its OWN leading colon off of
;;     (split-top-level splits on top-level commas inside the OUTER `:(...)`; a nested tuple
;;     element written `:((A,B),C)` arrives here as the bare group text `(A,B)`, no colon, because
;;     the colon belongs to the OUTER keyword, not this inner group). Prepend the colon back and
;;     recurse through `render-tuple` so arbitrarily-nested tuples resolve outside-in.
;;     (`tests/collection/probe_arc216_stone7_tuple_roundtrip.wat`'s p4-rt-nested case.)
;;   - contains "::" — a namespaced concrete type; colon-prefix it (no further nesting possible:
;;     a `::`-qualified path never itself contains an unrendered `<...>`/`(...)` group post-arc-③).
;;   - otherwise — a bare short identifier (K, V, T, Xt) — a lexical type VARIABLE; verbatim.
(:wat::core::defn :user::render-one-arg
  [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::string::starts-with? s "(")
    (:user::render-tuple (:wat::string::concat ":" s))
    (:wat::core::if (:wat::string::contains? s "::")
      (:wat::string::concat ":" s)
      s)))

(:wat::core::defn :user::render-args
  [args <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::String
  (:wat::string::join " "
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) a <- :wat::core::String]
        -> (:wat::core::Vector :- [:wat::core::String])
        (:wat::core::conj acc (:user::render-one-arg a)))
      (:wat::core::Vector :- [:wat::core::String])
      args)))

(:wat::core::defn :user::render-tuple
  [kw-text <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [inner         (:wat::string::subs kw-text 2 (:wat::i64::- (:wat::string::length kw-text) 1))
                    args          (:user::split-top-level inner 0 0 0 (:wat::core::Vector :- [:wat::core::String]))
                    rendered-args (:user::render-args args)]
    (:wat::string::interpolate "(:wat::core::Tuple :- [{a}])" :a rendered-args)))

;; ── the raw-text scanner — replaces fix.wat's `read-string`-based walk ──────────────────────────
;;
;; Walks `text` byte-index by byte-index (all corpus content here is ASCII in the positions that
;; matter — keyword bodies, brackets, commas — matching `is_symbol_continue`'s own ASCII-only
;; keyword-body contract), tracking two pieces of state a real lexer would also track:
;;   - `in-string?` — inside a `"..."` string literal; `;` and `:(` are inert there.
;;   - line comments (`;` to end of line) — skipped inline, never scanned for `:(`.
;; `\` outside a string escapes the NEXT character unconditionally (covers both string escapes
;; like `\"` and char literals like `\;`), so neither is mistaken for a comment/string boundary.

(:wat::core::defn :user::char-at
  [s <- :wat::core::String i <- :wat::core::i64] -> :wat::core::String
  (:wat::string::subs s i (:wat::i64::+ i 1)))

;; has-top-level-comma? — does `inner` (a tuple's paren-interior text) contain a depth-0 comma?
;; Reuses split-top-level: more than one segment means a real separator fired.
(:wat::core::defn :user::has-top-level-comma?
  [inner <- :wat::core::String] -> :wat::core::bool
  (:wat::i64::> (:wat::core::count (:user::split-top-level inner 0 0 0 (:wat::core::Vector :- [:wat::core::String]))) 1))

;; scan — the walk. `text` never mutates; `edits` accumulates in ASCENDING offset order (the walk
;; is strictly left-to-right), reversed once at the top level before `fix-text-apply` (which wants
;; right-to-left, per its own docstring — same convention `angle-brackets-to-binder.wat` uses).
(:wat::core::defn :user::scan
  [text <- :wat::core::String i <- :wat::core::i64 len <- :wat::core::i64
   in-string? <- :wat::core::bool
   edits <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::i64::>= i len)
    edits
    (:wat::core::let [c (:user::char-at text i)]
      (:wat::core::if (:wat::core::= c "\\")
        ;; Escape: swallow this char and the next, unconditionally (string- or char-literal escape).
        (:user::scan text (:wat::i64::+ i 2) len in-string? edits)
        (:wat::core::if in-string?
          (:wat::core::if (:wat::core::= c "\"")
            (:user::scan text (:wat::i64::+ i 1) len false edits)
            (:user::scan text (:wat::i64::+ i 1) len true edits))
          (:wat::core::if (:wat::core::= c "\"")
            (:user::scan text (:wat::i64::+ i 1) len true edits)
            (:wat::core::if (:wat::core::= c ";")
              (:user::scan text (:user::skip-to-eol text (:wat::i64::+ i 1) len) len false edits)
              (:wat::core::if (:wat::core::if (:wat::core::= c ":")
                                (:wat::core::= (:user::char-at text (:wat::i64::+ i 1)) "(")
                                false)
                (:user::scan-tuple-site text i len edits)
                (:user::scan text (:wat::i64::+ i 1) len false edits)))))))))

;; skip-to-eol — advance `i` to the index right after the next `\n` (or to `len` at EOF).
(:wat::core::defn :user::skip-to-eol
  [text <- :wat::core::String i <- :wat::core::i64 len <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i len)
    len
    (:wat::core::if (:wat::core::= (:user::char-at text i) "\n")
      (:wat::i64::+ i 1)
      (:user::skip-to-eol text (:wat::i64::+ i 1) len))))

;; scan-tuple-site — `text[i]` is the `:` of a `:(` keyword start. Find its matching close paren,
;; decide (top-level comma?) whether it is a retired tuple site, record an edit iff so, and resume
;; the walk right after the whole keyword either way (never re-enter what was just consumed).
(:wat::core::defn :user::scan-tuple-site
  [text <- :wat::core::String i <- :wat::core::i64 len <- :wat::core::i64
   edits <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let [close (:user::scan-for-close text (:wat::i64::+ i 2) 1)
                    inner (:wat::string::subs text (:wat::i64::+ i 2) close)
                    next-i (:wat::i64::+ close 1)]
    (:wat::core::if (:user::has-top-level-comma? inner)
      (:wat::core::let [old-text (:wat::string::subs text i next-i)
                        new-text (:user::render-tuple old-text)]
        (:user::scan text next-i len false (:wat::core::conj edits (:wat::core::Tuple i old-text new-text))))
      (:user::scan text next-i len false edits))))

(:wat::core::defn :user::convert
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [all-edits (:user::scan src 0 (:wat::string::length src) false
                                 (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))
                    rev-edits (:wat::core::reverse all-edits)]
    (:wat::fix::fix-text-apply src rev-edits)))

;; ── file/stdin harness — identical shape to every recorded migration ────────────────────────────
(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::convert (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[tuple-parens-to-binder] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
