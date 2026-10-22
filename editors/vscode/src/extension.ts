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

// An error that no further typing on its line can mend is not held: a
// field on something that is not a record stays wrong however the line
// goes on.
const UNMENDABLE = new Set(['not_a_record']);

function shown(uri: vscode.Uri): vscode.Diagnostic[] {
    const all = latest.get(uri.toString()) ?? [];
    const hold = holds.get(uri.toString());
    if (!hold) {
        return all;
    }
    return all.filter(d => UNMENDABLE.has(String(typeof d.code === 'object' ? d.code.value : d.code))
        || d.range.start.line < hold.start || d.range.start.line > hold.end);
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
        // The file's own error names its line: say that line, not the path
        // twice, and link to it; the server underlines the term there.
        const err = (json.stderr || text.stderr).trim();
        const at = /line (\d+): (.*)$/m.exec(err);
        const message = at ? `surety ${verb} could not run: line ${at[1]}: ${at[2]}` : `surety ${verb} could not run: ${err}`;
        const d = new vscode.Diagnostic(lineRange(doc, sig), message, vscode.DiagnosticSeverity.Error);
        d.source = 'surety';
        if (at) {
            d.relatedInformation = [
                new vscode.DiagnosticRelatedInformation(new vscode.Location(doc.uri, lineRange(doc, Number(at[1]) - 1)), at[2]),
            ];
        }
        verdicts.set(doc.uri, [d]);
        gapDetail.delete(doc.uri.toString());
        setLens(doc, undefined);
        return;
    }
    const lines = json.stdout.split('\n').filter(l => l.trim()).map(l => JSON.parse(l));
    const diags = useCheck ? checkDiagnostics(doc, lines, sig) : mutantDiagnostics(doc, lines, sig);
    verdicts.set(doc.uri, diags);
    setLens(doc, useCheck ? checkLens(lines) : mutantLens(lines));
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

// The verdict of the last run on a file, as one line above its signature:
// a CodeLens, not a diagnostic, so a pass is not drawn as a squiggle. What
// is wrong (a gap, a slipped mutant, a failing clause, a run that could not
// start) stays a warning or an error on its own line.
const verdictLens = new Map<string, { title: string; tooltip: string }>();
const lensChanged = new vscode.EventEmitter<void>();

function setLens(doc: vscode.TextDocument, lens: { title: string; tooltip: string } | undefined): void {
    if (lens) {
        verdictLens.set(doc.uri.toString(), lens);
    } else {
        verdictLens.delete(doc.uri.toString());
    }
    lensChanged.fire();
}

const MUTANTS_ARE = 'Mutants are deliberately wrong performances from a fixed library: return nothing, return the input unchanged, drop, repeat or swap an element, and others.';

/** The mutants run in a line: settled or not, and why, with the counts in the tooltip. */
export function mutantLens(lines: any[]): { title: string; tooltip: string } {
    const report = lines.find(l => l.kind === 'mutant_report') ?? {};
    const m = report.mutants ?? {};
    const applicable: number = m.applicable ?? 0;
    const total: number = m.total ?? 0;
    const slipped: number = m.slipped ?? 0;
    const gaps = lines.filter(l => l.kind === 'gap').length;
    const inputs = (report.example_inputs ?? 0) + (report.generated_inputs ?? 0);
    const searched: number = report.search?.inputs ?? 0;
    const cap: number = report.search?.candidates_cap ?? 0;
    const accepted: number = report.accepted_inputs ?? 0;
    const open = accepted ? ` · ${plural(accepted, 'input', 'inputs')} left open by \`accept\`` : '';
    const ran = `${MUTANTS_ARE} ${applicable} of the ${total} fit this signature and ran on ${inputs} inputs. `
        + `The search looked at ${plural(searched, 'input', 'inputs')}, up to ${cap} candidate outputs each, for two different outputs that both pass every clause`;
    const excluded: number = report.excluded_inputs ?? 0;
    if (gaps || slipped || excluded) {
        const problems = [];
        // No output at all on some inputs comes first: there the question of
        // which output does not arise, which is why a gap can vanish.
        if (excluded) {
            problems.push(`Unsatisfiable on ${excluded} of ${inputs} inputs, which a clause about the input alone rules out`);
        }
        const parts = [];
        if (gaps) {
            parts.push(`${plural(gaps, 'gap', 'gaps')}, where two different outputs both pass every clause`);
        }
        if (slipped) {
            parts.push(`${slipped} of the ${applicable} mutants that fit ${slipped === 1 ? 'passes' : 'pass'} every clause`);
        }
        if (parts.length) {
            problems.push(`Not settled: ${parts.join('; ')}`);
        }
        return {
            title: `$(warning) ${problems.join(' · ')}${open}`,
            tooltip: `${ran}. Each problem is a warning in the file. Click for the full report.`,
        };
    }
    const mutants = applicable ? `all ${applicable} mutants that fit are caught` : `none of the ${total} mutants fits this signature`;
    return {
        title: `$(pass) Settled: ${mutants}; no second passing output on the ${plural(searched, 'input', 'inputs')} searched${open}`,
        tooltip: `${ran}, and found none. ${applicable ? 'A clause rejected every mutant. ' : ''}Click for the full report.`,
    };
}

