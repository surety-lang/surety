//! Slice 6, acceptance test 6, the draft's half: the sentences stage
//! through `--reply`, no key. An example-wanted line gains its input as a
//! literal on a `// in:` line beneath; an output written after the literal
//! is stripped and said; a case that states its output in words is flagged;
//! and a literal that does not fit the signature goes back to the model as
//! a parse error, round by round.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn scratch(dir: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("draft")
        .join(dir);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

const PROBLEM: &str = "Keep the orders at or under a limit, in their order.\n";

fn reply(literal_under_threshold: &str) -> String {
    format!(
        "```surety\n// draft\n// drafted\ntype Order = {{ id: Id, amount: Int }}\n\nkeep_small(orders: Order[], limit: Int) -> Order[]\n\tnote \"keep the orders at or under the limit, in order\"\n\t# Every output order is an input order at or under the limit.\n\n\t# The output keeps the input's order.\n\t// example wanted: an order exactly at the limit: the threshold\n\t// in: {literal_under_threshold}\n\t// example wanted: no orders: the output is empty\n\t// in: ([], 3)\n\t// example wanted: one order over the limit\n\t// in: ([{{id: \"b\", amount: 9}}], 3) -> []\n```\n"
    )
}

/// `draft.py sentences` with saved replies, one per round.
fn sentences(dir: &Path, replies: &[String]) -> Output {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::write(dir.join("problem.md"), PROBLEM).unwrap();
    let mut cmd = Command::new(std::env::var("SURETY_PYTHON").unwrap_or_else(|_| "python3".into()));
    cmd.arg(root.join("experiments/draft.py"))
        .arg("sentences")
        .arg(dir.join("problem.md"))
        .arg("--out")
        .arg(dir.join("keep_small.sur"))
        .arg("--surety")
        .arg(env!("CARGO_BIN_EXE_surety"))
        .env("SURETY_DRAFT_RUNS", dir.join("runs"));
    for (i, r) in replies.iter().enumerate() {
        let path = dir.join(format!("reply{}.md", i + 1));
        fs::write(&path, r).unwrap();
        cmd.arg("--reply").arg(path);
    }
    cmd.output().expect("draft.py runs")
}

