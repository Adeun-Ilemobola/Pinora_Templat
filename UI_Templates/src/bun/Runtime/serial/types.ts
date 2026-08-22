export type SerialPortInfo = {
  path: string;
  manufacturer?: string;
  serialNumber?: string;
  vendorId?: string;
  productId?: string;
  locationId?: string;
  pnpId?: string;
};

export type SerialOpenOptions = {
  path: string;
  baudRate: number;
};

export type BunSerialOptions = {
  bridgePath: string;

  // Leave this as "node" for now.
  nodePath?: string;
};

export type SerialEventMap = {
  opened: {
    path: string;
  };

  data: Uint8Array;

  closed: {
    path?: string;
  };

  error: Error;
};

export type SerialEventName = keyof SerialEventMap;

export type SerialEventListener<T extends SerialEventName> = (
  data: SerialEventMap[T],
) => void;