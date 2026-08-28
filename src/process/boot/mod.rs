//! The boot wire — how a spawned child receives its program.
//!
//! # Why this exists
//!
//! Today a forked child gets its program for free: `spawn_process_peer` hands
//! `run_forms_as_server_child` a `Vec<WatAST>` that the child can simply read,
//! because fork gave it the parent's address space. `execve` ends that — it
//! replaces the address space, so the program must arrive as **bytes on a wire**.
//!
//! This module is that wire. It ships BEFORE the exec (arc 170 step 2) so the
//! stream path is proven while the closure is still there as the control; when the
//! exec lands, exactly one variable changes.
//!
//! # The shape
//!
//! ```text
//!    substrate frames 0..N        the substrate's — Config lives here
//!    marker  SubstrateDone
//!    program frames 0..M          the user's — reassembled by concatenation
//!    marker  ProgramDone
//!    ── HANDOVER ──               the stream is the program's
//! ```
//!
//! # Mini-TCP — and why the read boundary is safe
//!
//! Every frame is acked, and **the parent blocks on the ack before writing the
//! next frame**. That is the substrate's canonical mutex-replacement pattern,
//! named in arc 089 and documented at `docs/ZERO-MUTEX.md:252`: *"bounded(1) on
//! the request pipe means a producer can't enqueue another message while the
//! previous one is in-flight."*
//!
//! It is also what makes a chunked read correct here. A pipe has no peek and no
//! pushback, so a chunked read over-reads by design — if a marker lands at byte
//! 100 of a 64 KiB read, the rest is consumed and cannot be returned. Reading one
//! byte at a time is the only way to stop exactly at a delimiter, and it was
//! measured at **91.91 ms per 512 KiB frame against 0.41 ms chunked** (224x) —
//! unaffordable next to exec's ~170 ms.
//!
//! Under mini-TCP there is nothing to over-read: the parent has not written past
//! the marker, because it is blocked on that marker's ack. The class is not made
//! recoverable, it is made **unproducible**.
//!
//! **The invariant, stated once: the parent MUST NOT write past a marker until
//! that marker is acked.** Pipelining the first message behind `ProgramDone` for
//! latency silently re-opens the class.
//!
//! # Why the ack is not redundant with the kernel
//!
//! The kernel's pipe backpressure tells the parent *bytes were accepted*. An ack
//! tells it *the child received, decoded, and accepted them*. Different facts —
//! and for a startup handshake the second is the one that matters, because the
//! failure this arc exists to kill is "the child died during startup and the
//! parent proceeds not knowing." Acks also make the death **locatable**: dying on
//! a substrate frame is a bad handoff from the parent; on a program frame, a bad
//! program. Without them both read as *the child went away*.
//!
//! # The frames are EDN, read and written only by Rust
//!
//! Boot happens before any world exists in the child, so these types are never
//! registered as wat types and no wat program ever sees them. They ride the
//! existing `Sender<String>` / `Receiver<String>` process wire as EDN text, which
//! is already newline-framed (`comms/process.rs` — `edn_bytes + b'\n'`).

use crate::runtime::{RuntimeError, RuntimeErrorKind};
use crate::span::Span;

/// The per-frame transport budget. Mirrors `edn::render::DEFAULT_MAX_FRAME_BYTES`.
///
/// This is a limit on ONE FRAME, never on the program: a large program is chunked
/// across many program frames. The chunker below is what guarantees no single
/// frame exceeds it.
pub(crate) const MAX_BOOT_FRAME_BYTES: usize = crate::edn::render::DEFAULT_MAX_FRAME_BYTES;

/// A frame on the boot wire.
///
/// Markers are typed values, not magic strings — an unknown or malformed marker is
/// a located decode failure rather than a string comparison that quietly does not
/// match. They carry no count: every frame is acked, so a lost frame cannot go
/// unnoticed, and a count would be a second mechanism for a guarantee the
/// handshake already gives.
#[derive(Debug, Clone, PartialEq, wat_edn::Edn)]
#[to_edn(namespace = BOOT_NS)]
pub(crate) enum BootFrame {
    /// A slice of the current section's payload. Sections are reassembled by
    /// CONCATENATION, so a chunk boundary may fall anywhere — mid-form,
    /// mid-token, mid-string — and the reader never parses content.
    Chunk { text: String },
    /// The substrate section is complete.
    SubstrateDone,
    /// The program section is complete. The next thing on this wire belongs to
    /// the program.
    ProgramDone,
    /// The parent is holding the other end of this lifeline. Written once onto
    /// the lifeline pipe BEFORE clone, consumed by `was_spawned`. Presence of
    /// fd 3 is not enough — an inherited harness pipe is also "open."
    Here,
}

/// The child's reply to one frame.
///
/// An enum rather than a bare acknowledgement, because a reply is an OUTCOME: the
/// child may later need to say *why* it will not take a frame, and a `Refused`
/// variant joins here without a wire change. A parent that gets no ack today
/// learns only that the child stopped talking; a parent that gets a refusal
/// learns which frame and why.
#[derive(Debug, Clone, Copy, PartialEq, wat_edn::Edn)]
#[to_edn(namespace = BOOT_NS)]
pub(crate) enum BootReply {
    /// Received, decoded, accepted. Send the next frame.
    Ack,
}