#[test]
fn the_sentences_stage_writes_each_case_with_its_input_and_never_an_output() {
    let dir = scratch("literals");
    let out = sentences(&dir, &[reply("([{id: \"a\", amount: 5}], 5)")]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{stdout}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let file = fs::read_to_string(dir.join("keep_small.sur")).unwrap();
    // The tuple form for two inputs, as written; the output after the third goes.
    assert!(
        file.contains("\t// in: ([{id: \"a\", amount: 5}], 5)\n"),
        "{file}"
    );
    assert!(file.contains("\t// in: ([], 3)\n"), "{file}");
    assert!(
        file.contains("\t// in: ([{id: \"b\", amount: 9}], 3)\n"),
        "{file}"
    );
    assert!(
        !file
            .lines()
            .any(|l| l.trim_start().starts_with("// in:") && l.contains("->")),
        "{file}"
    );
    assert!(
        stdout.contains(
            "stripped an output after a // in: literal (the author's, never the model's): -> []"
        ),
        "{stdout}"
    );
    assert!(stdout.contains("parse ok"), "{stdout}");
    assert!(
        stdout.contains(
            "examples wanted: 3, as // example wanted lines, 3 with its input on a // in: line"
        ),
        "{stdout}"
    );
    // The case that says its output in words is flagged, and only it.
    assert!(
        stdout.contains("1 example-wanted line states an output in words; read before accepting")
            && stdout.contains("    no orders: the output is empty"),
        "{stdout}"
    );
}

#[test]
fn a_literal_that_does_not_fit_the_signature_goes_back_for_a_round() {
    let dir = scratch("rounds");
    let out = sentences(
        &dir,
        &[
            reply("([{id: \"a\", amount: 5}], 5, 1)"),
            reply("([{id: \"a\", amount: 5}], 5)"),
        ],
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{stdout}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains("round 1:")
            && stdout.contains("the `// in:` literal: the function has 2 inputs"),
        "{stdout}"
    );
    // The error went to the model in round 2's prompt, and round 2 parsed.
    let run = fs::read_dir(dir.join("runs"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let prompt2 = fs::read_to_string(run.join("prompt2.md")).unwrap();
    assert!(
        prompt2.contains("the `// in:` literal: the function has 2 inputs"),
        "{prompt2}"
    );
    assert!(stdout.contains("parse ok"), "{stdout}");
}

/// Slice 6, acceptance test 5: the lines stage through `--reply`, no key.
const SENTENCES: &str = "// draft\ntype Order = { id: Id, amount: Int }\n\nkeep_small(orders: Order[], limit: Int) -> Order[]\n\tnote \"keep the orders at or under the limit, in order\"\n\t# Every output order is an input order at or under the limit.\n\n\t# The output keeps the input's order.\n\n\t# No two output orders share an id.\n\n\t# The output holds at most half the orders.\n\t// example wanted: no orders: the empty case\n\t// in: ([], 3)\n";

const EXAMPLE: &str =
    "\te:\n\t\tin: ([{id: \"a\", amount: 5}], 5)\n\t\tout: [{id: \"a\", amount: 5}]\n";

fn lines(dir: &Path, file: &str, replies: &[&str]) -> Output {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::write(dir.join("keep_small.sur"), file).unwrap();
    let mut cmd = Command::new(std::env::var("SURETY_PYTHON").unwrap_or_else(|_| "python3".into()));
    cmd.arg(root.join("experiments/draft.py"))
        .arg("lines")
        .arg(dir.join("keep_small.sur"))
        .arg("--surety")
        .arg(env!("CARGO_BIN_EXE_surety"))
        .env("SURETY_DRAFT_RUNS", dir.join("runs"));
    for (i, r) in replies.iter().enumerate() {
        let path = dir.join(format!("reply{}.md", i + 1));
        fs::write(&path, r).unwrap();
        cmd.arg("--reply").arg(path);
    }
    cmd.output().expect("draft.py runs")
}

#[test]
fn the_lines_stage_refuses_a_file_with_no_example() {
    let dir = scratch("no-example");
    let out = lines(&dir, SENTENCES, &["```surety\n```\n"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("has no example, so stage 2 does not start: a line is asked for only against an example that can run it"),
        "{stderr}"
    );
    assert!(
        stderr.contains("  example wanted: no orders: the empty case\n    in: ([], 3)"),
        "{stderr}"
    );
    assert!(!dir.join("runs").exists(), "no run started");
}

#[test]
fn a_retry_asks_again_only_for_the_failing_pair() {
    let dir = scratch("retry");
    // Round 1: three lines, the third not parsing, and a reach-for; a
    // comment beneath the first pair.
    let round1 = "```surety\n# Every output order is an input order at or under the limit.\n\tall o in out: o in orders and o.amount <= limit\n\t// the limit is inclusive, as the note says\n# The output keeps the input's order.\n\tout same_order_as orders\n# No two output orders share an id.\n\tunique out.id id\n# The output holds at most half the orders.\n\treaching for: a ratio of lengths\n```\n";
    // Round 2 answers the failing sentence, and rewrites the first one too,
    // which the stage does not take: it asked for the failing one only.
    let round2 = "```surety\n# No two output orders share an id.\n\tunique out.id\n# Every output order is an input order at or under the limit.\n\tall o in out: o in orders\n```\n";
    let out = lines(&dir, &(SENTENCES.to_string() + EXAMPLE), &[round1, round2]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{stdout}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let file = fs::read_to_string(dir.join("keep_small.sur")).unwrap();
    assert!(file.contains("\tall o in out: o in orders and o.amount <= limit\n\t// drafted: the limit is inclusive, as the note says\n"), "{file}");
    assert!(file.contains("\tout same_order_as orders\n"), "{file}");
    assert!(file.contains("\tunique out.id\n"), "{file}");
    assert!(
        file.contains("\t// reaching for: a ratio of lengths\n"),
        "{file}"
    );
    assert!(!file.contains("unique out.id id"), "{file}");
    // Round 2's prompt asks for the failing sentence alone, with its error.
    let run = fs::read_dir(dir.join("runs"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let prompt2 = fs::read_to_string(run.join("prompt2.md")).unwrap();
    let again = prompt2
        .split("give the pairs again for these sentences only, corrected:")
        .nth(1)
        .unwrap();
    assert!(
        again.contains(
            "# No two output orders share an id.\n\tunique out.id id\nThe checker reports:"
        ),
        "{again}"
    );
    assert!(!again.contains("# Every output order"), "{again}");
    // Every round's errors and reach count are in the run's record.
    let rounds: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(run.join("rounds.json")).unwrap()).unwrap();
    assert_eq!(rounds.as_array().unwrap().len(), 2, "{rounds}");
    assert_eq!(rounds[0]["asked"].as_array().unwrap().len(), 4, "{rounds}");
    assert!(
        rounds[0]["errors"]["No two output orders share an id."]
            .as_str()
            .unwrap()
            .contains("two terms cannot stand side by side"),
        "{rounds}"
    );
    assert_eq!(rounds[0]["reach"], 1, "{rounds}");
    assert_eq!(
        rounds[1]["asked"],
        serde_json::json!(["No two output orders share an id."])
    );
    assert_eq!(rounds[1]["errors"], serde_json::json!({}));
    // The summary lists the model's comment and the cases left.
    assert!(
        stdout.contains("the model's comments, kept as // drafted: beneath their lines"),
        "{stdout}"
    );
    assert!(
        stdout.contains("  - the limit is inclusive, as the note says"),
        "{stdout}"
    );
    assert!(
        stdout.contains("examples wanted still in the file: 1, 1 with a // in: literal not yet turned into an e: block:\n  - no orders: the empty case\n      in: ([], 3)"),
        "{stdout}"
    );
}
