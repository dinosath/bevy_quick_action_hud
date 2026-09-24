# Bevy architecture audit

Date: 2026-09-24  
Target: Bevy 0.19 / `bevy_feathers` 0.19  
Scope: `src/`, examples, tests, Cargo configuration, project skills and
development documentation.

This is the pre-refactor audit requested for the feature-first, BSN-first and
Feathers-first reorganization. It records the current architecture, evidence,
priority and safe migration boundaries. It does not pretend that all runtime
behavior can become a scene asset in Bevy 0.19.

## Executive assessment

Scores for the current codebase:

- Architecture: 6/10
- ECS usage: 7/10
- Performance: 5/10
- Maintainability: 4/10
- Bevy compliance: 8/10

The implementation is Bevy 0.19-compatible and already uses messages,
observers, direct component spawning, BSN scene fragments and Feathers scene
components. Its principal debt is concentration: `src/lib.rs` (6,332 lines)
and `src/editor.rs` (7,150 lines) combine data models, plugins, systems,
rendering, scene definitions, persistence and editor interaction.

### Current module inventory

| Module | Responsibility today | Finding |
| --- | --- | --- |
| `src/lib.rs` | Core wheel ECS, serialized config, HUD renderer, custom wedge material, HUD editor canvas, plugin setup, tests | God feature module; strongest extraction target |
| `src/editor.rs` | Sidebar BSN scenes, Feathers controls, selection model, capture, navigation, undo/redo, persistence, validation | God editor module; split by ownership after behavior stabilizes |
| `src/touch.rs` | Touch resource, touch messages, gesture systems/plugin | Good feature boundary; messages are appropriate |
| `src/wasm.rs` | Viewport, orientation, virtual keyboard, mobile/WASM plugin | Good platform boundary; sync browser state is polled because browser APIs require it |
| `examples/simple.rs` | Minimal host/demo | Static HUD log shell is BSN; runtime log text and input remain ECS-driven |
| `tests/wasm_deploy.rs` | Static deployment smoke test/server | Tooling test, not runtime feature code |

Networking is absent. Save/load is RON file persistence in the editor. Rendering
is Bevy UI plus a custom `UiMaterial`/WGSL wedge. Input uses Bevy gamepad/key
input, `leafwing-input-manager` for wheel navigation, and custom capture logic.

## Architecture audit

### Strengths

- `QuickActionHudPlugin::core`, default and `with_editor` provide a useful
  capability split.
- Touch and WASM support are isolated plugins rather than scattered platform
  conditionals.
- Core communication uses Bevy 0.19 messages for selection, lifecycle,
  actions, hold progress, wheel switching and editing.
- Editor activation/value changes use observers where Feathers emits the
  appropriate entity events.
- Components generally represent wheel/slot state; global configuration is a
  resource and serialized data is separated from rendering entities.
- Static UI helpers already use `bsn!`, `spawn_scene`, scene composition and
  Feathers controls.

### Findings and priorities

