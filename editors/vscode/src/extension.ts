// The Surety extension. `surety lsp` answers diagnostics, hover, definition
// and symbols as the file changes; on save the extension runs the checker
// and puts the verdict on the signature line and each finding on the line
// of the clause it names. Nothing runs the checker on a keystroke.

import * as cp from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';
import { Block, blockAt } from './block';

let client: LanguageClient | undefined;
let verdicts: vscode.DiagnosticCollection;
let output: vscode.OutputChannel;

// The server's diagnostics are parse's on every change, and a clause is
// wrong until it is finished: `all Ok(r) in out:` has no body the moment
// it is typed. So the server's diagnostics inside the block of the last
// edit wait until the cursor leaves that block, the file is saved, or
// another editor takes focus. An edit that spans blocks (a paste, a file
// rewritten on disk) holds nothing.
const latest = new Map<string, vscode.Diagnostic[]>();
const holds = new Map<string, Block>();

function shown(uri: vscode.Uri): vscode.Diagnostic[] {
    const all = latest.get(uri.toString()) ?? [];
    const hold = holds.get(uri.toString());
    if (!hold) {
        return all;
    }
    return all.filter(d => d.range.start.line < hold.start || d.range.start.line > hold.end);
}

function release(uri: vscode.Uri): void {
    if (holds.delete(uri.toString())) {
        client?.diagnostics?.set(uri, shown(uri));
    }
}

function holdEdit(e: vscode.TextDocumentChangeEvent): void {
    const doc = e.document;
    if (doc.languageId !== 'surety' || e.contentChanges.length === 0) {
        return;
    }
    if (vscode.window.activeTextEditor?.document !== doc) {
        release(doc.uri);
        return;
    }
    const lines = doc.getText().split(/\r?\n/);
    let hold: Block | undefined;
    for (const c of e.contentChanges) {
        const first = c.range.start.line;
        const last = first + (c.text.match(/\n/g)?.length ?? 0);
        const block = blockAt(lines, first);
        if (last > block.end) {
            release(doc.uri);
            return;
        }
        if (hold && (block.start !== hold.start || block.end !== hold.end)) {
            release(doc.uri);
            return;
        }
        hold = block;
    }
    holds.set(doc.uri.toString(), hold!);
    client?.diagnostics?.set(doc.uri, shown(doc.uri));
}

function settings() {
    const c = vscode.workspace.getConfiguration('surety');
    return {
        path: c.get<string>('path') || 'surety',
        graph: c.get<string>('graph') || '',
        python: c.get<string>('python') || 'python3',
        checkOnSave: c.get<boolean>('checkOnSave') ?? true,
    };
}

/** The line of the obligation's signature: `name(…) -> …`. */
export function signatureLine(doc: vscode.TextDocument): number {
    for (let i = 0; i < doc.lineCount; i++) {
        if (/^[a-z_][A-Za-z0-9_]*\s*\(.*\)\s*->/.test(doc.lineAt(i).text)) {
            return i;
        }
    }
    return 0;
}

function run(bin: string, args: string[], cwd: string | undefined): Promise<{ code: number; stdout: string; stderr: string }> {
    return new Promise(resolve => {
        cp.execFile(bin, args, { cwd, maxBuffer: 64 * 1024 * 1024 }, (err, stdout, stderr) => {
            const code = err && typeof (err as cp.ExecFileException).code === 'number' ? (err as cp.ExecFileException).code as number : err ? 1 : 0;
            resolve({ code, stdout: stdout.toString(), stderr: stderr.toString() });
        });
    });
}

/** A line from its first character after the indentation to its end. */
function lineRange(doc: vscode.TextDocument, line: number): vscode.Range {
    const l = doc.lineAt(Math.max(0, Math.min(line, doc.lineCount - 1)));
    return l.isEmptyOrWhitespace ? l.range : l.range.with({ start: l.range.start.with({ character: l.firstNonWhitespaceCharacterIndex }) });
}

