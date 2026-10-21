// Slice 4, acceptance test 3: the grammar colours the corpus. Every keyword,
// built-in, quantifier, sentence, comment, string, number and type name
// carries a scope beyond source.surety, and a # line carries no comment
// scope. Runs under node with vscode-textmate; no editor needed.

import * as fs from 'fs';
import * as path from 'path';
import * as assert from 'assert';
import * as oniguruma from 'vscode-oniguruma';
import * as textmate from 'vscode-textmate';

const here = path.resolve(__dirname, '..', '..');
const corpus = path.resolve(here, '..', '..', 'crates', 'surety', 'tests', 'corpus');
const fixtures = ['p1', 'p2', 'p3', 'p4', 'p5a', 'p5b', 'p6', 'p7', 'p8', 'p14', 'apply_diff'];

async function registry(): Promise<textmate.Registry> {
    const wasm = fs.readFileSync(require.resolve('vscode-oniguruma/release/onig.wasm')).buffer;
    const onig = oniguruma.loadWASM(wasm).then(() => ({
        createOnigScanner: (patterns: string[]) => new oniguruma.OnigScanner(patterns),
        createOnigString: (s: string) => new oniguruma.OnigString(s),
    }));
    return new textmate.Registry({
        onigLib: onig,
        loadGrammar: async () => {
            const text = fs.readFileSync(path.join(here, 'syntaxes', 'surety.tmLanguage.json'), 'utf8');
            return textmate.parseRawGrammar(text, 'surety.tmLanguage.json');
        },
    });
}

const keywords = new Set(['all', 'any', 'none', 'exactly_one', 'in', 'where', 'else', 'and', 'or', 'not',
    'leftover', 'before', 'unordered', 'unique', 'sorted_by', 'same_order_as', 'same_elements_as',
    'drawn_from', 'asc', 'desc', 'type', 'note', 'accept']);
const builtins = new Set(['sum', 'len', 'min', 'max', 'count', 'flatten']);
const primitives = new Set(['Int', 'Float', 'Text', 'Bool', 'Id', 'Time', 'UUID', 'Option', 'Result']);

async function main(): Promise<void> {
    const grammar = await (await registry()).loadGrammar('source.surety');
    assert.ok(grammar, 'the grammar loads');
    let tokens = 0;
    let sentences = 0;
    let comments = 0;
    for (const name of fixtures) {
        const text = fs.readFileSync(path.join(corpus, `${name}.sur`), 'utf8');
        let state = textmate.INITIAL;
        for (const [n, line] of text.split('\n').entries()) {
            const result = grammar!.tokenizeLine(line, state);
            state = result.ruleStack;
            const where = `${name}.sur:${n + 1}`;
            const trimmed = line.trim();
            if (trimmed.startsWith('#')) {
                sentences += 1;
                for (const t of result.tokens) {
                    assert.ok(!t.scopes.some(s => s.includes('comment')),
                        `${where}: a # line is a sentence, not a comment: ${t.scopes}`);
                }
                assert.ok(result.tokens.some(t => t.scopes.some(s => s.startsWith('string.unquoted.sentence'))),
                    `${where}: the sentence carries its scope`);
            }
            if (trimmed.startsWith('//')) {
                comments += 1;
                assert.ok(result.tokens.every(t => t.scopes.some(s => s.startsWith('comment.line'))),
                    `${where}: a // line is a comment`);
            }
            for (const t of result.tokens) {
                const word = line.slice(t.startIndex, t.endIndex);
                if (word.trim() === '') { continue; }
                tokens += 1;
                assert.ok(t.scopes.length > 1, `${where}: '${word}' has only source.surety`);
                const scope = t.scopes[t.scopes.length - 1];
                if (keywords.has(word)) {
                    assert.ok(scope.startsWith('keyword.'), `${where}: keyword '${word}' is ${scope}`);
                } else if (builtins.has(word) && line.slice(t.endIndex).trimStart().startsWith('(')) {
                    assert.ok(scope.startsWith('support.function'), `${where}: builtin '${word}' is ${scope}`);
                } else if (primitives.has(word)) {
                    assert.ok(scope.startsWith('support.type'), `${where}: type '${word}' is ${scope}`);
                } else if (/^[A-Z][A-Za-z0-9_]*$/.test(word)) {
                    assert.ok(scope.startsWith('entity.name.type') || scope.startsWith('constant.language'),
                        `${where}: type name '${word}' is ${scope}`);
                } else if (/^-?\d+(\.\d+)?$/.test(word)) {
                    assert.ok(scope.startsWith('constant.numeric'), `${where}: number '${word}' is ${scope}`);
                } else if (word.startsWith('"') && word.endsWith('"') && word.length > 1) {
                    assert.ok(scope.startsWith('string.quoted'), `${where}: string ${word} is ${scope}`);
                }
            }
        }
    }
    assert.ok(sentences > 30, `the corpus has sentences: ${sentences}`);
    assert.ok(comments > 5, `the corpus has comments: ${comments}`);
    console.log(`grammar: ${tokens} tokens over ${fixtures.length} files, ${sentences} sentences, ${comments} comments, every token scoped`);
}

main().catch(e => { console.error(e); process.exit(1); });
