# DESIGN — STONE 255.1c-io: HOME #12, the `:wat::io::` carve

The kernel TIER closed at `6dd7bb18` — eleven homes, literal `:wat::kernel::` dispatch at **zero**.
This stone opens the next family, and it is chosen on a property no home so far has had.

## Why `:wat::io::`, measured

Census of `src/runtime.rs` literal dispatch arms taken this session (HEAD `4160b12a`), rolled to the
top-level family. ⚠ A grep, positive-controlled: `:wat::kernel::` and `:wat::time::` both return **0**,
which is the known ground truth for two carved families, so the pattern sees what it claims to see.

```
499 literal arms remain
core 286 · holon 99 · io 29 · rete 16 · std 14 · runtime 13 · edn 13
config 7 · stream 4 · verify 3 · program 3 · 11 `eval-*` singletons
```

`:wat::io::` clears all four of the criteria stone 255.1c-time established, and adds a fifth:

1. **CONTIGUOUS.** All 29 arms sit in `runtime.rs:6448–6539`. One block.
2. **DOWNSTREAM OF THE REGISTRY GUARD.** The guard is `runtime.rs:5285`
   (`if let Some(handler) = crate::intrinsic::registry().lookup(head)`). Every io arm is 6448+, so a
   registered io name is intercepted the instant it registers — no dispatch reordering, no shadowing.
3. **COLD.** `:wat::core::i64::+` dispatches at `runtime.rs:5036`, *before* the guard. The arithmetic
   hot path is untouched by this stone and stays untouched until `core::i64` carves, which is LAST.
4. **IT STRADDLES THE CATEGORY AXIS.** `IOReader/from-bytes` takes a `Bytes` and hands back a reader
   with no syscall; `open-file` claims an fd; `TempFile/path` projects a component off a handle;
   `flush` pushes bytes at the world. A home whose every row takes the same label cannot falsify the
   metadata contract — R59 `NISI FRANGAS, NIHIL PROBAS`. This one can, four ways.
5. ★ **THE GATE CAN SEE EVERY ROW — and that is new.** Proven by probe, not by grep.

## ★★ The load-bearing reason: ZERO of the 29 are blanket-accepted — proven, not grepped

`kernel/resource.rs` had to write it down that its gate verified **4 of 14** rows: ten verbs reach
the checker through bespoke `infer_list` arms, and `doc_arg_ret_types_match_checker_scheme` opens
`None => continue`, so it silently skipped them. Home #5's five are registered and still skipped.
That is the standing shape of this campaign — *registration is not typing* — and it is why the
seam says **do not report the carve as having closed #110**.

`:wat::io::` inverts it — and a probe, not a grep, is what says so. **`PROBE-255.1c-io-every-verb-is-scheme-enforced.sh`** (committed beside this stone) hands every verb seven arguments and reads
what the checker does, with `peer-pid` as the negative control. Re-runnable, no build:

```
NEGATIVE CONTROL  peer-pid + 5 args ....... EXIT 0, no error      ← the blanket-accept, demonstrated
28 of 29 io verbs ......................... ArityMismatch         ← plain TypeScheme, enforced
 1 of 29  IOReader/read-frame ............. MalformedForm         ← bespoke arm, see below
 0 of 29  fell through ....................                       ← none is in the blanket's shadow
```

> **Zero of the 29 are blanket-accepted, and all 29 are gate-VISIBLE** — every one is a registered
> `env.register(name, TypeScheme{…})`, so `check_env.get()` returns `Some` and
> `doc_arg_ret_types_match_checker_scheme` never hits its `None => continue`. The `@ret` line of
> every row is compared against the checker's own scheme, by the compiler, at every floor.

⚠ **The probe corrected this stone's first draft, which claimed a flat 29/29 enforcement.** That was
a grep result — and a grep is a guess with a number on it
(`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`). `IOReader/read-frame` has BOTH a
registered scheme (`check.rs:15794`, **one** param) and a bespoke `infer_ioreader_read_frame` arm
(`check.rs:2969`) that intercepts first and accepts **one or two** args. Two consequences the rider
must carry:

1. The gate's arg loop is guarded by `i < scheme.params.len()`, so **`read-frame`'s second `@arg`
   would be silently skipped** — documented or not, right or wrong. It is the one row in this home
   whose arg list the gate does not fully see. Give it a `//` (not `///`) maintainer comment naming
   `infer_ioreader_read_frame` as the real authority, per the `kernel/message.rs` shape.
