# Surety for VS Code

Obligations in Surety, in the editor: errors as you type (the same ones `surety parse` reports, held back in the clause you are typing until the cursor leaves it or the file is saved), names that resolve, and, with the second piece of slice 4, the verdict on save, as a line above the signature (a pass is never a squiggle; hover it for what was run, click it for the full report): each gap the mutants find is one warning saying where two passing outputs differ, with the input and both outputs on hover, and a quick fix that writes the input as a drafted example with both outputs between merge markers for you to choose; each finding opens with its label (Gap, Mutant gets through, Unneeded clause, faded like unused code, Input-only clause, and Untested clause), which is also its code; a call to a name no obligation declares is shown italic and dotted, unverified rather than wrong.

The extension starts `surety lsp`. Set `surety.path` to the binary when it is not on your PATH, and `surety.graph` to a graph directory to see a sibling obligation's state on hover.

Build: `npm install && npm run compile`. Package: `npx @vscode/vsce package`. Install the VSIX from the command palette, "Extensions: Install from VSIX".

**Formatting.** The server formats a file through `surety render`: sentences and comments wrap at 100 columns, every comment stays anchored to the line that follows it, and the obligation itself never changes (the hash is the test). Format on save is on by default for `.sur` files; a sentence or comment past the column shows as a hint until then.
