# Arc 214 Slice 3 — Stone C — HolonRepresentable serialization layer

## Mission

Stone C is the THIRD of five stepping stones in Slice 3. It proves **one** thing: `Sender<T: HolonRepresentable>` and `Receiver<T: HolonRepresentable>` correctly roundtrip arbitrary `HolonRepresentable` values through the io_uring pipe via HolonAST ↔ EDN bytes.

This stone closes the wire form contract that Slice 1's `HolonRepresentable` trait promised — and gives Slice 1's trait its FIRST CONCRETE IMPL (`impl HolonRepresentable for String`).

After Stone C, the process tier is a fully-typed peer channel for any `HolonRepresentable` payload. The bytes-only foundation from Stones A+B is preserved; Stone C wraps the byte layer in serialization.

Stone C still defers:
- `try_recv` + `Select` + Clone + close + len + trait impls (Stone D)
- Persistent IoUring + config tunable (Stone E)

## Stepping stone roadmap (Slice 3; informational — do not implement beyond Stone C)

- **Stone A (shipped):** io_uring bytes-only `pair()` + `Sender::send(&[u8])` + `Receiver::recv()` with newline framing
- **Stone B (shipped):** cascade-aware multi-arm POLL_ADD on `[data_fd, broadcast_fd]`; broadcast wins ties
- **Stone C (this work):** generic `Sender<T: HolonRepresentable>` / `Receiver<T: HolonRepresentable>` with HolonAST ↔ EDN bytes via wat-edn; `impl HolonRepresentable for String`
- **Stone D:** `try_recv` + `Select<'a, T>` + Clone + close + len + CommSender/CommReceiver trait impls
- **Stone E:** Persistent IoUring per Receiver + `:wat::config::set-process-tier-uring-depth!`

## Substrate context (substrate-truth verified pre-spawn)

- **`HolonRepresentable` trait** at `src/comms/mod.rs:58-63` (Slice 1):
  ```rust
  pub trait HolonRepresentable: Send + 'static {
      fn to_holon_ast(&self) -> holon::HolonAST;
      fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
      where Self: Sized;
  }
  ```
  Currently has ZERO concrete impls. Stone C adds the first (`impl HolonRepresentable for String`).

- **`holon::HolonAST`** variants (per `src/edn_shim.rs:1678+` `holon_ast_to_edn` arm-for-arm structure): `Symbol`, `String`, `I64`, `F64`, `Bool`, `Nil`, `Atom`, `Bind(role, filler)`, `Bundle(xs)`, `Permute(child)`, `Difference(a, b)`, ... Stone C only needs the `HolonAST::String` arm for `impl HolonRepresentable for String`.

- **wat-edn surface (PUBLIC):**
  - `wat_edn::write(&OwnedValue) -> String` — serialize OwnedValue to single-line EDN text (used by `src/typed_channel.rs:229`)
  - `wat_edn::parse_owned(&str) -> Result<OwnedValue, ...>` — parse EDN text to OwnedValue (used by edn_shim)
  - **Critical guarantee:** `wat_edn::write` produces SINGLE-LINE output; embedded newlines in user strings escape as `\n` literal. This is what makes newline framing safe across Stone C — payload bytes never contain an actual `'\n'` because EDN escapes it.

- **edn_shim PUBLIC HolonAST text surface (existing, READ side only):**
  - `pub fn read_holon_ast_tagged(s: &str) -> Result<Arc<holon::HolonAST>, EdnReadError>` at `src/edn_shim.rs:1997`
  - `pub fn read_holon_ast_natural(s: &str) -> Result<Arc<holon::HolonAST>, EdnReadError>` at `src/edn_shim.rs:2009`

- **edn_shim PRIVATE HolonAST surface (Stone C must promote):**
  - `fn holon_ast_to_edn(h: &holon::HolonAST) -> OwnedValue` at `src/edn_shim.rs:1678` — Stone C wraps this in a new public `write_holon_ast_tagged` (5 LOC; mirrors the existing `read_holon_ast_tagged` symmetry).

