import * as fs from 'fs';
import * as path from 'path';

import { ExtensionContext } from 'vscode';
import { LanguageClientOptions, LanguageClient, ServerOptions, Executable, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient;

export async function activate(_context: ExtensionContext) {
    const server = getServerPath();
    const run: Executable = {
        command: server,
        args: ["lsp"],
        transport: TransportKind.stdio,
    };

    const serverOptions: ServerOptions = {
        run,
        debug: run,
    }

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ "scheme": "file", "language": "zippy" }],
    };

    client = new LanguageClient("zippyc", serverOptions, clientOptions);
    await client.start();
}

export async function deactivate() { }

function getServerPath(): string {
    const baseDir = process.env['zc-dir'];
    if (baseDir !== undefined) {
        const exe = path.join(baseDir, "zc.exe");
        const bare = path.join(baseDir, "zc");

        if (fs.existsSync(exe)) {
            return exe;
        } else if (fs.existsSync(bare)) {
            return bare;
        }
    }

    return "zc";
}