/** The check in a line: accepted or not, with what was run in the tooltip. */
export function checkLens(lines: any[]): { title: string; tooltip: string } | undefined {
    const e = lines.find(l => l.kind === 'evidence');
    if (!e) {
        return undefined;
    }
    const u = e.underdetermination;
    const mutants = typeof u === 'object' && u
        ? `${MUTANTS_ARE} ${u.applicable} of the ${u.total} fit this signature; ${u.caught} caught, ${u.slipped} passing every clause.`
        : 'The mutants did not run.';
    const ran = `Examples: ${e.examples?.passed} of ${e.examples?.total} pass. `
        + `Clauses: ${e.properties?.passed} of ${e.properties?.total} hold over ${e.properties?.runs} runs, ${e.types?.generated_inputs} of them on generated inputs. ${mutants}`;
    if (e.verdict === 'ACCEPTED') {
        return {
            title: '$(pass) Accepted: the performance passes every example and clause, and the obligation has no findings',
            tooltip: `${ran} Click for the full report.`,
        };
    }
    const parts = [];
    if ((e.obligation_findings ?? []).length) {
        parts.push(`obligation: ${e.obligation_findings.join('; ')}`);
    }
    if ((e.performance_findings ?? []).length) {
        parts.push(`performance: ${e.performance_findings.join('; ')}`);
    }
    return {
        title: `$(warning) Not accepted: ${parts.join(' · ')}`,
        tooltip: `${ran} Each finding is a warning in the file. Click for the full report.`,
    };
}

function gapDiag(doc: vscode.TextDocument, sig: number, g: any): vscode.Diagnostic {
    const d = diagAt(outputRange(doc, sig), gapMessage(g), vscode.DiagnosticSeverity.Warning);
    d.code = GAP.code;
    return d;
}

