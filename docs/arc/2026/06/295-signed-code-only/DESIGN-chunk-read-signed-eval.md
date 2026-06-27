# Arc 295 (eval-side) — chunk-read signed eval over lazy byte-streams: the doctrine's deepest form

> **Status: SCOPED — co-designed 2026-06-27, fully grounded.** The signed-code doctrine (`you may only use signed
> code`) reaches its deepest form here: **not just load — EVAL itself must be signed, mandatory, parity with load.**
> And the requirement forces a chain of long-deferred work into being (the substrate-forces-our-hands ethos).

## The cascade (one doctrine drags four deferred things into being)
```
signed-code doctrine  →  EVAL must be signed (mandatory, no unsigned eval)
                      →  eval takes a length-bounded byte STREAM (the far side transmits over the wire)
                      →  that stream is a LAZY SEQ  ───────────────────►  finally builds arc 118 (deferred since 2026-05-01)
                      →  lazy seqs replace the thread-per-stage HOFs  ──►  annihilates wat/streams.wat (built wrong, successfully)
```
*"Did we make lazy-seqs a thing?"* — yes; signed-streaming-eval is the forcing function arc 118 waited for.

## "We've written this" — the grounded prior art (every part rides existing machinery)
| piece | on disk | 295 move |
|---|---|---|
| **signed eval** | `eval_signed_in_frozen(ast, algo, sig_b64, pubkey_b64)` (freeze.rs:1218, arc 028) — opt-in; verifies SHA-256 of canonical-EDN before any exec; failed → `EvalVerificationFailed`, NO code runs | flip to **mandatory**; modernize (below) |
| **lazy seqs** | DESIGNED arc 118 — **Option C: closures + recursion + thunks** (+ optional generator macro); metric *"threads exist only to guard mutable state; pure stages collapse onto the consumer"* — **never built** | **build** as the byte-chunk pull reader (minimal first cut) |
| **bounded read** | `:wat::io::IOReader/read-frame <reader> [max-bytes]` (io.rs:867); `DEFAULT_MAX_FRAME_BYTES = 512 KiB`; default-or-per-call-override ALREADY the contract; `read_framed_edn` (edn_shim.rs) | **reuse** — chunk-read = bounded frame read + the sig gate |
| **`wat/streams.wat`** | `:wat::stream::*` HOFs — a thread PER stage (map/filter/take) — *built wrong, successfully* | **annihilate** → reimplement over lazy seqs |
| **crypto** | Rust-internal: `verify_ast_signature`/`verify_program_signature` (hash.rs), `sign_*`, `sha2::Sha256`, `base64` dep | expose thin **wat-callable seam** (sha256 / ed25519 sign+verify), **no base64** |

## The model — chunk-read signed eval

### The flow (bounded → verify → eval, never out of order)
```
lazy byte-chunk seq   (arc 118 Option C; pull CHUNK_LIMIT bytes per step; empty seq at EOF; file OR wire — same shape)
  → bounded-load      (fold chunks into a buffer; ABORT EARLY the instant buf_len+chunk_len > MAX; refuse-lies; the
                       malicious tail is never pulled — bounded memory on untrusted wire input)
  → size contract     (accumulated len MUST equal the declared form-str-length; mismatch = a lie = reject)
  → verify sig        (the crypto seam: sha256(buf) → ed25519/alg verify under pubkey; bad sig → reject, NO eval)
  → eval              (only now lex+parse+eval the verified buffer; lazy form-lifting happens AFTER the gate, never before)
```
The lazy chunked read is the **security** mechanism, not ergonomics: you refuse mid-pull, so an untrusted far side
can never force an unbounded read (eager `read`-then-check is too late — the DoS already landed).

