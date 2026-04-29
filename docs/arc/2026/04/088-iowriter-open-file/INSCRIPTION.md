# Arc 088 — `:wat::io::IOWriter/open-file` — INSCRIPTION

**Status:** shipped 2026-04-29.

The substrate's IO surface gained a file-backed writer constructor.
Long-running wat programs that manage their own per-run logs (the
trader's three-files-per-run discipline:
`runs/<descriptor>-<epoch>.{out,err,db}`) now open `.out` and
`.err` writers themselves at `:user::main` startup, instead of
inheriting the parent process's stdio.

**Predecessors:**
- Arc 008 — `:wat::io::IOReader` / `IOWriter` abstract types.
  Today's `IOWriter` constructors covered in-memory
  (`StringIoWriter`) and pipe-via-fd (`PipeWriter`). Files were a
  gap — the substrate had no way to open a path for writing.
- Arc 087 — ConsoleLogger ships per-run files; without
  IOWriter/open-file, `:user::main` couldn't open them.

**Surfaced by:** the user's framing (2026-04-29):

> "the old format was something like runs/some-unique-name.{log,db}
> now we can have runs/some-uniq-name.{out,err,db}"

The lab needed three files per run: `.out` (info/debug), `.err`
(warn/error), `.db` (sqlite). The wat program needs to OPEN those
files itself — shell-redirect (`cargo run > x.out 2> x.err`) is
boilerplate the binary should manage internally. This arc closes
the substrate gap.

---

## What shipped

### `src/io.rs`

```rust
pub fn eval_iowriter_open_file(args, env, sym) -> Result<Value, RuntimeError>
```

Implementation:
- Eval the path arg (single `:String`).
- `std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(&path)`
  — open-or-create + truncate (clean per-run file every invocation).
- Convert the resulting `File` to `OwnedFd` via `file.into()`.
- Wrap in `PipeWriter::from_owned_fd` — the existing fd-backed
  writer impl that `libc::write(2)`s direct to the fd.
- Return as `Value::io__IOWriter(Arc<dyn WatWriter>)`.

Drop closes the file via `OwnedFd`'s stdlib impl. The Console
driver thread that owns the writer keeps the fd alive for its
lifetime; clean shutdown cascade closes everything.

### `src/runtime.rs`

Op-dispatch entry:

```rust
":wat::io::IOWriter/open-file" => crate::io::eval_iowriter_open_file(args, env, sym),
```

### `src/check.rs`

Type scheme: `:fn(:String) -> :wat::io::IOWriter`.

---

## Wat-side surface

```scheme
(:wat::io::IOWriter/open-file
  (path :String)
  -> :wat::io::IOWriter)
```

Usage at `:user::main`:

```scheme
((out-writer :wat::io::IOWriter)
 (:wat::io::IOWriter/open-file "runs/smoke-1777434179.out"))
((err-writer :wat::io::IOWriter)
 (:wat::io::IOWriter/open-file "runs/smoke-1777434179.err"))
((con-spawn :Console::Spawn)
 (:wat::std::service::Console/spawn out-writer err-writer 1))
```

Console writes to the files instead of inheriting parent stdio.
Caller chooses the path; substrate opens; fd cleans up at driver
join.

---

## Posture

Construction-time errors (bad path, permission denied, disk full,
parent dir missing) panic with a diagnostic per memory
`feedback_shim_panic_vs_option`. These are environment-shape
errors worth halting on; if a future consumer needs graceful
handling, a `try-open-file` Result-returning sibling lands then.

Open mode is hardcoded to write+create+truncate. The "append to
existing" case isn't shipped — wat programs that want append
semantics open the path themselves and pass an existing fd
(future arc; no consumer today).

---

## Verification

The lab's `wat/programs/smoke.wat` program (shipped same session)
opens two file-backed writers, spawns Console with them, runs a
producer that double-writes through both ConsoleLogger (file
output) and Sqlite/auto-spawn (db output). End-to-end:

```
$ cargo run -p holon-lab-trading
$ ls -lt runs/smoke-*
-rw-rw-r-- 1 watmin watmin   766 Apr 28 20:42 runs/smoke-1777434179.out
-rw-r--r-- 1 watmin watmin 12288 Apr 28 20:42 runs/smoke-1777434179.db
-rw-rw-r-- 1 watmin watmin   313 Apr 28 20:42 runs/smoke-1777434179.err
```

Three files per run; all written by the wat program itself; no
shell redirect needed.

---

## Consumer impact

Unblocks:
- Trader's per-run file discipline (`runs/<descriptor>-<epoch>.{out,err,db}`).
- Any future long-running wat program (proofs, MTG, truth-engine)
  that wants to own its log destinations.
- `:user::main` as the wiring diagram (CIRCUIT.md) — the lab
  finally has a `:user::main` that's not hello-world. It opens
  files, spawns drivers, distributes handles, joins cascade.

PERSEVERARE.
