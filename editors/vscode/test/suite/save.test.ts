import * as assert from 'assert';
import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import * as cp from 'child_process';
import { checkLens, mutantLens, signatureLine as signature } from '../../src/extension';

/** A run's `--json` lines, as the extension reads them. */
function run(verb: string, uri: vscode.Uri): any[] {
    const bin = vscode.workspace.getConfiguration('surety').get<string>('path')!;
    const out = cp.spawnSync(bin, [verb, '--json', uri.fsPath], { encoding: 'utf8' }).stdout;
    return out.split('\n').filter(l => l.trim()).map(l => JSON.parse(l));
}

function workspaceFile(name: string): vscode.Uri {
    const root = vscode.workspace.workspaceFolders![0].uri.fsPath;
    return vscode.Uri.file(path.join(root, name));
}

async function ours(uri: vscode.Uri, want: (d: vscode.Diagnostic[]) => boolean, timeoutMs = 90000): Promise<vscode.Diagnostic[]> {
    const started = Date.now();
    for (;;) {
        const all = vscode.languages.getDiagnostics(uri).filter(d => d.source === 'surety');
        if (want(all)) {
            return all;
        }
        if (Date.now() - started > timeoutMs) {
            throw new Error(`timed out waiting for diagnostics on ${uri.fsPath}: ${JSON.stringify(all.map(d => d.message))}`);
        }
        await new Promise(r => setTimeout(r, 500));
    }
}

const onSignature = (sig: number) => (d: vscode.Diagnostic) => d.range.start.line === sig;

/** The verdict line above a file's signature, once a run has written it. */
async function lens(uri: vscode.Uri, want: (title: string) => boolean, timeoutMs = 90000): Promise<vscode.CodeLens> {
    const started = Date.now();
    for (;;) {
        const lenses = await vscode.commands.executeCommand<vscode.CodeLens[]>('vscode.executeCodeLensProvider', uri);
        const hit = lenses?.find(l => l.command && want(l.command.title));
        if (hit) {
            return hit;
        }
        if (Date.now() - started > timeoutMs) {
            throw new Error(`timed out waiting for the verdict line on ${uri.fsPath}: ${JSON.stringify(lenses?.map(l => l.command?.title))}`);
        }
        await new Promise(r => setTimeout(r, 500));
    }
}

