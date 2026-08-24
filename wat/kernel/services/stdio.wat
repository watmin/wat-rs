;; wat/kernel/services/stdio.wat — arc 170 stdio-as-defservice, PHASE 1.
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
;; read-frame → ReadFrame…), exactly as the counter/query exemplars.
;;
;; Loads AFTER wat/service.wat (defservice) and wat/io.wat (the from-fd builtins' neighbourhood).
;; Baked stdlib → may define under `:wat::` (bypasses the reserved-prefix gate; expansion-born
;; `…/start` companions register via the stdlib-privilege bypass, as query/mem.wat relies on).

;; ─── StdOut ──────────────────────────────────────────────────────────────────────────────────
;; A DUMB, serialized RAW-byte writer: the op writes `bytes` VERBATIM (via IOWriter/write-string — NO
;; added newline). The newline that framed a line moved OUT of the service and INTO the verb's payload
;; (the verb appends "\n" before fragmenting — see stdio-write-out). `:max-request-bytes 524288` stays
;; as the DEFENSIVE FLOOR for a direct >budget caller; the `write-batched` client helper CHUNKS so this
;; is never tripped by println's own (possibly oversized) output — a program's output isn't a self-DoS.
(:wat::core::defsurface :wat::kernel::StdOut :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat::kernel::StdOut::WriteRequest [bytes <- :wat::core::String])
   (:wat::core::defenum :wat::kernel::StdOut::WriteResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(write [self <- :wat::kernel::StdOut  req <- :wat::kernel::StdOut::WriteRequest]
     -> :wat::kernel::StdOut::WriteResponse :max-request-bytes 524288)])

(:wat::service::defservice :wat::kernel::stdout-svc
  :satisfies :wat::kernel::StdOut
  :durable   []
  :ephemeral [out <- :wat::io::IOWriter]
  :init (:wat::core::fn [record <- :wat::kernel::stdout-svc::Record  fd <- :wat::core::i64]
          -> :wat::kernel::stdout-svc::State
          (:wat::kernel::stdout-svc::State :durable record :out (:wat::io::IOWriter/from-fd fd)))
  :impls
  [(write [s ctx req]
     (:wat::core::let [_bytes (:wat::io::IOWriter/write-string (:wat::kernel::stdout-svc::State/out s)
                                (:wat::kernel::StdOut::WriteRequest/bytes req))]
       (:wat::service::Outcome::Reply s (:wat::kernel::StdOut::WriteResponse::Ok))))])

;; ─── StdErr (raw write-serializer — identical to StdOut; the eprintln TERMINATE stays the verb's own
;;     act, never the service loop's — see DESIGN §3) ────────────────────────────────────────────
(:wat::core::defsurface :wat::kernel::StdErr :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat::kernel::StdErr::WriteRequest [bytes <- :wat::core::String])
   (:wat::core::defenum :wat::kernel::StdErr::WriteResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(write [self <- :wat::kernel::StdErr  req <- :wat::kernel::StdErr::WriteRequest]
     -> :wat::kernel::StdErr::WriteResponse :max-request-bytes 524288)])

(:wat::service::defservice :wat::kernel::stderr-svc
  :satisfies :wat::kernel::StdErr
  :durable   []
  :ephemeral [out <- :wat::io::IOWriter]
  :init (:wat::core::fn [record <- :wat::kernel::stderr-svc::Record  fd <- :wat::core::i64]
          -> :wat::kernel::stderr-svc::State
          (:wat::kernel::stderr-svc::State :durable record :out (:wat::io::IOWriter/from-fd fd)))
  :impls
  [(write [s ctx req]
     (:wat::core::let [_bytes (:wat::io::IOWriter/write-string (:wat::kernel::stderr-svc::State/out s)
                                (:wat::kernel::StdErr::WriteRequest/bytes req))]
       (:wat::service::Outcome::Reply s (:wat::kernel::StdErr::WriteResponse::Ok))))])

