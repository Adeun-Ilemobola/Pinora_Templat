// Device authentication status (audit Phase 2B). The backend runs the handshake and
// keeps the keys; this slice only mirrors `deviceAuth` and asks the backend to pair
// or unpair over USB serial. Results arrive as `deviceAuth` events.
import { invoke } from "@tauri-apps/api/core";
import type { AuthSlice, Slice } from "./types";

export const createAuthSlice: Slice<AuthSlice> = (set) => ({
  DeviceAuth: null,
  PairingError: null,
  PairDevice: async () => {
    try {
      set({ PairingError: null });
      await invoke("provision_device_key");
    } catch (error) {
      set({ PairingError: String(error) });
    }
  },
  UnpairDevice: async () => {
    try {
      set({ PairingError: null });
      await invoke("clear_device_key");
    } catch (error) {
      set({ PairingError: String(error) });
    }
  },
});