### The two size bounds (both load-bearing, kept distinct)
- **MAX (the ceiling)** — a **`wat.config/` item** (settable fleet-wide alongside `:dims`/`:capacity-mode`, via a
  `:wat::config::set-max-eval-bytes!`-style setter — intueri the name at the strike), **default = `DEFAULT_MAX_FRAME_BYTES`
  (512 KiB)** (shared const — eval and read-frame cap from the same place). Plus a **per-call override** (exactly
  `read-frame`'s `[max-bytes]` — "know better"). Both: config default + per-call override.
- **`form-str-length` (the declared contract)** — the payload's own declared size; the bounded-load result must
  **equal** it. *"Refuse lies on size."*

### The args shape (the modernized `eval_signed_in_frozen` — builder's `wat/eval$args`)
```
(:wat::core::defrecord :wat/eval$args
  [form-str-length    :- :wat.type/i64
   form-pubkey        :- (:wat.type/Vec :wat.type/u8)      ;; raw bytes — NO base64
   form-alg           :- :wat.crypto/Algorithm            ;; a defenum, dispatch on it (ed25519, p256, …)
   form-sig           :- (:wat.type/Vec :wat.type/u8)      ;; raw bytes — NO base64
   form-bytes         :- (:wat.type/Stream :wat.type/u8)]) ;; the lazy byte stream (eval owns lex+parse)
```
⊘ Refinements vs the builder's first sketch (decided in co-design): the stream is **`(Stream u8)` raw bytes** (eval
owns lexing+parsing — *"give eval a byte stream and it lifts tokens and forms"*), not `(Stream String)`; the
**per-call MAX override** is a 6th optional field/arg; the sig covers the form bytes (domain-tag/length-bound at the
strike so framing can't be tampered).

### Modernization (applies to BOTH eval AND load — parity)
- **NO base64.** `sig` / `pubkey` are raw `(Vec u8)`. The existing `*_b64: &str` shape retires. (`load` forms get the
  same: `:wat::verify::signed-ed25519`'s base64 payloads → raw bytes.)
- **`algo: &str` → `:wat.crypto/Algorithm` defenum.** Typed dispatch; each variant routes to its verifier.
- **opt-in → mandatory.** No unsigned eval path survives (parity with the load doctrine flip). The unsigned
  `eval_in_frozen`/`eval-ast!` surface is gated behind the build-trusted bootstrap only.

## The crypto seam (thin, pure, Rust — `src/intrinsic/crypto.rs`, arc-255 `#[wat_intrinsic]` home)
Bytes in, bytes out, no policy. NO base64.
- `:wat::crypto::sha256` `Bytes → Bytes`
- `:wat::crypto::ed25519-sign` `(priv: Bytes, msg: Bytes) → Bytes`
- `:wat::crypto::ed25519-verify` `(pub: Bytes, msg: Bytes, sig: Bytes) → Bool`
- (later `:sig` supports add p256-verify / PEM→bytes — additive, not v1)

All policy — the bounded-load, the size contract, the `Algorithm` dispatch, the manifest/chain (load side) — is
**wat over these.** Only the seam + the pinned pubkey + the minimal mandatory-verify gate stay irreducibly Rust.

## Build order (dependency-forced)
1. **lazy byte-chunk reader** (arc 118 Option C, minimal) — the bounded pull seq over a source fd, riding `read-frame`.
2. **crypto seam** (`src/intrinsic/crypto.rs`) — parallel, independent.
3. **chunk-read signed eval (mandatory)** — `wat/eval$args` + bounded-load + size-contract + verify(seam) + eval;
   the `:wat.crypto/Algorithm` defenum; the `wat.config` MAX default + per-call override.
4. **`wat/streams.wat` → lazy seqs** — annihilate the thread-per-stage HOFs; reimplement pure stages over lazy seqs
   (threads only where state is guarded). The general lazy-seq HOF library lands here.
5. **load parity** — same modernization (no base64, `Algorithm` defenum, mandatory) on the load forms.

## First strike
**The bounded byte-chunk lazy reader (arc 118 Option C) over the existing `read-frame` bounded read.** Ground the
real byte/seq primitives first (`bytes-concat`/`len`/`bytes-empty`, `reduce`, the source-fd reader) so the producer +
`bounded-load` land on real verbs. RED probe → BRIEF → strike.

## Pairs
`295/DESIGN.md` (the load-side model) · `freeze.rs:1218` (`eval_signed_in_frozen` — the thing modernized) ·
`docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN.md` (Option C — finally built) · `io.rs:867` /
`edn_shim.rs` (`read-frame` / `read_framed_edn` — the bounded read reused) · `wat/streams.wat` (annihilated) ·
`feedback_substrate_forces_idealized_state` (the cascade IS this ethos).