type Mode = 'auto' | 'check' | 'mutants';

/** Run `check` (a performance beside the file) or `mutants` (none) and show the result. */
async function checkDocument(doc: vscode.TextDocument, mode: Mode): Promise<void> {
    if (doc.languageId !== 'surety' || doc.uri.scheme !== 'file') {
        return;
    }
    const s = settings();
    const file = doc.uri.fsPath;
    const performance = file.replace(/\.sur$/, '.py');
    const useCheck = mode === 'check' || (mode === 'auto' && fs.existsSync(performance));
    const verb = useCheck ? 'check' : 'mutants';
    const cwd = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? path.dirname(file);
    const sig = signatureLine(doc);
    const started = Date.now();
    output.appendLine(`$ ${s.path} ${verb} ${path.basename(file)}`);
    const [json, text] = await Promise.all([
        run(s.path, [verb, '--json', file], cwd),
        run(s.path, [verb, file], cwd),
    ]);
    output.append(text.stdout);
    if (text.stderr) {
        output.append(text.stderr);
    }
    output.appendLine(`(${((Date.now() - started) / 1000).toFixed(1)} s)`);
    if (json.code === 2 || !json.stdout.trim()) {
        const d = new vscode.Diagnostic(lineRange(doc, sig), `surety ${verb} could not run: ${(json.stderr || text.stderr).trim()}`, vscode.DiagnosticSeverity.Error);
        d.source = 'surety';
        verdicts.set(doc.uri, [d]);
        gapDetail.delete(doc.uri.toString());
        return;
    }
    const lines = json.stdout.split('\n').filter(l => l.trim()).map(l => JSON.parse(l));
    const diags = useCheck ? checkDiagnostics(doc, lines, sig) : mutantDiagnostics(doc, lines, sig);
    verdicts.set(doc.uri, diags);
    gapDetail.set(doc.uri.toString(), lines.filter(l => l.kind === 'gap').map(g => ({ range: lineRange(doc, sig), md: gapMarkdown(g) })));
}

// A gap is the underdetermination check's finding as an author reads it:
// where two outputs that pass every clause differ, however many inputs
// show it. Its diagnostic is that one line; the input and both outputs in
// full, highlighted, are its hover. The inputs and mutants it holds carry
// its index and get no diagnostic of their own.
const gapDetail = new Map<string, { range: vscode.Range; md: vscode.MarkdownString }[]>();

const plural = (n: number, one: string, many: string) => `${n} ${n === 1 ? one : many}`;

function inline(s: string): string {
    return s.includes('`') ? `\`\` ${s} \`\`` : `\`${s}\``;
}

function gapMessage(g: any): string {
    return `gap: ${g.says}  (${plural(g.shown_by, 'input shows', 'inputs show')} it)`;
}

function gapMarkdown(g: any): vscode.MarkdownString {
    const md = new vscode.MarkdownString();
    md.appendMarkdown('**Two outputs pass every clause:** ');
    md.appendText(`${g.says}.`);
    md.appendMarkdown('\n\n');
    const first = g.a_is_expected ? `example ${g.example}` : 'passes every clause';
    md.appendCodeblock(`${g.inputs?.[0]?.sur ?? 'in: ?'}\nout: ${g.sur.a}    // ${first}\nout: ${g.sur.b}    // also passes every clause`, 'surety');
    if (g.at && g.says.includes('\u2026')) {
        md.appendMarkdown(`\n\nAt ${inline(g.place)}: ${g.at.a ? inline(g.at.a) : 'nothing'} or ${g.at.b ? inline(g.at.b) : 'nothing'}.`);
    }
    md.appendMarkdown(`\n\n${plural(g.shown_by, 'input shows', 'inputs show')} it; this is the smallest.`);
    const mutants: any[] = g.mutants ?? [];
    if (mutants.length) {
        const named = mutants.map(m => `${inline(m.mutant)} (${m.describes}${m.returns ? `: ${inline(m.returns)}` : ''})`);
        md.appendMarkdown(` The same hole is what ${named.join(' and ')} ${mutants.length === 1 ? 'returns' : 'return'}.`);
    }
    return md;
}

