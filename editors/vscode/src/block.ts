// The block a line belongs to, read from layout alone, so it holds for a
// file that does not parse: what a half-typed clause is. A top-level
// declaration is its line and, while a bracket it opens is unclosed, the
// lines inside; anything else is a clause, its head and the lines indented
// under it. Either takes the blank lines after it, up to the next block.
// No vscode import: the node test reads this module directly.

export interface Block {
    /** First and last line, 0-based, inclusive. */
    start: number;
    end: number;
}

const blank = (s: string) => s.trim() === '';

/** Leading whitespace in columns, a tab as four. */
function width(s: string): number {
    let w = 0;
    for (const c of s) {
        if (c === '\t') {
            w += 4;
        } else if (c === ' ') {
            w += 1;
        } else {
            break;
        }
    }
    return w;
}

/** The change in bracket depth across a line, strings, `//` comments and `#` sentences skipped. */
function depthChange(s: string): number {
    if (s.trim().startsWith('#')) {
        return 0;
    }
    let d = 0;
    let quoted = false;
    for (let i = 0; i < s.length; i++) {
        const c = s[i];
        if (quoted) {
            if (c === '\\') {
                i++;
            } else if (c === '"') {
                quoted = false;
            }
        } else if (c === '"') {
            quoted = true;
        } else if (c === '/' && s[i + 1] === '/') {
            break;
        } else if (c === '{' || c === '[' || c === '(') {
            d++;
        } else if (c === '}' || c === ']' || c === ')') {
            d--;
        }
    }
    return d;
}

const closes = (s: string) => /^\s*[}\])]/.test(s);

/** The last line of a top-level declaration: where its brackets close, or the line before the next declaration. */
function declarationEnd(lines: string[], top: number): number {
    let depth = 0;
    for (let k = top; k < lines.length; k++) {
        if (k > top && !blank(lines[k]) && width(lines[k]) === 0 && !closes(lines[k])) {
            return k - 1;
        }
        depth += depthChange(lines[k]);
        if (depth <= 0) {
            return k;
        }
    }
    return lines.length - 1;
}

/** The indent of a clause: the least non-zero indent in the file. */
function clauseWidth(lines: string[]): number {
    let least = Infinity;
    for (const l of lines) {
        const w = width(l);
        if (!blank(l) && w > 0 && w < least) {
            least = w;
        }
    }
    return least;
}

export function blockAt(lines: string[], line: number): Block {
    if (lines.length === 0) {
        return { start: line, end: line };
    }
    let h = Math.min(line, lines.length - 1);
    while (h > 0 && blank(lines[h])) {
        h--;
    }
    if (blank(lines[h])) {
        return { start: line, end: line };
    }
    let top = h;
    while (top > 0 && (blank(lines[top]) || width(lines[top]) > 0 || closes(lines[top]))) {
        top--;
    }
    if (!blank(lines[top]) && width(lines[top]) === 0) {
        let end = declarationEnd(lines, top);
        if (h <= end) {
            while (end + 1 < lines.length && blank(lines[end + 1])) {
                end++;
            }
            return { start: top, end: Math.max(end, line) };
        }
    }
    const clause = clauseWidth(lines);
    let head = h;
    while (head > top + 1 && width(lines[head]) > clause) {
        head--;
    }
    const w = width(lines[head]);
    let end = head;
    for (let k = head + 1; k < lines.length; k++) {
        if (!blank(lines[k]) && width(lines[k]) <= w) {
            break;
        }
        end = k;
    }
    return { start: head, end: Math.max(end, line) };
}
