// // src/types/bun-serialport.d.ts
// declare module "bun-serialport" {
//     export interface SerialPortOptions {
//         path: string;
//         baudRate: number;
//         autoOpen?: boolean;
//     }

//     export interface PortInfo {
//         path: string;
//         manufacturer?: string;
//         serialNumber?: string;
//         vendorId?: string;
//         productId?: string;
//     }

//     export class SerialPort {
//         constructor(options: SerialPortOptions);

//         open(callback?: (error?: Error | null) => void): void;

//         close(callback?: (error?: Error | null) => void): void;

//         write(
//             data: string | Uint8Array,
//             callback?: (error?: Error | null) => void
//         ): boolean;

//         on(
//             event: "data",
//             listener: (data: Uint8Array) => void
//         ): this;

//         on(
//             event: "open" | "close",
//             listener: () => void
//         ): this;

//         on(
//             event: "error",
//             listener: (error: Error) => void
//         ): this;

//         static list(): Promise<PortInfo[]>;
//     }
// }