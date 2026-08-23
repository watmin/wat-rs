;; wat-scripts/fixes/service-locus-to-user-rendezvous.wat — arc 278, the namespacing wall's
;; `Reserved` arm, made honest. Self-hosted: no hand-edit of wat source (use-the-tool).
;;
;; THE FLAW THE WALL FOUND (not a naming nit — a real design flaw):
;;   `wat/spawn.wat`'s process `launch` prepends `(def :wat::spawn::service-locus (process))` into
;;   the CHILD's program forms, and the child's agnostic `:user::main` binds on that coordinate
;;   (`wat/service.wat:1710` reads it back through `listener`). But privilege does NOT survive a
;;   process boundary: by the time the child freezes those forms they are the post-`register_defines`
;;   USER residue, so a `def` minting into the RESERVED `:wat::` tree is exactly what
;;   `resolve::gate` -> `Registration::Reserved` exists to refuse. It was invisible only because
;;   scalar `def` never reached the gate (the hole this arc's stone closes).
;;
;; THE FIX — move it to the tree it always belonged in. `:user::` is not "a user's namespace"; it is
;; the RENDEZVOUS COORDINATE SPACE (`wat/bracket.wat:22` — "not a user's namespace; a rendezvous
;; space"), direction-agnostic. A parent-planted coordinate that a child resolves at startup IS the
;; definition, and the precedent is one file over and identical in shape:
;;   `:user::bracket::work-fn` — `bracket.wat:207` emits `(def :user::bracket::work-fn (fn …))` into
;;   child forms; `:289` mints the same coordinate by string; the child's generated `:user::main`
;;   passes its VALUE in. `service-locus` is that, for the transport literal.
;;
;;   :wat::spawn::service-locus  ->  :user::spawn::service-locus
;;
;; PREFIX, not EXACT (arc 278 24t): `rename-keyword-exact` keys on the FULL ast-name, so any
;; qualified or parametric use would be left byte-identical and the migration would be a scattered
;; half. Prefix catches every form and is still boundary-safe here (no longer name shares the head).
;; Idempotent BY CONSTRUCTION: the rewrite REMOVES every `:wat::spawn::service-locus`, so a re-run
;; matches nothing (this is a prefix SWAP, not an append — cf. `rename-keyword-prefix`'s
;; non-idempotence for the append-`'` case that forced `rename-keyword-exact` into existence).
;;
;; SURFACES ENUMERATED before the run (arc 278 24t — a rename touches five, this tool reaches one
;; and a half). Measured on the corpus, whole-tree, all extensions:
;;   1. `.wat` keyword forms ............ 2 code sites (wat/spawn.wat, wat/service.wat) — THIS TOOL
;;   2. `.wat` strings BUILDING keywords . 0 (no `keyword/from-string` / interpolate / concat / split)
;;   3. other extensions (.wat.bad/.disabled/.expr/.intueri) ... 0
;;   4. `src/**/*.rs` literals (both `":wat::…"` and bare `"wat::…"` head forms) ... 0
;;   5. `tests/**/*.rs` literals ......... 0
;; The 4 prose COMMENT lines naming the old coordinate (spawn.wat:489,491 · service.wat:1666,1668)
;; are invisible to a form-tree codemod BY CONSTRUCTION and are the manual tail.
;; NOT a hit: `wat-tests/service-locus-parity.wat` — a FILENAME carrying the substring (arc 272
;; locus parity); it does not reference the coordinate.
;;
;; Usage (one EDN vector of EVERY path on stdin — list them ALL):
;;   printf '["wat/spawn.wat" "wat/service.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/service-locus-to-user-rendezvous.wat

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix
    ":wat::spawn::service-locus" ":user::spawn::service-locus"
    src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[rendezvous] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
