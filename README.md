# quick_action_hud

A headless, gamepad-driven radial (wheel) menu library for [Bevy](https://bevyengine.org/) 0.19.

**Headless** means the library handles all logic — input, hover detection, casting modes, time scaling, slot cycling — and emits events your app reacts to for rendering.  You own the visuals.

---

## Features

| Feature | Description |
|---|---|
| **Casting modes** | `Vanilla` (button press), `ReleaseToUse` (stick release), `HoldToActivate` (dwell), `Direct` (instant on hover) |
| **Time scaling** | `Normal`, `Slow(scale)`, or `Pause` virtual time while the wheel is open |
| **Multi-item slots** | Each slice holds a `Vec<ActionItem>`; player cycles with thumbstick buttons |
| **Action data model** | `ActionItem` enum (`Weapon`, `Spell`, `Consumable`, `Shout`, `Custom`) + `ActionBehavior` trait |
| **Wheel sets** | Cycle between multiple wheels with shoulder buttons |
| **Hold-to-activate** | Per-wheel dwell timer with progress events for UI feedback |
| **Low-count warnings** | Emit once when a slice's count drops below a threshold |
| **Edit mode** | D-pad reorder slices at runtime |
| **Lifecycle events** | `WheelOpened` / `WheelClosed` on hover transitions |
| **Conflict-free config** | Enum variants make invalid combinations impossible |

---

## Quick start

Add the dependency (local path or once published, crates.io):

```toml
[dependencies]
quick_action_hud = { path = "../bevy_wheel_menu" }
bevy = "0.19"
```

### Minimal example

```rust
use bevy::prelude::*;
use quick_action_hud::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WheelMenuPlugin)   // or ActionWheelPlugin
        .add_systems(Startup, spawn_wheel)
        .add_systems(Update, on_select)
        .run();
}

fn spawn_wheel(mut commands: Commands) {
    commands.spawn((
        WheelMenu { slices: 8, radius: 160.0, inner_radius: 45.0, ..default() },
        WheelState::default(),
        WheelMenuConfig {
            casting_mode: CastingMode::ReleaseToUse,
            ..default()
        },
    ));
}

fn on_select(mut events: MessageReader<WheelMenuSelected>) {
    for ev in events.read() {
        println!("Selected slice {}", ev.index);
    }
}
```

---

## Core types

### `WheelMenu`

The shape descriptor — attach to any entity.

```rust
WheelMenu {
    slices: 8,
    radius: 160.0,       // outer radius (px)
    inner_radius: 45.0,  // hole radius (px)
    deadzone: 0.25,      // stick deadzone (0–1)
    gap: 0.04,           // gap between slices (radians)
}
```

### `WheelMenuConfig`

All behaviour flags replaced by mutually-exclusive **enums** — no invalid combinations.

```rust
WheelMenuConfig {
    time_mode:            TimeMode::Slow(0.2),
    casting_mode:         CastingMode::HoldToActivate { duration: 0.8 },
    toggle_mode:          WheelToggleMode::Hold,
    auto_snap:            true,
    block_gameplay_input: false,
}
```

### `TimeMode`

```rust
TimeMode::Normal       // no time manipulation
TimeMode::Slow(0.15)   // slow Time<Virtual> to 15 %
TimeMode::Pause        // fully freeze virtual time
```

### `CastingMode`

```rust
CastingMode::Vanilla                         // South/A button confirms
CastingMode::ReleaseToUse                    // release stick to confirm
CastingMode::HoldToActivate { duration: 0.8} // dwell for N seconds
CastingMode::Direct                          // fire immediately on hover
```

### `WheelSlot` + `ActionItem`

Attach `WheelSlot` alongside `WheelSlice` to store multiple items per slice.
The player cycles through them with right/left thumbstick press.

```rust
commands.spawn((
    WheelSlice { index: 0 },
    WheelSlot::new(vec![
        ActionItem::Weapon { name: "Sword".into(), icon: "⚔️".into() },
        ActionItem::Weapon { name: "Bow".into(),   icon: "🏹".into() },
    ]),
));
```

Implement `ActionBehavior` for fully custom actions:

```rust
struct HealPotion;

impl ActionBehavior for HealPotion {
    fn execute(&self, commands: &mut Commands) { /* apply healing */ }
    fn label(&self) -> &str { "Heal Potion" }
    fn icon(&self)  -> &str { "🧪" }
}

ActionItem::Custom(Box::new(HealPotion))
```

---

## Events (messages)

| Event | When |
|---|---|
| `WheelMenuSelected { index, menu_entity }` | Slice confirmed (all modes) |
| `WheelMenuHoverChanged { previous, current, menu_entity }` | Hover changes |
| `WheelOpened { menu_entity }` | First slice hovered this session |
| `WheelClosed { menu_entity }` | Stick returns to centre |
| `SlotSelected { slot_index, menu_entity }` | Normalised selection signal |
| `ActionTriggered { slot_index, menu_entity }` | Prompt to call `ActionBehavior::execute` |
| `WheelSlotItemChanged { slot_index, previous_item, current_item, menu_entity }` | Slot item cycled |
| `WheelMenuHoldProgress { index, progress, menu_entity }` | Hold progress 0–1 each frame |
| `WheelMenuHoldActivated { index, menu_entity }` | Hold threshold reached |
| `WheelMenuLowCount { index, current, threshold, slice_entity }` | Count crossed low threshold |
| `WheelSwitched { previous, current, menu_entity }` | Active wheel in a set changed |
| `WheelEditModeChanged { active, menu_entity }` | Edit mode toggled |
| `WheelSliceReorder { from_index, to_index, menu_entity }` | Reorder requested |

---

## Optional components

| Component | Purpose |
|---|---|
| `WheelHoldState` | Required for `CastingMode::HoldToActivate`; tracks dwell progress |
| `WheelSliceCount { current, max, low_threshold }` | Drives low-count warnings |
| `WheelSet { count, prev_button, next_button }` | Multi-wheel cycling |
| `WheelEditMode { toggle_button }` | Runtime slice reorder |

---

## UI helpers (BSN)

The library ships three `bsn!`-authored scene builders for `bevy_ui`:

```rust
// Full-screen centered overlay — attach WheelMenu + WheelState here.
commands.spawn_scene(wheel_overlay()).insert((menu, WheelState::default(), config));

// Zero-size hub at screen center — parent slices to this.
let hub = commands.spawn_scene(wheel_hub()).id();

// Absolutely-positioned rounded panel for slice `i`.
let slice = commands.spawn_scene(wheel_slice_panel(&menu, i, 96.0, Color::srgba(0.1, 0.1, 0.1, 0.9))).id();
```

---

## Examples

```sh
# Diablo-style skill wheel (Release-to-Use, low-count warnings)
cargo run --example gamepad

# FPS weapon / ability wheel (slow-time, hold-to-activate, ammo tracking)
cargo run --example fps
```

---

## Bevy compatibility

| `quick_action_hud` | Bevy |
|---|---|
| `0.1` | `0.19` |

---

## License

Licensed under either of [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.

---

## Credits

| Asset | Author | License |
|---|---|---|
| Controller button icons (`assets/icons/`) | [Julio Cácko](https://juliocacko.itch.io/free-input-prompts) | [CC0](https://creativecommons.org/publicdomain/zero/1.0/) |
| Editor UI icons (`assets/icons/editor/`) | [CoreUI](https://github.com/coreui/coreui-icons) | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) |

---

## WebAssembly — GitHub Pages deployment

The repository includes everything needed to build the **FPS demo** (`examples/fps.rs`) for WebAssembly and publish it to **GitHub Pages** automatically.

### Try it in the browser

Once deployed, the demo is available at:

```
https://<owner>.github.io/bevy_quick_action_hud/
```

Replace `<owner>` with the GitHub organisation or user that owns the repository.

### Prerequisites (local development)

Install the required tools:

```sh
# Install the WebAssembly target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen CLI (version must match the wasm-bindgen dependency in Cargo.lock)
cargo install wasm-bindgen-cli --version 0.2.108
```

### Build the WASM release

This project uses the same approach as the official Bevy examples: `cargo build` for the WASM binary followed by `wasm-bindgen` to generate the JavaScript loader.

```sh
# 1. Build the example for the WASM target
cargo build --release --example fps --target wasm32-unknown-unknown

# 2. Generate JS bindings
wasm-bindgen \
  --out-dir dist \
  --out-name wasm_example \
  --target web \
  target/wasm32-unknown-unknown/release/examples/fps.wasm

# 3. Copy the HTML entry point
cp index.html dist/index.html
```

The output is written to the `dist/` directory:

```
dist/
├── index.html              # Entry point (imports wasm_example.js)
├── wasm_example.js         # JavaScript loader (wasm-bindgen)
└── wasm_example_bg.wasm    # WebAssembly binary
```

To test the build locally, serve from a **subdirectory** that matches the GitHub Pages URL:

```sh
# Copy files into a subdirectory and serve the parent
mkdir -p /tmp/test-site/bevy_quick_action_hud
cp -r dist/* /tmp/test-site/bevy_quick_action_hud/
cd /tmp/test-site && python3 -m http.server 8080

# Then visit http://localhost:8080/bevy_quick_action_hud/
```

> **Note:** The FPS example uses gamepad input. Make sure your browser supports the [Gamepad API](https://developer.mozilla.org/en-US/docs/Web/API/Gamepad_API) and connect a controller before pressing L2 to open the wheel.

### How deployment works

A **GitHub Actions workflow** (`.github/workflows/deploy.yml`) runs on every push to the `main` branch. It:

1. Checks out the repository
2. Installs the stable Rust toolchain with the `wasm32-unknown-unknown` target
3. Builds the `fps` example with `cargo build --release --target wasm32-unknown-unknown`
4. Generates JavaScript bindings with `wasm-bindgen` (using the version from `Cargo.lock`)
5. Copies `index.html` into the output directory
6. Runs the deployment smoke test to verify all files return HTTP 200
7. Uploads the `dist/` folder as a Pages artifact
8. Deploys to GitHub Pages using the official `actions/deploy-pages` action

The workflow can also be triggered manually from the **Actions** tab in your repository.

### One-time GitHub repository settings

After the first workflow run succeeds, you must configure the Pages source:

1. Go to your repository on GitHub
2. Navigate to **Settings** → **Pages**
3. Under **Source**, select **GitHub Actions** (not a branch)
4. No further configuration is needed — the workflow handles everything

> **Required permissions:** The workflow uses the default `GITHUB_TOKEN` which is automatically granted `write` access to the `pages` and `id-token` scopes for public repositories. If your repo is private, verify that **Actions → General → Workflow permissions** includes **Read and write permissions**.

### Verifying the deployment

1. Push to `main` (or trigger a manual workflow run from the Actions tab)
2. Wait for the workflow to complete (approx. 5–10 minutes for the first build)
3. Visit `https://<owner>.github.io/bevy_quick_action_hud/`
4. The Bevy application should load and render the FPS demo
5. Open the browser's developer console — there should be no JavaScript or WASM loading errors
6. All embedded assets (shaders, icons) load without 404 errors