/// The wire namespace every boot tag lives under.
///
/// `#[derive(Edn)]` emits the write impl AND submits an `EdnSchema` into the
/// link-time inventory that `register_builtin_types` drains — so these frames are
/// REGISTERED wat types, readable by `edn::read`, not a private Rust dialect. That
/// is deliberate: a boot frame a wat program cannot name is a stringly holdout of
/// exactly the kind arc 296 exists to delete, and the handshake is testable from
/// wat because of it.
pub(crate) const BOOT_NS: &str = "wat.boot";
const TAG_CHUNK: &str = "Chunk";
const TAG_SUBSTRATE_DONE: &str = "SubstrateDone";
const TAG_PROGRAM_DONE: &str = "ProgramDone";
const TAG_HERE: &str = "Here";
const TAG_ACK: &str = "Ack";

fn boot_err(reason: String) -> RuntimeError {
    RuntimeError::new(
        crate::rust_caller_span!(),
        RuntimeErrorKind::MalformedForm {
            head: ":wat::process::boot".into(),
            reason,
        },
    )
}

impl BootFrame {
    /// Encode to the EDN line this frame rides as.
    ///
    /// The shape comes from `#[derive(Edn)]` — there is no hand-written encoder to
    /// drift from the type. This is only the bridge to `comms`'s string wire.
    pub(crate) fn to_wire(&self) -> String {
        wat_edn::write(&wat_edn::ToEdn::to_edn(self))
    }

    /// Decode one frame.
    ///
    /// **This reader is hand-written, and that asymmetry is deliberate.** The
    /// derive registers an `EdnSchema` so `edn::read` can reconstruct these types
    /// — but `reconstruct_record` needs a `TypeEnv`, and the child has no world
    /// when it reads these frames; the frames are what BUILD the world. So the
    /// boot decoder cannot use the registry it populates.
    ///
    /// Kept small and total for that reason: three tags, structural comparison,
    /// and an unknown tag is a LOCATED refusal rather than a guess. A child must
    /// never improvise about a frame it does not recognise.
    pub(crate) fn from_wire(line: &str) -> Result<BootFrame, RuntimeError> {
        let parsed = wat_edn::parse_owned(line.trim())
            .map_err(|e| boot_err(format!("boot frame is not EDN: {e}")))?;
        // Compare the tag STRUCTURALLY (namespace + name), never through a
        // rendered string — a Display form is a second encoding to get wrong, and
        // this decoder is the thing standing between a child and a program it
        // does not understand.
        let (tag, body) = match &parsed {
            wat_edn::OwnedValue::Tagged(tag, body) => (tag, body.as_ref()),
            other => {
                return Err(boot_err(format!(
                    "boot frame must be a tagged value; got {other:?}"
                )))
            }
        };
        if tag.namespace() != BOOT_NS {
            return Err(boot_err(format!(
                "boot frame tag is not in the {BOOT_NS} namespace: #{}/{}",
                tag.namespace(),
                tag.name()
            )));
        }
        match tag.name() {
            TAG_CHUNK => {
                let text = named_string_field(body, "text")?;
                Ok(BootFrame::Chunk { text })
            }
            TAG_SUBSTRATE_DONE => Ok(BootFrame::SubstrateDone),
            TAG_PROGRAM_DONE => Ok(BootFrame::ProgramDone),
            TAG_HERE => Ok(BootFrame::Here),
            other => Err(boot_err(format!(
                "unknown boot frame tag #{BOOT_NS}/{other} — the child refuses to guess"
            ))),
        }
    }
}

/// Pull one named String field out of a derived variant's map body.
fn named_string_field(body: &wat_edn::OwnedValue, key: &str) -> Result<String, RuntimeError> {
    let entries = match body {
        wat_edn::OwnedValue::Map(entries) => entries,
        other => {
            return Err(boot_err(format!(
                "expected a map body carrying :{key}; got {other:?}"
            )))
        }
    };
    for (k, v) in entries {
        let matches = matches!(k, wat_edn::OwnedValue::Keyword(kw) if kw.name() == key);
        if matches {
            return match v {
                wat_edn::OwnedValue::String(s) => Ok(s.to_string()),
                other => Err(boot_err(format!(":{key} must be a String; got {other:?}"))),
            };
        }
    }
    Err(boot_err(format!("boot frame body is missing :{key}")))
}

impl BootReply {
    /// `self` by value, not `&self` — `BootReply` is `Copy` (a fieldless-plus-`Ack`
    /// enum), so borrowing costs a pointer to save nothing. This is an INHERENT
    /// method, not an impl of the `to_wire` trait in `crate::comms`, so its
    /// signature is ours to choose; the trait's `&self` shape is correct for the
    /// large payload types that implement it, and is not a constraint here.
    pub(crate) fn to_wire(self) -> String {
        wat_edn::write(&wat_edn::ToEdn::to_edn(&self))
    }

    /// Decode the child's reply. Anything that is not an `Ack` means the child did
    /// NOT accept the frame — the parent must never read silence or a surprise as
    /// success.
    pub(crate) fn from_wire(line: &str) -> Result<BootReply, RuntimeError> {
        let parsed = wat_edn::parse_owned(line.trim())
            .map_err(|e| boot_err(format!("boot reply is not EDN: {e}")))?;
        match &parsed {
            wat_edn::OwnedValue::Tagged(tag, _)
                if tag.namespace() == BOOT_NS && tag.name() == TAG_ACK =>
            {
                Ok(BootReply::Ack)
            }
            other => Err(boot_err(format!(
                "expected #{BOOT_NS}/{TAG_ACK}; got {other:?} — the child did not accept the frame"
            ))),
        }
    }
}

