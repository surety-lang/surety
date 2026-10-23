//! Slice 6, piece 4, the half in `surety-syntax` and `surety lsp`: an
//! example-wanted line with its input as a literal on a `// in:` line
//! beneath. The literal is checked as an example's `in:` is, `render` never
//! wraps it, the long-line hint passes it by, and the editor's code action
//! turns the two comments into an `e:` block whose output is left to the
//! author, the file not parsing there until it is written.

use std::fs;

use serde_json::json;
use surety_syntax::{ErrorKind, Graph, parse};

mod common;

/// The frozen 13 with one example-wanted block, its case wrapped over two
/// comment lines and its literal holding two spaces.
fn thirteen_with(block: &str) -> String {
    fs::read_to_string(common::corpus_dir().join("drafts/p13.sur")).unwrap() + block
}

const BLOCK: &str = "\t// example wanted: four items, size 2, page 1: page plus one, times size, equals the item count,\n\t// the has_next boundary\n\t// in: ([{id: \"a\"},  {id: \"b\"}, {id: \"c\"}, {id: \"d\"}], 1, 2)\n";

fn check(text: &str) -> Result<(), surety_syntax::Error> {
    parse(text).and_then(|f| Graph::single("p13", f).check_file(0))
}

#[test]
fn the_literal_is_checked_as_an_examples_input() {
    assert_eq!(check(&thirteen_with(BLOCK)), Ok(()));
    // The wrong count of inputs, and a field the input's type does not have.
    let arity = thirteen_with("\t// example wanted: two only\n\t// in: ([], 1)\n");
    let e = check(&arity).unwrap_err();
    assert_eq!(
        (e.kind, e.line),
        (ErrorKind::ExampleInNotLiteral, 55),
        "{e}"
    );
    assert!(
        e.message
            .starts_with("the `// in:` literal: the function has 3 inputs"),
        "{e}"
    );
    let field =
        thirteen_with("\t// example wanted: a stray field\n\t// in: ([{sku: \"a\"}], 0, 1)\n");
    let e = check(&field).unwrap_err();
    assert_eq!(e.line, 55, "{e}");
    // An `in:` comment that is not directly under an example-wanted line
    // is a comment: the frozen 13 ends with one, so a blank line first.
    assert_eq!(
        check(&thirteen_with("\n\t// in: not a literal at all\n")),
        Ok(())
    );
}

#[test]
fn render_never_wraps_the_literal_and_keeps_its_spaces() {
    let text = thirteen_with(BLOCK);
    let rendered = surety_syntax::render::file(&parse(&text).unwrap(), false);
    let literal = "\t// in: ([{id: \"a\"},  {id: \"b\"}, {id: \"c\"}, {id: \"d\"}], 1, 2)\n";
    assert!(rendered.ends_with(literal), "{rendered}");
    // The case above it still wraps at the column, and a second render is the first.
    assert!(rendered.contains("\t// example wanted: four items, size 2, page 1: page plus one, times size, equals the item count,\n\t// the has_next boundary\n"), "{rendered}");
    assert_eq!(
        surety_syntax::render::file(&parse(&rendered).unwrap(), false),
        rendered
    );
    // A literal past the column is left whole.
    let long = format!(
        "\t// example wanted: many\n\t// in: ([{}], 0, 9)\n",
        ["{id: \"item\"}"; 12].join(", ")
    );
    let rendered = surety_syntax::render::file(&parse(&thirteen_with(&long)).unwrap(), false);
    assert!(rendered.lines().last().unwrap().chars().count() > surety_syntax::render::WIDTH);
    let hints = surety_lsp::long_prose_lines(&thirteen_with(&long));
    let last = thirteen_with(&long).lines().count() - 1;
    assert!(
        !hints.iter().any(|h| h["range"]["start"]["line"] == last),
        "{hints:?}"
    );
}

#[test]
fn the_code_action_writes_the_block_with_the_output_left_to_the_author() {
    let text = thirteen_with(BLOCK);
    // The frozen 13 has a case of the same words; this block is the last.
    let lines: Vec<&str> = text.lines().collect();
    let first = lines
        .iter()
        .rposition(|l| l.contains("example wanted: four items"))
        .unwrap();
    for line in first..first + 3 {
        let actions = surety_lsp::code_actions("file:///p13.sur", &text, Some(line), &[]);
        let action = &actions[0];
        assert_eq!(action["title"], surety_lsp::WRITE_EXAMPLE);
        let edit = &action["edit"]["changes"]["file:///p13.sur"][0];
        assert_eq!(
            edit["range"]["start"],
            json!({ "line": first, "character": 0 })
        );
        assert_eq!(edit["range"]["end"]["line"], first + 2);
        assert_eq!(
            edit["newText"],
            "\te:\n\t\tnote \"four items, size 2, page 1: page plus one, times size, equals the item count, the has_next boundary\"\n\t\tin: ([{id: \"a\"},  {id: \"b\"}, {id: \"c\"}, {id: \"d\"}], 1, 2)\n\t\tout: "
        );
        assert_eq!(action["command"]["command"], "surety.cursorAt");
        assert_eq!(
            action["command"]["arguments"],
            json!(["file:///p13.sur", first + 3, 7])
        );
    }
    // Applied, the file stops at the empty `out:` and nowhere else.
    let written = format!(
        "{}\n\te:\n\t\tnote \"case\"\n\t\tin: ([{{id: \"a\"}}], 0, 1)\n\t\tout: \n",
        lines[..first].join("\n")
    );
    let e = check(&written).unwrap_err();
    assert_eq!(
        (e.kind, e.line),
        (ErrorKind::ExampleOutMissing, first + 4),
        "{e}"
    );
    assert!(e.message.contains("the output is yours to write"), "{e}");
    // Writing the output clears it.
    let done = written.replace(
        "out: \n",
        "out: Ok({items: [{id: \"a\"}], total_pages: 1, has_next: false})\n",
    );
    assert_eq!(check(&done), Ok(()));
    // No literal, no action: the case alone does not make an example.
    let bare = thirteen_with("\t// example wanted: no literal yet\n");
    let line = bare.lines().count() - 1;
    let actions = surety_lsp::code_actions("file:///p13.sur", &bare, Some(line), &[]);
    assert_eq!(actions, json!([]));
}
