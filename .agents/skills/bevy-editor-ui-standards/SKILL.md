---
name: bevy-editor-ui-standards
description: Enforce Feathers Gallery-based architecture for all Bevy editor UI, including dedicated inspector windows, reusable property widgets, typed input capture, and event-driven live preview updates.
---

# Bevy Editor UI Standards

Apply this skill automatically whenever editor UI is created, modified, refactored, debugged, or reviewed. It covers radial-menu, button, wheelset, HUD-page, inspector, settings, dialog, toolbar, and future editor tooling.

Canonical reference: [Bevy Feathers Gallery](https://bevy.org/examples/ui-user-interface/feathers-gallery/). Verify the actual Bevy 0.19 API in this project before using a widget or component; do not invent a Feathers type from a newer release.

## Non-negotiable UX rules

- Do not create generic editor buttons that open undocumented dialogs.
- Do not use temporary popups, ad-hoc modals, or entity-specific one-off editors for editable data.
- Every editable asset/component gets a dedicated, named inspector window. Examples include `Radial Menu Editor`, `Button Editor`, `Wheelset Editor`, and `HUD Page Editor`.
- A settings control may select/open the corresponding dedicated inspector, but it must not be the inspector itself or hide undocumented configuration behind a generic popup.
- Keep one clear owner for the active inspector. Opening a new inspector closes the previous one; selection changes must not reopen unrelated windows.

## Standard editor window

Every inspector uses the same retained hierarchy:

```text
EditorWindow
├── TitleBar
│   ├── Title
│   └── CloseButton
├── ScrollView
│   └── PropertyGrid
│       └── PropertyRow(Label, EditorControl)
└── Footer (optional actions)
```

Use Feathers composition and theme tokens for the hierarchy, spacing, typography, focus, and interaction states. Use BSN scene fragments for stable structure where practical; keep runtime values, entity references, capture state, and hit testing in ECS systems.

Every property row follows the same two-column pattern:

```text
PropertyGrid
└── PropertyRow
    ├── Label
    └── Typed editor control
```

## Required reusable widgets

Create and reuse these project-owned widget/scene fragments before adding a new control:

- `EditorWindow`
- `PropertyGrid`
- `PropertyRow`
- `TextField`
- `InputCaptureField`
- `SliderField`
- `ToggleField`
- `DropdownField`
- `SectionHeader`

They may wrap Feathers widgets, but must preserve Feathers focus, theme, keyboard/gamepad activation, accessibility labels, and interaction semantics. Do not duplicate a control implementation inside a component-specific inspector.

Recommended feature layout:

```text
editor/
├── widgets/
│   ├── editor_window.rs
│   ├── property_grid.rs
│   ├── property_row.rs
│   ├── text_field.rs
│   ├── input_capture.rs
│   ├── slider_field.rs
│   ├── toggle_field.rs
│   └── dropdown_field.rs
└── inspectors/
    ├── radial_menu.rs
    ├── button.rs
    └── wheelset.rs
```

Keep `components.rs`, `events.rs`, `observers.rs`, `ui.rs`, and `plugin.rs` separate when a new editor feature warrants them. Avoid giant UI construction systems.

## Required inspectors

### Radial Menu Editor

When `RadialMenu`/`RadialMenuConfig` is editable, the dedicated window must expose:

- Name — `TextField`
- Thumbstick — `InputCaptureField` with `ThumbstickOnly`
- Cooldown — `SliderField`
- Labels — `ToggleField`
- Icons — `ToggleField`
- Inner Radius — `SliderField`
- Opacity — `SliderField`

Additional radial-menu properties belong in the same property grid, grouped with `SectionHeader`; they must not become a separate ad-hoc popup.

### Button Editor

When `QuickAction`/button configuration is editable, the dedicated window must expose:

- Name — `TextField`
- Button Binding — `InputCaptureField` with `ButtonOnly`
- Cooldown — `SliderField` when supported by the authored model
- Labels — `ToggleField` when supported by the authored model
- Icons — `ToggleField` when supported by the authored model
- Opacity — `SliderField`

If the current data model lacks a requested property, document that limitation and add the model field/event path before silently omitting the control. Do not invent a fake control that does not update data.

## Input capture

`InputCaptureField` is the single reusable binding-capture surface. It must provide:

- A clear listening/capture state
- Current binding display
- Clear binding action
- Invalid-device rejection and visible validation feedback
- Capture consumption so the captured input cannot also close the HUD, activate a button, or trigger a global shortcut

Supported modes:

- `ThumbstickOnly` — accept only analog thumbstick input; reject face buttons, triggers, keyboard, and mouse
- `ButtonOnly` — accept only gamepad buttons or the explicitly supported digital button devices
- `KeyboardOnly` — accept only keyboard input
- `AnyInput` — accept the project-supported input types

The capture field emits an event/message or editor action. It does not directly mutate gameplay or authored ECS data from widget construction code.

## ECS and data flow

Use this flow for edits:

```text
Widget interaction
    → Editor event/message or EditorAction
    → observer/action application system
    → authored component/resource update
    → live-preview refresh event/message
    → retained UI value/state update
```

Do not wire a widget directly to arbitrary gameplay mutation. Components hold entity state, resources hold genuinely global editor state, and transient selection/capture/window state remains separate from authored configuration.

Every editable value must support immediate preview refresh where meaningful, including opacity, inner radius, cooldown, labels, icons, name, and bindings. Prefer targeted messages/events and `Changed<T>`/`Added<T>` filters over rebuilding the entire UI tree every frame.

## Bevy 0.19 and Feathers practices

- Prefer Feathers widgets shown in the Gallery for buttons, toggles, sliders, text inputs, menus, and disclosure controls.
- Use `bsn!`, scene fragments, `Children[]`, and composition for stable editor hierarchy.
- Keep runtime capture, dynamic values, focus ownership, and entity-reference wiring in ECS systems.
- Use observers/events/messages for activation and edit application where appropriate.
- Use required components and focused feature plugins; avoid global mutable UI managers.
- Use theme tokens rather than scattered hardcoded colors, spacing, and typography.
- Give controls accessible labels and preserve keyboard/gamepad navigation.
- Do not recreate inspector entities for hover or ordinary value changes; update retained components in place.
- If a bespoke widget is unavoidable, document the missing Feathers capability and why a wrapper or stock widget is insufficient.

## Review behavior

Automatically flag:

- Generic “Settings” buttons with hidden or undocumented behavior
- Popup/dialog editors instead of dedicated inspector windows
- Direct widget-to-data mutation
- Duplicate text fields, sliders, toggles, or binding controls
- Hardcoded, inconsistent property layouts
- Missing close buttons, scroll views, property grids, or typed controls
- Thumbstick capture that accepts non-thumbstick input
- Captured input leaking into HUD close, activation, or global shortcuts
- Live preview values that require a full UI rebuild or do not update

Recommend the smallest incremental refactor toward:

- A dedicated named inspector window
- Shared `EditorWindow`/`PropertyGrid`/`PropertyRow` composition
- Feathers-first typed controls
- `InputCaptureField` with an explicit device mode
- Events/messages/observers and live preview refresh
- Feature-owned components, UI, systems, and plugin registration

## Generation behavior

When asked to create an editor, add settings, or edit radial menus/buttons, produce and implement:

1. The dedicated window hierarchy with title bar and close action.
2. The property-grid rows and typed reusable controls.
3. The component/event/message and observer/action path.
4. Input-capture filtering and consumption behavior.
5. Immediate preview refresh behavior.
6. Feature/plugin organization and tests appropriate to the change.

Never make a generic button the primary editing mechanism. If a requested requirement conflicts with the current model or Bevy/Feathers 0.19 API, state the limitation, preserve the window architecture, and implement the closest valid typed path with a documented follow-up.