/// Split `payload` into `Chunk` frames, none exceeding the per-frame budget.
///
/// The split is on BYTES, not on form boundaries — reassembly is concatenation, so
/// a boundary may fall anywhere. It respects UTF-8 character boundaries only
/// because a `String` cannot be split mid-codepoint; that is a property of the
/// type, not a guarantee this wire needs.
///
/// Returns a located error rather than a silent truncation if a single chunk would
/// somehow exceed the budget — that would be a bug in this function, and STOP-2
/// says it must be loud.
pub(crate) fn chunk_payload(payload: &str) -> Result<Vec<BootFrame>, RuntimeError> {
    // The budget applies to the ENCODED frame, which is larger than the payload
    // (the tag, the brackets, and EDN string escaping). Leave headroom rather than
    // encode-and-hope: escaping can in the worst case double a payload's size.
    let slice_budget = MAX_BOOT_FRAME_BYTES / 2;

    let mut frames = Vec::new();
    let mut rest = payload;
    while !rest.is_empty() {
        let mut cut = slice_budget.min(rest.len());
        // Do not split a codepoint.
        while cut > 0 && !rest.is_char_boundary(cut) {
            cut -= 1;
        }
        if cut == 0 {
            return Err(boot_err(
                "boot chunker: a single codepoint exceeds the frame budget".into(),
            ));
        }
        let (head, tail) = rest.split_at(cut);
        let frame = BootFrame::Chunk {
            text: head.to_string(),
        };
        let encoded_len = frame.to_wire().len();
        if encoded_len > MAX_BOOT_FRAME_BYTES {
            return Err(boot_err(format!(
                "boot chunker produced a {encoded_len}-byte frame, over the \
                 {MAX_BOOT_FRAME_BYTES}-byte budget — this is a chunker bug, not a caller error"
            )));
        }
        frames.push(frame);
        rest = tail;
    }
    Ok(frames)
}

#[allow(dead_code)]
fn _span_type_is_used(_: &Span) {}

/// Render a form vector as the EDN the program already is — the payload a
/// program section carries.
///
/// # This replaced a SOURCE-TEXT payload, and the reason matters
///
/// Step 2 first shipped wat source (`write_wat_source`), on the stated grounds
/// that "the EDN path would emit `:wat.core/fn` and could not be read back as
/// the same form". That was true of the bridge as it stood and is no longer:
/// what EDN cannot spell now crosses verbatim, and
/// `probe_arc170_edn_bridge_unspellable::c03` holds the whole corpus — 1223
/// files — to an exact round trip.
///
/// Source text could never have worked here, for a reason no amount of printer
/// fixing would reach: `spawn-process` ships forms a MACRO built
/// (`kernel/spawn.rs:485`), and those carry hygiene scopes that source text has
/// no syntax for. Printing them dropped the scopes and the child raised
/// `HygieneScopeDivergence` — the STOP-4 that stopped step 2d the first time.
/// EDN carries them (`#wat.ast/ScopedSymbol`), so the program survives.
///
/// The inverse is [`wire_to_forms`]. Together they are a round trip, which is
/// the property the child's oracle actually needs — see `spawn_process_peer`.
pub(crate) fn forms_to_wire(forms: &[crate::ast::WatAST]) -> String {
    crate::edn::bridge::program_to_edn(forms)
}

/// Rebuild the program from the wire — the inverse of [`forms_to_wire`].
///
/// This is what makes step 2d a real handover: the child runs what it DECODED,
/// not what it inherited through the fork, so the stream is load-bearing before
/// the exec removes the inheritance entirely.
pub(crate) fn wire_to_forms(frame: &str) -> Result<Vec<crate::ast::WatAST>, RuntimeError> {
    crate::edn::bridge::edn_to_program(frame)
        .map_err(|e| boot_err(format!("program frame did not decode: {e}")))
}

// ─── The boot-phase transport ────────────────────────────────────────────────
//
// Plain `libc::read` / `libc::write` on raw fds, deliberately. This runs BEFORE
// any world exists in the child — there is no `Receiver`, no service, no wat. The
// comms layer's framing (`edn_bytes + b'\n'`) is reproduced here in its simplest
// form so the boot phase depends on nothing that boot has to build.
//
// The reader buffers: it reads in chunks and keeps whatever it pulled past a
// newline. That is safe ONLY because of mini-TCP — the writer is blocked on this
// frame's ack and has not sent the next one, so there is nothing past the newline
// to keep. See the module doc; the invariant is not optional.

/// Chunk size for boot reads. Measured: byte-at-a-time costs 91.91 ms per 512 KiB
/// against 0.41 ms chunked (224x), which is why the accumulator exists at all.
const BOOT_READ_CHUNK: usize = 64 * 1024;

/// A newline-framed reader over a raw fd, for the pre-world boot phase.
pub(crate) struct BootReader {
    fd: i32,
    acc: Vec<u8>,
}

impl BootReader {
    pub(crate) fn new(fd: i32) -> Self {
        BootReader {
            fd,
            acc: Vec::new(),
        }
    }

