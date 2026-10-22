import * as assert from 'assert';
import * as path from 'path';
import * as vscode from 'vscode';

function workspaceFile(name: string): vscode.Uri {
    const root = vscode.workspace.workspaceFolders![0].uri.fsPath;
    return vscode.Uri.file(path.join(root, name));
}

const parseErrors = (uri: vscode.Uri) =>
    vscode.languages.getDiagnostics(uri).filter(d => d.source === 'surety' && d.severity === vscode.DiagnosticSeverity.Error);

async function until(what: string, ok: () => boolean, timeoutMs = 30000): Promise<void> {
    const started = Date.now();
    while (!ok()) {
        if (Date.now() - started > timeoutMs) {
            throw new Error(`timed out waiting for ${what}`);
        }
        await new Promise(r => setTimeout(r, 100));
    }
}

async function never(what: string, bad: () => boolean, forMs = 2000): Promise<void> {
    const started = Date.now();
    while (Date.now() - started < forMs) {
        assert.ok(!bad(), what);
        await new Promise(r => setTimeout(r, 100));
    }
}

suite('diagnostics wait while a clause is typed', () => {
    test('a quantifier head with no body yet: held in its block, shown on leaving it', async () => {
        const uri = workspaceFile('typing.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        const editor = await vscode.window.showTextDocument(doc);
        await until('the server to see the file clean', () => parseErrors(uri).length === 0);
        // Typed as an author types it, at the cursor: Enter after the note,
        // the head, Enter.
        const at = (line: number, ch: number) => new vscode.Position(line, ch);
        const type = async (where: vscode.Position, text: string) => {
            editor.selection = new vscode.Selection(where, where);
            await editor.edit(e => e.insert(where, text));
        };
        await type(at(2, doc.lineAt(2).text.length), '\n\t');
        await type(at(3, 1), 'all o in out:');
        await type(at(3, doc.lineAt(3).text.length), '\n\t\t');
        editor.selection = new vscode.Selection(at(4, 2), at(4, 2));
        const onHead = () => parseErrors(uri).some(d => d.range.start.line === 3);
        await never('no error on the head while the cursor is under it', onHead);
        // Moving within the block keeps it held.
        editor.selection = new vscode.Selection(at(3, 5), at(3, 5));
        await never('no error while the cursor is on the head', onHead, 1000);
        // Leaving the block shows parse's error on the head.
        editor.selection = new vscode.Selection(at(0, 0), at(0, 0));
        await until('the error on the head', onHead);
        const d = parseErrors(uri).find(x => x.range.start.line === 3)!;
        assert.ok(d.message.includes('no body'), d.message);
        assert.strictEqual(d.range.start.character, 1, 'the squiggle starts after the tab');
        // Back into the block without an edit: the error stays where it is.
        editor.selection = new vscode.Selection(at(4, 2), at(4, 2));
        await never('the error stays shown when the cursor only returns', () => !onHead(), 1000);
        await vscode.commands.executeCommand('workbench.action.files.revert');
    });

    test('a field on something that is not a record shows at once, even in the block being typed', async () => {
        const uri = workspaceFile('typing.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        const editor = await vscode.window.showTextDocument(doc);
        await until('the server to see the file clean', () => parseErrors(uri).length === 0);
        const at = (line: number, ch: number) => new vscode.Position(line, ch);
        // `o.amount` is an Int: no field can follow it, however the line goes on.
        const type = async (where: vscode.Position, text: string) => {
            editor.selection = new vscode.Selection(where, where);
            await editor.edit(e => e.insert(where, text));
        };
        await type(at(2, doc.lineAt(2).text.length), '\n\tall o in out: o.amount. > 0');
        // The field name typed inside the line, at the cursor: the edit holds this block.
        await type(at(3, 24), 'x');
        editor.selection = new vscode.Selection(at(3, 25), at(3, 25));
        await until('the error, with the cursor still on the line', () =>
            parseErrors(uri).some(d => d.range.start.line === 3 && d.code === 'not_a_record'));
        const d = parseErrors(uri).find(x => x.range.start.line === 3)!;
        assert.ok(d.message.startsWith('`o.amount` is Int, not a record'), d.message);
        await vscode.commands.executeCommand('workbench.action.files.revert');
    });

    test('an edit away from the cursor is not held', async () => {
        const uri = workspaceFile('typing.sur');
        const doc = await vscode.workspace.openTextDocument(uri);
        const editor = await vscode.window.showTextDocument(doc);
        await until('the server to see the file clean', () => parseErrors(uri).length === 0);
        // The cursor stays on line 1 while a script, editing inside line 3
        // alone, deletes the body of `all o in out: o in orders`.
        editor.selection = new vscode.Selection(1, 0, 1, 0);
        const head = doc.lineAt(3).text;
        assert.strictEqual(head, '\tall o in out: o in orders');
        await editor.edit(e => e.delete(new vscode.Range(3, head.indexOf(':') + 1, 3, head.length)));
        await until('the error, though nobody moved the cursor', () =>
            parseErrors(uri).some(d => d.range.start.line === 3 && d.message.includes('no body')));
        await vscode.commands.executeCommand('workbench.action.files.revert');
    });
});
