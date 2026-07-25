;; wat/kernel/services/stdio-primes.wat — arc 170 stdio-as-defservice, PHASE 1.
;;
;; The three stdio streams reborn as `defservice`s — the PRIMES. They COEXIST with the hand-rolled
;; path (the old :wat::kernel::services::Std{In,Out,Err}Service/handle fns + the Rust
;; spawn_service_peer loop + the five eval_kernel_* verbs). Nothing is flipped or deleted in Phase 1:
;; the five caller verbs still route to the OLD path; these primes are freeze-bootstrapped on the real
;; fds and proven on CONTROLLED pipe fds, but no caller reaches them yet (Strikes 2–4).
;;
;; THE FD LIVES IN `:ephemeral` — an impure IOWriter/IOReader, thread-owned, born INSIDE `:init`.
;; It is NEVER an :init PARAMETER: init params ride the generated `Admin` enum, which is
;; unconditionally `:wat::enum::Pure` (it must cross the fork wire for process-tier services), so an
;; impure field there trips the containment wall (arc 293.W ImpureVariantFieldInPureEnum). Instead
;; `:init` takes the PURE fd NUMBER (an i64, rides Admin::Init clean) and materializes the handle in
;; its body via the kernel-restricted `(:wat::io::IOWriter/from-fd fd)` / `IOReader/from-fd`
;; (dup-then-own — the service owns a dup, never the real fd 0/1/2). This mirrors the sift service's
;; connect'-inside-init pattern (query.wat): the live resource is BORN inside init from a pure seed.
;;
;; The per-op message records are convention-named `<Surface>::<PascalOp>Request/Response` — the
;; `defservice :satisfies` macro synthesizes req-ty/resp-ty from the OP name (write-line → WriteLine…,
;; read-line → ReadLine…), exactly as the counter/query exemplars.
;;
;; Loads AFTER wat/service.wat (defservice) and wat/io.wat (the from-fd builtins' neighbourhood).
;; Baked stdlib → may define under `:wat::` (bypasses the reserved-prefix gate; expansion-born
;; `…/start` companions register via the stdlib-privilege bypass, as query/mem.wat relies on).

;; ─── StdOut' ──────────────────────────────────────────────────────────────────────────────────
(:wat::core::defsurface :wat::kernel::StdOut' :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :wat::kernel::StdOut'::WriteLineRequest [line <- :wat::core::String])
   (:wat::core::defenum :wat::kernel::StdOut'::WriteLineResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(write-line [self <- :wat::kernel::StdOut'  req <- :wat::kernel::StdOut'::WriteLineRequest]
     -> :wat::kernel::StdOut'::WriteLineResponse :max-request-bytes 524288)])

(:wat::service::defservice :wat::kernel::stdout-svc'
  :satisfies :wat::kernel::StdOut'
  :durable   []
  :ephemeral [out <- :wat::io::IOWriter]
  :init (:wat::core::fn [record <- :wat::kernel::stdout-svc'::Record  fd <- :wat::core::i64]
          -> :wat::kernel::stdout-svc'::State
          (:wat::kernel::stdout-svc'::State :durable record :out (:wat::io::IOWriter/from-fd fd)))
  :impls
  [(write-line [s req]
     (:wat::core::let [_bytes (:wat::io::IOWriter/writeln (:wat::kernel::stdout-svc'::State/out s)
                                (:wat::kernel::StdOut'::WriteLineRequest/line req))]
       (:wat::service::Outcome::Reply s (:wat::kernel::StdOut'::WriteLineResponse::Ok))))])

;; ─── StdErr' (write-serializer — identical to StdOut'; the eprintln TERMINATE stays the verb's own
;;     act, never the service loop's — see DESIGN §3) ────────────────────────────────────────────
(:wat::core::defsurface :wat::kernel::StdErr' :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :wat::kernel::StdErr'::WriteLineRequest [line <- :wat::core::String])
   (:wat::core::defenum :wat::kernel::StdErr'::WriteLineResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(write-line [self <- :wat::kernel::StdErr'  req <- :wat::kernel::StdErr'::WriteLineRequest]
     -> :wat::kernel::StdErr'::WriteLineResponse :max-request-bytes 524288)])

(:wat::service::defservice :wat::kernel::stderr-svc'
  :satisfies :wat::kernel::StdErr'
  :durable   []
  :ephemeral [out <- :wat::io::IOWriter]
  :init (:wat::core::fn [record <- :wat::kernel::stderr-svc'::Record  fd <- :wat::core::i64]
          -> :wat::kernel::stderr-svc'::State
          (:wat::kernel::stderr-svc'::State :durable record :out (:wat::io::IOWriter/from-fd fd)))
  :impls
  [(write-line [s req]
     (:wat::core::let [_bytes (:wat::io::IOWriter/writeln (:wat::kernel::stderr-svc'::State/out s)
                                (:wat::kernel::StdErr'::WriteLineRequest/line req))]
       (:wat::service::Outcome::Reply s (:wat::kernel::StdErr'::WriteLineResponse::Ok))))])

;; ─── StdIn' (EOF-as-matchable-value upgrade: today's EOF panics-kills-the-loop; here it is a
;;     matchable `ReadLineResponse::Eof`, no-hidden-failures R55/R57 — DESIGN §7(c)) ─────────────
(:wat::core::defsurface :wat::kernel::StdIn' :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :wat::kernel::StdIn'::ReadLineRequest [max-buffer-bytes <- :wat::core::i64])
   (:wat::core::defenum :wat::kernel::StdIn'::ReadLineResponse :wat::enum::Pure
     :Line            [line <- :wat::core::String]
     :Eof             []                                     ;; NULLARY (matchable), constructed (::Eof) — mirrors ::Ok
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(read-line [self <- :wat::kernel::StdIn'  req <- :wat::kernel::StdIn'::ReadLineRequest]
     -> :wat::kernel::StdIn'::ReadLineResponse :max-request-bytes 524288)])

(:wat::service::defservice :wat::kernel::stdin-svc'
  :satisfies :wat::kernel::StdIn'
  :durable   []
  :ephemeral [in <- :wat::io::IOReader]
  :init (:wat::core::fn [record <- :wat::kernel::stdin-svc'::Record  fd <- :wat::core::i64]
          -> :wat::kernel::stdin-svc'::State
          (:wat::kernel::stdin-svc'::State :durable record :in (:wat::io::IOReader/from-fd fd)))
  :impls
  [(read-line [s req]
     (:wat::service::Outcome::Reply s
       (:wat::core::match (:wat::io::IOReader/read-frame (:wat::kernel::stdin-svc'::State/in s)
                            (:wat::kernel::StdIn'::ReadLineRequest/max-buffer-bytes req))
         ;; a full line read → ::Line; EOF (read-frame returns None) → the matchable ::Eof value
         ;; (NOT a panic that kills the serve loop — the no-hidden-failures upgrade).
         ((:wat::core::Some line) (:wat::kernel::StdIn'::ReadLineResponse::Line line))
         (:wat::core::None        (:wat::kernel::StdIn'::ReadLineResponse::Eof)))))])

;; ─── freeze-bootstrap helper (arc 170 PHASE 1) ──────────────────────────────────────────────────
;; Called ONCE from Rust (src/freeze.rs `bootstrap_wat_vm_process`) via `apply_function` with the
;; three real fds (0/1/2). Starts the three primed stdio defservices and returns their Handles as a
;; tuple. The three `<svc>/start` KWARGS-DEFN macros expand at NORMAL freeze time — inside this plain
;; defn body — which sidesteps the kwargs-defn-via-`eval_in_frozen` macro-eval gap (that path
;; mis-resolves the companion's `$impl` keyword to a live fn). Rust holds the returned tuple (keeping
;; each admin lineage Peer' — hence each service — alive for the process lifetime) and extracts each
;; Handle's `addr` into the `PrimedStdio` carrier for Strike 2's verb flip.
(:wat::core::defn :wat::kernel::start-primed-stdio
  [stdin-fd  <- :wat::core::i64
   stdout-fd <- :wat::core::i64
   stderr-fd <- :wat::core::i64]
  -> :(wat::kernel::stdin-svc'::Handle,wat::kernel::stdout-svc'::Handle,wat::kernel::stderr-svc'::Handle)
  (:wat::core::Tuple
    (:wat::kernel::stdin-svc'/start  :locus (:wat::spawn::thread) :record (:wat::kernel::stdin-svc'::Record)  :fd stdin-fd)
    (:wat::kernel::stdout-svc'/start :locus (:wat::spawn::thread) :record (:wat::kernel::stdout-svc'::Record) :fd stdout-fd)
    (:wat::kernel::stderr-svc'/start :locus (:wat::spawn::thread) :record (:wat::kernel::stderr-svc'::Record) :fd stderr-fd)))