    /// Read one newline-terminated line, or `None` at EOF.
    ///
    /// EOF mid-handshake is not silently tolerated by callers — it means the peer
    /// went away before finishing, which is a failure with a name (see
    /// `read_section`), never an empty success.
    fn read_line(&mut self) -> Result<Option<String>, RuntimeError> {
        loop {
            if let Some(nl) = self.acc.iter().position(|b| *b == b'\n') {
                let line: Vec<u8> = self.acc.drain(..=nl).collect();
                let text = String::from_utf8(line[..nl].to_vec())
                    .map_err(|e| boot_err(format!("boot frame is not valid UTF-8: {e}")))?;
                return Ok(Some(text));
            }
            let mut buf = [0u8; BOOT_READ_CHUNK];
            // SAFETY: `buf` is a live stack array of exactly this length; `fd` is
            // the caller-supplied boot fd, open for the duration of the handshake.
            let n =
                unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(boot_err(format!("boot read failed: {err}")));
            }
            if n == 0 {
                // EOF. Anything still buffered is a truncated frame, not a frame.
                if self.acc.is_empty() {
                    return Ok(None);
                }
                return Err(boot_err(format!(
                    "boot stream ended mid-frame with {} unterminated bytes",
                    self.acc.len()
                )));
            }
            self.acc.extend_from_slice(&buf[..n as usize]);
        }
    }

    /// Read frames until the marker `is_end` accepts, concatenating every
    /// `Chunk`'s text. Acks each frame on `ack_fd` as it is accepted.
    ///
    /// Reassembly is CONCATENATION — a frame boundary may fall mid-form or
    /// mid-token and this never inspects the content. The section is parsed once,
    /// by the caller, after the marker.
    pub(crate) fn read_section(
        &mut self,
        ack_fd: i32,
        section: &str,
        is_end: impl Fn(&BootFrame) -> bool,
    ) -> Result<String, RuntimeError> {
        let mut out = String::new();
        loop {
            let line = self.read_line()?.ok_or_else(|| {
                boot_err(format!(
                    "boot stream closed during the {section} section — the peer went                      away before its terminating marker"
                ))
            })?;
            let frame = BootFrame::from_wire(&line)?;
            if is_end(&frame) {
                write_boot_line(ack_fd, &BootReply::Ack.to_wire())?;
                return Ok(out);
            }
            match frame {
                BootFrame::Chunk { text } => out.push_str(&text),
                other => {
                    return Err(boot_err(format!(
                        "unexpected {other:?} in the {section} section — a section                          carries Chunks until its own marker"
                    )))
                }
            }
            write_boot_line(ack_fd, &BootReply::Ack.to_wire())?;
        }
    }
}

/// Did a wat parent announce itself on this fd?
///
/// The routing question is not "is fd 3 open" — a harness control pipe is
/// also open. It is "did the other end write [`BootFrame::Here`] before we
/// started?" The parent writes that frame onto the lifeline BEFORE clone,
/// so a real child always finds it ready. An inherited pipe is empty, or
/// carries something that is not `#wat.boot/Here`. Either way we are a CLI.
///
/// Never blocks: `poll` timeout 0, then one `read`. Failures are "not a
/// parent," not a boot error — a false child is worse than a missed one
/// at this gate (a missed child fails the later handshake by name).
pub(crate) fn lifeline_has_here(fd: i32) -> bool {
    let mut pfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: poll interrogates readiness; timeout 0 never sleeps.
    let n = unsafe { libc::poll(&mut pfd, 1, 0) };
    if n <= 0 || pfd.revents & libc::POLLIN == 0 {
        return false;
    }
    let mut buf = [0u8; 256];
    // SAFETY: `fd` is open; we read into a stack buffer we own.
    let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
    if n <= 0 {
        return false;
    }
    let bytes = &buf[..n as usize];
    let line = match bytes.iter().position(|&b| b == b'\n') {
        Some(i) => &bytes[..i],
        None => bytes,
    };
    let Ok(text) = std::str::from_utf8(line) else {
        return false;
    };
    matches!(BootFrame::from_wire(text), Ok(BootFrame::Here))
}

/// Write one newline-terminated line to a raw fd, retrying short writes.
pub(crate) fn write_boot_line(fd: i32, line: &str) -> Result<(), RuntimeError> {
    let mut bytes = line.as_bytes().to_vec();
    bytes.push(b'\n');
    let mut written = 0usize;
    while written < bytes.len() {
        // SAFETY: `bytes` is a live Vec; the pointer and length are derived from
        // its unwritten tail. `fd` is open for the duration of the handshake.
        let n = unsafe {
            libc::write(
                fd,
                bytes[written..].as_ptr() as *const libc::c_void,
                bytes.len() - written,
            )
        };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(boot_err(format!("boot write failed: {err}")));
        }
        written += n as usize;
    }
    Ok(())
}

/// Write one frame and BLOCK until the peer acks it — mini-TCP's producer half.
///
/// This is where the parent gives up the ability to run ahead, and it is the whole
/// reason the reader may buffer: while this waits, nothing further has been sent,
/// so there is nothing past the frame's newline for a chunked read to swallow.
///
/// **This is also why the handshake cannot deadlock on a dead child.** The child
/// holds the ack pipe's write end; if it dies, that end closes, this read sees EOF,
/// and the failure is NAMED rather than a parent blocked forever. A `None` from the
/// ack reader is never "keep waiting" — it is "the peer is gone."
pub(crate) fn send_frame_and_await_ack(
    write_fd: i32,
    acks: &mut BootReader,
    frame: &BootFrame,
) -> Result<(), RuntimeError> {
    write_boot_line(write_fd, &frame.to_wire())?;
    let line = acks.read_line()?.ok_or_else(|| {
        boot_err(format!(
            "the child closed the ack channel while {frame:?} was in flight — it died \
             during startup; its reason rides the err channel"
        ))
    })?;
    // Anything that is not an Ack means the frame was not accepted. Never read a
    // surprise as success.
    BootReply::from_wire(&line)?;
    Ok(())
}

