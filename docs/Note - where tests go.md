# Note: where tests go

2026-10-21. A question that comes up as soon as someone sees the checker: "so do I still write tests?" The answer is also a good way to explain what Surety is.

## What a person writes

Examples. An example is a test: concrete input, concrete output, hand-written, cheap, and the one layer a model cannot bend toward itself. That is why examples stay human-written in the spec. Properties are the other thing a person writes, and a property is a property-based test whose generator the checker supplies. So "writing tests in Surety" is writing the obligation. There is no third file.

## What the checker produces

Everything else. Generated inputs, the two-run determinism check, totality, the mutant library, the search for underdetermined inputs. That is the test suite a careful engineer would have written for the function, produced from the contract instead of beside it. It is why the evidence record can say "503 runs, 0 findings" rather than "tests pass."

## What a person does not write

Tests of a performance's internals. A performance is the model's. If someone wants a unit test of a helper inside it, that helper should be its own obligation in the graph, with its own examples and its own evidence. Testing execution traces is what design principle 10 keeps out of properties, and the same rule keeps it out of tests.

The model may write scratch tests while producing a performance, inside the sandbox. That is fine. It is not evidence. Evidence comes from the checker running the obligation, never from tests the performance's author wrote for itself. The referee is a separate party on purpose.

## What a target generates: tests as a projection

Each target can render the obligation into its native test idiom. For Python: a `test_<name>.py` with one test per example and one property-based test per property. This buys adoption (Surety fits an existing CI as a test file nobody has to understand) and it is derived, never authored, the same as the reference docs: edit the generated test and you have created drift. A target-facing projection beside the human-facing ones. On the reached-for list; not a slice.

## What is open

Effectful code: IO, databases, clocks. There "test" means something the pure-transform v0 does not cover. That is the capabilities work in v1, and it grows the obligation language, not the test story.

## The short version

You write what must be true: a few examples, a few properties. The checker writes and runs every test that follows from that. The model writes the code and its own tests do not count. The graph is where you put the next function.
