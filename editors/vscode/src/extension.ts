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
    gapDetail.set(doc.uri.toString(), lines.filter(l => l.kind === 'gap').map(g => ({ range: outputRange(doc, sig), md: gapMarkdown(g), gap: g, sig })));
}

/** The output type on the signature line: what a gap, a claim about `out`, sits under. */
function outputRange(doc: vscode.TextDocument, sig: number): vscode.Range {
    const text = doc.lineAt(Math.max(0, Math.min(sig, doc.lineCount - 1))).text;
    const arrow = text.lastIndexOf('->');
    if (arrow < 0) {
        return lineRange(doc, sig);
    }
    const start = arrow + 2 + (text.slice(arrow + 2).length - text.slice(arrow + 2).trimStart().length);
    const end = text.trimEnd().length;
    return new vscode.Range(sig, start, sig, Math.max(start, end));
}

// A gap is the underdetermination check's finding as an author reads it:
// where two outputs that pass every clause differ, however many inputs
// show it. Its diagnostic is that one line, under the output type; its
// hover is the example that shows it, laid out and highlighted. The inputs
// and mutants it holds carry its index and get no diagnostic of their own.
const gapDetail = new Map<string, { range: vscode.Range; md: vscode.MarkdownString; gap: any; sig: number }[]>();

const plural = (n: number, one: string, many: string) => `${n} ${n === 1 ? one : many}`;

function gapMessage(g: any): string {
    return `underdetermined: ${g.says}  (${plural(g.shown_by, 'input', 'inputs')})`;
}

function gapMarkdown(g: any): vscode.MarkdownString {
    const md = new vscode.MarkdownString();
    const a = g.a_is_expected ? `    // example ${g.example}` : '';
    md.appendCodeblock(`in: ${g.pretty.in}\nout: ${g.pretty.a}${a}\nout: ${g.pretty.b}`, 'surety');
    const mutants: string[] = (g.mutants ?? []).map((m: any) => `\`${m.mutant}\``);
    if (mutants.length) {
        md.appendMarkdown(`Also what ${mutants.join(' and ')} ${mutants.length === 1 ? 'returns' : 'return'}.`);
    }
    return md;
}

/** A clause no mutant breaks: one that never reads `out` cannot be broken by any output. */
function unbreakable(inputOnly: boolean): string {
    return inputOnly
        ? 'this clause never reads `out`: a claim about the input alone, which no output can break'
        : 'no mutant breaks this clause: check that it claims what you mean';
}

const QUICK_FIX_TITLE = 'Add this input as an example, with both outputs to choose from';

/**
 * The quick fix on a gap: the gap's input as a drafted example whose two
 * `out:` lines sit between merge-conflict markers, so the author keeps the
 * one they mean (the editor offers Accept Current and Accept Incoming) and
 * the file does not parse until they do. Appended to the obligation's block.
 */
function chooseOutput(doc: vscode.TextDocument, sig: number, g: any): vscode.WorkspaceEdit {
    let last = sig;
    let unit = '\t';
    for (let k = sig + 1; k < doc.lineCount; k++) {
        const t = doc.lineAt(k).text;
        if (t.trim() === '') {
            continue;
        }
        if (!/^\s/.test(t)) {
            break;
        }
        if (last === sig) {
            unit = t.match(/^\s+/)![0];
        }
        last = k;
    }
    const body = unit + unit;
    const value = (head: string, v: string) => `${body}${head}: ${v.split('\n').join(`\n${body}`)}`;
    const text = [
        '',
        '',
        `${unit}// drafted: two outputs pass every clause on this input; keep the one you mean`,
        `${unit}e:`,
        value('in', g.pretty.in),
        '<<<<<<< passes every clause',
        value('out', g.pretty.a),
        '=======',
        value('out', g.pretty.b),
        '>>>>>>> also passes every clause',
    ].join('\n');
    const edit = new vscode.WorkspaceEdit();
    edit.insert(doc.uri, doc.lineAt(last).range.end, text);
    return edit;
}

function diag(doc: vscode.TextDocument, line: number, message: string, severity: vscode.DiagnosticSeverity): vscode.Diagnostic {
    return diagAt(lineRange(doc, line - 1), message, severity);
}

function diagAt(range: vscode.Range, message: string, severity: vscode.DiagnosticSeverity): vscode.Diagnostic {
    const d = new vscode.Diagnostic(range, message, severity);
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
            out.push(diagAt(outputRange(doc, sig), gapMessage(l), vscode.DiagnosticSeverity.Warning));
            continue;
        }
        if (l.kind === 'look_here' && l.clause?.line !== undefined) {
            if (!/\bunordered$/.test(l.clause.text ?? '')) {
                out.push(diag(doc, l.clause.line, unbreakable(!!l.input_only), vscode.DiagnosticSeverity.Information));
            }
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
    const parts = [`mutants: ${applicable} of ${total} apply, ${slipped.length} slipped`];
    if (accepted.length) {
        parts.push(`${accepted.length} accepted`);
    }
    const clean = slipped.length === 0 && underdetermined.length === 0;
    // Gaps say it themselves; the counts go to the output channel.
    if (gaps.length === 0) {
        out.push(diag(doc, sig + 1, `${clean ? 'NOT UNDERDETERMINED' : 'UNDERDETERMINED'}  ${parts.join('; ')}`, clean ? vscode.DiagnosticSeverity.Information : vscode.DiagnosticSeverity.Warning));
    }
    for (const g of gaps) {
        out.push(diagAt(outputRange(doc, sig), gapMessage(g), vscode.DiagnosticSeverity.Warning));
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
    const report = lines.find(l => l.kind === 'mutant_report');
    // `x unordered` claims nothing by design: no hint for it.
    for (const c of report?.uncaught ?? []) {
        if (c.line !== undefined && !/\bunordered$/.test(c.text ?? '')) {
            out.push(diag(doc, c.line, unbreakable(!!c.input_only), vscode.DiagnosticSeverity.Information));
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
        vscode.languages.registerCodeActionsProvider({ language: 'surety' }, {
            provideCodeActions(doc, _range, context) {
                const gaps = gapDetail.get(doc.uri.toString()) ?? [];
                const actions: vscode.CodeAction[] = [];
                for (const d of context.diagnostics) {
                    // An input that is already an example has its output chosen.
                    const hit = gaps.find(g => d.source === 'surety' && d.message === gapMessage(g.gap) && !g.gap.a_is_expected);
                    if (hit) {
                        const action = new vscode.CodeAction(QUICK_FIX_TITLE, vscode.CodeActionKind.QuickFix);
                        action.diagnostics = [d];
                        action.edit = chooseOutput(doc, hit.sig, hit.gap);
                        actions.push(action);
                    }
                }
                return actions;
            },
        }, { providedCodeActionKinds: [vscode.CodeActionKind.QuickFix] }),
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
