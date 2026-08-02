# End-to-end tests

Every `.sc` program in this directory is a test case. `tests/run.sh` compiles each one with
`scrapc`, optionally runs it, and checks the result against expectations the program declares
itself as `//@` directives in a comment header.

```sh
tests/run.sh                    # everything except the interop cases
tests/run.sh --interop          # everything
tests/run.sh spawn              # only cases whose name contains "spawn"
tests/run.sh --interop rust_    # combine both
```

## Directives

| Directive | Meaning |
| --- | --- |
| `//@ run` | Compile, link and execute the program. |
| `//@ exit: 42` | Required with `run` — the expected process exit code. |
| `//@ stdout: text` | Substring the program must print. Repeatable. |
| `//@ compile-fail` | `scrapc` must reject the program. |
| `//@ error: text` | Substring `scrapc` must report. Repeatable. |
| `//@ manifest: Scrap.toml` | Passed to `--manifest`. Also marks the case as interop. |
| `//@ ignore: reason` | Skip this program. The reason is required. |

`run` and `compile-fail` are mutually exclusive.

**A program with no directives is an error, not a skip.** That rule is the point of the harness:
before it existed, expected exit codes lived only in a scratch notes file, and a program could stop
being covered without anyone noticing.

## Writing a case

Put the directives at the top of the file:

```rust
//@ run
//@ exit: 42

fn main() -> i32 {
    42
}
```

For a negative case, keep the `error:` substring short and semantic rather than pasting a full
rendered diagnostic — it should survive rewording of the surrounding message but still fail if the
*meaning* changes:

```rust
//@ compile-fail
//@ error: cannot assign to immutable variable `x`
```

**Predict the expected value by reading the program, then run it — don't record whatever it
currently does.** Annotating from observed behaviour freezes bugs into the suite as "correct". The
first pass over these files caught two real ones that way: a counter underflow in the coroutine
stack pool that made `stack_growth` silently exit 0, and an `if_basic.sc` that predated mutability
checking and no longer compiled.

## Interop cases

`rust_interop_abi/` and `rust_interop_value/` exercise native Rust interop and are skipped unless
you pass `--interop`. They need the `scrap-rustc` metadata driver, which links `rustc_private`, so
`run.sh` builds `scrapc` with `--features interop-driver` for them. That requires the `rustc-dev`
component:

```sh
rustup component add rustc-dev
```

Without the feature, `scrapc` reports that interop is disabled instead of failing obscurely. In CI
these run in a separate `e2e-interop` job, gated on the `interop` PR label or a manual dispatch,
because pulling `rustc-dev` and building the anchor is slow.

Crate names are derived from the path (`rust_interop_abi/methods.sc` → `rust_interop_abi__methods`)
because `scrapc` has no `--out-dir` and everything lands in `target/scrap/`, so `tests/methods.sc`
would otherwise collide.
