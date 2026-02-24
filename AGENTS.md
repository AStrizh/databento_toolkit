# AGENTS.md

## Project Safety Rules

- This project connects to Databento, which is a paid live service.
- Never run any command that can trigger live data downloads unless the user explicitly asks for it in the current conversation.
- Do not run `cargo run`, `cargo test`, or any script/tool that may hit Databento by default.
- Default behavior for analysis tasks is static code inspection only.
- If runtime verification is needed, ask the user to run commands and share output.

## Testing Notes

- Test coverage is limited overall.
- Some tests are mock/unit-style, and some smoke-style tests may call paid APIs.
- Never assume a test is safe to run without confirming it cannot trigger Databento usage.
- Treat test results as advisory unless the user asks to repair or expand tests.

## Workflow Rules for Future Changes

- After making code modifications, do not run the code or tests unless the user explicitly requests execution.
- Do not delete downloaded data artifacts unless the user explicitly requests cleanup.
- When uncertain whether an action may incur API cost, stop and ask first.
