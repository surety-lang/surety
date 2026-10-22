import * as assert from 'assert';
import * as path from 'path';
import * as vscode from 'vscode';

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

suite('the verdict on save', () => {
    test('P5a with its performance: ACCEPTED on the signature line', async () => {
        const uri = workspaceFile('p5a.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        const editor = await vscode.window.showTextDocument(doc);
        // A save that changes nothing semantically: a trailing newline.
        await editor.edit(e => e.insert(new vscode.Position(doc.lineCount, 0), '\n'));
        assert.ok(await doc.save(), 'the document saves');
        const sig = 2; // get_first_customer_orders(orders: Order[]) -> Order[] is line 3
        const diags = await ours(uri, all => all.some(d => onSignature(sig)(d) && /^(ACCEPTED|NOT ACCEPTED)/.test(d.message)));
        const verdict = diags.find(onSignature(sig))!;
        assert.ok(verdict.message.startsWith('ACCEPTED'), verdict.message);
        assert.strictEqual(verdict.severity, vscode.DiagnosticSeverity.Information);
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
    });

    test('P5a without its accept line and no performance: the mutants\' findings', async () => {
        const uri = workspaceFile('ties.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        await vscode.window.showTextDocument(doc);
        await vscode.commands.executeCommand('surety.mutants');
        const sig = 2;
        const diags = await ours(uri, all => all.some(d => onSignature(sig)(d) && d.message.startsWith('underdetermined: ')));
        // The ties are one gap, read where the two passing outputs differ,
        // under the output type; no input gets a diagnostic of its own, and
        // no summary line stacks on it.
        const gaps = diags.filter(d => d.message.startsWith('underdetermined: '));
        assert.strictEqual(gaps.length, 1, JSON.stringify(diags.map(d => d.message)));
        assert.ok(gaps[0].message.includes('].amount can be '), gaps[0].message);
        assert.ok(!diags.some(d => /^UNDERDETERMINED|^underdetermined: two outputs/.test(d.message)), JSON.stringify(diags.map(d => d.message)));
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

    test('a clause about the input alone: a hint on its line that no output can break it', async () => {
        const uri = workspaceFile('inputs.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        await vscode.window.showTextDocument(doc);
        await vscode.commands.executeCommand('surety.mutants');
        const line = doc.getText().split('\n').findIndex(l => l.includes('all o in orders: o.amount >= 0'));
        const diags = await ours(uri, all => all.some(d => d.range.start.line === line));
        const hint = diags.find(d => d.range.start.line === line)!;
        assert.strictEqual(hint.severity, vscode.DiagnosticSeverity.Information);
        assert.ok(hint.message.startsWith('this clause never reads `out`'), hint.message);
        assert.strictEqual(hint.range.start.character, 1, 'after the tab');
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
});
