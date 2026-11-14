# Contributing to arity

Thank you for your help. This file tells you how to make a change that we can
accept.

## Before you start

- For a large change, open an issue first. Describe the problem and your plan.
- For a data error, open an issue with the report ID and the correct text from
  the source.

## Build and test

You need the Rust toolchain that `rust-toolchain.toml` specifies. `rustup`
installs it for you.

Do these steps before each commit:

1. Format the code: `cargo fmt --all`
2. Run the linter: `cargo clippy --workspace --all-targets -- -D warnings`
3. Run the tests: `cargo test --workspace`

## How we write code

- Write the test first. Make sure that it fails. Then write the code that makes
  it pass.
- Keep each commit small. Each commit must build and pass all tests.
- Do not use `unsafe` code. The workspace forbids it.
- Do not use `unwrap`, `expect`, or `panic` outside tests. Return an error.
- Write a comment only when the code cannot show the reason.

## How we write text

All documentation, commit messages, error messages, and user interface text
use ASD-STE100 Simplified Technical English:

- Use short sentences. Use a maximum of 20 words in an instruction and 25
  words in a description.
- Use the active voice.
- Use one instruction in each sentence.
- Use one word for one meaning. For example, use "report" only for a statement
  from a source.

## Commit messages

Use this form: `area: imperative sentence`. The area is the crate name without
the `arity-` prefix, or `web`, `docs`, `ci`, or `project`.

Examples:

- `units: add the SI prefix table`
- `protocol: reject quotes that do not occur in the source`

## License of contributions

arity uses the Apache License, Version 2.0. When you send a contribution, you
agree to license it under the same terms, as section 5 of the license states.
