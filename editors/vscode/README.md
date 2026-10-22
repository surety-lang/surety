# Surety for VS Code

Obligations in Surety, in the editor: errors as you type (the same ones `surety parse` reports, held back in the clause you are typing until the cursor leaves it or the file is saved), names that resolve, and, with the second piece of slice 4, the verdict on save: each gap the mutants find is one warning saying where two passing outputs differ, with the input and both outputs on hover, and a quick fix that writes the input as a drafted example with both outputs between merge markers for you to choose; a clause no mutant can break gets a hint on its line.

The extension starts `surety lsp`. Set `surety.path` to the binary when it is not on your PATH, and `surety.graph` to a graph directory to see a sibling obligation's state on hover.

Build: `npm install && npm run compile`. Package: `npx @vscode/vsce package`. Install the VSIX from the command palette, "Extensions: Install from VSIX".
