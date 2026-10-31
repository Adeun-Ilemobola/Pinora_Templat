# Module boundaries

How a Pinora "module" is defined, registered, addressed and rendered today, and where the boundaries leak.

## 1. The firmware module contract

`Firmware_Templates/src/core/module.rs`

```rust
pub struct ModuleCore { pub id: String /* UUID v4, new each boot */, pub module_type: ModuleType,
                        pub lookup_name: String /* sent as lool_up_id */, pub parent_id: String /* "" if none */,
                        emitter: TransportEmitter }

pub trait Module {
    fn core(&self) -> &ModuleCore;
    fn tick(&mut self) -> Result<(), ModuleError>;                         // called every loop iteration
    fn handle_command(&mut self, command: &ModuleCommand) -> anyhow::Result<()>;
    fn register(&self) -> Result<(), EmitterError>   { self.emit_registration() } // reliable
    fn first_emit(&mut self) -> Result<(), EmitterError> { Ok(()) }        // once after register
    fn emit(&self, event: ModuleEvent)   { /* try_emit EventPackage{id, event} */ }  // lossy
    fn log(&self, data: SysLogEvent)     { /* emit(SysLog) */ }
}
```

**Lifecycle.**
1. Construction in `main.rs`. This is where hardware is acquired, and actuators currently **move here** (PIN-TPL-004).
2. Insertion into `HashMap<String, Box<dyn Module>>` keyed by UUID.
3. `register()` then `first_emit()` for each module, in HashMap order.
4. `tick()` forever.
5. `handle_command()` when a command's `id` matches.

There is no shutdown, suspend, reset or safe-state hook. Modules are dropped only if `main` returns.

**Each module receives `ModuleCommand` (the enum of all module families)** and must ignore other families with a `_ => {}` arm. Type safety between a module and its command family is enforced only by convention.

## 2. Ownership and hardware boundaries

| Resource | Owner | Sharing mechanism | Notes |
|---|---|---|---|
| `Peripherals` | `main` | Moved into drivers | `p.uart0` is not moved, but UART0 is driven via raw `sys` (PIN-TPL-026) |
| LEDC timer0 | `HardwareContext.led_timer` | `&LedTimer` borrowed by each `LedcDriver::new` | Shared by all LEDs: 5 kHz, 13-bit |
| LEDC channels 0–2, GPIO 12/14/27 | each `Ledmodule` | exclusive | GPIO12 is a strapping pin (PIN-TPL-018) |
| I²C0 (GPIO21 SDA / GPIO22 SCL, 100 kHz) | `Rc<RefCell<I2cDriver>>` in `HardwareContext.i2c_bus` | `RcDevice` per consumer | No active consumer in the current image |
| PCA9685 | would be `Rc<RefCell<Pca9685<RcDevice>>>` | cloned into servos | Creation commented out |
| Modem | `TransportCore` → `Bluetooth`/`Wifi` | exclusive | One transport per image |
| NVS default partition | cloned into transport | ESP-IDF handle | Used for Bluetooth bonding / Wi-Fi config |

Modules and buses are deliberately non-`Send` (`Rc`), which keeps all hardware access on the main task. That is a good invariant, and Phase 0 must document it rather than change it.

## 3. Registration and identity

| Concept | Firmware | Wire | Desktop |
|---|---|---|---|
| Identity | `ModuleCore.id` (UUID v4 generated at construction) | `Registration.id`, `EventPackage.id`, `IncomingCommand.id` | `ModuleRegistry[id]` |
| Human name | `lookup_name` (caller string, e.g. `"led1"`) | `Registration.lool_up_id` | `LookUpId[name] = id` (last writer wins) |
| Type | `ModuleType` enum | `Registration.module_type` | factory map `{Led, Button}`; others ignored |
| Parent | `parent_id: String` (`""` when none) | `Registration.parent_id` | `hasParent: boolean` only |

Identity is **not stable across reboots**. The desktop must rediscover after every reset, and it relies on replay or on the reset itself (PIN-TPL-012).

## 4. Composite modules

LiDAR owns `servo_x`, `servo_y` and `rangefinder` by value. Its `register()` also registers the children, so they appear on the desktop with `parent_id = lidar.id`. The main registry contains only the LiDAR, so a command addressed to a child UUID is rejected. Children are controlled only through `LidarCommandPayload::ChangeMotorAngle { id, step }`. The joystick follows the same pattern with an embedded button. This is the boundary gap recorded as PIN-TPL-017.

## 5. Desktop module boundary

For each family, a file in `UI/src/lib/Modules/` contains:
1. Zod schemas for the family's events and commands, duplicating Rust.
2. A `create<Family>(hasParent, id, lookUpId)` vanilla Zustand store with command actions that call `IncomingCommand` → `invoke("send_data")`, plus `handleEvent`.
3. React views (`<Family>View` by lookup name, `Registered<Family>View` by UUID).

`useModuleFront` dispatches `ModuleEvent`s to a module store only when the event's top-level key equals the store's `kind`. Only `Led` and `Button` have dispatch branches. All other families are dropped silently, apart from a console line.

## 6. What it takes to add a module today

1. Add the Rust payload types to `protocol/src/module/<x>.rs` and to the `ModuleCommand`/`ModuleEvent`/`ModuleType` enums.
2. Implement `Module` in `Firmware_Templates/src/modules/<group>/<x>.rs` and declare it in that group's `mod.rs`.
3. Construct and insert it in `board/mod.rs` (and pass its peripherals from `main.rs`), choosing pins by hand.
4. Write the Zod schema, store and view in `UI/src/lib/Modules/<x>/{schema.ts,store.ts,view.tsx}` and re-export them from `index.ts`.
5. Add the factory to `RegistrationEvent` and the dispatch branch to `ModuleEvent` in `Modulefront.ts`.
6. Add the case to `Dashboard.tsx`'s kind filter and switch.
7. Optionally add the family to the aggregate schemas in `UI/src/lib/protocol/*.ts`.

That is seven locations in three languages and contexts, with no generation and no compile-time link between them. It is the main long-term expansion risk (PIN-TPL-007, PIN-TPL-020).

## 7. Boundaries that Phase 0 should make explicit (without changing behavior)

- Separate *board wiring* (`main.rs` construction) from the *runtime loop*. Splitting files is behavior-neutral if the code is moved verbatim.
- Group firmware modules by role: actuators, sensors, composite.
- Make the transport layer's internal pieces visible: emitter, sender thread, per-transport worker, discovery cache.
- Split each frontend module file into schema / store / view.
- Document the non-`Send` hardware invariant, the reliable/lossy emission rule, and the module lifecycle in doc comments.
