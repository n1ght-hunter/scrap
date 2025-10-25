import * as path from 'path';
import * as fs from 'fs';
import { ExtensionContext, window } from 'vscode';
import {
    Executable,
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
} from 'vscode-languageclient/node';
import "./util";
import { registerCommands } from './command';
import { logger } from './util';

const resolveExe = (input: string) => {
    if (process.platform === 'win32') {
        return `${input}.exe`;
    }
    return input;
};

export let client: LanguageClient;

export function activate(context: ExtensionContext) {
    logger.setOutput(window.createOutputChannel("Scrap Extension", {
        log: true,
    }));
    // Create output channel for debugging
    logger.info('Scrap LSP extension is starting...');

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

        logger.info(`Looking for LSP server at: ${serverCommand}`);

        // Check if the server binary exists
        if (!fs.existsSync(serverCommand)) {
            const errorMsg = `Scrap LSP server binary not found at: ${serverCommand}\nPlease run 'cargo build' to compile the LSP server.`;
            logger.error(`ERROR: ${errorMsg}`);
            window.showErrorMessage(errorMsg);
            return;
        }

        logger.info('LSP server binary found, starting client...');

        const newEnv = { ...process.env };
        const run: Executable = {
            command: serverCommand,
            options: { env: newEnv },
        };

        const serverOptions: ServerOptions = {
            run,
            debug: run,
        };

        const clientOptions: LanguageClientOptions = {
            // Register the server for 'scrap' documents
            documentSelector: [{ scheme: 'file', language: 'scrap' }],
        };

        // Create and start the client.
        client = new LanguageClient(
            'scrapLsp',
            'Scrap Language Server',
            serverOptions,
            clientOptions,
        );

        logger.info('Starting Scrap Language Server client...');

        // Start the client and handle errors
        client.start().then(() => {
            logger.info('Scrap LSP client started successfully!');
        }).catch((error) => {
            logger.error(`Failed to start Scrap LSP client: ${error}`);
            window.showErrorMessage(`Failed to start Scrap LSP: ${error}`);
        });

        registerCommands(context);
    } catch (error) {
        const errorMsg = `Failed to activate Scrap LSP extension: ${error}`;
        logger.error(`ERROR: ${errorMsg}`);
        window.showErrorMessage(errorMsg);
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