- **Existing PipeFd reference pattern** at `src/typed_channel.rs:228-230`:
  ```rust
  let edn = crate::edn_shim::value_to_edn_with(&value, types);
  let mut payload = wat_edn::write(&edn);
  payload.push('\n');
  ```
  This is the substrate's existing send-encode pattern. Stone C mirrors it: encode HolonAST → OwnedValue → String, append '\n', write through Stone A's libc::write loop.

- **`SendError<T>`** at `src/comms/mod.rs:114` (Slice 1): `pub struct SendError<T>(pub T);`. Stone C's `Sender<T>::send` takes ownership of `value: T`; on failure returns `Err(SendError(value))` — no clone needed (Stone A had to clone bytes via `.to_vec()` because input was `&[u8]`; Stone C owns the input).

## Concrete deliverables

### 1. Add `pub fn write_holon_ast_tagged` to `src/edn_shim.rs`

Mint the symmetric counterpart to the existing `read_holon_ast_tagged`. Insert immediately above `pub fn read_holon_ast_tagged` (the natural pairing spot; ~line 1996):

```rust
/// Render a HolonAST as a tagged-EDN string (single-line).
///
/// Inverse of [`read_holon_ast_tagged`]. The roundtrip `read . write`
/// is an identity on valid HolonASTs.
///
/// Output is single-line per `wat_edn::write` guarantee — embedded
/// newlines in payload strings escape as `\n` literal. This makes
/// the output safe for newline-framed wire protocols (process-tier
/// pipe framing per arc 214 Slice 3 Stone C).
pub fn write_holon_ast_tagged(h: &holon::HolonAST) -> String {
    wat_edn::write(&holon_ast_to_edn(h))
}
```

This is the ONLY change to edn_shim.rs. Five LOC. Minimal exposure delta.

### 2. Add `impl HolonRepresentable for String` to `src/comms/mod.rs`

Insert immediately after the `HolonRepresentable` trait definition (~line 64; before the `// ─── Tier-agnostic sender / receiver traits ───` divider):

```rust
/// First concrete `HolonRepresentable` impl (Slice 3 Stone C).
///
/// Encodes `String` as `HolonAST::String`. The roundtrip is exact —
/// `String::from_holon_ast(s.to_holon_ast())` returns the original
/// string (including any embedded `'\n'` which wat-edn escapes
/// during serialization).
///
/// Used by Stone C's probe tests as the test type. Future arcs may
/// add impls for other substrate types (StdInServiceEvent,
/// SpawnOutcome, etc.) as Slice 4/5 consumers require.
impl HolonRepresentable for String {
    fn to_holon_ast(&self) -> holon::HolonAST {
        holon::HolonAST::String(self.clone())
    }

    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        match ast {
            holon::HolonAST::String(s) => Ok(s.clone()),
            other => Err(WireError::new(format!(
                "expected HolonAST::String, got {:?}",
                other
            ))),
        }
    }
}
```

### 3. Refactor `src/comms/process.rs` — make Sender/Receiver/pair generic over T

Stone C is the BIGGEST process.rs refactor in this slice. The structural changes:

**Module-level doc update** — replace `## Current scope (through Stone B)` section with:

```rust
//! ## Current scope (through Stone C)
//!
//! Generic `Sender<T: HolonRepresentable>` / `Receiver<T: HolonRepresentable>`
//! with HolonAST ↔ EDN bytes via wat-edn (Stone C). Cascade-aware
//! multi-arm POLL_ADD (Stone B). io_uring bytes foundation with
//! newline framing (Stone A). Still NO try_recv / Select / Clone /
//! close / len / trait impls (Stone D); NO persistent ring / config
//! tunable (Stone E).
```

Also retire the Stone A "Payload bytes MUST NOT contain '\n'" caveat — Stone C resolves it via wat-edn's single-line escape guarantee. Update the `## Framing` section:

```rust
//! ## Framing
//!
//! Each `send` encodes `T` as a tagged-EDN single-line string via
//! `write_holon_ast_tagged`, appends `'\n'`, and writes atomically
//! (writes ≤ PIPE_BUF = 4096 are atomic per POSIX). The receiver
//! reads bytes into an internal accumulator and splits on `'\n'`;
//! the trailing newline does not appear in EDN output because
//! wat-edn produces single-line text (embedded newlines escape as
//! `\n` literal). Frames are decoded back via
//! `read_holon_ast_tagged` + `T::from_holon_ast`.
```

