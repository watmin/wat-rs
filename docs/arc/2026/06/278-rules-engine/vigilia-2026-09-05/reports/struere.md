# ward `struere` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

I have enough verified ground. Report follows.

---

# `struere` — wat-rs @ `21530efab` (src/, tests/, wat/)

Ground: values flowing through vs. mutating in place; composition holding under load; abstraction at the caller's level. All citations read this session.

---

## L1 — DEFECTS

### 1. Four shipping sites bypass `record_token` / `record_tokens` — and the doc says they cannot exist

**What.** `src/rete/kernel/fire/pass/mod.rs:38` defines the one insertion verb for "a token reached node N": gate on `beta_readers`, count the census, push the durable beta, push the round delta — one act. Its doc at `:24-28` states the structural claim verbatim:

> *"It was previously a SECOND statement next to the push at each of **thirteen sites** — two things that must agree, written twice, in **seven files**. Here they are one act, so a future site cannot push without counting or count without pushing."*

That claim is false at HEAD. Four shipping sites still open-code the three-part act:

| site | file:line |
|---|---|
| `left_activate_join` | `src/rete/kernel/fire/pass/mod.rs:150-157` |
| `dispatch_where_tests`, proven-reuse arm | `src/rete/kernel/fire/mod.rs:2069-2073` |
| `dispatch_where_tests`, eval arm (tree branch) | `src/rete/kernel/fire/mod.rs:2079-2083` |
| `dispatch_where_tests`, eval arm (fallback branch) | `src/rete/kernel/fire/mod.rs:2103-2107` |

**Mechanism.** Count the converted call sites: `filter.rs` 3, `filter_after_join.rs` 3, `accumulate.rs` 2, `hash_join.rs` 2, `root_join.rs` 1, `join_after_filter.rs` 1 = **12 in 6 files**. The doc's own prior-state figure was 13 in 7. The thirteenth site *is* `left_activate_join` — it lives in the file that defines the verb, and it was not converted. `dispatch_where_tests` was never in the count at all: it sits in `fire/mod.rs`, outside the `pass/` tree the extraction audited.

**Why it matters.** This is the identical shape to the D2 cure that HEAD's own commit message celebrates ("*the bypass is now unrepresentable*"). It is representable. The `JoinRightIndex` cure at `src/rete/kernel/session.rs:236` explicitly cites `record_token` as its precedent — *"Same shape, and the same reason, as `fire::pass::record_token` and its beta census"* — so the precedent it rests on is weaker than stated. Today all four sites happen to pair correctly, so there is no wrong answer *now*; what is broken is the guarantee. The `beta_written` census is what the round-census tests read, so a drift here stamps a broken arm green.

**Fix.** Replace the four bodies with the verb. Both are one-liners and both borrow-check: `sink.wm.beta` / `sink.d_beta` are disjoint fields of one `&mut WhereSink`, and `arm.beta_readers` is a shared read across a `&mut wm.beta`. Then either the doc's claim becomes true, or — better — make it structural: `beta` and `d_beta` become private fields of one `BetaWrite` type whose only door is `record_token`, exactly as `JoinRightIndex`/`RightIndexWriter` did.

---

### 2. `ChildHandle::mark_reaped` + `wait_or_cached_exit` compose into an unbounded spin

**Where.** `src/process/handle.rs:82-84` and `:98-125`.

```rust
pub fn mark_reaped(&self) {
    self.reaped.store(true, Ordering::Release);   // :83
}
```

`wait_or_cached_exit` then:
1. `cached_exit.get()` → `None` (nothing set it),
2. `compare_exchange(false→true)` → **fails** (already true),
3. falls into `loop { if let Some(&code) = self.cached_exit.get() { return code } std::hint::spin_loop() }` (`:107-113`).