/// Deliver both sections to a child over `write_fd`, reading acks from `ack_fd`.
///
/// The substrate section carries `Config` (arc 170 step 3). It used to be empty because
/// step 2 does not exec. The marker is sent anyway so the wire's shape is the
/// final one and step 3 only fills a section that already exists.
pub(crate) fn deliver_to_child(
    write_fd: i32,
    ack_fd: i32,
    config_wire: &str,
    program_source: &str,
) -> Result<(), RuntimeError> {
    let mut acks = BootReader::new(ack_fd);
    // The SUBSTRATE section, first: what the child used to inherit through COW
    // and — once step 4 execs — must be told.
    for frame in chunk_payload(config_wire)? {
        send_frame_and_await_ack(write_fd, &mut acks, &frame)?;
    }
    send_frame_and_await_ack(write_fd, &mut acks, &BootFrame::SubstrateDone)?;
    for frame in chunk_payload(program_source)? {
        send_frame_and_await_ack(write_fd, &mut acks, &frame)?;
    }
    send_frame_and_await_ack(write_fd, &mut acks, &BootFrame::ProgramDone)?;
    Ok(())
}

/// The child's half: read both sections, acking as it goes, and return the
/// program source.
pub(crate) fn receive_in_child(
    read_fd: i32,
    ack_fd: i32,
) -> Result<(String, String), RuntimeError> {
    let mut reader = BootReader::new(read_fd);
    let substrate = reader.read_section(ack_fd, "substrate", |f| {
        matches!(f, BootFrame::SubstrateDone)
    })?;
    let program =
        reader.read_section(ack_fd, "program", |f| matches!(f, BootFrame::ProgramDone))?;
    Ok((substrate, program))
}

/// Report a boot-handshake failure on the child's err channel and terminate.
///
/// Called from the child branch, where fd 2 is already the err channel — so the
/// reason reaches the owner as a `Lost` cause on its `recv` rather than vanishing
/// into a mute exit. A child that cannot receive its program has no world to
/// build a structured error in, so this writes plain bytes: the message is the
/// diagnostic, and it is never silent.
///
/// Exits `EXIT_STARTUP_ERROR` — a boot failure IS a startup failure; exiting 2
/// would mislabel it as a runtime panic.
pub(crate) fn report_boot_failure_and_exit(reason: &str) -> ! {
    let line = format!("wat: boot handshake: {reason}\n");
    let bytes = line.as_bytes();
    let mut written = 0usize;
    while written < bytes.len() {
        // SAFETY: fd 2 is the child's err channel, dup2'd before this point;
        // `bytes` is a live buffer. Best-effort: a failed write cannot itself be
        // reported, so the loop exits on error rather than spinning.
        let n = unsafe {
            libc::write(
                2,
                bytes[written..].as_ptr() as *const libc::c_void,
                bytes.len() - written,
            )
        };
        if n <= 0 {
            break;
        }
        written += n as usize;
    }
    unsafe { libc::_exit(crate::process::EXIT_STARTUP_ERROR) };
}

/// Exhaustiveness guard for the hand-written decoder.
///
/// `#[derive(Edn)]` owns the WRITE side, so adding a `BootFrame` variant updates
/// the encoder for free — and would leave `from_wire` behind. That failure is
/// loud (an unknown tag is a located refusal, never a guess), but loud-at-runtime
/// is a rung below caught-at-build.
///
/// This match is the rung: a new variant breaks the build HERE, which sends the
/// author to `from_wire` and to the round-trip test that covers every variant.
/// It costs one match and never runs.
///
/// The proper cure is arc 296.3 — see `docs/arc/2026/06/296-diagnostics-fully-edn/
/// NOTE-pre-world-decode-is-hand-written.md`. Until then, this guard.
#[allow(dead_code)]
fn _every_boot_frame_variant_is_covered(f: &BootFrame) {
    match f {
        BootFrame::Chunk { .. } => {}
        BootFrame::SubstrateDone => {}
        BootFrame::ProgramDone => {}
        BootFrame::Here => {}
    }
}

#[allow(dead_code)]
fn _every_boot_reply_variant_is_covered(r: &BootReply) {
    match r {
        BootReply::Ack => {}
    }
}

// ─── The substrate section's payload: Config ─────────────────────────────────

/// `Config` on the wire, as EDN.
///
/// # Why this exists at all
///
/// Until step 4, `Config` reached the child through COW — the fork copied the
/// parent's address space and the snapshot came along for free. `execve` is
/// exactly the thing that ends that, so the substrate section stops being empty:
/// what the child used to inherit, it must now be TOLD.
///
/// # The exhaustive destructure is the point
///
/// The `let Config { .. } = cfg` below binds every field by name with no `..`
/// rest pattern, so ADDING A FIELD TO `Config` BREAKS THIS BUILD. That is
/// deliberate and it is the whole guarantee: a field that silently defaulted in
/// the child is precisely the parent/child divergence this mechanism exists to
/// prevent, and it would be invisible — the child would just quietly run under
/// different settings. Do not reach for `..`.
pub(crate) fn substrate_to_wire(cfg: Option<&crate::config::Config>, env_fn: &str) -> String {
    format!("{{:config {} :env-fn {:?}}}", config_to_wire(cfg), env_fn)
}