function diag(doc: vscode.TextDocument, line: number, message: string, severity: vscode.DiagnosticSeverity): vscode.Diagnostic {
    const d = new vscode.Diagnostic(lineRange(doc, line - 1), message, severity);
    d.source = 'surety';
    return d;
}

function checkDiagnostics(doc: vscode.TextDocument, lines: any[], sig: number): vscode.Diagnostic[] {
    const out: vscode.Diagnostic[] = [];
    const evidence = lines.find(l => l.kind === 'evidence');
    if (evidence) {
        const verdict: string = evidence.verdict ?? (evidence.strict_passed ? 'ACCEPTED' : 'NOT ACCEPTED');
        const parts = [verdict];
        if ((evidence.obligation_findings ?? []).length) {
            parts.push(`obligation: ${evidence.obligation_findings.join('; ')}`);
        }
        if ((evidence.performance_findings ?? []).length) {
            parts.push(`performance: ${evidence.performance_findings.join('; ')}`);
        }
        const severity = verdict === 'ACCEPTED' ? vscode.DiagnosticSeverity.Information : vscode.DiagnosticSeverity.Warning;
        out.push(diag(doc, sig + 1, parts.join('  '), severity));
    }
    const binding = evidence && evidence.passed === false;
    for (const l of lines) {
        if (l.kind === 'evidence' || (l.gap !== undefined && l.kind !== 'gap')) {
            continue;
        }
        if (l.kind === 'gap') {
            out.push(diag(doc, sig + 1, gapMessage(l), vscode.DiagnosticSeverity.Warning));
            continue;
        }
        const party = l.party ? ` (${l.party})` : '';
        const line: number = l.clause?.line ?? l.slipped?.[0]?.line ?? l.accept?.line ?? sig + 1;
        const detail: string = l.message ?? l.inputs?.[0]?.sur ?? '';
        const severity = binding && (l.kind === 'example_mismatch' || l.example !== undefined)
            ? vscode.DiagnosticSeverity.Error
            : vscode.DiagnosticSeverity.Warning;
        out.push(diag(doc, line, `${l.kind}${party}: ${detail}`, severity));
    }
    return out;
}

function mutantDiagnostics(doc: vscode.TextDocument, lines: any[], sig: number): vscode.Diagnostic[] {
    const out: vscode.Diagnostic[] = [];
    const gaps = lines.filter(l => l.kind === 'gap');
    const slipped = lines.filter(l => l.kind === 'mutant_slipped');
    const underdetermined = lines.filter(l => l.kind === 'underdetermined');
    const accepted = lines.filter(l => l.kind === 'accepted');
    const applicable = lines.filter(l => l.kind === 'mutant_slipped' || l.kind === 'mutant_caught').length;
    const total = applicable + lines.filter(l => l.kind === 'mutant_not_applicable').length;
    const parts = [plural(gaps.length, 'gap', 'gaps'), `mutants: ${applicable} of ${total} apply, ${slipped.length} slipped`];
    if (accepted.length) {
        parts.push(`${accepted.length} accepted`);
    }
    const clean = slipped.length === 0 && underdetermined.length === 0;
    out.push(diag(doc, sig + 1, `${clean ? 'NOT UNDERDETERMINED' : 'UNDERDETERMINED'}  ${parts.join('; ')}`, clean ? vscode.DiagnosticSeverity.Information : vscode.DiagnosticSeverity.Warning));
    for (const g of gaps) {
        out.push(diag(doc, sig + 1, gapMessage(g), vscode.DiagnosticSeverity.Warning));
    }
    for (const m of slipped.filter(m => m.gap === undefined)) {
        const name = `${m.candidate?.mutant} (${m.candidate?.describes})`;
        const clauses: any[] = m.slipped ?? [];
        if (clauses.length === 0) {
            out.push(diag(doc, sig + 1, `mutant ${name} slips through every clause`, vscode.DiagnosticSeverity.Warning));
        }
        for (const c of clauses) {
            out.push(diag(doc, c.line ?? sig + 1, `mutant ${name} slips through this clause`, vscode.DiagnosticSeverity.Warning));
        }
    }
    for (const u of underdetermined.filter(u => u.gap === undefined)) {
        out.push(diag(doc, sig + 1, `underdetermined: two outputs pass every clause on ${u.inputs?.[0]?.sur ?? 'an input'}`, vscode.DiagnosticSeverity.Warning));
    }
    return out;
}