| Priority | Location | Finding | Why it matters | Safe direction |
| --- | --- | --- | --- | --- |
| P0 | `src/lib.rs`, `src/editor.rs` | UI visibility/selection state and structural rebuilds are coupled | A write or stale owner can invalidate hit targets and cause flicker | Guard writes; make rebuild reasons explicit; test owner transitions |
| P1 | `src/lib.rs:2772` | `WheelHudState` mixes HUD lifecycle, editor selection, hover, focus, flash timers and rebuild invalidation | God resource and hidden coupling between renderer/editor/input | Split into `HudRuntimeState`, `HudSelectionState`, and `EditorCanvasState` behind a compatibility migration |
| P1 | `src/editor.rs:1195` | Nearly all editor systems are one long `.chain()` | Correctness is explicit but parallelism and ownership are obscured | Introduce named `SystemSet`s and retain only true dependencies |
| P1 | `src/lib.rs:898` | Core, HUD, editor and platform setup are registered in one plugin implementation | Feature boundaries are conceptual, not module boundaries | Extract `CoreWheelPlugin`, `HudCanvasPlugin`, `EditorPlugin`, and platform plugins incrementally |
| P1 | `src/lib.rs`, `src/editor.rs` | Entire UI subtrees are rebuilt on dirty state | Acceptable for low-frequency document changes, but expensive and invalidates focus/hit targets | Keep structural rebuilds event-driven; update visual state in place |
| P2 | `src/lib.rs:5538` | Autoload performs synchronous filesystem IO in `PostStartup` | Blocks startup and is not a browser asset workflow | Add explicit persistence plugin/API; use async/platform adapter later |
| P2 | `examples/simple.rs`, `README.md` | Static example hierarchy and commands were stale/imperative | Confuses current architecture and BSN usage | Use a reusable BSN scene fragment and update docs to `simple` |
| P2 | `src/lib.rs` | Hand-authored editor palette colors coexist with Feathers theme tokens | Styling drift and duplicated theme ownership | Move common editor tokens into a theme resource; retain game HUD colors |
| P3 | `src/lib.rs` | `WheelHierarchy` stores entity vectors manually | Relationship data can become stale and couples persistence to runtime IDs | Use Bevy relationships or a runtime-only index where nested wheels are enabled |

## ECS and component organization

The following is the component/resource classification. It focuses on public
and architectural types rather than every marker used by a single editor row.

| Type | Current classification | Problems | Recommended structure |
| --- | --- | --- | --- |
| `WheelData` | Component + serialized wheel definition | Appropriate for an entity-owned wheel, but large | Keep as authored wheel data; split presentation-only fields if renderer needs independent updates |
| `WheelMenuConfig` | Component | Behavior config is correctly entity-local; name is broad | Keep; consider `WheelInteractionConfig` and `WheelTimingConfig` only if queries need independent change detection |
| `WheelState` | Component | Runtime direction/open/hover is focused | Keep; never serialize it |
| `WheelHoldState` | Component | Runtime hold state | Keep; activate only for hold casting |
| `WheelSet` | Component | Runtime active/count/input state; overlaps serialized `WheelSetData` conceptually | Keep runtime-only and explicitly map from config; avoid duplicating visual settings |
| `WheelSlot`, `WheelSlice`, `WheelSliceCount` | Components | Data and presentation ownership are split across runtime entities | Keep; use narrow marker/data components and avoid mirroring `WheelSlotData` unless a renderer requires it |
| `WheelStyle`, `WheelAudio` | Components | Authored presentation/audio values are attached to runtime entities | Keep when per-wheel; consider assets/handles for reusable skins and audio collections |
| `WheelHierarchy` | Component | Stores child entity IDs and may become stale | Runtime relationship/index; do not serialize entity IDs |
| `WheelInputOverride` | Component | Correct scoped override | Keep; use messages for binding changes rather than scanning all overrides |
| `ActiveSlotContext` | Component | Derived context duplicated from `WheelSliceLink` + `WheelState` | Keep only if downstream queries need fast local access; otherwise replace with a targeted query/message |
| `GlobalBindings` | Resource | Correct global fallback table | Keep as resource; persistence should use a separate settings DTO |
| `QuickActionConfig` | Resource | Correct global document, but large and combines authored config with editor document | Keep as document resource short term; split persistence DTO from transient document/editor state in a compatibility phase |
| `WheelHudState` | Resource | God resource: lifecycle, selection, hover, focus, flash and rebuild state | Highest-value ECS refactor; split by ownership and provide accessors/messages during migration |
| `EditorUiState` | Resource | Large editor navigation, capture, undo/redo and dirty state | Split into navigation/capture/history resources after system extraction |
| `WheelHudRoot`, `EditorRoot`, `EditorScrollArea` | Marker components | Valid ownership markers; not proliferation by themselves | Keep, but each must have one owner and cleanup path |
| `WedgeMaterial` | Asset | Correct custom render asset | Keep; procedural geometry is not a Feathers/BSN widget |
| `TouchConfig`, `ViewportInfo`, `VirtualKeyboardState` | Resources | Platform-global state | Keep in platform plugins; emit transition messages when consumers should react |
| `TouchTapEvent`, drag/long-press messages and wheel messages | Messages | Appropriate buffered communication | Keep; observers are better for entity-targeted widget activation, messages for cross-feature streams |