2. Its registered scheme **under-describes the verb**. That is a finding, not a defect to fix here —
   the contract decision below forbids touching `check.rs`.

That is 28 rows whose arg lists are fully compiler-checked and one that is honestly annotated —
still, by a wide margin, the best gate coverage any home in this campaign has had, and the first
where a wrong `@ret` cannot ship green anywhere in the file.

## The decomposition — a directory from the first commit

`kernel/` spent nine stones as flat `kernel_*.rs` files and then paid a tenth
(`255.1c-kernel-becomes-a-directory`) to become `kernel/`. The precedent now exists; io takes it up
front. **Three subjects, three files** — not the prefix repeated three times:

```
src/intrinsic/io/
  mod.rs      the family claim + the "bodies do not live here" statement
  reader.rs   10 — IOReader/{from-bytes,from-string,open-file,from-fd,read,read-all,
                   read-all-string,read-line,read-frame,rewind}
  writer.rs   13 — IOWriter/{new,open-file,from-fd,to-bytes,to-string,write,write-all,
                   write-string,print,println,writeln,flush,close}
  fs.rs        6 — TempFile/{new,path}, TempDir/{new,path}, read-file, list-dir
```

All 30 bodies in the block delegate into `crate::io::` — one module, so "the bodies do not live
here" holds for this family the same way it holds for the kernel tier.

⚠ **`:wat::io::` is a FAMILY, not a TIER.** kernel earned the tier language by braiding seven
unrelated concerns; io is one subject — bytes crossing the process boundary — asked three ways. The
rider must not import the tier vocabulary. If its body-reads refute this, that refutation is the
stone's most valuable output.

## The one contract decision

**No stub `TypeScheme` is minted, and no existing one is touched.** The docs conform to the checker,
never the reverse. If a body's real return type disagrees with its registered scheme, that is a
FINDING — surface it, do not reconcile it by editing `check.rs`.

## Out of scope — affirmatively cut, not deferred

- **`:wat::stdlib::sources`** (`runtime.rs:6527`) sits *inside* the io block and also delegates to
  `crate::io::eval_stdlib_sources`. It is **not** `:wat::io::`. The carve boundary is the family, not
  the line range — it stays in `runtime.rs`, and the block is deleted around it.
- **`core::i64` / `core::f64`** — the hot path, and it carves LAST by the time-home stone's ruling.
- **#110, the blanket-accept.** Untouched here, and io does not shrink it: these 29 already have
  schemes, so none of them was ever in the blanket's shadow. Saying otherwise would be the exact
  overclaim the seam warns about.

## The goldens — this stone MOVES them, and by a computable amount

Five fixtures pin `src/runtime.rs`, and all five pin **`:line 25277`** (`:col 17`) — the provenance
Span of a runtime-built keyword. Deleting the io block at 6448 sits **entirely above** 25277, so
every deleted line shifts it, and none of the deletion is below it.

★ The "proven by ABSENCE" best case (`git diff --stat src/runtime.rs` empty) is **not available**
here — removing arms from `runtime.rs` is what the stone *is*. So the standing step runs in full:
after the rider returns, read `git diff src/runtime.rs`, confirm **every hunk precedes 25277**,
apply *that* delta — never the net `numstat`, which was wrong by 52 lines on a prior stone — and
verify `:col` is unchanged and only `:line` moved. Then the floor.

## The strike order — three stones, and the first one is the template

| stone | file | rows | why this order |
|---|---|---|---|
| **255.1c-io-reader** | `io/mod.rs` + `io/reader.rs` | 10 | mints the directory, the family claim, and the doc-vs-`TypeScheme` pattern |
| 255.1c-io-writer | `io/writer.rs` | 13 | the largest, on settled foundation |
| 255.1c-io-fs | `io/fs.rs` | 6 | the two RAII temp handles + the two one-shots |

The stepping-stone test answers YES: the writer stone is 13 rows of pure repetition *once* the
directory, the mod-doc, and the "@ret must match the scheme" pattern are on disk — and 13 rows of
repetition plus three fresh structural decisions if they are not.

## The four questions

- **Obvious?** YES — `ls src/intrinsic/io/` reads as reader / writer / filesystem.
- **Simple?** YES — each file is one subject; every body already lives in `crate::io::`.
- **Honest?** YES — the gate verifies all 29 rows, and the two things this stone does NOT close
  (#110, `stdlib::sources`) are named as cuts rather than left to be inferred from silence.
- **Good UX?** YES — a wrong `@ret` goes red at the floor instead of shipping as prose.