export function activate(context: vscode.ExtensionContext): void {
    const s = settings();
    output = vscode.window.createOutputChannel('Surety');
    verdicts = vscode.languages.createDiagnosticCollection('surety-check');
    const args = ['lsp', ...(s.graph ? ['--graph', s.graph] : [])];
    const cwd = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    const serverOptions: ServerOptions = { command: s.path, args, options: { cwd } };
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'surety' }],
        middleware: {
            handleDiagnostics: (uri, diagnostics, next) => {
                latest.set(uri.toString(), diagnostics);
                next(uri, shown(uri));
            },
        },
    };
    client = new LanguageClient('surety', 'Surety', serverOptions, clientOptions);
    client.start();
    context.subscriptions.push(
        output,
        verdicts,
        { dispose: () => client?.stop() },
        vscode.workspace.onDidChangeTextDocument(holdEdit),
        vscode.window.onDidChangeTextEditorSelection(e => {
            const doc = e.textEditor.document;
            const hold = holds.get(doc.uri.toString());
            const line = e.selections[0]?.active.line;
            if (hold && line !== undefined && (line < hold.start || line > hold.end)) {
                release(doc.uri);
            }
        }),
        vscode.window.onDidChangeActiveTextEditor(editor => {
            for (const key of [...holds.keys()]) {
                if (key !== editor?.document.uri.toString()) {
                    release(vscode.Uri.parse(key));
                }
            }
        }),
        vscode.workspace.onDidCloseTextDocument(doc => {
            holds.delete(doc.uri.toString());
            latest.delete(doc.uri.toString());
            gapDetail.delete(doc.uri.toString());
        }),
        vscode.languages.registerHoverProvider({ language: 'surety' }, {
            provideHover(doc, position) {
                const hits = (gapDetail.get(doc.uri.toString()) ?? []).filter(g => g.range.contains(position));
                return hits.length ? new vscode.Hover(hits.map(g => g.md), hits[0].range) : undefined;
            },
        }),
        vscode.workspace.onDidSaveTextDocument(doc => {
            release(doc.uri);
            if (settings().checkOnSave) {
                void checkDocument(doc, 'auto');
            }
        }),
        vscode.commands.registerCommand('surety.check', () => {
            const doc = vscode.window.activeTextEditor?.document;
            if (doc) { return checkDocument(doc, 'check'); }
        }),
        vscode.commands.registerCommand('surety.mutants', () => {
            const doc = vscode.window.activeTextEditor?.document;
            if (doc) { return checkDocument(doc, 'mutants'); }
        }),
        vscode.commands.registerCommand('surety.draft', () => {
            const doc = vscode.window.activeTextEditor?.document;
            const root = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
            if (!doc || !root) { return; }
            const script = path.join(root, 'experiments', 'draft.py');
            if (!fs.existsSync(script)) {
                void vscode.window.showWarningMessage(`No experiments/draft.py in ${root}.`);
                return;
            }
            // The draft calls a model with the credentials in your shell, so it runs in a terminal.
            const terminal = vscode.window.createTerminal({ name: 'Surety draft', cwd: root });
            terminal.show();
            terminal.sendText(`${settings().python} ${script} lines ${doc.uri.fsPath} --surety ${settings().path}`);
        }),
    );
}

export function deactivate(): Thenable<void> | undefined {
    return client?.stop();
}