### Component smells found

- `WheelHudState` and `EditorUiState` are god resources, not because resources
  are wrong, but because unrelated ownership domains share invalidation and
  selection fields.
- `WheelHierarchy` is a runtime relationship representation inside a component;
  it must not leak into serialized data or outlive its referenced entities.
- Marker components are mostly legitimate ownership/query filters. Do not
  replace them with booleans in resources; remove only markers with no query,
  cleanup or ownership purpose.
- `ActionBehavior::execute(&mut Commands)` is an inversion point: it lets
  content behavior own structural commands. Prefer an action message/command
  emitted to the host application for large games, while preserving this API
  for compatibility.

## Plugin organization

### Current

`QuickActionHudPlugin` conditionally registers core wheel systems, HUD systems,
Feathers/editor systems, touch and WASM plugins. `WheelMenuPlugin` and
`WheelHudPlugin` are compatibility wrappers.

### Target feature graph

```text
QuickActionHudPlugin (compatibility facade)
├── WheelCorePlugin
│   ├── components/data
│   ├── input systems
│   └── messages and runtime systems
├── HudCanvasPlugin (optional)
│   ├── scenes
│   ├── wedge rendering
│   └── HUD interaction systems
├── EditorPlugin (optional)
│   ├── scenes / Feathers adapters
│   ├── selection and capture
│   ├── history and validation
│   └── persistence adapter
└── PlatformSupportPlugin
    ├── TouchInteractionPlugin
    └── WasmSupportPlugin / MobileSupportPlugin
```

The current facade should remain public. Extract modules behind it in slices;
do not duplicate public types or create circular re-exports.

## System and scheduling audit

Core systems are chained in a meaningful order because input updates state,
hover emits selection/lifecycle, hold/set/edit systems derive behavior, and
resolution emits actions. This is correct but too coarse. The editor chain is
also correctness-oriented but serializes unrelated validation/UI work.

Recommended sets:

```rust
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WheelCoreSet { Input, Derive, Emit, Resolve }

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum HudSet { Input, Interaction, Presentation, Rebuild }

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EditorSet { Capture, Navigation, Mutation, Presentation, Rebuild }
```

Use ordering only between sets with a documented data dependency. Keep input
capture before shortcuts; keep rebuild after mutation; keep presentation after
selection state changes. Do not make `validate_config` or independent focus
styling wait behind all mutation systems unless required.

### Polling-to-reactive opportunities

- `rebuild_hud` and `rebuild_editor` already have dirty gates; replace ad hoc
  dirty writes with typed `HudRebuildRequested` / `EditorRebuildRequested`
  messages when ownership is split.
- `hud_context_visibility` should update only when editor-open or selection
  state changes, and should guard writes to avoid generating unnecessary UI
  change detection.
- `update_log_window` correctly uses `Res<InputLog>::is_changed()` and is a
  good model for retained text updates.
- Browser viewport and gamepad state are legitimately sampled each frame, but
  should emit transition messages for expensive downstream work.

## BSN migration audit

### Fully migrated

- HUD layout primitives: overlay, hub, panels, discs/rings, labels/icons.
- Editor shell: sidebar/card/header/footer/field-row scene fragments.
- Feathers scrollbar hierarchy and generated thumb, with a runtime target and
  field-level BSN patch.

### Partially migrated

- Runtime wheel/segment composition: scene fragments are declarative; count,
  geometry, asset handles, hit markers and wedge material are runtime data.
