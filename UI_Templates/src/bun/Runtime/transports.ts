


type TransportsModeType = "Serial" | "Wifi" | "Bluetooth";

export class Transports {
    private Serial: any = null
    private Wifi: any = null
    private Bluetooth: any = null

    constructor(private mode: TransportsModeType = "Serial") {
        this.Init()
    }
    private Init() {
        switch (this.mode) {
            case "Serial": {
                return
            }

            case "Wifi": {
                return
            }

            case "Bluetooth": {
                return
            }
            default:
                break;
        }

    }
    public GetMode(): TransportsModeType {
        return this.mode
    }

    public async send(data: Uint8Array) {

    }

    private async connect() {

    }
    public async disconnect() {

    }
    //Serial
    public async switchToSerial() {

    }
    //Wifi
    public async switchToWifi() {

    }
    //Bluetooth

    public async switchToBluetooth() {

    }
}