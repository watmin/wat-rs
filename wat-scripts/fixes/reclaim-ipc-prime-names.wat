;; wat-scripts/fixes/reclaim-ipc-prime-names.wat — arc 278 "0z": the IPC/kernel primes reclaim
;; their plain names. Every non-prime counterpart is annihilated and PROVEN dead (each exits 1
;; with UnknownFunction on a real run; the whole-tree sweep reads zero across src/ and corpus).
;;
;; ⚠ PREFIX, NOT EXACT — measured, not assumed. `rename-keyword-exact` keys on the FULL ast-name,
;; and a parametric use carries its params INSIDE the keyword token, so an exact rule for
;; `:wat::kernel::Peer'` leaves `:wat::kernel::Peer'<S,R>` untouched — proven on a scratch file
;; where exact left BOTH parametric uses byte-identical. Across 249 corpus files that would have
;; renamed every bare use and silently stranded every parametric one. `rename-keyword-prefix` is
;; boundary-aware and catches both — and, verified in the same run, renaming `Peer'` does NOT eat
;; `ThreadSelfPeer'`, because a prefix must match from the START of the keyword. Order is
;; therefore irrelevant; the list below is grouped for reading, not for precedence.
;;
;; Idempotent by construction: this DROPS a trailing `'`, so a re-run matches nothing.
;;
;; NOT here, each for its own reason:
;;   peer-pair'   ANNIHILATED (890b60a4) — a deletion, not a rename
;;   readln'      STRUCTURALLY REQUIRED: `:wat::kernel::readln` is a live defmacro
;;                (wat/kernel/readln.wat:59) that EXPANDS TO `readln'`. Same name, two
;;                forms — drop the `'` and the macro collides with the verb it expands into.
;;   Frame'       the positional-CONSTRUCTOR idiom — Frame is the record type, Frame' builds one
;;   fire-rules' / fire-once' / fire-rules-explain' / step-payload'
;;                the rete DUAL-IMPL — unprimed is the wat ORACLE, primed is the native kernel,
;;                differential-tested against each other (R9/R22). Never collapse.
;;
;; The four defservice-emitted kernel internals ARE included (builder-ruled): allow' / deny' /
;; retag-op' / serve-dispatch-op' come out of the `defservice` quasiquote (wat/service.wat:1245/
;; 1264/1293/1299) — but unlike readln', NO same-name non-prime macro exists for any of them
;; (`defmacro :wat::kernel::allow` etc. = 0), so nothing collides and the `'` marks nothing.
;;
;; ⚠ THE STASH-DANCE APPLIES (wat/fix.wat header) — ships alongside a src/ change that makes the
;; old form illegal: stash the rust change, build the OLD checker, rewrite the corpus, pop,
;; rebuild. GENERATE the path list — never hand-type it; a missed file breaks the build.
;; Dry-run on a COPY first and `diff`.
;;
;; Usage:
;;   printf '[…EVERY path…]\n' | ./target/release/wat ./wat-scripts/fixes/reclaim-ipc-prime-names.wat

;; The migration as DATA — one line per name. Adding a name is one line; nothing to re-balance.
;; (An earlier draft nested 24 `rename-keyword-prefix` calls into a staircase; the closing-paren
;; count stopped being eyeballable and was wrong twice. A fold over a list is the honest form.)
(:wat::core::defn :user::renames [] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::Vector :(wat::core::String,wat::core::String)
    ;; spawn family
    (:wat::core::Tuple ":wat::kernel::spawn-program'"     ":wat::kernel::spawn-program")
    (:wat::core::Tuple ":wat::kernel::spawn-thread'"      ":wat::kernel::spawn-thread")
    (:wat::core::Tuple ":wat::kernel::spawn-process'"     ":wat::kernel::spawn-process")
    ;; wire verbs
    (:wat::core::Tuple ":wat::kernel::try-send'"          ":wat::kernel::try-send")
    (:wat::core::Tuple ":wat::kernel::send'"              ":wat::kernel::send")
    (:wat::core::Tuple ":wat::kernel::recv-all-loop'"     ":wat::kernel::recv-all-loop")
    (:wat::core::Tuple ":wat::kernel::recv-all'"          ":wat::kernel::recv-all")
    (:wat::core::Tuple ":wat::kernel::recv'"              ":wat::kernel::recv")
    (:wat::core::Tuple ":wat::kernel::select'"            ":wat::kernel::select")
    (:wat::core::Tuple ":wat::kernel::poll'"              ":wat::kernel::poll")
    (:wat::core::Tuple ":wat::kernel::close'"             ":wat::kernel::close")
    ;; socket tier
    (:wat::core::Tuple ":wat::kernel::connect'"           ":wat::kernel::connect")
    (:wat::core::Tuple ":wat::kernel::accept'"            ":wat::kernel::accept")
    (:wat::core::Tuple ":wat::kernel::listener'"          ":wat::kernel::listener")
    (:wat::core::Tuple ":wat::kernel::Listener'"          ":wat::kernel::Listener")
    (:wat::core::Tuple ":wat::kernel::Address'"           ":wat::kernel::Address")
    ;; peer types (parametric — the reason this is a PREFIX rename)
    (:wat::core::Tuple ":wat::kernel::ThreadSelfPeer'"    ":wat::kernel::ThreadSelfPeer")
    (:wat::core::Tuple ":wat::kernel::Peer'"              ":wat::kernel::Peer")
    (:wat::core::Tuple ":wat::kernel::Thread'"            ":wat::kernel::Thread")
    (:wat::core::Tuple ":wat::kernel::Process'"           ":wat::kernel::Process")
    ;; defservice-emitted kernel internals
    (:wat::core::Tuple ":wat::kernel::serve-dispatch-op'" ":wat::kernel::serve-dispatch-op")
    (:wat::core::Tuple ":wat::kernel::retag-op'"          ":wat::kernel::retag-op")
    (:wat::core::Tuple ":wat::kernel::allow'"             ":wat::kernel::allow")
    (:wat::core::Tuple ":wat::kernel::deny'"              ":wat::kernel::deny")))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String
                     pr  <- :(wat::core::String,wat::core::String)] -> :wat::core::String
      (:wat::fix::rename-keyword-prefix (:wat::core::first pr) (:wat::core::second pr) acc))
    src
    (:user::renames)))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))
