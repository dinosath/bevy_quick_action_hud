# Bevy 0.19 → 0.20 migration notes (verified against 0.20.0-rc.1)

This project was migrated from 0.19 to 0.20.0-rc.1. Every item below was either
hit while compiling this repository or checked in the local crate sources at
`~/.cargo/registry/src/*/bevy_*-0.20.0-rc.1/`. RC APIs can still change before
0.20.0 ships, so re-check this list when you move to the final release.

## Toolchain

- `bevy 0.20.0-rc.1` declares `rust-version = "1.96.0"`.
- Pin the prerelease exactly: `bevy = "0.20.0-rc.1"`. Plain `"0.20"` does not
  match a prerelease. If you pin `bevy_feathers` separately, pin it to the same
  prerelease.

## BSN syntax (`bevy_scene`)

| 0.19 | 0.20 | Notes |
| --- | --- | --- |
| `Children [ a, b ]`, `bsn_list![a, b]` | `Children [ a -- b ]`, `bsn_list! { a -- b }` | Entities in a scene list are separated by `--`. Components on the same entity are separated by whitespace. |
| `{ scene_expr }` as a scene entry | `@{ scene_expr }` | The old form gives `error: expected identifier`. |
| `my_scene(args)` include | `@my_scene(args)` | Paths also work: `@super::bsn::canvas()`. Without `@`, the compiler reports `Template`/`SceneEffect` not implemented for `impl Scene`. |
| `{ list }` inside a relationship | unchanged | Splice a `SceneList` with `Children [ #A -- {list} -- #B ]`. |
| `:scene` cached include | assets only (`:"file.bsn"`) | Using `:` on a function scene or `SceneComponent` is a compile error. |
| `.bsn` files | still not released | The loader hooks exist, but the format has not shipped. |

## UI interaction

- `bevy::ui::Interaction` and `bevy::ui::widget::Button` are deprecated aliases
  of **private** types. You cannot use them in system parameters, queries, or
  `bsn!`: the build fails with `type ... DeprecatedInteraction is private`.
  They also can't be used as values (`expected value, found type alias`).
  Migrate as follows:
  - **Drop-in state:** `bevy::picking::hover::PickingInteraction`
    (`Pressed` / `Hovered` / `None`). Picking updates it for any entity that has
    the component. Add it with `#[require(PickingInteraction)]` on your
    clickable marker, replacing what the old `Button` provided through
    `#[require(Interaction)]`.
  - **Split state (Feathers style):** `bevy::picking::hover::Hovered(bool)`,
    which is immutable, so react with change detection or observers. Pair it
    with the `bevy::ui::Pressed` marker, which `ui_widgets::Button` inserts on
    press.
  - **Button semantics:** `bevy::ui_widgets::Button` (accessibility role,
    `Activate`, `Pressed`). Import it explicitly, because the prelude `Button`
    still resolves to the deprecated alias.
  - The old `Button` also required `FocusPolicy::Block`. Picking blocks through
    `Pickable`, which UI nodes block by default. Audit overlapping hit targets
    after migrating.
- `UiWidgetsPlugins` is now part of `DefaultPlugins`. Do not add it (or
  `ButtonPlugin`) again, because that panics with a duplicate plugin.
- `Activate` is still a non-propagating `EntityEvent`. `ui_widgets::Button`
  still calls `propagate(false)` on pointer press, click, release, and cancel.

## Feathers

- `FeathersSliderProps` now has only `min` and `max`. Set the initial value
  by patching the component:
  `@FeathersSlider { @min: 0., @max: 10. } SliderValue({ value })`.
- `SceneComponent` plus `#[scene(Props)]` and `Box<dyn SceneList>` slot props
  are unchanged. `FeathersLazyMenu` shows a closure render prop:
  `Arc<dyn Fn() -> Box<dyn Scene> + Send + Sync>`.

## ECS

- Lifecycle observers use event patterns: `On<Add<T>>`, `On<Insert<T>>`, and
  `On<Remove<T>>`, where 0.19 used `On<Add, T>`.
- The `on_replace` component hook is renamed `on_discard`.
- `Resource: Component` still holds, so derive only `Resource`.
- Messages (`add_message`, `MessageReader`, `MessageWriter`) are unchanged, as
  far as this repository exercises them.

## Picking

- Pointer events are flat structs: `PointerOver`, `PointerOut`,
  `PointerEnter`, `PointerLeave`, `PointerPress`, `PointerRelease`,
  `PointerClick`, `PointerMove`, `PointerCancel`, `PointerDragStart`,
  `PointerDrag`, `PointerDragEnd`, `PointerDragEnter`, `PointerDragOver`,
  `PointerDragLeave`, `PointerDragDrop`, and `PointerScroll`.
  0.19 used `Pointer<Click>` and similar.

## Migration procedure

1. Bump `bevy` (and `bevy_feathers` if you pin it) in `Cargo.toml`. Confirm
   that every feature flag still exists in
   `bevy-0.20.0-rc.1/Cargo.toml` `[features]`.
2. Run `cargo check --all-targets --all-features`. Macro errors in `bsn!` hide
   type errors, so fix the syntax first (`@{}`, `@scene()`, `--`).
3. Replace `Interaction`/`Button` in queries, inserts, and scenes. Keep the
   interaction semantics identical, and write the old-to-new mapping in the
   change description.
4. Run the tests, run clippy, run each example, and exercise hover, press, and
   selection by hand. If CI builds WASM, check `wasm32-unknown-unknown` too.
5. Update README compatibility tables and the skills' version references.