;; ─── StdIn (EOF-as-matchable-value upgrade: today's EOF panics-kills-the-loop; here it is a
;;     matchable `ReadFrameResponse::Eof`, no-hidden-failures R55/R57 — DESIGN §7(c)) ─────────────
(:wat::core::defsurface :wat::kernel::StdIn :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat::kernel::StdIn::ReadFrameRequest [max-buffer-bytes <- :wat::core::i64])
   (:wat::core::defenum :wat::kernel::StdIn::ReadFrameResponse :wat::enum::Pure
     :Frame            [line <- :wat::core::String]
     :Eof             []                                     ;; NULLARY (matchable), constructed (::Eof) — mirrors ::Ok
     ;; Arc 170 stdin-joins-the-lock-step — a process-wide stop was requested while
     ;; the read was blocked. NOT ::Eof (the peer didn't close) and NOT a
     ;; RequestMalformed/RequestTooLarge (nothing wrong with the request) — its own
     ;; outcome, so a caller cannot mistake a stop for the writer hanging up. Named
     ;; `Stopped`, not `Shutdown`, by the arc-170 intueri cast (2026-07-28): wat
     ;; already has `(:wat::kernel::stopped?)` for this fact, and nothing is
     ;; shutting down here — a stop was merely requested.
     :Stopped         []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(read-frame [self <- :wat::kernel::StdIn  req <- :wat::kernel::StdIn::ReadFrameRequest]
     -> :wat::kernel::StdIn::ReadFrameResponse :max-request-bytes 524288)])

(:wat::service::defservice :wat::kernel::stdin-svc
  :satisfies :wat::kernel::StdIn
  :durable   []
  :ephemeral [in <- :wat::io::IOReader]
  :init (:wat::core::fn [record <- :wat::kernel::stdin-svc::Record  fd <- :wat::core::i64]
          -> :wat::kernel::stdin-svc::State
          (:wat::kernel::stdin-svc::State :durable record :in (:wat::io::IOReader/from-fd fd)))
  :impls
  [(read-frame [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::core::match (:wat::io::IOReader/read-frame (:wat::kernel::stdin-svc::State/in s)
                            (:wat::kernel::StdIn::ReadFrameRequest/max-buffer-bytes req))
         ;; a full line read → ::Line; EOF → the matchable ::Eof value (NOT a panic that
         ;; kills the serve loop — the no-hidden-failures upgrade). Arc 170: IOReader/read-frame's
         ;; return became :wat::io::IOReader::ReadFrameOutcome (was (Option :- [String])) so a stop
         ;; request could get its OWN outcome rather than being folded into ::Eof — ::Stopped
         ;; carries it straight through.
         ((:wat::io::IOReader::ReadFrameOutcome::Frame line) (:wat::kernel::StdIn::ReadFrameResponse::Frame line))
         (:wat::io::IOReader::ReadFrameOutcome::Eof          (:wat::kernel::StdIn::ReadFrameResponse::Eof))
         (:wat::io::IOReader::ReadFrameOutcome::Stopped      (:wat::kernel::StdIn::ReadFrameResponse::Stopped)))))])

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
  -> (:wat::core::Tuple :- [:wat::kernel::stdin-svc::Handle :wat::kernel::stdout-svc::Handle :wat::kernel::stderr-svc::Handle])
  (:wat::core::Tuple
    (:wat::kernel::stdin-svc/start  :locus (:wat::spawn::thread) :record (:wat::kernel::stdin-svc::Record)  :fd stdin-fd)
    (:wat::kernel::stdout-svc/start :locus (:wat::spawn::thread) :record (:wat::kernel::stdout-svc::Record) :fd stdout-fd)
    (:wat::kernel::stderr-svc/start :locus (:wat::spawn::thread) :record (:wat::kernel::stderr-svc::Record) :fd stderr-fd)))