suite('the verdict on save', () => {
    test('P5a with its performance: accepted, in the line above the signature and not as a squiggle', async () => {
        const uri = workspaceFile('p5a.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        const editor = await vscode.window.showTextDocument(doc);
        // A save that changes nothing semantically: a trailing newline.
        await editor.edit(e => e.insert(new vscode.Position(doc.lineCount, 0), '\n'));
        assert.ok(await doc.save(), 'the document saves');
        const sig = 2; // get_first_customer_orders(orders: Order[]) -> Order[] is line 3
        const verdict = await lens(uri, t => /Accepted|Not accepted/.test(t));
        assert.ok(verdict.command!.title.startsWith('$(pass) Accepted: the performance passes every example and clause'), verdict.command!.title);
        assert.strictEqual(verdict.range.start.line, sig);
        // The tooltip, which the lens API does not hand back: what was run.
        const tip = checkLens(run('check', uri))!.tooltip;
        assert.ok(tip.startsWith('Examples: 3 of 3 pass. Clauses: 5 of 5 hold over 503 runs, 500 of them on generated inputs.'), tip);
        assert.strictEqual(verdict.command!.command, 'surety.report');
        const diags = vscode.languages.getDiagnostics(uri).filter(d => d.source === 'surety');
        assert.ok(!diags.some(d => onSignature(sig)(d) && d.severity === vscode.DiagnosticSeverity.Information), JSON.stringify(diags.map(d => d.message)));
    });

    test('P1 with a wrong performance: NOT ACCEPTED, the finding on its clause', async () => {
        const uri = workspaceFile('p1.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        await vscode.window.showTextDocument(doc);
        await vscode.commands.executeCommand('surety.check');
        const sig = 2; // combine(c: Customer) -> { name: Text, email: Text } is line 3
        const diags = await ours(uri, all => all.some(d => onSignature(sig)(d) && d.message.startsWith('NOT ACCEPTED')));
        const verdict = diags.find(d => d.message.startsWith('NOT ACCEPTED'))!;
        assert.ok(verdict.message.includes('performance:'), verdict.message);
        assert.strictEqual(verdict.severity, vscode.DiagnosticSeverity.Warning);
        // The frame clause that caught it sits on the signature line in P1, so
        // the finding lands there too, as a property_failed of the performance's.
        const finding = diags.find(d => d.message.startsWith('property_failed (performance)'));
        assert.ok(finding, JSON.stringify(diags.map(d => d.message)));
        assert.strictEqual(finding!.range.start.line, sig);
        assert.ok(diags.some(d => d.message.startsWith('example_mismatch (performance)')), 'the examples fail too');
        const above = await lens(uri, t => t.includes('Not accepted'));
        assert.ok(above.command!.title.startsWith('$(warning) Not accepted: '), above.command!.title);
    });

    test('P5a without its accept line and no performance: the mutants\' findings', async () => {
        const uri = workspaceFile('ties.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        await vscode.window.showTextDocument(doc);
        await vscode.commands.executeCommand('surety.mutants');
        const sig = 2;
        const diags = await ours(uri, all => all.some(d => onSignature(sig)(d) && d.message.startsWith('Gap: ')));
        // The ties are one gap, read where the two passing outputs differ,
        // under the output type; no input gets a diagnostic of its own, and
        // no summary line stacks on it.
        const gaps = diags.filter(d => d.message.startsWith('Gap: '));
        assert.strictEqual(gaps.length, 1, JSON.stringify(diags.map(d => d.message)));
        const verdict = await lens(uri, t => t.includes('settled'));
        assert.ok(verdict.command!.title.startsWith('$(warning) Not settled: 1 gap, where two different outputs both pass every clause'), verdict.command!.title);
        assert.ok(gaps[0].message.includes('].amount can be '), gaps[0].message);
        assert.strictEqual(gaps[0].code, 'gap');
        assert.ok(!diags.some(d => /^UNDERDETERMINED|^Gap: two outputs/.test(d.message)), JSON.stringify(diags.map(d => d.message)));
        const signature = doc.lineAt(sig).text;
        assert.strictEqual(gaps[0].range.start.character, signature.indexOf('-> ') + 3, 'under the output type');
        // The hover on it is the example, laid out and highlighted as Surety.
        const hovers = await vscode.commands.executeCommand<vscode.Hover[]>('vscode.executeHoverProvider', uri, gaps[0].range.start);
        const text = hovers.flatMap(h => h.contents.map(c => (typeof c === 'string' ? c : c.value))).join('\n');
        assert.ok(/```surety\nin: \[\n    \{/.test(text), text);
        assert.strictEqual((text.match(/\nout: /g) ?? []).length, 2, text);
        // The quick fix writes the input as a drafted example, the two
        // outputs as a choice to resolve; until it is, the file does not parse.
        const actions = await vscode.commands.executeCommand<vscode.CodeAction[]>('vscode.executeCodeActionProvider', uri, gaps[0].range);
        const fix = actions.find(a => a.title === 'Add this input as an example, with both outputs to choose from');
        assert.ok(fix?.edit, JSON.stringify(actions.map(a => a.title)));
        assert.ok(await vscode.workspace.applyEdit(fix!.edit!));
        const written = doc.getText();
        assert.ok(/\t\/\/ drafted: two outputs pass every clause on this input; keep the one you mean\n\te:\n\t\tin: \[\n/.test(written), written);
        assert.ok(/\n<<<<<<< passes every clause\n\t\tout: [\s\S]*\n=======\n\t\tout: [\s\S]*\n>>>>>>> also passes every clause/.test(written), written);
        await ours(uri, all => all.some(d => d.message.startsWith('a choice left open')));
        await vscode.commands.executeCommand('workbench.action.files.revert');
        // `out unordered` claims nothing by design and gets no hint.
        assert.ok(!diags.some(d => d.severity === vscode.DiagnosticSeverity.Information && doc.lineAt(d.range.start.line).text.trim().endsWith('unordered')), JSON.stringify(diags.map(d => d.message)));
    });

    test('clauses about the input alone: one that rules inputs out is a warning, one that rules nothing out is faded', async () => {
        const uri = workspaceFile('inputs.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        await vscode.window.showTextDocument(doc);
        await vscode.commands.executeCommand('surety.mutants');
        const lines = doc.getText().split('\n');
        const amounts = lines.findIndex(l => l.includes('all o in orders: o.amount >= 0'));
        const always = lines.findIndex(l => l.includes('len(orders) >= 0'));
        const diags = await ours(uri, all => all.some(d => d.range.start.line === amounts) && all.some(d => d.range.start.line === always));
        // False on the negative amounts the generator brings: no output passes there.
        const ruling = diags.find(d => d.range.start.line === amounts)!;
        assert.strictEqual(ruling.severity, vscode.DiagnosticSeverity.Warning);
        assert.strictEqual(ruling.code, 'input-only-clause');
        assert.ok(/^Input-only clause: it never reads `out`, so it limits the inputs, not the output: it is false on \d+ of the \d+ inputs tried, and no output passes there/.test(ruling.message), ruling.message);
        assert.strictEqual(ruling.range.start.character, 1, 'after the tab');
        // True on every input: unneeded, faded like unused code.
        const idle = diags.find(d => d.range.start.line === always)!;
        assert.strictEqual(idle.severity, vscode.DiagnosticSeverity.Hint);
        assert.strictEqual(idle.code, 'unneeded-clause');
        assert.ok(idle.message.startsWith('Unneeded clause: it never reads `out` and holds on every input tried'), idle.message);
        assert.deepStrictEqual(idle.tags, [vscode.DiagnosticTag.Unnecessary]);
        // The verdict says why there is nothing to settle on those inputs.
        const verdict = await lens(uri, t => t.includes('Unsatisfiable') || t.includes('ettled'));
        assert.ok(/^\$\(warning\) Unsatisfiable on \d+ of \d+ inputs, which a clause about the input alone rules out$/.test(verdict.command!.title), verdict.command!.title);
    });

    test('a run that settles: the pass is the line above the signature, with no squiggle under it', async () => {
        const uri = workspaceFile('settled.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        await vscode.window.showTextDocument(doc);
        await vscode.commands.executeCommand('surety.mutants');
        const verdict = await lens(uri, t => t.includes('ettled') || t.includes('Unsatisfiable'));
        assert.ok(/^\$\(pass\) Settled: all \d+ mutants that fit are caught; no second passing output on the \d+ inputs searched$/.test(verdict.command!.title), verdict.command!.title);
        const tip = mutantLens(run('mutants', uri)).tooltip;
        assert.ok(tip.startsWith('Mutants are deliberately wrong performances from a fixed library'), tip);
        assert.ok(/fit this signature and ran on \d+ inputs\. The search looked at \d+ inputs, up to \d+ candidate outputs each, for two different outputs that both pass every clause, and found none\./.test(tip), tip);
        const sig = signature(doc);
        assert.ok(!vscode.languages.getDiagnostics(uri).some(d => d.source === 'surety' && d.range.start.line === sig), 'nothing on the signature');
    });

});

suite('formatting through the server', () => {
    test('P7, formatted: no prose line past 100 columns, the obligation unchanged', async () => {
        const uri = workspaceFile('p7.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        const before = doc.getText();
        const columns = (l: string) => [...l].reduce((n, c) => n + (c === '\t' ? 4 : 1), 0);
        const prose = (t: string) => t.split('\n').filter(l => /^\s*(#|\/\/)/.test(l));
        assert.ok(prose(before).some(l => columns(l) > 100), 'the fixture has a prose line to wrap');
        let edits: vscode.TextEdit[] | undefined;
        for (let i = 0; i < 60 && !edits?.length; i++) {
            edits = await vscode.commands.executeCommand<vscode.TextEdit[]>('vscode.executeFormatDocumentProvider', uri, { tabSize: 4, insertSpaces: false });
            if (!edits?.length) { await new Promise(r => setTimeout(r, 500)); }
        }
        assert.ok(edits && edits.length > 0, 'edits from the server');
        // VS Code minimizes the server's whole-document edit into a diff; apply it.
        const we = new vscode.WorkspaceEdit();
        we.set(uri, edits!);
        assert.ok(await vscode.workspace.applyEdit(we), 'the edit applies');
        const after = doc.getText();
        assert.ok(prose(after).every(l => columns(l) <= 100), `a prose line past the column:\n${after}`);
        assert.ok(after.includes('enrich_orders('), 'the obligation is still there');
        assert.ok(prose(after).length > prose(before).length, 'wrapping added prose lines');
    });

    test('a file that does not parse: the term underlined on its line, and the verdict linking to it', async () => {
        const uri = workspaceFile('broken.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        await vscode.window.showTextDocument(doc);
        const line = doc.getText().split('\n').findIndex(l => l.includes('all o leftover in out:'));
        // The server's error covers `leftover in out`, not the line.
        const parsed = await ours(uri, all => all.some(d => d.range.start.line === line && d.code === 'empty_leftover'));
        const term = parsed.find(d => d.range.start.line === line)!;
        const text = doc.lineAt(line).text;
        assert.strictEqual(text.slice(term.range.start.character, term.range.end.character), 'leftover in out');
        // The check cannot run; its verdict on the signature names the line and links to it.
        await vscode.commands.executeCommand('surety.mutants');
        const sig = 1;
        const diags = await ours(uri, all => all.some(d => d.range.start.line === sig && d.message.startsWith('surety mutants could not run')));
        const verdict = diags.find(d => d.range.start.line === sig && d.message.startsWith('surety mutants could not run'))!;
        assert.ok(verdict.message.startsWith(`surety mutants could not run: line ${line + 1}: \`leftover in out\``), verdict.message);
        assert.strictEqual(verdict.relatedInformation?.[0]?.location.range.start.line, line);
    });

    test('a rebuilt binary restarts the language server', async () => {
        const api = vscode.extensions.getExtension('surety-lang.surety')!.exports as { restarts(): number };
        const before = api.restarts();
        const bin = vscode.workspace.getConfiguration('surety').get<string>('path')!;
        const now = new Date();
        fs.utimesSync(bin, now, now);
        const started = Date.now();
        while (api.restarts() === before) {
            assert.ok(Date.now() - started < 20000, 'no restart within 20 s of the binary changing');
            await new Promise(r => setTimeout(r, 200));
        }
        // The new server answers.
        const uri = workspaceFile('p5a.sur');
        await vscode.workspace.openTextDocument(uri);
        let symbols: vscode.DocumentSymbol[] | undefined;
        for (let i = 0; i < 50 && !symbols?.length; i++) {
            symbols = await vscode.commands.executeCommand<vscode.DocumentSymbol[]>('vscode.executeDocumentSymbolProvider', uri);
            await new Promise(r => setTimeout(r, 200));
        }
        assert.ok(symbols?.some(x => x.name.startsWith('get_first_customer_orders(')), JSON.stringify(symbols?.map(x => x.name)));
    });
});