**Imports** — add to the existing import block at top of file:

```rust
use std::marker::PhantomData;
```

**Sender refactor** — make generic over T:

```rust
/// Process-tier send endpoint. Generic over the payload type T (Stone C).
/// Owns the pipe's write-end fd. Encodes `T` via
/// `HolonRepresentable::to_holon_ast` → `write_holon_ast_tagged` →
/// newline-framed bytes.
///
/// NOT Clone (Stone D adds). NOT close-able (Stone D adds `close(self)`).
/// Drop closes the fd automatically (OwnedFd Drop impl).
#[derive(Debug)]
pub struct Sender<T: HolonRepresentable> {
    write_fd: OwnedFd,
    /// Type marker — `T` doesn't appear in any field but constrains
    /// what `send` accepts. `PhantomData<T>` makes `Sender<T>` invariant
    /// in T which is correct for this use case.
    _phantom: PhantomData<T>,
}

impl<T: HolonRepresentable> Sender<T> {
    /// Send `value` to the channel. Encodes via
    /// `T::to_holon_ast` → `edn_shim::write_holon_ast_tagged` →
    /// newline-framed bytes → `libc::write` retry loop.
    ///
    /// Returns `Err(SendError(value))` when the peer's read-end is
    /// closed (EPIPE) or when the write fails for any other reason.
    /// The error carries the original `T` so the caller can recover
    /// or re-send.
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        // Encode T → HolonAST → tagged EDN string (single-line).
        let ast = value.to_holon_ast();
        let edn_str = crate::edn_shim::write_holon_ast_tagged(&ast);

        // Frame: EDN bytes + '\n'. Single allocation; single contiguous write.
        let edn_bytes = edn_str.as_bytes();
        let mut framed: Vec<u8> = Vec::with_capacity(edn_bytes.len() + 1);
        framed.extend_from_slice(edn_bytes);
        framed.push(b'\n');

        let fd = self.write_fd.as_raw_fd();
        let mut written = 0usize;
        while written < framed.len() {
            // SAFETY: `fd` is valid for the lifetime of `self.write_fd`
            // (OwnedFd-managed; not closed until Drop). The pointer
            // derived from `framed[written..]` is valid for
            // `framed.len() - written` bytes — `framed` is a live Vec
            // on this function's stack and is not freed until after
            // this loop completes.
            let n = unsafe {
                libc::write(
                    fd,
                    framed[written..].as_ptr() as *const _,
                    framed.len() - written,
                )
            };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                // EPIPE (peer closed) or other write failure — caller
                // gets the original value back.
                return Err(SendError(value));
            }
            written += n as usize;
        }
        Ok(())
    }
}
```

**Receiver refactor** — make generic over T; recv returns T:

```rust
/// Process-tier receive endpoint. Generic over the payload type T (Stone C).
/// Owns the pipe's read-end fd and a small internal byte accumulator
/// for cross-call frame splitting.
///
/// Cascade-aware (Stone B): `recv` wakes on substrate shutdown via
/// io_uring multi-arm POLL_ADD on `SHUTDOWN_BROADCAST_READ_FD`.
/// NOT Clone (Stone D adds). Per-call `IoUring` instance (Stone E
/// persistifies).
#[derive(Debug)]
pub struct Receiver<T: HolonRepresentable> {
    read_fd: OwnedFd,
    /// Bytes read from the pipe but not yet returned to a caller.
    /// `RefCell` provides interior mutability so `recv(&self)` can
    /// update the accumulator without `&mut self`. `Receiver` is `!Sync`
    /// by construction (RefCell is !Sync); the substrate's threading
    /// model never shares a single Receiver across threads — clones
    /// (Stone D) create independent endpoints.
    accumulator: RefCell<Vec<u8>>,
    /// Type marker — `T` doesn't appear in any field but constrains
    /// what `recv` produces. `PhantomData<T>` makes `Receiver<T>`
    /// invariant in T which is correct for this use case.
    _phantom: PhantomData<T>,
}

impl<T: HolonRepresentable> Receiver<T> {
    /// Blocking recv. Returns the next complete `T` decoded from the
    /// pipe (newline-framed; EDN-encoded). Reads from the internal
    /// accumulator first; if no complete frame is buffered, drives
    /// the cascade-aware io_uring multi-arm POLL_ADD + Read loop
    /// until a `'\n'` is observed; then decodes the frame via
    /// `read_holon_ast_tagged` + `T::from_holon_ast`.
    ///
    /// Returns `Err(RecvError)` on peer-close (EOF; read returns 0),
    /// on io_uring submission/completion failure, on substrate
    /// shutdown (cascade-arm fires; Stone B), on UTF-8 decode failure,
    /// on EDN parse failure, or on `T::from_holon_ast` failure.
    pub fn recv(&self) -> Result<T, RecvError> {
        // Fast path — accumulator already has a complete frame.
        if let Some(frame) = take_frame(&mut self.accumulator.borrow_mut()) {
            return decode_frame::<T>(&frame);
        }

        let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD
            .load(std::sync::atomic::Ordering::SeqCst);
        let read_fd = self.read_fd.as_raw_fd();

        loop {
            // Cascade-aware step — poll both arms (data + broadcast).
            // Bootstrap fallback: when broadcast_fd is -1 (pre-init or
            // test bypass), skip the poll and fall through to bare Read
            // (Stone A behavior; no cascade available).
            if broadcast_fd >= 0 {
                match wait_for_data_or_cascade(read_fd, broadcast_fd)? {
                    PollOutcome::Shutdown => return Err(RecvError),
                    PollOutcome::DataReady => {
                        // Data is ready; fall through to Read step.
                    }
                }
            }

            // Read step — same as Stones A+B. Per-call IoUring; ring size 2.
            // (Stone E persistifies the ring.)
            let mut ring = IoUring::new(2).map_err(|_| RecvError)?;
            let mut buf = [0u8; 4096];
            let read_e = opcode::Read::new(
                types::Fd(read_fd),
                buf.as_mut_ptr(),
                buf.len() as _,
            )
            .build()
            .user_data(1);

            // SAFETY: read_e's buf pointer (buf) outlives submit_and_wait
            // because buf is on this function's stack and not freed until
            // after the wait completes.
            unsafe {
                ring.submission()
                    .push(&read_e)
                    .map_err(|_| RecvError)?;
            }

            ring.submit_and_wait(1).map_err(|_| RecvError)?;
            let cqe = ring.completion().next().ok_or(RecvError)?;
            let result = cqe.result();
            if result < 0 {
                return Err(RecvError);
            }
            if result == 0 {
                // EOF — peer closed the write-end.
                return Err(RecvError);
            }
            let n = result as usize;
            self.accumulator
                .borrow_mut()
                .extend_from_slice(&buf[..n]);

            if let Some(frame) = take_frame(&mut self.accumulator.borrow_mut()) {
                return decode_frame::<T>(&frame);
            }
            // No complete frame yet; loop and poll/read more bytes.
        }
    }
}
```

**Add `decode_frame` private fn** — insert immediately above `take_frame`:

```rust
/// Decode a newline-framed payload to `T` via the Stone C wire chain:
/// UTF-8 bytes → tagged-EDN string → HolonAST → T.
///
/// Returns `Err(RecvError)` on any layer's failure (utf8, EDN parse,
/// or `T::from_holon_ast`). The error type collapses all three causes
/// because the caller cannot meaningfully distinguish them — wire
/// failures all mean "the frame did not roundtrip cleanly; the channel
/// is in an honest but unrecoverable state per this call".
fn decode_frame<T: HolonRepresentable>(bytes: &[u8]) -> Result<T, RecvError> {
    let s = std::str::from_utf8(bytes).map_err(|_| RecvError)?;
    let ast_arc = crate::edn_shim::read_holon_ast_tagged(s).map_err(|_| RecvError)?;
    T::from_holon_ast(&ast_arc).map_err(|_| RecvError)
}
```

**`take_frame` UNCHANGED** — stays as-is from Stones A/B; concern is "split first newline-frame from a Vec<u8> buffer". Decoding is `decode_frame`'s concern.

**`pair` factory refactor** — generic over T:

