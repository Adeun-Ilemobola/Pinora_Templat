import type {
    BunSerialOptions,
    SerialEventListener,
    SerialEventMap,
    SerialEventName,
    SerialOpenOptions,
    SerialPortInfo,
} from "./types";

type PendingRequest = {
    resolve: (value: any) => void;
    reject: (error: Error) => void;
};

export class BunSerial {
    private bridgePath: string;
    private nodePath: string;

    private child: any = null;

    private nextRequestId = 1;

    private pendingRequests = new Map<number, PendingRequest>();

    private listeners = new Map<
        SerialEventName,
        Set<(data: any) => void>
    >();

    constructor(options: BunSerialOptions) {
        this.bridgePath = options.bridgePath;
        this.nodePath = options.nodePath ?? "node";
    }

    private start() {
        if (this.child) {
            return;
        }

        this.child = Bun.spawn({
            cmd: [this.nodePath, this.bridgePath],

            stdin: "pipe",
            stdout: "pipe",

            // Let Node-side debugging appear normally
            // without contaminating our stdout IPC.
            stderr: "inherit",
        });

        console.log("Serial bridge PID:", this.child.pid);

        this.readOutput();
    }

    private async readOutput() {
        const reader = this.child.stdout.getReader();
        const decoder = new TextDecoder();

        let buffer = "";

        try {
            while (true) {
                const { value, done } = await reader.read();

                if (done) {
                    break;
                }

                buffer += decoder.decode(value, {
                    stream: true,
                });

                const lines = buffer.split("\n");

                buffer = lines.pop() ?? "";

                for (const line of lines) {
                    const text = line.trim();

                    if (!text) {
                        continue;
                    }

                    try {
                        const message = JSON.parse(text);

                        this.handleMessage(message);
                    } catch (error) {
                        console.error(
                            "Invalid message from serial bridge:",
                            text,
                            error,
                        );
                    }
                }
            }
        } catch (error) {
            this.emit(
                "error",
                error instanceof Error
                    ? error
                    : new Error(String(error)),
            );
        }
    }

    private handleMessage(message: any) {
        /*
         * Response to something WE requested.
         *
         * Example:
         *
         * {
         *   type: "response",
         *   id: 1,
         *   ok: true,
         *   result: {}
         * }
         */
        if (message.type === "response") {
            const pending = this.pendingRequests.get(message.id);

            if (!pending) {
                return;
            }

            this.pendingRequests.delete(message.id);

            if (message.ok) {
                pending.resolve(message.result);
            } else {
                pending.reject(
                    new Error(
                        message.error ?? "Serial bridge request failed",
                    ),
                );
            }

            return;
        }

        /*
         * Unsolicited events from the serial device.
         */
        if (message.type === "event") {
            switch (message.event) {
                case "opened":
                    this.emit("opened", {
                        path: message.path,
                    });
                    break;

                case "closed":
                    this.emit("closed", {
                        path: message.path,
                    });
                    break;

                case "error":
                    this.emit(
                        "error",
                        new Error(message.message),
                    );
                    break;

                case "data": {
                    const buffer = Buffer.from(
                        message.data,
                        "base64",
                    );

                    this.emit(
                        "data",
                        new Uint8Array(buffer),
                    );

                    break;
                }
            }
        }
    }

    private request<T = void>(
        message: Record<string, unknown>,
    ): Promise<T> {
        this.start();

        const id = this.nextRequestId++;

        return new Promise<T>((resolve, reject) => {
            this.pendingRequests.set(id, {
                resolve,
                reject,
            });

            const payload =
                JSON.stringify({
                    ...message,
                    id,
                }) + "\n";

            try {
                this.child.stdin.write(payload);
                this.child.stdin.flush();
            } catch (error) {
                this.pendingRequests.delete(id);

                reject(
                    error instanceof Error
                        ? error
                        : new Error(String(error)),
                );
            }
        });
    }

    async list(): Promise<SerialPortInfo[]> {
        const result = await this.request<{
            ports: SerialPortInfo[];
        }>({
            type: "list",
        });

        return result.ports;
    }

    async open(options: SerialOpenOptions) {
        await this.request({
            type: "open",
            path: options.path,
            baudRate: options.baudRate,
        });
    }

    async write(
        data: string | Uint8Array,
    ) {
        const bytes =
            typeof data === "string"
                ? Buffer.from(data, "utf8")
                : Buffer.from(data);

        await this.request({
            type: "write",
            data: bytes.toString("base64"),
        });
    }

    async close() {
        await this.request({
            type: "close",
        });
    }

    on<T extends SerialEventName>(
        event: T,
        listener: SerialEventListener<T>,
    ) {
        let eventListeners = this.listeners.get(event);

        if (!eventListeners) {
            eventListeners = new Set();

            this.listeners.set(
                event,
                eventListeners,
            );
        }

        eventListeners.add(listener as any);

        return () => {
            eventListeners?.delete(listener as any);
        };
    }

    private emit<T extends SerialEventName>(
        event: T,
        data: SerialEventMap[T],
    ) {
        const eventListeners =
            this.listeners.get(event);

        if (!eventListeners) {
            return;
        }

        for (const listener of eventListeners) {
            listener(data);
        }
    }

    async dispose() {
        try {
            await this.close();
        } catch {
            // Ignore close failure during shutdown
        }

        if (this.child) {
            this.child.kill();
            this.child = null;
        }
    }
}