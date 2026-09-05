# ward `struere-recon` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

Reconnaissance complete. Ranked candidates below — all verified against the source, none carrying a `rune:struere` marker or an existing ⛔/⚠ block for the named hazard.

---

### 1. `ChildHandle::mark_reaped` — `/home/john/work/holon/wat-rs/src/process/handle.rs:83`
**Shape 1** (two fields, more than one writer, only some writers maintain the second). This is the strongest find: `reaped` and `cached_exit` must move together, and `mark_reaped` moves only the first. A subsequent `wait_or_cached_exit()` finds `cached_exit == None`, loses the `compare_exchange`, and enters an **unbounded `spin_loop()` on a `OnceLock` nothing will ever set**. The doc even says Drop deliberately skips `cached_exit` "because nobody can read it" — which is true for Drop and false for `mark_reaped`, whose whole point is that the handle stays alive afterward.

```rust
    /// Mark the child as already-reaped by an external wait (e.g., wat-cli's
    /// own `wait_child`). Prevents `Drop` from attempting a second reap on
    /// an already-collected pid. Call ONLY after the caller has completed a
    /// successful wait — this is the external coordination hook for the
    /// reap-once invariant.
    pub fn mark_reaped(&self) {
        self.reaped.store(true, Ordering::Release);
    }
```
and the reader that hangs, at `handle.rs:107`:
```rust
        if self.reaped.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
            loop {
                if let Some(&code) = self.cached_exit.get() {
                    return code;
                }
                std::hint::spin_loop();
            }
        }
```
The caller who "completed a successful wait" *has* the exit code — the signature throws it away. `mark_reaped(&self)` should be `mark_reaped(&self, code: i64)`.

---

### 2. `cached_stdio_peer` — `/home/john/work/holon/wat-rs/src/services/client.rs:91`
**Shape 4** (abstraction at the wrong level) plus an unenforced correlated-argument pair. The caller's intent is "give me my stderr peer"; it must instead supply six infrastructure handles, two of which — `connect_helper` (a stringly-typed symbol name) and `select` (a field projection) — must agree with each other *and* with the `addr` passed separately. Nothing types that agreement; `(stdin_addr, stdio-connect-in, |io| &io.stdout_peer)` compiles and caches a stdin peer in the stdout slot.

```rust
pub(crate) fn cached_stdio_peer(
    op: &'static str,
    span: &crate::span::Span,
    sym: &crate::runtime::SymbolTable,
    addr: crate::runtime::Value,
    connect_helper: &'static str,
    select: fn(&ThreadIO) -> &RefCell<Option<crate::runtime::Value>>,
) -> Result<crate::runtime::Value, crate::runtime::RuntimeError> {
```
Call sites at `services/verbs.rs:242`, `:104`, `:287` each re-spell the triple by hand. Secondary defect in the same function (`client.rs:112`): the cache store is silently skipped if `THREAD_IO` became `None` across the `apply_function` connect —
```rust
    THREAD_IO.with(|cell| {
        if let Some(io) = cell.borrow().as_ref() {
            *select(io).borrow_mut() = Some(peer.clone());
        }
    });
```
so a documented "cached" function can quietly never cache, reconnecting on every `println`. The doc promises the caching; the `if let` drops it.

---

### 3. `sender_receiver_from_split_fds` — `/home/john/work/holon/wat-rs/src/comms/process.rs:2066`
**Shape 3 / 5**. Every other receiver constructor in this file has a `_with_budget` sibling, and `Arc 278 Stone 1` documents `max_frame_bytes` as a per-service declared invariant. This one hardcodes the default and its doc never mentions the budget at all — the omission is invisible at the call site.

```rust
pub fn sender_receiver_from_split_fds<T: EdnRepresentable>(
    read_fd: OwnedFd,
    write_fd: OwnedFd,
) -> std::io::Result<(Sender<T>, Receiver<T>)> {
    let receiver = Receiver {
        source: Source::Pipe { read_fd },
        accumulator: RefCell::new(Vec::new()),
        max_frame_bytes: DEFAULT_MAX_FRAME_BYTES,
```
Its one caller is the child's own self-peer at `/home/john/work/holon/wat-rs/src/process/verbs.rs:378`, so a `defservice` that declares `max-message-bytes` gets that budget on the parent's read leg and silently 512 KiB on the child's read leg.