/// Split the substrate section back into its parts. Both fields are REQUIRED —
/// a missing one is a located error, never a default, for the same reason the
/// encode side destructures exhaustively.
pub(crate) fn wire_to_substrate(
    frame: &str,
) -> Result<(Option<crate::config::Config>, String), RuntimeError> {
    let parsed = wat_edn::parse_owned(frame.trim())
        .map_err(|e| boot_err(format!("substrate section is not EDN: {e}")))?;
    let wat_edn::OwnedValue::Map(fields) = &parsed else {
        return Err(boot_err(format!(
            "substrate section must be a map, got {}",
            parsed.type_name()
        )));
    };
    let get = |want: &str| {
        fields.iter().find_map(|(k, v)| match k {
            wat_edn::Value::Keyword(kw) if kw.namespace().is_none() && kw.name() == want => Some(v),
            _ => None,
        })
    };
    let cfg_val =
        get("config").ok_or_else(|| boot_err("substrate section is missing :config".to_owned()))?;
    let env_fn = match get("env-fn") {
        Some(wat_edn::Value::String(s)) => s.as_ref().to_owned(),
        Some(other) => {
            return Err(boot_err(format!(
                ":env-fn must be a String, got {}",
                other.type_name()
            )))
        }
        None => return Err(boot_err("substrate section is missing :env-fn".to_owned())),
    };
    Ok((wire_to_config(&wat_edn::write(cfg_val))?, env_fn))
}

fn config_to_wire(cfg: Option<&crate::config::Config>) -> String {
    let Some(cfg) = cfg else {
        // No snapshot: the program forms carry their own setters (the
        // entry-file discipline). `nil` says so explicitly rather than by
        // absence.
        return "nil".to_owned();
    };
    // EXHAUSTIVE — see the doc above. No `..`.
    let crate::config::Config {
        capacity_mode,
        global_seed,
        dim_count,
        presence_sigma_ast,
        coincident_sigma_ast,
        redef_allowed,
        eval_redef_allowed,
        max_fire_rounds,
    } = cfg;

    let ast_field = |a: &Option<crate::ast::WatAST>| match a {
        Some(ast) => crate::edn::bridge::program_to_edn(std::slice::from_ref(ast)),
        None => "nil".to_owned(),
    };
    let mode = match capacity_mode {
        crate::config::CapacityMode::Error => ":error",
        crate::config::CapacityMode::Panic => ":panic",
    };
    format!(
        "{{:capacity-mode {mode} :global-seed {global_seed} :dim-count {dim_count} \
         :presence-sigma {} :coincident-sigma {} :redef-allowed {redef_allowed} \
         :eval-redef-allowed {eval_redef_allowed} :max-fire-rounds {max_fire_rounds}}}",
        ast_field(presence_sigma_ast),
        ast_field(coincident_sigma_ast),
    )
}

