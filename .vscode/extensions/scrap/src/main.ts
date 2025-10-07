import * as path from 'path';
import * as fs from 'fs';
import { workspace, ExtensionContext, window, LogOutputChannel } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
} from 'vscode-languageclient/node';

const resolveExe = (input: string) => {
    if (process.platform === 'win32') {
        return `${input}.exe`;
    }
    return input;
};

let client: LanguageClient;
let outputChannel: LogOutputChannel;

export function activate(context: ExtensionContext) {
    // Create output channel for debugging
    outputChannel = window.createOutputChannel('Scrap LSP', { log: true });
    outputChannel.info('Scrap LSP extension is starting...');

    try {
        // IMPORTANT:
        // The server is the compiled Rust binary.
        // You can compile it with `cargo build` and copy it to the extension's root,
        // or provide a relative path from this extension's root to the binary.
        // For development, a path like this works well.
        const serverCommand = context.asAbsolutePath(
            path.join('../../..', 'target', 'debug', resolveExe('scrap_lsp')), // Adjust this path!
        );
        // For a release, you'd bundle the binary at `server/scrap_lsp`.
        // const serverCommand = context.asAbsolutePath(path.join('server', 'scrap_lsp'));

        outputChannel.info(`Looking for LSP server at: ${serverCommand}`);

        // Check if the server binary exists
        if (!fs.existsSync(serverCommand)) {
            const errorMsg = `Scrap LSP server binary not found at: ${serverCommand}\nPlease run 'cargo build' to compile the LSP server.`;
            outputChannel.error(`ERROR: ${errorMsg}`);
            window.showErrorMessage(errorMsg);
            return;
        }

        outputChannel.info('LSP server binary found, starting client...');

        const serverOptions: ServerOptions = {
            run: { command: serverCommand, transport: TransportKind.stdio },
            debug: { command: serverCommand, transport: TransportKind.stdio },
        };

        const clientOptions: LanguageClientOptions = {
            // Register the server for 'scrap' documents
            documentSelector: [{ scheme: 'file', language: 'scrap' }],
            synchronize: {
                fileEvents: workspace.createFileSystemWatcher('**/*.sc'),
            },
            outputChannel: outputChannel,

        };

        // Create and start the client.
        client = new LanguageClient(
            'scrapLsp',
            'Scrap Language Server',
            serverOptions,
            clientOptions,
        );

        outputChannel.appendLine('Starting Scrap Language Server client...');

        // Start the client and handle errors
        client.start().then(() => {
            outputChannel.appendLine('Scrap LSP client started successfully!');
        }).catch((error) => {
            outputChannel.error(`Failed to start Scrap LSP client: ${error}`);
            window.showErrorMessage(`Failed to start Scrap LSP: ${error}`);
        });
    } catch (error) {
        const errorMsg = `Failed to activate Scrap LSP extension: ${error}`;
        outputChannel.error(`ERROR: ${errorMsg}`);
        window.showErrorMessage(errorMsg);
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
