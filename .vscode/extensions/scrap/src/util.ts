import { LogOutputChannel } from "vscode";

export class Log {
    output?: LogOutputChannel;

    constructor() {
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    info(message: string, ...args: any[]) {
        this.output?.info(message, ...args);
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    error(message: string, ...args: any[]) {
        this.output?.error(message, ...args);
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    warn(message: string, ...args: any[]) {
        this.output?.warn(message, ...args);
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    debug(message: string, ...args: any[]) {
        this.output?.debug(message, ...args);
    }

    setOutput(output: LogOutputChannel) {
        this.output = output;
    }
}

export const logger = new Log();
