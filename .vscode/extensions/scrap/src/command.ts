import { commands, ExtensionContext, ProgressLocation, window } from "vscode";
import { client } from "./main";

export function registerCommands(context: ExtensionContext) {
    // Register restart command
    const restartCommand = commands.registerCommand('scrap.restart', async () => {
        await restartServer();
    });
    context.subscriptions.push(restartCommand);

    // Register stop command
    const stopCommand = commands.registerCommand('scrap.stop', async () => {
        await stopServer();
    });
    context.subscriptions.push(stopCommand);

    // Register start command
    const startCommand = commands.registerCommand('scrap.start', async () => {
        await startServer();
    });
    context.subscriptions.push(startCommand);
}

async function stopServer(): Promise<void> {
    if (!client) {
        window.showWarningMessage('Scrap Language Server is not running.');
        return;
    }

    await window.withProgress({
        location: ProgressLocation.Notification,
        title: "Scrap Language Server",
        cancellable: false,
    }, async (progress) => {
        try {
            progress.report({ increment: 0, message: "Stopping server..." });
            console.info('Stopping Scrap Language Server...');

            await client.stop();

            progress.report({ increment: 100, message: "Server stopped successfully!" });
            console.info('Scrap Language Server stopped successfully!');
            window.showInformationMessage('Scrap Language Server stopped.');

            // Brief delay to show completion message
            await new Promise((resolve) => setTimeout(resolve, 500));
        } catch (error) {
            const errorMsg = `Failed to stop Scrap Language Server: ${error}`;
            console.error(`ERROR: ${errorMsg}`);
            window.showErrorMessage(errorMsg);
        }
    });
}

async function startServer(): Promise<void> {
    if (client && client.state === 2) { // State.Running
        window.showWarningMessage('Scrap Language Server is already running.');
        return;
    }

    await window.withProgress({
        location: ProgressLocation.Notification,
        title: "Scrap Language Server",
        cancellable: false,
    }, async (progress) => {
        try {
            progress.report({ increment: 0, message: "Starting server..." });
            console.info('Starting Scrap Language Server...');

            if (!client) {
                window.showErrorMessage('Language client not initialized. Please reload the extension.');
                return;
            }

            await client.start();

            progress.report({ increment: 100, message: "Server started successfully!" });
            console.info('Scrap Language Server started successfully!');
            window.showInformationMessage('Scrap Language Server started.');

            // Brief delay to show completion message
            await new Promise((resolve) => setTimeout(resolve, 500));
        } catch (error) {
            const errorMsg = `Failed to start Scrap Language Server: ${error}`;
            console.error(`ERROR: ${errorMsg}`);
            window.showErrorMessage(errorMsg);
        }
    });
}

async function restartServer(): Promise<void> {
    if (!client) {
        window.showWarningMessage('Scrap Language Server is not running.');
        return;
    }

    await window.withProgress({
        location: ProgressLocation.Notification,
        title: "Scrap Language Server",
        cancellable: false,
    }, async (progress) => {
        try {
            progress.report({ increment: 0, message: "Stopping server..." });
            console.info('Restarting Scrap Language Server...');

            progress.report({ increment: 50, message: "Restarting server..." });
            await client.restart();

            progress.report({ increment: 100, message: "Server restarted successfully!" });
            console.info('Scrap Language Server restarted successfully!');

            // Brief delay to show completion message
            await new Promise((resolve) => setTimeout(resolve, 500));
        } catch (error) {
            const errorMsg = `Failed to restart Scrap Language Server: ${error}`;
            console.error(`ERROR: ${errorMsg}`);
            window.showErrorMessage(errorMsg);
        }
    });
}