/// Rebuild `Config` from the substrate section — the inverse of
/// [`config_to_wire`].
///
/// Every field is REQUIRED. A missing one is a located error, never a default:
/// defaulting here would reintroduce exactly the silent parent/child divergence
/// the exhaustive destructure exists to prevent, just on the read side.
pub(crate) fn wire_to_config(frame: &str) -> Result<Option<crate::config::Config>, RuntimeError> {
    let trimmed = frame.trim();
    if trimmed == "nil" || trimmed.is_empty() {
        return Ok(None);
    }
    let parsed = wat_edn::parse_owned(trimmed)
        .map_err(|e| boot_err(format!("substrate section is not EDN: {e}")))?;
    let wat_edn::OwnedValue::Map(fields) = &parsed else {
        return Err(boot_err(format!(
            "substrate section must be a Config map, got {}",
            parsed.type_name()
        )));
    };
    let get = |want: &str| {
        fields.iter().find_map(|(k, v)| match k {
            wat_edn::Value::Keyword(kw) if kw.namespace().is_none() && kw.name() == want => Some(v),
            _ => None,
        })
    };
    let need = |want: &str| {
        get(want).ok_or_else(|| boot_err(format!("substrate Config is missing :{want}")))
    };
    let int = |want: &str| -> Result<i64, RuntimeError> {
        match need(want)? {
            wat_edn::Value::Integer(n) => Ok(*n),
            other => Err(boot_err(format!(
                ":{want} must be an integer, got {}",
                other.type_name()
            ))),
        }
    };
    let boolean = |want: &str| -> Result<bool, RuntimeError> {
        match need(want)? {
            wat_edn::Value::Bool(b) => Ok(*b),
            other => Err(boot_err(format!(
                ":{want} must be a boolean, got {}",
                other.type_name()
            ))),
        }
    };
    let ast = |want: &str| -> Result<Option<crate::ast::WatAST>, RuntimeError> {
        match need(want)? {
            wat_edn::Value::Nil => Ok(None),
            other => {
                let text = wat_edn::write(other);
                let mut forms = crate::edn::bridge::edn_to_program(&text)
                    .map_err(|e| boot_err(format!(":{want} did not decode as a form: {e}")))?;
                match forms.len() {
                    1 => Ok(Some(forms.pop().expect("len checked"))),
                    n => Err(boot_err(format!(
                        ":{want} carried {n} forms, want exactly 1"
                    ))),
                }
            }
        }
    };
    let capacity_mode = match need("capacity-mode")? {
        wat_edn::Value::Keyword(kw) if kw.name() == "error" => crate::config::CapacityMode::Error,
        wat_edn::Value::Keyword(kw) if kw.name() == "panic" => crate::config::CapacityMode::Panic,
        other => {
            return Err(boot_err(format!(
                ":capacity-mode must be :error or :panic, got {}",
                wat_edn::write(other)
            )))
        }
    };
    Ok(Some(crate::config::Config {
        capacity_mode,
        global_seed: int("global-seed")? as u64,
        dim_count: int("dim-count")? as usize,
        presence_sigma_ast: ast("presence-sigma")?,
        coincident_sigma_ast: ast("coincident-sigma")?,
        redef_allowed: boolean("redef-allowed")?,
        eval_redef_allowed: boolean("eval-redef-allowed")?,
        max_fire_rounds: int("max-fire-rounds")? as usize,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_round_trip_through_the_wire() {
        for f in [
            BootFrame::Chunk {
                text: "(:user::main)".into(),
            },
            BootFrame::Chunk {
                text: "with \"quotes\" and\nnewlines".into(),
            },
            BootFrame::SubstrateDone,
            BootFrame::ProgramDone,
            BootFrame::Here,
        ] {
            let wire = f.to_wire();
            assert!(
                !wire.contains('\n'),
                "an encoded frame must carry no RAW newline — the wire is \
                 newline-framed, so a raw newline would split one frame into two: {wire:?}"
            );
            assert_eq!(BootFrame::from_wire(&wire).expect("decode"), f);
        }
    }

    #[test]
    fn ack_round_trips() {
        assert_eq!(
            BootReply::from_wire(&BootReply::Ack.to_wire()).expect("decode"),
            BootReply::Ack
        );
    }

    #[test]
    fn an_unknown_tag_is_refused_not_guessed() {
        // A DIFFERENTIAL, not a substring check: the only difference between these
        // two lines is the tag name, so a refusal of the second can be attributed
        // to the tag and nothing else.
        assert!(
            BootFrame::from_wire("#wat.boot/ProgramDone {}").is_ok(),
            "the control must decode — otherwise the negative case proves nothing"
        );
        assert!(
            BootFrame::from_wire("#wat.boot/Sideways {}").is_err(),
            "an unrecognised tag must be refused; a child must never improvise"
        );
        // And a tag from another namespace is refused too, so `Chunk` alone is not
        // a password.
        assert!(BootFrame::from_wire("#other.ns/Chunk {:text \"x\"}").is_err());
    }

    #[test]
    fn an_empty_or_foreign_pipe_is_not_a_parent() {
        use std::os::fd::AsRawFd;
        let (r, w) = super::super::clone::make_pipe(":test").expect("pipe");
        assert!(
            !lifeline_has_here(r.as_raw_fd()),
            "an inherited empty pipe must not route us into the child path"
        );
        write_boot_line(w.as_raw_fd(), "watmin").expect("write prose");
        assert!(
            !lifeline_has_here(r.as_raw_fd()),
            "prose on fd 3 is a harness, not a wat parent"
        );
    }

    #[test]
    fn here_on_the_lifeline_is_a_parent() {
        use std::os::fd::AsRawFd;
        let (r, w) = super::super::clone::make_pipe(":test").expect("pipe");
        write_boot_line(w.as_raw_fd(), &BootFrame::Here.to_wire()).expect("write Here");
        assert!(
            lifeline_has_here(r.as_raw_fd()),
            "only #wat.boot/Here on the lifeline answers \"a wat parent spawned me\""
        );
    }

    #[test]
    fn an_ack_that_is_not_an_ack_is_refused() {
        // The child answering a frame with anything else means it did not accept
        // it — the parent must not read that as success.
        assert!(BootReply::from_wire("#wat.boot/ProgramDone {}").is_err());
    }

    #[test]
    fn chunking_never_exceeds_the_frame_budget() {
        // A payload well past one frame, with multibyte characters at the split
        // points to prove the codepoint guard holds.
        let payload: String = "λ(:demo::x 1)".repeat(200_000);
        let frames = chunk_payload(&payload).expect("chunk");
        assert!(frames.len() > 1, "a large payload must span frames");
        for f in &frames {
            assert!(
                f.to_wire().len() <= MAX_BOOT_FRAME_BYTES,
                "every encoded frame must fit the budget"
            );
        }
        // Reassembly is concatenation — the whole point.
        let rejoined: String = frames
            .iter()
            .map(|f| match f {
                BootFrame::Chunk { text } => text.as_str(),
                _ => panic!("chunk_payload emits only Chunks"),
            })
            .collect();
        assert_eq!(
            rejoined, payload,
            "concatenation must reproduce the payload exactly"
        );
    }

    /// The transport, driven end-to-end over a REAL pipe pair.
    ///
    /// A writer sends a chunked section plus its marker; a `BootReader` reassembles
    /// it and acks each frame. This is the mechanism the handshake rests on, proven
    /// before either side of the fork is rewired.
    #[test]
    fn a_section_round_trips_over_a_real_pipe_with_acks() {
        // data: writer → reader.   ack: reader → writer.
        let (data_r, data_w) = super::super::clone::make_pipe(":test").expect("data pipe");
        let (ack_r, ack_w) = super::super::clone::make_pipe(":test").expect("ack pipe");
        use std::os::fd::AsRawFd;

        // A payload big enough to span frames, with multibyte characters so a
        // boundary can land mid-codepoint if the chunker is wrong.
        let payload: String = "λ(:demo::form 1)".repeat(80_000);
        let frames = chunk_payload(&payload).expect("chunk");
        assert!(
            frames.len() > 1,
            "the payload must span frames for this to prove anything"
        );

        let data_w_fd = data_w.as_raw_fd();
        let ack_r_fd = ack_r.as_raw_fd();
        let expected_acks = frames.len() + 1; // every chunk, plus the marker

        // The writer runs on another thread because mini-TCP BLOCKS it: it waits
        // for each ack before sending the next frame, exactly as the parent will.
        let writer = std::thread::spawn(move || {
            let mut acks = BootReader::new(ack_r_fd);
            for f in frames {
                write_boot_line(data_w_fd, &f.to_wire()).expect("write chunk");
                let line = acks.read_line().expect("ack read").expect("ack present");
                assert_eq!(
                    BootReply::from_wire(&line).expect("ack decode"),
                    BootReply::Ack
                );
            }
            write_boot_line(data_w_fd, &BootFrame::ProgramDone.to_wire()).expect("write marker");
            let line = acks
                .read_line()
                .expect("final ack read")
                .expect("final ack present");
            assert_eq!(
                BootReply::from_wire(&line).expect("ack decode"),
                BootReply::Ack
            );
            expected_acks
        });

        let mut reader = BootReader::new(data_r.as_raw_fd());
        let got = reader
            .read_section(ack_w.as_raw_fd(), "program", |f| {
                matches!(f, BootFrame::ProgramDone)
            })
            .expect("read_section");

        let acked = writer.join().expect("writer thread");
        assert_eq!(
            acked, expected_acks,
            "every frame AND the marker must be acked"
        );
        assert_eq!(
            got, payload,
            "concatenation must reproduce the payload to the byte"
        );
    }

    /// A stream that stops before its marker is a NAMED failure, never an empty
    /// success — the parent going away mid-handshake is exactly the hidden-failure
    /// shape this arc exists to kill.
    #[test]
    fn a_section_that_ends_before_its_marker_is_a_named_failure() {
        let (data_r, data_w) = super::super::clone::make_pipe(":test").expect("data pipe");
        let (ack_r, ack_w) = super::super::clone::make_pipe(":test").expect("ack pipe");
        use std::os::fd::AsRawFd;

        write_boot_line(
            data_w.as_raw_fd(),
            &BootFrame::Chunk {
                text: "(:demo::x 1)".into(),
            }
            .to_wire(),
        )
        .expect("write chunk");
        drop(data_w); // EOF before the marker — the peer vanished.
                      // ack_r stays ALIVE: the reader still acks the chunk it did receive, and
                      // dropping the read end would make that ack fail with EPIPE — surfacing a
                      // write error instead of the truncation this test is about.
        let _ack_r_held = ack_r;

        let mut reader = BootReader::new(data_r.as_raw_fd());
        assert!(
            reader
                .read_section(ack_w.as_raw_fd(), "program", |f| matches!(
                    f,
                    BootFrame::ProgramDone
                ))
                .is_err(),
            "a section whose marker never arrives must NOT read as an empty success"
        );
    }

    /// Parent half of the handshake: the child is already gone, and the parent
    /// does not hold a copy of the ack write end. `deliver_to_child` must name
    /// that death. Holding the write end is the hang `spawn_process_peer`
    /// used to have — this test is the invariant that drop is load-bearing.
    #[test]
    fn deliver_to_child_names_death_when_the_ack_write_end_is_gone() {
        let (data_r, data_w) = super::super::clone::make_pipe(":test").expect("data pipe");
        let (ack_r, ack_w) = super::super::clone::make_pipe(":test").expect("ack pipe");
        use std::os::fd::AsRawFd;
        // Keep the data read end open so the first frame write succeeds; the
        // failure under test is the ack wait, not EPIPE on the write. Drop
        // only the ack write end — that is the parent's leaked `output_tx`.
        let _data_r_held = data_r;
        drop(ack_w);

        let err = deliver_to_child(
            data_w.as_raw_fd(),
            ack_r.as_raw_fd(),
            "{:config nil :env-fn \"\"}",
            "(:user::main)",
        )
        .expect_err("a dead child must not look like a successful handoff");
        let first = chunk_payload("{:config nil :env-fn \"\"}")
            .expect("the test's own config wire must chunk")
            .into_iter()
            .next()
            .expect("a non-empty config wire yields at least one frame");
        let expected = format!(
            "the child closed the ack channel while {first:?} was in flight — it died \
             during startup; its reason rides the err channel"
        );
        match err.kind() {
            RuntimeErrorKind::MalformedForm { head, reason } => {
                assert_eq!(head, ":wat::process::boot");
                assert_eq!(reason, &expected);
            }
            other => panic!("dead-child handshake must be a named boot MalformedForm, got {other:?}"),
        }
    }

    /// The control for the truncation test: the SAME stream, with its marker,
    /// reads clean. Without this the negative case only proves "something failed."
    #[test]
    fn the_same_section_with_its_marker_reads_clean() {
        let (data_r, data_w) = super::super::clone::make_pipe(":test").expect("data pipe");
        let (ack_r, ack_w) = super::super::clone::make_pipe(":test").expect("ack pipe");
        use std::os::fd::AsRawFd;
        let _ack_r_held = ack_r;

        write_boot_line(
            data_w.as_raw_fd(),
            &BootFrame::Chunk {
                text: "(:demo::x 1)".into(),
            }
            .to_wire(),
        )
        .expect("write chunk");
        write_boot_line(data_w.as_raw_fd(), &BootFrame::ProgramDone.to_wire())
            .expect("write marker");

        let mut reader = BootReader::new(data_r.as_raw_fd());
        let got = reader
            .read_section(ack_w.as_raw_fd(), "program", |f| {
                matches!(f, BootFrame::ProgramDone)
            })
            .expect("a terminated section must read clean");
        assert_eq!(got, "(:demo::x 1)");
    }

    #[test]
    fn an_empty_payload_produces_no_frames() {
        assert!(chunk_payload("").expect("chunk").is_empty());
    }
}
