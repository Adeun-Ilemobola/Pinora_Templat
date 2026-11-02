// useModuleFront: the desktop's central store, composed from slices (audit Phase 2D,
// PIN-TPL-020; before, one 500-line object did everything):
//
// * store/transport.ts — connection lifecycle and Tauri listeners (the transport
//   service; the only slice that talks to the backend besides pairing).
// * store/registry.ts — modules, lookup names and the system snapshot.
// * store/logs.ts — the bounded, batched protocol log history (PIN-TPL-011).
// * store/auth.ts — device authentication status and USB pairing (Phase 2B).
//
// Field and action names are unchanged, so components did not change; the behavior
// is pinned by UI/tests/store.test.ts.
import { create } from "zustand";

import { createAuthSlice } from "./store/auth";
import { createLogSlice } from "./store/logs";
import { createRegistrySlice } from "./store/registry";
import { createTransportSlice } from "./store/transport";
import type { StoreState } from "./store/types";

export type { AnyModule } from "./store/types";
export { registrationHasParent } from "./registry/registration";

export const useModuleFront = create<StoreState>()((...args) => ({
  ...createTransportSlice(...args),
  ...createRegistrySlice(...args),
  ...createLogSlice(...args),
  ...createAuthSlice(...args),
}));