```rust
/// Create a new process-tier channel pair (Stone C — generic over T).
///
/// Allocates an anonymous pipe via `libc::pipe(2)` and wraps the two
/// file descriptors as `Sender<T>` / `Receiver<T>`. The type parameter
/// `T` constrains what values flow through the channel; both endpoints
/// must agree on `T` (typically inferred at call site).
///
/// Returns the OS-level `io::Error` on `pipe(2)` failure (rare; out
/// of fds or kernel OOM).
pub fn pair<T: HolonRepresentable>() -> std::io::Result<(Sender<T>, Receiver<T>)> {
    let mut fds = [0i32; 2];
    // SAFETY: `fds` is a valid `[i32; 2]` stack allocation whose
    // lifetime covers this call; `libc::pipe` writes two file
    // descriptors into it.
    let result = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: pipe(2) returned two valid, owned fds. Wrap each as OwnedFd
    // so Drop closes them; never call OwnedFd::from_raw_fd on the same
    // fd twice (would double-close).
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    Ok((
        Sender {
            write_fd,
            _phantom: PhantomData,
        },
        Receiver {
            read_fd,
            accumulator: RefCell::new(Vec::new()),
            _phantom: PhantomData,
        },
    ))
}
```

### 4. Rewrite `tests/probe_comms_process.rs` — String round-trip

The Stone A/B probe used bytes (`pair() -> (Sender, Receiver)`; `tx.send(b"hello")`; `rx.recv() -> Vec<u8>`). Stone C uses String:

```rust
//! Arc 214 Slice 3 Stone C smoke probe — verify HolonRepresentable
//! round-trip through io_uring pipe via String as the test type.
//!
//! Six tests:
//!   1. pair() constructs successfully
//!   2. single-string round-trip preserves the string
//!   3. FIFO ordering across multiple sends
//!   4. sender drop wakes recv with Err(RecvError)
//!   5. accumulator correctly splits two frames received in one read
//!   6. large string spans multiple io_uring reads
//!
//! Stone C wires generic T: HolonRepresentable. The wire chain is
//! T → HolonAST → tagged EDN string → newline-framed bytes →
//! libc::write → io_uring Read → bytes → EDN → HolonAST → T.
//!
//! Embedded `\n` in strings escape during wat-edn serialization, so
//! the Stone A "no newlines in payload" constraint no longer applies
//! at the wire layer.

use std::thread;
use std::time::Duration;

use wat::comms::{RecvError, SendError};
use wat::comms::process::pair;

#[test]
fn probe_slice3c_pair_constructs_successfully() {
    // Verifies the libc::pipe → OwnedFd wrapping path works under
    // the generic T parameter.
    let result = pair::<String>();
    assert!(result.is_ok(), "pair() must return Ok; got {:?}", result.err());
    let (_tx, _rx) = result.expect("pair");
}

#[test]
fn probe_slice3c_single_string_round_trip() {
    // Verifies the core contract: a String sent is the exact same
    // String received (after the EDN roundtrip).
    let (tx, rx) = pair::<String>().expect("pair");
    tx.send("hello".to_string()).expect("send must succeed on live channel");
    let got = rx.recv().expect("recv must return the sent string");
    assert_eq!(got, "hello", "received string must equal sent string");
}

#[test]
fn probe_slice3c_fifo_ordering_preserved_across_sends() {
    // Verifies that N sends followed by N recvs preserve order.
    let (tx, rx) = pair::<String>().expect("pair");
    tx.send("first".to_string()).expect("send 1");
    tx.send("second".to_string()).expect("send 2");
    tx.send("third".to_string()).expect("send 3");
    assert_eq!(rx.recv().expect("recv 1"), "first");
    assert_eq!(rx.recv().expect("recv 2"), "second");
    assert_eq!(rx.recv().expect("recv 3"), "third");
}

#[test]
fn probe_slice3c_sender_drop_wakes_recv_with_err() {
    // Verifies that dropping the Sender causes recv to return
    // Err(RecvError) (EOF on the pipe; io_uring Read returns 0).
    let (tx, rx) = pair::<String>().expect("pair");
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        drop(tx);
    });
    let result = rx.recv();
    assert_eq!(
        result,
        Err(RecvError),
        "recv must return Err(RecvError) after sender drop (EOF)"
    );
    handle.join().expect("sender-drop thread");
}

#[test]
fn probe_slice3c_accumulator_splits_two_frames_from_one_read() {
    // Verifies that when the sender writes two EDN frames in quick
    // succession (kernel may deliver both atomically), the first
    // recv() returns frame 1 and the second recv() returns frame 2
    // WITHOUT another io_uring read.
    let (tx, rx) = pair::<String>().expect("pair");
    tx.send("alpha".to_string()).expect("send 1");
    tx.send("beta".to_string()).expect("send 2");
    let first = rx.recv().expect("recv 1");
    let second = rx.recv().expect("recv 2");
    assert_eq!(first, "alpha", "first recv must return first string");
    assert_eq!(second, "beta", "second recv must return second string");
}

#[test]
fn probe_slice3c_large_string_spans_multiple_io_uring_reads() {
    // Verifies that a String whose EDN encoding exceeds the io_uring
    // read buffer (4096) is correctly assembled across multiple loop
    // iterations of recv(). 10_000-char ASCII string — the EDN
    // encoding is even bigger (tagged-EDN wraps it), guaranteeing
    // multi-iteration reads.
    let (tx, rx) = pair::<String>().expect("pair");
    let payload: String = (0..10_000u32)
        .map(|i| (i % 26) as u8 + b'a')
        .map(|b| b as char)
        .collect();
    let payload_clone = payload.clone();
    let send_handle = thread::spawn(move || {
        tx.send(payload_clone).expect("send large");
    });
    let got = rx.recv().expect("recv large");
    assert_eq!(got.len(), payload.len(), "received length must match sent");
    assert_eq!(got, payload, "received string must equal sent string");
    send_handle.join().expect("sender thread");
}
```