**Mechanism.** `reaped` and `cached_exit` are two fields encoding one fact ("the exit code has been collected"). There are three writers of `reaped` — `mark_reaped`, `wait_or_cached_exit`, `Drop` — and exactly **one** of them writes `cached_exit`. The spin loop's exit condition is "the compare-exchange winner will publish"; after `mark_reaped` there is no winner and never will be. Result: a busy spin at 100% CPU with no timeout, no yield, no bound. `Drop`'s omission is documented as safe (`:20-22`, "nobody can read it — the Arc is going away") and is; `mark_reaped`'s is the same omission where the handle *stays alive*, which is its entire purpose.

The caller who "completed a successful wait" (`:79-81`) is holding the exit code. The signature throws it away.

**Fix.** `mark_reaped(&self, code: i64)`, setting `cached_exit` before `reaped` — or collapse the pair into a single `OnceLock<i64>` where presence *is* reapedness, and delete the `AtomicBool`.

**Honest scope — and a second finding on top of it.** `mark_reaped`, `wait_or_cached_exit`, and `ChildHandle::new` have **zero callers anywhere** — src/, tests/, wat/, wat-scripts/, crates/, tools/. `Value::wat__kernel__ChildHandle` is never constructed either (only matched). So this cannot fire today. But the type is `pub` on a `pub mod` (`src/process/mod.rs:45,63`), the method docs claim live consumers that do not exist (`:88-91`, *"Used by arc-112's unified ProgramHandle Forked variant + Process/join-result"*), and `src/types.rs:2551` already records that *"the ChildHandle is no longer wat-visible"*. Meanwhile the dead variant still costs a match arm in the purity wall (`src/check.rs:13895`), the `KeyEligibility` wall that `clippy.toml`'s `ignore-interior-mutability` exemption rests on (`src/value/value.rs:1474`), `src/edn_shim.rs:4053`, `src/value/observe.rs:420`, `src/closure_extract.rs:2255`, and `src/runtime.rs:9346, 12544, 18522`. **Either retire the variant and the module, or wire it and fix the spin. Half-built is the current state.**

---

### 3. A declared `max-message-bytes` binds one of four receiver legs

**Where.** `src/kernel/spawn.rs:882-933`. The parameter is documented at `:599` as *"the per-receiver frame-size budget"* — plural receivers, and it is applied to exactly one:

| leg | line | budget |
|---|---|---|
| `input` (parent→child; child holds `input_rx`) | `:896` | `pair()` → `DEFAULT_MAX_FRAME_BYTES` |
| `output` (child→parent) | `:906` | `pair_with_budget(max_frame_bytes)` ✓ |
| `err` (child→parent, structured exit) | `:925` | `pair()` → `DEFAULT_MAX_FRAME_BYTES` |
| child's self-peer over fd0/fd1 | `src/process/verbs.rs:377` | hardcoded default (see below) |

**Mechanism.** Both directions of the mistake are live. Lowering the declared budget below 512 KiB does not tighten `input`/`err`; raising it above 512 KiB does not loosen them, so a message the user declared legal is refused with `RecvError::FrameTooLarge` on three of four legs. Per `take_frame`'s own doc (`src/comms/process.rs:1145-1148`), `FrameTooLarge` is the "tear down the peer immediately" outcome — so an over-budget structured exit on the err channel takes the peer down rather than being delivered.

The fourth leg cannot even be fixed at the call site: `sender_receiver_from_split_fds` (`src/comms/process.rs:2066`) hardcodes `max_frame_bytes: DEFAULT_MAX_FRAME_BYTES` at `:2073`. It is the **only** receiver constructor in that file without a `_with_budget` sibling (`pair`/`pair_with_budget` at `:1962`/`:1975`; `sender_receiver_from_fd`/`_with_budget` at `:2024`/`:2037`), and its doc never mentions the budget at all, so the omission is invisible from `verbs.rs:377`.

**Fix.** Add `sender_receiver_from_split_fds_with_budget`; thread `max_frame_bytes` to all three `spawn_process_peer` pairs and to the child's self-peer. The deeper fix is that a budget is a property of the *peer*, not of one pipe — one `PeerBudget` value constructed once and consumed by every leg makes "which legs got it" unaskable.

---

## L2 — WEAKNESSES

