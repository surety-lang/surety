// The extension-host run removes the workspace it prepares, whether the
// suite passes or fails, so test runs leave nothing in the temp
// directory. Runs under node, VS Code replaced by a stand-in.

import * as assert from 'assert';
import * as fs from 'fs';
import { TestOptions } from '@vscode/test-electron';
import { main } from './runTest';

async function workspaceAfter(outcome: 'passes' | 'fails'): Promise<string> {
    let workspace = '';
    const run = async (options: TestOptions): Promise<number> => {
        workspace = options.launchArgs![0];
        assert.ok(fs.existsSync(`${workspace}/typing.sur`), 'the workspace is prepared before the run');
        if (outcome === 'fails') {
            throw new Error('the suite failed');
        }
        return 0;
    };
    await main(run).then(
        () => assert.strictEqual(outcome, 'passes'),
        e => assert.strictEqual(e.message, 'the suite failed'),
    );
    return workspace;
}

async function suite(): Promise<void> {
    // Any file will do for the binary: the stand-in never starts it.
    process.env.SURETY_BIN = process.execPath;
    for (const outcome of ['passes', 'fails'] as const) {
        const workspace = await workspaceAfter(outcome);
        assert.ok(workspace, 'the run was handed a workspace');
        assert.ok(!fs.existsSync(workspace), `a run that ${outcome} leaves ${workspace} behind`);
    }
    console.log('workspace: 2 passed');
}

suite().catch(e => {
    console.error(e);
    process.exit(1);
});