Tests preserve the 6-test structure of Stone A/B but exercise the typed surface. Naming changes from `probe_slice3a_*` to `probe_slice3c_*` (the slice-stone-prefix convention; tests track the stone that owns their contract).

## Verification

```
cargo build --release                                       # MUST be clean (no new warnings)
cargo test --release --test probe_comms_process             # 6/6 PASS
cargo test --release --test probe_comms_thread              # 10/10 PASS unchanged
cargo test --release --test probe_comms_foundation          # 3/3 PASS unchanged (Slice 1)
cargo test --release --test probe_channel_primitive         # 3/3 PASS unchanged (χ-1)
cargo test --release --test probe_pidfd_primitive           # 2/2 PASS unchanged (α)
```

Per `feedback_no_hang_vector_in_additive_scorecard`: **DO NOT** run `wat_arc170_program_contracts` or any workspace tests.

## Out of scope (STOP triggers)

- **DO NOT implement try_recv** — Stone D
- **DO NOT implement Select** — Stone D
- **DO NOT implement Clone** — Stone D
- **DO NOT implement close(self)** — Stone D
- **DO NOT implement len()** — Stone D
- **DO NOT implement CommSender / CommReceiver trait impls** — Stone D
- **DO NOT add config tunable** — Stone E
- **DO NOT optimize to persistent IoUring** — Stone E
- **DO NOT add HolonRepresentable impls for substrate types** beyond `String` — Slice 4/5 territory (StdInServiceEvent, SpawnOutcome, etc., as consumers require)
- **DO NOT replace newline framing with length-prefix** — wat-edn's single-line output guarantee makes newline framing safe; Stone A's "Stone C migrates to length-prefix" prediction was wrong (correct per typed_channel.rs precedent)
- **DO NOT touch the dirty tree** (`src/fork.rs` + `src/spawn_process.rs`)
- **DO NOT touch `src/typed_channel.rs`** (existing PipeFd; Slice 5 migrates callers later)
- **DO NOT run `wat_arc170_program_contracts`** (per additive-scorecard discipline)
- **DO NOT touch Cargo.toml** (wat-edn + holon already deps; no new deps needed)
- **ZERO modifications** outside the 4-file scope (`src/edn_shim.rs` +5 LOC, `src/comms/mod.rs` +~18 LOC, `src/comms/process.rs` refactor, `tests/probe_comms_process.rs` rewrite)

## Pre-emptive ward discipline (lessons from Slices 1+2 + Stones A+B)

