// Slice 4, acceptance test 4: the verdict on save, in VS Code's own test
// host, and parse's diagnostics waiting while a clause is typed. Prepares a workspace of corpus copies, points the extension at the
// repository's debug binary, and runs the suite under test/suite.

import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import { runTests } from '@vscode/test-electron';

async function main(): Promise<void> {
    const extensionDevelopmentPath = path.resolve(__dirname, '..', '..');
    const extensionTestsPath = path.resolve(__dirname, 'suite', 'index');
    const repo = path.resolve(extensionDevelopmentPath, '..', '..');
    const corpus = path.join(repo, 'crates', 'surety', 'tests', 'corpus');
    const binary = process.env.SURETY_BIN || path.join(repo, 'target', 'debug', 'surety');
    if (!fs.existsSync(binary)) {
        throw new Error(`no surety binary at ${binary}; run cargo build first`);
    }
    const workspace = fs.mkdtempSync(path.join(os.tmpdir(), 'surety-vscode-'));
    // P5a with its performance: ACCEPTED on save.
    fs.copyFileSync(path.join(corpus, 'p5a.sur'), path.join(workspace, 'p5a.sur'));
    fs.copyFileSync(path.join(corpus, 'p5a.py'), path.join(workspace, 'p5a.py'));
    // P1 with a wrong performance in the performance's place: NOT ACCEPTED, the finding on its clause.
    fs.copyFileSync(path.join(corpus, 'p1.sur'), path.join(workspace, 'p1.sur'));
    fs.copyFileSync(path.join(corpus, 'mutants', 'p1_changes_email.py'), path.join(workspace, 'p1.py'));
    // P5a without its accept line and with no performance: the mutants' findings.
    const p5a = fs.readFileSync(path.join(corpus, 'p5a.sur'), 'utf8');
    const withoutAccept = p5a.split('\n').filter(l => !l.trim().startsWith('accept ')).join('\n');
    fs.writeFileSync(path.join(workspace, 'ties.sur'), withoutAccept);
    // P2 to type a half-finished clause into: its diagnostics wait while the cursor is in that clause.
    fs.copyFileSync(path.join(corpus, 'p2.sur'), path.join(workspace, 'typing.sur'));
    // P7, whose comment at line 21 runs past the column: the formatting test.
    fs.copyFileSync(path.join(corpus, 'p7.sur'), path.join(workspace, 'p7.sur'));
    // P2 with a clause about the input alone: a hint that no output can break it.
    const p2 = fs.readFileSync(path.join(corpus, 'p2.sur'), 'utf8');
    // P2 with two clauses about the input alone: one false on negative
    // amounts, which the generator brings, and one that always holds.
    fs.writeFileSync(path.join(workspace, 'inputs.sur'), p2.replace('\tout same_order_as orders\n', '\tout same_order_as orders\n\tall o in orders: o.amount >= 0\n\tlen(orders) >= 0\n'));
    // P2 as it is, for a run that settles.
    fs.writeFileSync(path.join(workspace, 'settled.sur'), p2);
    // P2 with its left-out clause over `out`: a file that does not parse.
    fs.writeFileSync(path.join(workspace, 'broken.sur'), p2.replace('all o leftover in orders:', 'all o leftover in out:'));
    fs.mkdirSync(path.join(workspace, '.vscode'));
    fs.writeFileSync(path.join(workspace, '.vscode', 'settings.json'), JSON.stringify({ 'surety.path': binary }, null, 2));
    await runTests({
        extensionDevelopmentPath,
        extensionTestsPath,
        launchArgs: [workspace, '--disable-extensions', '--disable-gpu'],
    });
}

main().catch(e => {
    console.error(e);
    process.exit(1);
});
