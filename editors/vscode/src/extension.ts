// The Surety extension: starts `surety lsp` for the file's diagnostics,
// hover, definition and symbols. Check on save and the commands arrive with
// the second piece of the slice.

import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext): void {
    const config = vscode.workspace.getConfiguration('surety');
    const bin = config.get<string>('path') || 'surety';
    const graph = config.get<string>('graph') || '';
    const args = ['lsp', ...(graph ? ['--graph', graph] : [])];
    const cwd = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    const serverOptions: ServerOptions = { command: bin, args, options: { cwd } };
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'surety' }],
    };
    client = new LanguageClient('surety', 'Surety', serverOptions, clientOptions);
    client.start();
    context.subscriptions.push({ dispose: () => client?.stop() });
}

export function deactivate(): Thenable<void> | undefined {
    return client?.stop();
}