1. **Module-level doc honestly reflects current state** (Stone B gaze L1 lesson) — replace "Current scope (through Stone B)" with "Current scope (through Stone C)" naming the new generic-T + EDN-serialization shape.
2. **Receiver struct doc accurately reflects T-generic + cascade-aware** (Stone B gaze L1 lesson) — don't leave any "NOT generic over T (Stone C)" stale claim.
3. **Sender/Receiver struct docs explain the WIRE CHAIN** (gaze WHY-not-WHAT lesson) — name each layer: `T → HolonAST → EDN string → bytes`.
4. **All `unsafe` blocks have SAFETY comments** (Stone A round-1 forge lesson) — Stone C preserves Stone A's SAFETY comments verbatim (unsafe-block locations unchanged by Stone C's refactor; just need to be careful not to drop them during the generic-T conversion).
5. **PhantomData is a forge-relevant type-system tool** — `PhantomData<T>` makes `Sender<T>` / `Receiver<T>` invariant in T which is correct for this use case (T is consumed by Sender and produced by Receiver; invariance is honest). Document this in the field doc-comment.
6. **`impl HolonRepresentable for String` is Slice 1's FIRST concrete impl** — the doc comment names this milestone explicitly + names the roundtrip exactness invariant (`from_holon_ast(to_holon_ast(s)) == s`).
7. **`write_holon_ast_tagged` mirrors the existing `read_holon_ast_tagged` shape** (sever discipline) — symmetry signals "these are companions"; one concern per fn.

## Concrete deliverables list

1. **Edit** `src/edn_shim.rs` — add `pub fn write_holon_ast_tagged(h: &HolonAST) -> String` (5 LOC) immediately above the existing `pub fn read_holon_ast_tagged` (~line 1996)
2. **Edit** `src/comms/mod.rs` — add `impl HolonRepresentable for String { ... }` (~18 LOC) immediately after the `HolonRepresentable` trait definition (~line 64)
3. **Edit** `src/comms/process.rs` — module-level doc + `use std::marker::PhantomData;` + generic `Sender<T>` / `Receiver<T>` / `pair<T>()` + new private `decode_frame::<T>` helper above `take_frame`; `take_frame` UNCHANGED
4. **Rewrite** `tests/probe_comms_process.rs` — 6 tests use `pair::<String>()` and `String` payloads; test names migrate from `probe_slice3a_*` to `probe_slice3c_*`
5. **New file** SCORE doc: `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-3C-HOLON-REPRESENTABLE.md`

Estimated LOC: ~80-100 LOC delta across the 4 source files (most is generic-T threading + new helper + first HolonRepresentable impl). Test file is ~110 LOC rewrite.

## Critical constraints

- **DO NOT commit.** Orchestrator commits after SCORE verification + 5-ward pass per kernel-impeccability protocol.
- **Anchor cwd:** `/home/watmin/work/holon/wat-rs/` — `pwd` as first action; reject any `.claude/worktrees/` path.
- **Use `git -C`** for any git status / git diff inspections.

## Cross-references

- BRIEF-214-SLICE-3A-IO-URING-BYTES.md — Stone A foundation (bytes-only)
- BRIEF-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md — Stone B cascade contract
- WARD-PASS-3A-IO-URING-BYTES.md — Stone A 5-ward round-trip
- WARD-PASS-3B-CASCADE-AWARE-MULTI-ARM.md — Stone B 5-ward round-trip
- `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` — Slice 3 generic-T description
- `src/comms/mod.rs:58-63` — `HolonRepresentable` trait (Slice 1; Stone C provides first impl)
- `src/edn_shim.rs:1678` — `holon_ast_to_edn` (PRIVATE; Stone C's new `write_holon_ast_tagged` wraps it)
- `src/edn_shim.rs:1997` — `read_holon_ast_tagged` (existing PUBLIC; Stone C uses for recv decode)
- `src/typed_channel.rs:228-230` — existing PipeFd encode reference (`wat_edn::write` + `.push('\n')`)
- `holon::HolonAST` — the universal wire form per arc 057+ (project_holon_universal_ast)
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol
- `feedback_no_hang_vector_in_additive_scorecard` — verification discipline