### 4. `cached_stdio_peer`: three free parameters encode one enum, and the cache write can silently vanish

**Where.** `src/services/client.rs:91-101`.

```rust
pub(crate) fn cached_stdio_peer(
    op: &'static str, span: &Span, sym: &SymbolTable,
    addr: Value,
    connect_helper: &'static str,
    select: fn(&ThreadIO) -> &RefCell<Option<Value>>,
) -> Result<Value, RuntimeError>
```

The caller's intent is "give me this thread's stderr peer." It must instead keep **four** facts mutually consistent by hand — `primed.<x>_addr`, the stringly-typed `":wat::kernel::stdio-connect-<x>"`, the field projection `|io| &io.<x>_peer`, and (at the caller) `":wat::kernel::stdio-write-<x>"`. Nothing types the agreement. Re-spelled by hand at four sites: `src/services/verbs.rs:82, 92, 104, 287`.

Consequence of one slip: `(stdin_addr, stdio-connect-in, |io| &io.stdout_peer)` compiles and caches the stdin peer in the stdout slot. Because it is *cached*, the misroute is permanent for the thread's life and invisible to any test that only asserts "something was written."

**Secondary, same function** — `:114-118`:
```rust
THREAD_IO.with(|cell| {
    if let Some(io) = cell.borrow().as_ref() {
        *select(io).borrow_mut() = Some(peer.clone());
    }
});
```
The `ThreadIO` was verified present at `:106`, but the `apply_function` connect at `:112` runs arbitrary wat in between. If it uninstalled the `ThreadIO`, the `if let` drops the cache write and returns `Ok`. A function named "cached" then reconnects on every `println` forever, and says nothing.

**Fix.** `enum StdioStream { Out, Err, In }` with `addr_of(&PrimedStdio)`, `connect_helper()`, `write_helper()`, and `slot(&ThreadIO)` as methods. The caller names one value; the four facts stop being separable. For the cache write, make the `None` branch an explicit outcome rather than a silent skip.

---

### 5. `mark_wat_entry` sets the flag before it knows the fd exists

**Where.** `src/process/exec_plan.rs:143-166`.

```rust
pub(crate) fn mark_wat_entry() {
    ENTERED_WAT_ENTRY.store(true, Ordering::Relaxed);      // :144 — unconditional, FIRST
    if SELF_IMAGE_FD.load(Ordering::Relaxed) >= 0 { return; }
    let Ok(path) = std::env::current_exe() else { return; };   // :148 — leaves fd unset
    let Ok((fd, _)) = open_named(path.as_os_str()) else { return; };  // :151 — io::Error dropped
    ...
}
```

`image_source` (`:84-101`) treats exactly `(entered_wat: true, held_fd: None)` as fatal:

> `"this process's image was not captured at entry; refusing to execve a /proc path or a deleted current_exe() readlink"`

**Mechanism.** Two process-lifetime globals encode one fact ("we can re-exec ourselves"), and the single writer commits the first before establishing the second. Any failure to open the running image — EMFILE at startup, a filesystem that refuses `O_PATH`, a binary unlinked before entry — permanently poisons `spawn-process` for the whole process. The `std::io::Error` naming the real cause is discarded at `:151`, and what the user sees is a *policy* message about `/proc` paths that describes a refusal the code never actually made.

Note the refusal itself may well be the intended design for the deleted-exe case. What is not defensible is that three distinct failure modes collapse into it silently, with the diagnostic thrown away and the `()` return making the failure unobservable at the point it happens.

**Fix.** Store `ENTERED_WAT_ENTRY` only after a successful capture — or delete it and let `SELF_IMAGE_FD >= 0` be the whole witness, so the two cannot disagree. Return `io::Result<()>` so the open error reaches a log line.

---

### 6. `eval_form_against_defs` returns a pair whose halves must agree, and its two callers disagree about them

**Where.** `src/runtime.rs:29320-29325`:
```rust
) -> Result<(Value, Option<SymbolTable>), EvalBreak>
```
The `Value` is the wat `FormOutcome` enum (Declared / Evaluated / CheckFailed / Raised); the `Option<SymbolTable>` must line up with which variant came back. The doc at `:29316-29317` states the pairing in prose. The type does not.