;; ─── Strike 3 client-side helpers (arc 170 PHASE 2 — the verb flip) ──────────────────────────────
;; The five caller verbs (eval_kernel_{println,pprintln,eprintln,epprintln,readln}, src/services/verbs.rs)
;; now route THROUGH the primed defservices. Rust caches a per-thread client Peer' (connect' once), then
;; drives these thin helpers via apply_function. The connect'/send'/recv' + typed-response match live
;; in wat (kernel-namespaced — the from-fd restriction et al. stay satisfied); Rust keeps only the EDN
;; formatting, the cache, the readln decode, and the eprintln terminate. Contracts are preserved exactly
;; (see the verb docs). COEXIST: the old spawn_service_peer path stays bootstrapped-but-idle.

;; connect' the shared Address' → this thread's OWN client Peer' (raise on any failure arm — a stdio
;; service that cannot be dialed is fatal, mirroring the old path's ChannelDisconnected).
(:wat::core::defn :wat::kernel::stdio-connect-out
  [addr <- (:wat::kernel::Address :- [:wat::kernel::StdOut::Op :wat::kernel::StdOut::Reply])]
  -> (:wat::kernel::Peer :- [:wat::kernel::StdOut::Op :wat::kernel::StdOut::Reply])
  (:wat::core::match (:wat::kernel::connect addr)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :wat::kernel::stdio-connect-err
  [addr <- (:wat::kernel::Address :- [:wat::kernel::StdErr::Op :wat::kernel::StdErr::Reply])]
  -> (:wat::kernel::Peer :- [:wat::kernel::StdErr::Op :wat::kernel::StdErr::Reply])
  (:wat::core::match (:wat::kernel::connect addr)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :wat::kernel::stdio-connect-in
  [addr <- (:wat::kernel::Address :- [:wat::kernel::StdIn::Op :wat::kernel::StdIn::Reply])]
  -> (:wat::kernel::Peer :- [:wat::kernel::StdIn::Op :wat::kernel::StdIn::Reply])
  (:wat::core::match (:wat::kernel::connect addr)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

;; ─── write-batched fragmentation (arc 170) ────────────────────────────────────────────────────────
;; A program's own output isn't a self-DoS: an oversized `println` must FIT the op budget by CHUNKING,
;; not fail with `RequestTooLarge`. `stdio-write-out`/`stdio-write-err` take the FULL payload String
;; (the verb has already appended the trailing "\n") and emit it as N in-order raw `write`s, each chunk
;; ≤ the budget — so the defensive `:max-request-bytes` floor is NEVER tripped by this path.
;;
;; CHUNK SIZE — grounded, not guessed. The serve-loop's #16.2 budget guard measures
;; `(string::length (edn::write req))` — i.e. the CHAR count of the EDN-encoded `WriteRequest`, which
;; RE-ESCAPES the chunk's chars (`"`,`\`,`\n`,`\r`,`\t`,`\b`,`\f` → 2 chars; any other C0/DEL → `\uXXXX`
;; = 6 chars) plus the record framing (`#…WriteRequest {:bytes "…"}`, ~55 chars). Worst case is 6× per
;; char, so a chunk of `STDIO-WRITE-CHUNK-CHARS` chars encodes to at most 6·65536 + 55 = 393271 chars —
;; comfortably under the 524288 budget for ANY payload content. (This is why the chunk is far below the
;; naive "budget − framing": the check is on the DOUBLE-encoded request, not the raw chunk bytes.)
;; String/subs + String/length are CHAR-indexed and the budget is CHAR-measured, so chunking by chars is
;; internally consistent (no split UTF-8 scalar). A single ≤budget println = ONE chunk = identical bytes
;; to the pre-fragmentation writeln path.
(:wat::core::def :wat::kernel::STDIO-WRITE-CHUNK-CHARS 65536)

;; stdio-write-out — emit the full `payload` to the primed StdOut peer as in-order ≤budget raw `write`s
;; (tail-recursive over the remaining suffix; empty payload → nil, the terminating base case). Each
;; chunk's ::Ok → recurse on the rest; ::RequestTooLarge (impossible under this chunking — the defensive
;; floor) / a lost/closed peer → SURFACE (never silently drop — a stdio write failure is loud).
(:wat::core::defn :wat::kernel::stdio-write-out
  [peer    <- (:wat::kernel::Peer :- [:wat::kernel::StdOut::Op :wat::kernel::StdOut::Reply])
   payload <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::let [len (:wat::core::string::length payload)]
    (:wat::core::if (:wat::core::= len 0)
      nil
      (:wat::core::let
        [take  (:wat::core::if (:wat::core::i64::< len :wat::kernel::STDIO-WRITE-CHUNK-CHARS) len :wat::kernel::STDIO-WRITE-CHUNK-CHARS)
         chunk (:wat::core::string::subs payload 0 take)
         rest  (:wat::core::string::subs payload take len)
         _ack  (:wat::core::match (:wat::kernel::StdOut/write peer (:wat::kernel::StdOut::WriteRequest :bytes chunk))
                 ((:wat::kernel::RecvOutcome::Message resp)
                   (:wat::core::match resp
                     ((:wat::kernel::StdOut::WriteResponse::Ok) nil)
                     ((:wat::kernel::StdOut::WriteResponse::RequestTooLarge b cap)
                       (:wat::kernel::assertion-failed! "println: stdout write exceeded max-request-bytes (RequestTooLarge) — a write-batched chunk overran the budget (should be impossible)" :wat::core::None :wat::core::None))
                     ((:wat::kernel::StdOut::WriteResponse::RequestMalformed mpath mexpected mgot)
                       (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
                 ((:wat::kernel::RecvOutcome::Lost cause)
                   (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                 ;; arc 278 #73 — a stop, not a close. The stdout service was ALIVE.
                 (:wat::kernel::RecvOutcome::Stopped
                   (:wat::kernel::assertion-failed! "println: stop requested — the stdout service was ALIVE and the channel open" :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Closed
                   (:wat::kernel::assertion-failed! "println: stdout service peer closed" :wat::core::None :wat::core::None)))]
        (:wat::kernel::stdio-write-out peer rest)))))

;; stdio-write-err — the StdErr twin of stdio-write-out (same chunking; fd 2).
(:wat::core::defn :wat::kernel::stdio-write-err
  [peer    <- (:wat::kernel::Peer :- [:wat::kernel::StdErr::Op :wat::kernel::StdErr::Reply])
   payload <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::let [len (:wat::core::string::length payload)]
    (:wat::core::if (:wat::core::= len 0)
      nil
      (:wat::core::let
        [take  (:wat::core::if (:wat::core::i64::< len :wat::kernel::STDIO-WRITE-CHUNK-CHARS) len :wat::kernel::STDIO-WRITE-CHUNK-CHARS)
         chunk (:wat::core::string::subs payload 0 take)
         rest  (:wat::core::string::subs payload take len)
         _ack  (:wat::core::match (:wat::kernel::StdErr/write peer (:wat::kernel::StdErr::WriteRequest :bytes chunk))
                 ((:wat::kernel::RecvOutcome::Message resp)
                   (:wat::core::match resp
                     ((:wat::kernel::StdErr::WriteResponse::Ok) nil)
                     ((:wat::kernel::StdErr::WriteResponse::RequestTooLarge b cap)
                       (:wat::kernel::assertion-failed! "eprintln: stderr write exceeded max-request-bytes (RequestTooLarge) — a write-batched chunk overran the budget (should be impossible)" :wat::core::None :wat::core::None))
                     ((:wat::kernel::StdErr::WriteResponse::RequestMalformed mpath mexpected mgot)
                       (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
                 ((:wat::kernel::RecvOutcome::Lost cause)
                   (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                 ;; arc 278 #73 — a stop, not a close. The stderr service was ALIVE.
                 (:wat::kernel::RecvOutcome::Stopped
                   (:wat::kernel::assertion-failed! "eprintln: stop requested — the stderr service was ALIVE and the channel open" :wat::core::None :wat::core::None))
                 (:wat::kernel::RecvOutcome::Closed
                   (:wat::kernel::assertion-failed! "eprintln: stderr service peer closed" :wat::core::None :wat::core::None)))]
        (:wat::kernel::stdio-write-err peer rest)))))

;; read one line via the primed StdIn peer, returning the RAW line String (Rust decodes it via the
;; self-describing EDN wire, exactly as the old readln' did). ::Line → the line; ::Eof → reproduce the
;; old EOF-on-fd0 behavior EXACTLY: a terminal raise (the old StdInService/handle assertion-failed!'d on
;; EOF → the caller saw ChannelDisconnected; the matchable ::Eof variant is BANKED, not yet exposed to
;; the 72 readln callers). ::RequestTooLarge → SURFACE.
(:wat::core::defn :wat::kernel::stdio-read
  [peer <- (:wat::kernel::Peer :- [:wat::kernel::StdIn::Op :wat::kernel::StdIn::Reply])
   cap  <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::match (:wat::kernel::StdIn/read-frame peer (:wat::kernel::StdIn::ReadFrameRequest :max-buffer-bytes cap))
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::kernel::StdIn::ReadFrameResponse::Frame line) line)
        ((:wat::kernel::StdIn::ReadFrameResponse::Eof)
          (:wat::kernel::assertion-failed! "readln: EOF on stdin — client (parent process or pipe writer) disconnected" :wat::core::None :wat::core::None))
        ;; Arc 170 — honest about WHY: `readln` raises on every non-Line outcome (it
        ;; reproduces the pre-arc-170 EOF-on-fd0 behavior for its 72 callers, and there
        ;; is no caller-facing value form for "raise" to hand a stop through), but the
        ;; message must NOT say "EOF" — that is the exact defect this brief removes.
        ((:wat::kernel::StdIn::ReadFrameResponse::Stopped)
          (:wat::kernel::assertion-failed! "readln: a stop was requested while blocked reading stdin" :wat::core::None :wat::core::None))
        ((:wat::kernel::StdIn::ReadFrameResponse::RequestTooLarge b cap2)
          (:wat::kernel::assertion-failed! "readln: stdin read exceeded max-buffer-bytes (RequestTooLarge)" :wat::core::None :wat::core::None))
        ((:wat::kernel::StdIn::ReadFrameResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    ;; arc 278 #73 — a stop, not a close. The stdin service was ALIVE.
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "readln: stop requested — the stdin service was ALIVE and the channel open" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "readln: stdin service peer closed" :wat::core::None :wat::core::None))))

;; read one frame via the primed StdIn peer, returning the MATCHABLE outcome rather than the raw
;; String — the honest sibling of `stdio-read` above. Where `stdio-read` collapses every non-happy
;; outcome into a raise (it had to: it reproduces the pre-arc-170 EOF-on-fd0 behavior for the 72
;; `readln` callers, and its comment says so), this one hands `::Eof` BACK as a value. That bank is
;; exactly what left a REPL loop unable to stop cleanly, and this is where it is spent.
;;
;; The two request-framing variants are NOT propagated: `::RequestTooLarge` / `::RequestMalformed`
;; exist because defservice mandates them on every op-Response, but this op's request is one i64 the
;; kernel builds, so neither can fire. They raise here rather than widening the caller's enum with
;; arms that are unreachable by construction. (An over-cap READ is a different thing entirely — it
;; surfaces from inside `IOReader/read-frame` — which is why `stdio-read`'s ::RequestTooLarge message
;; mentioning max-buffer-bytes is itself misleading; noted, not fixed here.)
(:wat::core::defn :wat::kernel::stdio-read-frame
  [peer <- (:wat::kernel::Peer :- [:wat::kernel::StdIn::Op :wat::kernel::StdIn::Reply])
   cap  <- :wat::core::i64]
  -> :wat::kernel::ReadFrameOutcome
  (:wat::core::match (:wat::kernel::StdIn/read-frame peer (:wat::kernel::StdIn::ReadFrameRequest :max-buffer-bytes cap))
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::kernel::StdIn::ReadFrameResponse::Frame line)
          (:wat::kernel::ReadFrameOutcome::Frame line))
        ((:wat::kernel::StdIn::ReadFrameResponse::Eof)
          ;; a unit variant is a bare keyword, not a call — it evaluates straight to its
          ;; pre-built EnumValue via the SymbolTable's unit_variants table
          :wat::kernel::ReadFrameOutcome::Eof)
        ;; Arc 170 — carried straight through, not collapsed into ::Eof. Same bare-keyword
        ;; shape as ::Eof (both are Rust builtin-registered unit variants). Named `Stopped`
        ;; on both sides by the arc-170 intueri cast — see StdIn::ReadFrameResponse above.
        ((:wat::kernel::StdIn::ReadFrameResponse::Stopped)
          :wat::kernel::ReadFrameOutcome::Stopped)
        ((:wat::kernel::StdIn::ReadFrameResponse::RequestTooLarge b cap2)
          (:wat::kernel::assertion-failed! "read-frame: stdin request framing rejected (RequestTooLarge) — unreachable for a kernel-built request" :wat::core::None :wat::core::None))
        ((:wat::kernel::StdIn::ReadFrameResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "read-frame: stdin request framing rejected (RequestMalformed) — unreachable for a kernel-built request" :wat::core::None :wat::core::None))))
    ;; ★ arc 278 #73 — THIS SITE IS THE WHOLE STONE, VISIBLE IN ONE PLACE.
    ;;
    ;; Arc 170 already knew a stop is not a death: the client's `recv'` wakes on the
    ;; shutdown broadcast before the service's own read does, so the stop arrived HERE
    ;; first — and, having nowhere else to ride, it arrived wearing `Lost`. This code
    ;; was therefore written to receive a DEATH and then open the death report to ask
    ;; whether anyone had actually died. That nested `match cause` was the cost of the
    ;; missing variant, paid at the one site that most needed the truth.
    ;;
    ;; It is now a top-level arm, and two things fall out with it: the inner match is
    ;; gone, and so is its `_` wildcard — which was a doctrine violation
    ;; (`109/NOTE-full-enum-match-mandatory-no-wildcard-arm.md`) that only existed
    ;; because the outer arm was carrying two unrelated facts at once.
    (:wat::kernel::RecvOutcome::Stopped :wat::kernel::ReadFrameOutcome::Stopped)
    ;; `Lost` now means what it says: the peer died. Raise its own cause.
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "read-frame: stdin service peer closed" :wat::core::None :wat::core::None))))

;; ─── write-fd-raw (arc 170) — the RAW, un-terminated write-side sibling of `from-fd`: emit `payload`
;;     verbatim to a whitelisted fd (no framing, no newline, no op budget), returning the byte count. ─
;;
;; The one and only caller today is the `select'`-flood deadlock probe
;; (tests/comms/probe_select_flood_no_deadlock): after the Strike-3 verb flip, `println` is BOUNDED by
;; StdOut's `:max-request-bytes` op budget, so a conforming peer can no longer flood the wire past the
;; frame cap (correct — that is the whole point). The parent-side guard (`select'` FrameTooLarge → Lost,
;; no deadlock) must still be exercised, which needs a NON-CONFORMING peer.
;;
;; GATED (arbitrary-fd danger sealed): `{:restricted-to [:wat::kernel:: :wat::test::]}` — an arbitrary-fd
;; raw write is privileged (it reopens an UNBOUNDED write to whatever fd the caller names, bypassing the
;; framing/budget discipline the flip established), so only a `:wat::kernel::`/`:wat::test::` ENCLOSING FN
;; may call it. The gate is SAFE: the reserved-prefix gate forbids `:user::` code from authoring a
;; `:wat::` caller, so no user program can construct a passing call site. A `:user::` flood child reaches
;; the raw write only through the NARROW fixed-fd `flood-stdout-raw` below.
(:wat::core::defn :wat::kernel::write-fd-raw
  {:restricted-to [:wat::kernel:: :wat::test::]}
  [fd <- :wat::core::i64  payload <- :wat::core::String]
  -> :wat::core::i64
  (:wat::io::IOWriter/write-string (:wat::io::IOWriter/from-fd fd) payload))

;; flood-stdout-raw — the NARROW flood entry: fd is HARDCODED to 1 (the caller's OWN stdout / peer
;; wire), so it can never be abused for arbitrary-fd writes (that danger stays sealed in the gated
;; `write-fd-raw`). ALSO gated `[:wat::kernel:: :wat::test::]` (builder ruling: users cannot flood —
;; the tooling forbids it structurally; there is NO user-callable raw-write escape hatch). Its own
;; enclosing fn is `:wat::kernel::`, so its call to the gated write-fd-raw passes. Reached only from
;; kernel/test code; the select'-flood probe's child reaches it through a `:wat::test::` flood body.
;; (Lives in stdio.wat — not wat/test.wat, which loads BEFORE this file — so the defn→defn
;; eval-dep on write-fd-raw stays intra-file / correctly ordered.)
(:wat::core::defn :wat::kernel::flood-stdout-raw
  {:restricted-to [:wat::kernel:: :wat::test::]}
  [payload <- :wat::core::String]
  -> :wat::core::i64
  (:wat::kernel::write-fd-raw 1 payload))

;; str-double — internal (kernel/test-gated) pure helper: `2^n` copies of `s` via repeated concat.
;; Gated so it is NOT a user-callable knob; it exists only to build flood-own-stdout's fixed payload.
(:wat::core::defn :wat::kernel::str-double
  {:restricted-to [:wat::kernel:: :wat::test::]}
  [s <- :wat::core::String  n <- :wat::core::i64]
  -> :wat::core::String
  (:wat::core::if (:wat::core::= n 0)
    s
    (:wat::kernel::str-double (:wat::core::String/concat s s) (:wat::core::- n 1))))

;; flood-own-stdout — the TEST-namespaced entry, the ONLY flood path a `:user::` program can reach.
;; CHOKED DOWN so it cannot be exploited: ZERO ARGS, no knobs, fully HARDCODED behavior — flood the
;; caller's OWN stdout with a FIXED ~1 MiB (2^20 bytes of 'x') un-terminated payload. The user controls
;; NOTHING: not the fd (own stdout), not the size, not the content — only whether to trigger the single
;; fixed flood. Every controllable primitive it composes — `str-double` (size), `flood-stdout-raw`
;; (payload), `write-fd-raw` (fd) — is kernel/test-gated, so a `:user::` form can reach NONE of them
;; directly; this zero-arg fixed action is the entire user-reachable surface. The composition passes
;; the gates because this fn's enclosing scope is `:wat::test::` (matches every helper's whitelist).
;; The select'-flood deadlock probe's `:user::` child calls `(:wat::test::flood-own-stdout)` to
;; simulate a non-conforming peer.
;;
;; (Lives in stdio.wat, not wat/test.wat: test.wat loads earlier, so a defn there would
;; eval-depend on the later kernel helpers — a deporder violation. Namespace ≠ file; a baked stdlib
;; source may define `:wat::test::` here under stdlib privilege.)
(:wat::core::defn :wat::test::flood-own-stdout
  [] -> :wat::core::i64
  (:wat::kernel::flood-stdout-raw (:wat::kernel::str-double "x" 20)))