function gapMessage(g: any): string {
    return `Gap: ${g.says}  (${plural(g.shown_by, 'input', 'inputs')})`;
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

// Every finding the extension adds opens with a plain label, the same one
// as its code (shown as `surety(code)`): what kind of thing it is before
// why. A parse error's code is its kind, from the server.
type Finding = { label: string; code: string };
const GAP: Finding = { label: 'Gap', code: 'gap' };
const MUTANT: Finding = { label: 'Mutant gets through', code: 'mutant' };
const UNNEEDED: Finding = { label: 'Unneeded clause', code: 'unneeded-clause' };
const UNTESTED: Finding = { label: 'Untested clause', code: 'untested-clause' };
const INPUT_ONLY: Finding = { label: 'Input-only clause', code: 'input-only-clause' };

function finding(range: vscode.Range, f: Finding, message: string, severity: vscode.DiagnosticSeverity): vscode.Diagnostic {
    const d = diagAt(range, `${f.label}: ${message}`, severity);
    d.code = f.code;
    return d;
}

/**
 * A clause no mutant breaks. One that never reads `out` limits the inputs,
 * not the output: where it is false no output passes, so it makes the
 * obligation unsatisfiable there, which is a warning; if it is false on
 * no input it rules nothing out at all, unneeded, faded the way the editor
 * fades unused code. One that reads `out` may still mean something the
 * library cannot exercise: untested, a hint to read it again.
 */
function unbreakable(doc: vscode.TextDocument, line: number, c: { input_only?: boolean; false_on?: number }, tried: number): vscode.Diagnostic {
    const range = lineRange(doc, line - 1);
    if (c.input_only && (c.false_on ?? 0) > 0) {
        return finding(range, INPUT_ONLY, `it never reads \`out\`, so it limits the inputs, not the output: it is false on ${c.false_on} of the ${tried} inputs tried, and no output passes there, so the obligation is unsatisfiable on them. To rule those inputs out, give them an \`Err\` case`, vscode.DiagnosticSeverity.Warning);
    }
    if (c.input_only) {
        const d = finding(range, UNNEEDED, 'it never reads `out` and holds on every input tried, so it rules nothing out', vscode.DiagnosticSeverity.Hint);
        d.tags = [vscode.DiagnosticTag.Unnecessary];
        return d;
    }
    return finding(range, UNTESTED, 'no mutant breaks it, so nothing here shows it rules any output out; check that it says what you mean', vscode.DiagnosticSeverity.Information);
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
        // A pass is the line above the signature, not a squiggle.
        if (verdict !== 'ACCEPTED') {
            out.push(diag(doc, sig + 1, parts.join('  '), vscode.DiagnosticSeverity.Warning));
        }
    }
    const binding = evidence && evidence.passed === false;
    for (const l of lines) {
        if (l.kind === 'evidence' || (l.gap !== undefined && l.kind !== 'gap')) {
            continue;
        }
        if (l.kind === 'gap') {
            out.push(gapDiag(doc, sig, l));
            continue;
        }
        if (l.kind === 'look_here' && l.clause?.line !== undefined) {
            if (!/\bunordered$/.test(l.clause.text ?? '')) {
                const tried = (evidence?.types?.example_inputs ?? 0) + (evidence?.types?.generated_inputs ?? 0);
                out.push(unbreakable(doc, l.clause.line, l, tried));
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
    // The run's verdict is the line above the signature; only problems are
    // diagnostics, each on its own line.
    for (const g of gaps) {
        out.push(gapDiag(doc, sig, g));
    }
    for (const m of slipped.filter(m => m.gap === undefined)) {
        const name = `${m.candidate?.mutant} (${m.candidate?.describes})`;
        const clauses: any[] = m.slipped ?? [];
        if (clauses.length === 0) {
            out.push(finding(lineRange(doc, sig), MUTANT, `${name} passes every clause`, vscode.DiagnosticSeverity.Warning));
        }
        for (const c of clauses) {
            out.push(finding(lineRange(doc, (c.line ?? sig + 1) - 1), MUTANT, `${name} passes this clause`, vscode.DiagnosticSeverity.Warning));
        }
    }
    const report = lines.find(l => l.kind === 'mutant_report');
    // `x unordered` claims nothing by design: no hint for it.
    for (const c of report?.uncaught ?? []) {
        if (c.line !== undefined && !/\bunordered$/.test(c.text ?? '')) {
            out.push(unbreakable(doc, c.line, c, (report.example_inputs ?? 0) + (report.generated_inputs ?? 0)));
        }
    }
    for (const u of underdetermined.filter(u => u.gap === undefined)) {
        out.push(finding(outputRange(doc, sig), GAP, `two outputs pass every clause on ${u.inputs?.[0]?.sur ?? 'an input'}`, vscode.DiagnosticSeverity.Warning));
    }
    return out;
}

// A call to a name nothing declares is a graph reference whose performance
// is supplied unchecked: not an error, so not a squiggle, but marked in a
// style of its own, italic with a dotted underline, the hover saying why.
// The server lists them; the ranges are kept per file for the test.
let unverifiedStyle: vscode.TextEditorDecorationType;
const unverifiedRanges = new Map<string, vscode.Range[]>();
let unverifiedTimer: NodeJS.Timeout | undefined;

async function markUnverified(editor: vscode.TextEditor | undefined): Promise<void> {
    if (!editor || editor.document.languageId !== 'surety' || !client?.isRunning()) {
        return;
    }
    const doc = editor.document;
    const found = await client.sendRequest<{ range: { start: { line: number; character: number }; end: { line: number; character: number } } }[]>(
        'surety/unverified', { textDocument: { uri: doc.uri.toString() } });
    const ranges = found.map(f => new vscode.Range(f.range.start.line, f.range.start.character, f.range.end.line, f.range.end.character));
    unverifiedRanges.set(doc.uri.toString(), ranges);
    editor.setDecorations(unverifiedStyle, ranges);
}

function markUnverifiedSoon(): void {
    if (unverifiedTimer) {
        clearTimeout(unverifiedTimer);
    }
    unverifiedTimer = setTimeout(() => void markUnverified(vscode.window.activeTextEditor), 300);
}

// The server is the binary at `surety.path`. A build that replaces it leaves
// the running server the old one until a restart, so the extension watches
// the file and restarts the client when it changes: a rebuild reaches the
// editor without a reload. The count is what the test reads.
let restarts = 0;

function watchBinary(bin: string): vscode.Disposable {
    if (!path.isAbsolute(bin) || !fs.existsSync(bin)) {
        return { dispose() {} };
    }
    let timer: NodeJS.Timeout | undefined;
    const changed = (now: fs.Stats, before: fs.Stats) => {
        if (now.mtimeMs === before.mtimeMs || now.size === 0) {
            return;
        }
        if (timer) {
            clearTimeout(timer);
        }
        // A build writes the file more than once; restart when it settles.
        timer = setTimeout(async () => {
            output.appendLine(`${bin} changed; restarting the language server`);
            await client?.restart();
            restarts++;
            markUnverifiedSoon();
        }, 500);
    };
    fs.watchFile(bin, { interval: 500 }, changed);
    return {
        dispose() {
            fs.unwatchFile(bin, changed);
            if (timer) {
                clearTimeout(timer);
            }
        },
    };
}

export function activate(context: vscode.ExtensionContext): { restarts(): number; unverified(uri: vscode.Uri): vscode.Range[] } {
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
    unverifiedStyle = vscode.window.createTextEditorDecorationType({ fontStyle: 'italic', textDecoration: 'underline dotted' });
    void client.start().then(() => markUnverified(vscode.window.activeTextEditor));
    context.subscriptions.push(
        output,
        verdicts,
        { dispose: () => client?.stop() },
        watchBinary(s.path),
        unverifiedStyle,
        vscode.window.onDidChangeActiveTextEditor(markUnverifiedSoon),
        vscode.workspace.onDidChangeTextDocument(e => {
            if (e.document === vscode.window.activeTextEditor?.document) {
                markUnverifiedSoon();
            }
        }),
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
            verdictLens.delete(doc.uri.toString());
        }),
        lensChanged,
        vscode.languages.registerCodeLensProvider({ language: 'surety' }, {
            onDidChangeCodeLenses: lensChanged.event,
            provideCodeLenses(doc) {
                const v = verdictLens.get(doc.uri.toString());
                if (!v) {
                    return [];
                }
                const range = doc.lineAt(signatureLine(doc)).range;
                return [new vscode.CodeLens(range, { title: v.title, tooltip: v.tooltip, command: 'surety.report' })];
            },
        }),
        vscode.commands.registerCommand('surety.report', () => output.show(true)),
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
    return {
        restarts: () => restarts,
        unverified: uri => unverifiedRanges.get(uri.toString()) ?? [],
    };
}

export function deactivate(): Thenable<void> | undefined {
    return client?.stop();
}
