# Bevy development guidelines

These are the project defaults for new code and refactors.

## Architecture

- Organize by feature ownership: core wheel logic, HUD canvas, editor, input,
  platform support, rendering and persistence.
- A feature owns its components, resources, messages, observers, systems,
  scenes and plugin registration.
- Keep `QuickActionHudPlugin` as a compatibility facade, but do not add new
  unrelated behavior to its implementation.
- Components store entity state only. Global application state belongs in
  resources; transient communication belongs in messages/events or observers.
- Prefer small focused components and composition. Avoid god resources,
  service-locator components, mirrored resource/component state and marker
  proliferation without a query/ownership purpose.

## Systems and communication

- Give every system one primary responsibility.
- Prefer messages and entity observers for reactive workflows. Poll only when
  the input source is inherently sampled (axes, browser viewport, timers).
- Use `Changed<T>`, `Added<T>`, `RemovedComponents<T>` and narrow queries to
  avoid scanning or rewriting unrelated entities.
- Use named `SystemSet`s for feature phases. Document real ordering constraints;
  do not chain an entire feature merely for convenience.
- Input capture must run before global shortcuts and consume the triggering
  input in the same update.
- UI hover/focus changes update existing components. Structural rebuilds are
  reserved for explicit document/layout changes.

## BSN-first UI and scenes

- Prefer `bsn!`, reusable scene functions, `SceneComponent`, templates,
  `Children[]`, named references, scene lists and field-level patches for
  declarative hierarchy.
- Do not mechanically wrap imperative spawn calls in BSN.
- Keep runtime geometry, input/hit testing, asset readiness and entity wiring
  in focused ECS systems when values are not known at scene resolution.
- In Bevy 0.19, Rust-authored BSN is available; disk-backed `.bsn` authoring
  must not be assumed without verifying the current release.

## Feathers-first editor UI

- Use Feathers buttons, tool buttons, toggles, inputs, lists, scrollbars,
  menus, focus indicators and theme tokens before creating custom controls.
- A custom widget requires a documented reason: missing capability,
  specialized workflow, or measured performance requirement.
- Preserve value-change messages, focus, keyboard/gamepad navigation,
  accessibility and rebinding/capture semantics during migration.
- Do not replace a Feathers `Node` wholesale when applying a style; use BSN
  field patches or the widget's theme system.

## Data, assets and persistence

- Keep authored configuration serializable and separate from transient ECS
  presentation state.
- Use `AssetServer` handles and explicit loading/error policy. Avoid blocking
  filesystem work in frame systems.
- Version persisted documents before making schema changes; prefer atomic
  writes for user data.
- Do not add networking or ecosystem plugins without a demonstrated ownership
  and compatibility need.

## Verification

Before handoff, run `cargo fmt --all`, `cargo check`, relevant example checks,
`cargo test --lib`, and `git diff --check`. For UI changes, add a behavioral
test or a targeted debug trace for selection ownership, visibility transitions,
input capture and rebuild causes.