Its two callers read that prose differently:
- `src/runtime.rs:29300` — `Ok(eval_form_against_defs(&form, defs, env, sym)?.0)` discards the table entirely.
- `src/distribution/mcp.rs:365-404` — pushes `form` into `session.defs` unconditionally inside the `"Declared"` arm (`:383`), then installs the table separately under `if keep { if let Some(sym) = next_sym { … } }` (`:400-404`). The two halves of the session's world are written by two statements with a `match` between them.

**Fix.** One Rust enum: `Declared(SymbolTable) | Evaluated(Value, SymbolTable) | CheckFailed(Value) | Raised(Value)`. The doc at `:29313-29315` deliberately keeps the *wat* enum as the shipped contract to avoid a second definition — fine; that argument is about the wat-visible outcome, and does not require the Rust return channel to be a tuple whose halves can disagree.

### 6b. The panic-safety comment beside it is now false

`src/distribution/mcp.rs:262-266`:
> *"we do not persist a half-mutated `defs` across the catch (eval_turn only pushes on Declared after the eval returns)"*

`eval_turn` (`:344-357`) loops over **every** form in the payload — that multi-form behaviour was added later, and its own ⚠ at `:331-337` records why. If form 3 panics, forms 1–2 have already been pushed to `defs` and their tables installed, and `AssertUnwindSafe` keeps that state. The comment was true when the turn was one form. It is an alibi now.

---

### 7. `next_ticket` advances a counter and hands back a value it does not install

**Where.** `src/distribution/mcp.rs:127-140`. `session.ticket_seq` is bumped at `:129`; `session.ticket` is written only by `consume_ticket` at `:143`. Two fields, one fact ("the current rendezvous"), and the function named for the mint maintains only one of them. Any second caller burns a sequence number and orphans it — and the anti-repeat guard at `:135` (`if t == session.ticket`) is only meaningful because the caller is *about to* overwrite `session.ticket`, which nothing in the signature says.

Compounding it: `handle_tools_call` must call `reject_stale_ticket` **before** `eval_turn` and `consume_ticket` **after**, in both the `"eval"` (`:259, :276`) and `"reset"` (`:288, :291`) arms, with nothing encoding that order. The ticket's entire purpose is preventing a dual-fire; the ordering that delivers it is convention.

**Fix.** Fold into `consume_ticket` (make `next_ticket` a private inner or inline it), and lift the guard/turn/rotate sequence into one `fn with_ticket(session, f: impl FnOnce(&mut Session) -> String) -> String` so the three steps cannot be reordered or dropped.

---

## L3 — JUDGEMENT

**8. `src/alloc_counter.rs:76` cites a function that does not exist.** The module header points both ceiling doors at `rete::kernel::session::check_session_ceiling`. The real name is `session_ceiling_breach` (`src/rete/kernel/session.rs:1533`); `check_session_ceiling` appears nowhere in `src/`. It is also stale in `tests/rete/probe_arc278_import_accounting_ceiling.wat:3`. Two-line fix; worth it because this header is the one place that explains the per-session-vs-per-thread distinction, and a reader who greps the name it gives finds nothing.

**9. `session_ceiling_breach` conflates "no encoding context" with "the default limit."** `src/rete/kernel/session.rs:1537-1540` does `sym.encoding_ctx().map(|c| c.config.max_session_bytes).unwrap_or(DEFAULT_MAX_SESSION_BYTES)`. A program that configured 64 MiB would be judged against 1 GiB — 16× its contract — on any path reaching this without a context. The sibling `require_encoding_ctx` (`src/runtime.rs:26578`) already exists and raises `NoEncodingCtx` for exactly this. I could not establish that the arm is reachable (the note at `:26575-26576` says the table carries a ctx after freeze, which would make it dead), so this is a shape flag, not a bug report.