---

### 4. `mark_wat_entry` — `/home/john/work/holon/wat-rs/src/process/exec_plan.rs:144`
**Shape 1**. Two process-lifetime globals encode one fact ("we can re-exec ourselves"), and this sole writer sets the flag *first*, then takes two early returns that leave the fd uncaptured — permanently poisoning `spawn-process` for the process.

```rust
pub(crate) fn mark_wat_entry() {
    ENTERED_WAT_ENTRY.store(true, Ordering::Relaxed);
    if SELF_IMAGE_FD.load(Ordering::Relaxed) >= 0 {
        return;
    }
    let Ok(path) = std::env::current_exe() else {
        return;
    };
    let Ok((fd, _)) = open_named(path.as_os_str()) else {
        return;
    };
```
`image_source` (same file, line 96) treats exactly that state as fatal — `entered_wat: true, held_fd: None` → `Err("this process's image was not captured at entry…")`. The store belongs *after* the successful `compare_exchange`, or `SELF_IMAGE_FD >= 0` should be the only witness and `ENTERED_WAT_ENTRY` deleted.

---

### 5. `next_ticket` — `/home/john/work/holon/wat-rs/src/distribution/mcp.rs:127`
**Shape 2** (name/signature says mint, body writes) **+ Shape 1**. Reads as a pure derivation from `session`; line 129 is a hidden state advance. `session.ticket_seq` and `session.ticket` must move together, and this function moves only the first — leaving `consume_ticket` as the sole writer that keeps them consistent. Any second caller of `next_ticket` (it is `fn`, file-visible) burns a sequence number and orphans it.

```rust
fn next_ticket(session: &mut Session) -> i64 {
    use std::hash::{BuildHasher, Hasher, RandomState};
    session.ticket_seq = session.ticket_seq.saturating_add(1);
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(session.ticket_seq);
    hasher.write_i64(session.gen);
    hasher.write_u32(std::process::id());
    let t = ((hasher.finish() & JSON_SAFE_INT) as i64).max(1);
    if t == session.ticket {
        (t % (JSON_SAFE_INT as i64 - 1)).saturating_add(1).max(1)
    } else {
        t
    }
}
```
Also **Shape 5**: `handle_tools_call` (`mcp.rs:259` and `:288`) must call `reject_stale_ticket` *before* `eval_turn` and `consume_ticket` *after*, in both arms, with nothing encoding that order.

---

### 6. `wait_or_cached_exit` / `exit_status_to_i64` — `/home/john/work/holon/wat-rs/src/process/handle.rs:98` and `/home/john/work/holon/wat-rs/src/process/clone.rs:245`
**Shape 2** (a returned number whose meaning is not in the type). One `i64` carries four disjoint meanings, and `-1` is minted independently in two places for two unrelated conditions.

```rust
pub(super) fn exit_status_to_i64(status: ExitStatus) -> i64 {
    match status {
        ExitStatus::Exited(code) => code as i64,
        ExitStatus::Signaled(sig) => 128 + sig as i64,
        ExitStatus::Stopped(_) => -1, // WIFSTOPPED — only with WUNTRACED; should not fire here
    }
}
```
```rust
        let code = match self.pidfd.wait_status() {
            Ok(status) => exit_status_to_i64(status),
            Err(_) => -1, // waitid failure (rare; ECHILD or EINTR). Sentinel.
        };
```
A caller receiving `-1` cannot tell "stopped" from "waitid failed", and the `128+sig` overlay means the type admits values no consumer can decode without the prose. `ExitStatus` already exists one module away.

---

### 7. `spawn_process_peer` budget asymmetry — `/home/john/work/holon/wat-rs/src/kernel/spawn.rs:896`
**Shape 5 / 1**. Three sibling channel mints, one declared budget, silently applied to exactly one of them, with no comment marking the choice.

```rust
    let (input_tx, input_rx) = crate::comms::process::pair::<String>().map_err(|io_err| {
        ...
                reason: format!("comms::process::pair (input) failed: {}", io_err),
    })?;

    let (output_tx, output_rx) = crate::comms::process::pair_with_budget::<String>(max_frame_bytes)
        .map_err(|io_err| {
        ...
    let (err_tx, err_rx) = crate::comms::process::pair::<String>().map_err(|io_err| {
        ...
                reason: format!("comms::process::pair (err) failed: {}", io_err),
    })?;
```
The parameter is documented at `spawn.rs:598` as "the per-receiver frame-size budget" — plural receivers, singular application. Combined with #3, a declared `FOO` binds one of four legs.

