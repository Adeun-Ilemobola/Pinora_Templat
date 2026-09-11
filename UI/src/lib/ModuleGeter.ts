
 import { useModuleFront, type AnyModule } from './Modulefront';
 import type { StoreApi } from 'zustand/vanilla';
import { ModuleType } from './protocol/module-type';

 export function getModule<K extends ModuleType & AnyModule['kind']>(id: string, kind: K): StoreApi<Extract<AnyModule, { kind: K }>> {
     const entry = useModuleFront.getState().ModuleRegistry[id];
     if (!entry) throw new Error(`No module registered for id "${id}"`);
     if (entry.getState().kind !== kind) {
         throw new Error(`Module "${id}" is a ${entry.getState().kind}, not ${kind}`);
     }
     return entry as StoreApi<Extract<AnyModule, { kind: typeof kind }>>;
 }

export function getModuleByLookupId<
    K extends ModuleType & AnyModule["kind"]
>(
    lookUpId: string,
    kind: K
): StoreApi<Extract<AnyModule, { kind: K }>> | undefined {

    const state = useModuleFront.getState();

    const id = state.LookUpId[lookUpId];

    if (!id) {
        return undefined;
    }

    const entry = state.ModuleRegistry[id];

    if (!entry) {
        return undefined;
    }

    if (entry.getState().kind !== kind) {
        throw new Error(
            `Module "${id}" is a ${entry.getState().kind}, not ${kind}`
        );
    }

    return entry as StoreApi<
        Extract<AnyModule, { kind: K }>
    >;
}