- Editor forms: static shells are BSN; data-dependent labels, capture state,
  observers and focus entities are runtime-injected.
- `simple` log panel: static shell can be BSN; log text remains runtime-updated.
- The removed standalone gamepad/editor examples are now covered by the
  library API and `simple` host; avoid reintroducing redundant targets.

### Blocked by Bevy 0.19 limitations or runtime semantics

- Disk-backed `.bsn` asset authoring/loading is not a completed 0.19 workflow.
- Procedural annular geometry and WGSL materials.
- Per-frame input capture, hit testing, hold progress and selection state.
- Deferred entity references when the builder must append dynamic rows in the
  same command pass.

## Feathers audit

Already appropriate: buttons, tool buttons, checkboxes/toggles, themed text,
focus indicators and scrollbar. Remaining candidates: text input, number
input/slider, list view, nested tree rows, menus and radio groups.

Every candidate must pass focus, activation message, keyboard/gamepad,
accessibility, capture-consumption and theme tests before replacing a custom
control. Retain the radial HUD and specialized capture controls where Feathers
has no equivalent; document the exception in the migration report.

## Assets, persistence, networking and rendering

- Asset handles are loaded through `AssetServer` and embedded assets are
  registered by the plugin. A loading state/asset collection would be better
  for large applications, but adding one to a library with optional embedded
  icons is a compatibility decision, not an automatic refactor.
- `try_autoload_config` uses synchronous `std::fs` in `PostStartup`. Move this
  behind an explicit persistence feature/adapter before supporting large or
  browser-hosted documents.
- RON is suitable for current editor documents. Add schema versioning and
  atomic writes before treating it as production save data.
- No networking feature exists; no network plugin should be added speculatively.
- The wedge shader is a valid custom rendering boundary. Keep shader params
  small and avoid rebuilding materials when only UI state changes.

## Refactoring plan

### Phase 1 — safe correctness/performance (completed)

1. Guard UI component writes and make rebuild causes observable.
2. Extract static example UI into BSN.
3. Update README commands and development guidelines.
4. Add audit/report artifacts and skill routing.

The platform composition is now extracted to `src/platform.rs`; the RON
autoload adapter is isolated in `src/persistence.rs`; core wheel runtime
components are grouped in `src/radial_menu/components.rs`; and named
`WheelCoreSet`, `HudSet`, and `EditorSet` ownership seams live in
`src/scheduling.rs`. Existing local chains are intentionally retained while
their dependencies are being mapped.

The editor selection/focus model, editor ECS markers, and editor presentation
tokens now live in `src/editor/{state,components,theme}.rs`. HUD rendering state,
wedge material definitions, selection messages, and HUD presentation tokens now
live in `src/hud/{state,theme}.rs`; public re-exports preserve the existing
library API. These are ownership extractions only: runtime behavior remains in
the existing systems until their query and command dependencies are mapped.

Editor-only selected-component windows and radial settings/move/resize/delete
controls are implemented in `src/editor/components.rs`. The HUD module keeps
only forwarding seams for the existing canvas builder, avoiding a second public
interaction API while making editor presentation ownership explicit.

### Phase 2 — feature module extraction (in progress)

1. Move pure data types and messages to `core/components.rs` and
   `core/messages.rs` without changing public re-exports.
2. Move core systems into `core/systems/` and register named sets.
3. Move HUD scenes/rendering to `hud/scenes.rs` and `hud/rendering.rs`.
4. Split editor into `editor/scenes`, `editor/interaction`, `editor/history`,
   `editor/persistence`.

### Phase 3 — state/resource and retained UI improvements

1. Split `WheelHudState` and `EditorUiState` by ownership.
2. Replace dirty booleans with typed rebuild messages where useful.
3. Retain editor rows across value-only edits; rebuild only on structural
   document changes.
4. Migrate text/number/list controls to Feathers after behavior tests.

Each phase should compile and run the existing library tests before proceeding.