---

### 8. `update_newest` — `/home/john/work/holon/wat-rs/src/distribution/staleness.rs:148`
**Shape 2** (a `&mut` param that is only sometimes written, with no signal on the skip). Two nested `if let Ok` swallow both an unreadable file and an unsupported-mtime filesystem; the caller cannot distinguish "scanned, nothing newer" from "read nothing at all."

```rust
/// Update `newest` if `path`'s mtime is later.
fn update_newest(newest: &mut Option<SystemTime>, path: &Path) {
    if let Ok(meta) = std::fs::metadata(path) {
        if let Ok(mtime) = meta.modified() {
            match *newest {
                None => *newest = Some(mtime),
                Some(n) if mtime > n => *newest = Some(mtime),
                _ => {}
            }
        }
    }
}
```
`newest_source_mtime` (line 170) then returns `Option<SystemTime>` where `None` means "nothing readable" — and `check_dev_staleness` treats `None` as "not stale, return silently." A permission-denied `src/` renders the whole guard a silent no-op. `-> Option<SystemTime>` is the plain type not enforcing the documented "newest mtime found" promise.

---

### 9. `Select::listener` — `/home/john/work/holon/wat-rs/src/comms/process.rs:1502`
**Shape 1 / 2**. Takes a bare `RawFd` with no lifetime relationship to the `Select`, stores it, and re-poll-registers it on every `select()` iteration. The "replaces the previous fd" behaviour is a silent overwrite of a resource the `Select` does not own and cannot validate.

```rust
    /// Arc 209 C0b.3a-i — register a listen fd as the reactor listener arm.
    /// On `select()`, a `PollAdd POLLIN` SQE is pushed for this fd with
    /// `LISTENER_TOKEN`. When the CQE fires, `select()` returns
    /// `Ok(SelectOutcome::Listener)`. The caller then accepts non-blocking.
    /// One listener per `Select` (re-registering replaces the previous fd).
    pub fn listener(&mut self, fd: std::os::fd::RawFd) {
        self.listener_fd = Some(fd);
    }
```
Contrast `recv(&mut self, rx: &'a Receiver<T>)` two lines below, which *does* tie the borrow to the `Select`'s lifetime. The listener arm is the one arm that can dangle. (Today's sole caller, `kernel/listener.rs:355`, keeps `sel` local and shorter-lived than `self.listener` — so this is latent, not live.)

---

### 10. `walk_files` / `walk_wat_files` — `/home/john/work/holon/wat-rs/src/distribution/staleness.rs:102` and `:124`
**Shape 2** (the doc says one thing, the body does another). Doc: "Skips **sub-directories** named `target` or `.git`." The `continue` runs before `file_type()` is consulted, so it also skips any regular *file* with those names.

```rust
fn walk_files(dir: &Path, f: &mut dyn FnMut(&Path)) {
    ...
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // Prune heavy/irrelevant subtrees.
        if name == "target" || name == ".git" {
            continue;
        }
        match entry.file_type() {
            Ok(ft) if ft.is_dir() => walk_files(&path, f),
            Ok(_) => f(&path),
```
Minor in effect (nobody names a source file `target`), but it is the same class as #8: the name-based prune and the type-based dispatch are two decisions collapsed into one branch, and the doc describes only the second.

---

**Deliberately excluded** (verified as already-documented hazards, not new): `take_frame`/`next_complete_frame` (`comms/process.rs:1169`, long Arc-278 block), `Stream`/`LazyCell` single-pass (`stream/mod.rs:52`, explicit ⛔/⚠ pair), `BootReader::read_line`'s read-ahead (`process/boot/mod.rs:360`, module doc names the mini-TCP invariant as "not optional"), `Thread::drain_and_join` order (`kernel/peer.rs:355`, "Load-bearing order" heading), `SocketListener::deny` (`kernel/listener.rs:291`, "future accepts" stated), `typed_send`'s dead `_types`/`_span` (`channel/transfer.rs:54`, retention reason given), and the `rune:struere(invariant-coupling)` field-order site at `kernel/spawn.rs:305`.