**10. `Hologram.capacity` names a bound nothing enforces.** `src/hologram.rs:63-77`: the field is a *slot count* derived from `kanerva_capacity(d)`, cached beside `slots` whose length it must equal — an invariant only `make` (`:100`) maintains, where `self.slots.len()` would be free. Meanwhile the struct doc says "Unbounded; entries persist until the store is dropped" while `kanerva_capacity`'s doc (`:83-88`) calls the same number "the SAME physical bound (how many items superpose at `d` dims)." `put` (`:127`) caps nothing. Rename to `slots()` and drop the field, or enforce the bound.

**11. Canonical-EDN is canonical over scopes, not over set/map element order.** `src/hash.rs:255-273`: `WatAST::Map` and `WatAST::Set` are emitted in source order. The module invests heavily in making scope IDs canonical (`:44-67`) so identical programs hash identically across runs — and then two source forms that denote the same value (`#{a b}` / `#{b a}`) produce different identities. I am not claiming a live consequence: for Ed25519 signing bytes-are-bytes, and for content-addressed caching this is a miss, not a wrong answer. What is worth fixing is the silence — the file's determinism section never says which orderings are canonicalized and which are not.

**12. `newest_source_mtime` does not scan what it says it scans.** `src/distribution/staleness.rs:158-166` promises "mirroring what `cargo build` bakes into the binary" and lists `Cargo.toml`, `src/**`, `wat/**/*.wat`, `crates/*/…`. `build.rs` exists at the repo root and is a rebuild trigger; it is not scanned, so editing it rebuilds the binary without ever tripping the staleness warning. (Same file, cosmetic: `walk_files:102-111` prunes on `name == "target"` *before* consulting `file_type()`, so it skips regular files with those names too, which its doc does not say.) Warning-only path, low stakes — listed because the claim is checkable and false.

---

## What I could not check, and why

- **I ran nothing.** Read-only mandate: no build, no floor, no `cargo wat`. Every claim above is from reading. The four `record_token` bypasses (#1) *look* like they borrow-check as one-line replacements — disjoint field borrows through one `&mut` — but I did not compile it, so treat "one-liner" as a prediction.
- **`wat/` produced nothing.** I sized it (20,905 lines across 24 files) and read `wat/cache.wat`'s design header, which was exemplary. I did **not** systematically sweep the stdlib. A struere pass over 21k lines of wat is its own cast and I did not do it — do not read the absence of wat findings as a clean bill.
- **`src/runtime.rs` (40,883 lines) and `src/check.rs` (22,509 lines) were sampled, not swept.** Together they are 33% of the tree. I read specific functions reached by trails from elsewhere. Anything living only inside those files is unexamined.
- **`src/rete/` is 47k lines; I covered the fire path, `where_tree`, `session.rs`, and `alloc_counter`.** `export.rs`, `purity.rs`, `reachability.rs`, `vocabulary.rs`, `compiled_cond.rs`, `expr_ir/`, `validate/` — roughly 12k lines — I did not open.
- **`src/collection/`, `src/value/`, `src/types/`, `src/macros/`, `src/resolve/`, `src/check/`, `src/freeze/`, `src/remedy/`, `src/function/` (~22k lines) got no coverage at all.** I dispatched a second reconnaissance agent over that territory and it was refused — the concurrent-subagent limit was already saturated by the other wards in this vigilia. I did not have budget to walk it by hand. That is a real hole in this cast, not a judgement that the territory is clean.
- **`#2`'s liveness is settled; `#3`'s consequence is not.** I confirmed the budget asymmetry by reading all four construction sites. I did **not** verify that an over-budget frame on the `err` leg actually reaches `take_frame`'s `FrameTooLarge` arm in practice — that reading is inferred from `take_frame`'s doc block, not driven.
- **I did not attempt any mutation proof.** Under a read-only mandate every "this gate cannot see it" above is an argument from reading the gate, not from breaking it. `[[mutation-prove-every-gate]]` is unpaid here, and #1 in particular deserves it: the claim that the round-census tests would not catch a `record_token` drift is my reading of what they assert, not a demonstration.
