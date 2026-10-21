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
        const diags = await ours(uri, all => all.some(d => onSignature(sig)(d) && /UNDERDETERMINED/.test(d.message)));
        const summary = diags.find(d => d.message.startsWith('UNDERDETERMINED'))!;
        assert.ok(summary, JSON.stringify(diags.map(d => d.message)));
        assert.ok(diags.some(d => d.message.startsWith('underdetermined: two outputs pass every clause on')), 'each undecided input is named');
    });
});
