# BRIEF — make `--check` as loud as the runtime on a malformed `fn`

DESIGN: `DESIGN-STONE-the-checker-is-silent-where-the-runtime-is-loud.md`. Read it first; it carries
the measurements and the reason this is a DELETION rather than a new check.

## The work, in one paragraph

`--check` accepts a malformed `(:wat::core::fn …)` and returns a fresh unconstrained placeholder, so
that fn's body and every call to it go unchecked. The runtime rejects the same forms with a located
error. Two guards in `src/function/infer.rs` cause it, both justified by comments that the call sites
refute. Remove them; the loud path and its wording already exist.

## Read in order

| where | why |
|---|---|
| `src/function/infer.rs:30-70` | `SigParse`, and the one guard that routes `ArgsVecNotVector` to `SilentReject` |
| `src/function/infer.rs:105-125` | the second silent path, `sig_args.len() < 3` |
| `src/function/eval.rs:40-52` | ★ the **sister sequence**, which its own comment names — same peel, same guard, and it ERRORS. Its `reason` string is the wording to mirror. |
| `src/check.rs:2377` and `:4704` | the ONLY two callers, both `":wat::core::fn" => infer_fn(…)`. This is what makes both rationales false. |
| `src/function/infer.rs:229-255` | the test pinning the behaviour you are removing |

## Implementation sketch

```
infer.rs:57    delete the `Err(step) if matches!(… ArgsVecNotVector …) => SigParse::SilentReject` arm
               → the following `Err(step) => SigParse::Diagnosed(…)` already handles it

infer.rs:110   replace `return CheckResult::ok(fresh.fresh())` with a located CheckError::MalformedForm,
               head FN_HEAD, reason mirroring eval.rs:49 VERBATIM:
                 "expected [name <- :T ...] -> :Ret body ...; got {N} element(s)"

infer.rs:40    delete `SigParse::SilentReject` — now dead. Update the enum doc: the silent-vs-
               diagnostic distinction it exists to carry no longer exists.

infer.rs:236   delete infer_fn_non_vector_args_returns_silent_placeholder. Replace with a `.wat`
               probe under wat-scripts/scratch-pad/ that a caller can run: the malformed form must
               now FAIL `--check`, and the message must match the runtime's.
```

## What "done" looks like

Each of these must now FAIL `--check`, with a located error whose text matches what the runtime
already prints for the same form:

```clojure
(:wat::core::fn :foo [x <- :wat::core::i64] -> :wat::core::i64 x)
(:wat::core::fn 42   [x <- :wat::core::i64] -> :wat::core::i64 x)
(:wat::core::fn "s"  [x <- :wat::core::i64] -> :wat::core::i64 x)
(:wat::core::fn)
(:wat::core::fn [x <- :wat::core::i64])
(:wat::core::fn [x <- :wat::core::i64] ->)
```

And each of these must still CHECK — the negative controls:

```clojure
(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)          ; well-formed
(:wat::core::fn :- [T] [x <- :T] -> :T x)                             ; the γ-i binder
(:wat::core::fn {:doc "m"} [x <- :wat::core::i64] -> :wat::core::i64 x) ; metadata preamble
```

★ **The metadata and binder controls are the ones that bite.** Both are peeled BEFORE the guard, so
both look like a non-Vector in slot 0 until the peel runs. If your change fires before the peels,
they go red and you have moved the check to the wrong side of them.

## Boundaries

- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.
- `src/function/eval.rs` is the reference, not a target — the runtime side is already correct.
- Do NOT touch `parse_fn_signature_prefix`'s `&[WatAST; 3]`.
- Author no new message text. Every string you need already exists in `ParseStepKind::reason()` or
  at `eval.rs:49`.

## Your own checks

`cargo build --bin wat`, then `target/debug/wat --check <file>` on files under
`wat-scripts/scratch-pad/`. Diagnostics go to **stderr**; judge by exit code AND empty output, never
by grep alone. Prefix long commands with
`systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 900`.

## STOP triggers — ship nothing further and report

- **STOP-1.** If removing the guards makes a MACRO TEMPLATE go red — `wat/core.wat`,
  `wat/service.wat` or `wat/bracket.wat` quasiquote `(:wat::core::fn …)` in six places — STOP and
  report WHICH templates and the verbatim error. **This is an anticipated outcome, not a failure:**
  it means the exemption was load-bearing for unquote nodes, and the builder re-decides toward a slot
  rule. Do not special-case your way past it.
- **STOP-2.** If either negative control (metadata preamble, `:- [T]` binder) goes red, STOP — the
  check landed on the wrong side of the peels.
- **STOP-3.** If deleting `SigParse::SilentReject` requires changing `src/check.rs`, STOP and report.

## Your report

The diff per file. Every "done" row and every negative control with verbatim output. Whether any
macro template moved. What surprised you. Anything you inspected and left alone, with the reason.
