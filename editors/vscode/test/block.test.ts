// The block a line belongs to, which the extension holds diagnostics in
// while the author types: a clause with the lines under it, a multi-line
// declaration whole, the blank lines after either. Runs under node.

import * as assert from 'assert';
import * as fs from 'fs';
import * as path from 'path';
import { blockAt } from '../src/block';

const corpus = path.resolve(__dirname, '..', '..', '..', '..', 'crates', 'surety', 'tests', 'corpus');
const fixtures = ['p1', 'p2', 'p3', 'p4', 'p5a', 'p5b', 'p6', 'p7', 'p8', 'p14', 'apply_diff'];

const file = [
    'type Item = { sku: Id, qty: Int }', // 0
    'type Order = {',                    // 1
    '\tid: Id,',                         // 2
    '\tamount: Int',                     // 3
    '}',                                 // 4
    '',                                  // 5
    'f(items: Item[]) -> Order[]',       // 6
    '\tnote "n"',                        // 7
    '\t# a sentence',                    // 8
    '\tall Ok(r) in out:',               // 9
    '\t\t',                              // 10
    '',                                  // 11
    '\t# next',                          // 12
    '\tall o in out:',                   // 13
    '\t\to.amount > 0',                  // 14
    '\te:',                              // 15
    '\t  in: []',                        // 16
    '\t  out: []',                       // 17
];

function is(line: number, start: number, end: number, lines = file): void {
    assert.deepStrictEqual(blockAt(lines, line), { start, end }, `line ${line}`);
}

// A quantifier head with no body yet: the head, the line the cursor sits
// on under it, and the blank line after, which is where its error waits.
is(9, 9, 11);
is(10, 9, 11);
is(11, 9, 11);
// A vertical body belongs to its head.
is(13, 13, 14);
is(14, 13, 14);
// An example with its in and out.
is(16, 15, 17);
// A sentence is a line of its own.
is(12, 12, 12);
// A multi-line type is one block, and so is its closing brace.
is(1, 1, 5);
is(3, 1, 5);
is(4, 1, 5);
// A one-line declaration and the signature stand alone.
is(0, 0, 0);
is(6, 6, 6);
// A type whose brace is not closed yet stops before the next declaration.
const open = ['type A = {', '\tx: Int', '', 'g(a: A) -> Int', '\tout > 0'];
is(1, 0, 2, open);
is(3, 3, 3, open);

// Every corpus line falls inside its own block.
let lines = 0;
for (const name of fixtures) {
    const text = fs.readFileSync(path.join(corpus, `${name}.sur`), 'utf8').split(/\r?\n/);
    text.forEach((_, i) => {
        const b = blockAt(text, i);
        assert.ok(b.start <= i && i <= b.end, `${name}.sur line ${i + 1}: ${JSON.stringify(b)}`);
        lines++;
    });
}
console.log(`blocks: the layout cases and ${lines} corpus lines`);
