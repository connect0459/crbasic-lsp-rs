# Example Programs

Curated CRBasic programs showcasing specific extension features. Open any
file in VSCode with the extension installed and try the suggested actions.

Unlike the fixtures in [`docs/sample-codes/`](../sample-codes/) (real-world
programs used as parser regression tests, one per datalogger model), these
files are small and heavily commented, each focused on one or two features.
Their expected diagnostics are checked by
[`crates/crbasic-parser/tests/example_programs.rs`](../../crates/crbasic-parser/tests/example_programs.rs),
so they stay in sync with the analyzer.

## Files

| File | Demonstrates | Try |
| :--- | :--- | :--- |
| [`01-getting-started.CR6`](./01-getting-started.CR6) | The basic program shape: declarations, a `DataTable`, and the main `Scan` loop | Hover over `BeginProg`/`Scan`/`DataTable`; trigger completion for the `ScanLoop`/`NewProgram` pattern snippets; Go to Definition / Find All References on `BattVolt` |
| [`02-scope-and-copyback.CR6`](./02-scope-and-copyback.CR6) | `Public` variables are global even when declared inside a `Sub`; `Function` parameters are not copied back, `Sub` parameters are | Go to Definition / Find All References on `RunningTotal` from inside `AddSample`; hover over `Function`/`Sub` |
| [`03-cr200x-length-pitfalls.CR2`](./03-cr200x-length-pitfalls.CR2) | CR200X's 12-character output-name truncation, and the field-name collision it causes | Open as `.CR2` (targets the CR200X model) and check the Problems panel for the predicted max-length, recommended-length, and truncation-collision diagnostics |

## Why `.CR2`/`.CR6`?

The extension detects the datalogger model from the file extension and
applies model-specific variable name length rules (see the main
[README](../../README.md#supported-file-extensions)). Example 3 uses `.CR2`
specifically to trigger the stricter CR200X rules.
