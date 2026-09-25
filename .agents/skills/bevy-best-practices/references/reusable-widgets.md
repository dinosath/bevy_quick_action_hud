# Reusable widgets: props, slots, and consumer-injected affordances (Bevy 0.20)

Use this reference when a feature module owns a reusable UI widget (radial
menu, inventory grid, node graph, minimap) and a consumer (HUD, editor,
gameplay screen) needs to add identity, behavior, or extra controls such as
edit-mode buttons without the widget module knowing about the consumer.

## Verify the syntax against the pinned version first

BSN syntax changed between 0.19 and 0.20. Check `Cargo.lock`, then read
`~/.cargo/registry/src/*/bevy_scene-<version>/src/lib.rs`, which contains the
BSN syntax table. Examples on `bevyengine/bevy` `main` can already be past
the pinned release, so read them at the matching tag.

| Concept | Bevy 0.20.0-rc.1 (this repo, verified) | Bevy 0.19 (verified) |
| --- | --- | --- |
| Scene list separator | `Children [ a -- b ]`, `bsn_list! { a -- b }` | `Children [ a, b ]`, `bsn_list![a, b]` |
| Include a scene fn | `@my_scene(arg)` | `my_scene(arg)` |
| Insert a scene expression | `@{ expr }` | `{ expr }` |
| Cached include `:` | scene assets only | not implemented |
| Scene component with props | `@Widget { @prop: v, field: v }` | same |
| Lifecycle observer | `On<Add<T>>`, `On<Remove<T>>` | `On<Add, T>`, `On<Remove, T>` |
| Pointer events | `PointerClick`, `PointerOver`, `PointerPress` | `Pointer<Click>`, `Pointer<Over>`, `Pointer<Press>` |
| Discard hook | `on_discard` | `on_replace` |
| UI hover/press state | `PickingInteraction` or `Hovered` + `Pressed` | `Interaction` |
| Button marker | `bevy::ui_widgets::Button` (import explicitly) | `bevy::ui::Button` |
| Render-prop closure | `FeathersLazyMenu { popup: Arc<dyn Fn() -> Box<dyn Scene>> }` | plain Rust only |

See [migration-0.19-to-0.20.md](migration-0.19-to-0.20.md) for the upgrade
errors and fixes.

The official examples that show composition are
`examples/scene/bsn.rs`, `examples/ui/widgets/feathers_gallery.rs`,
`feathers_counter.rs`, `standard_widgets_observers.rs`,
`examples/ecs/observer_propagation.rs`, `relationships.rs`, and
`component_hooks.rs`. Read them at the matching release tag.

## Pattern 1: a `SceneComponent` with a props struct

Declare the widget the same way Feathers declares its controls
(`bevy_feathers-0.20.0-rc.1/src/controls/button.rs`):

```rust
#[derive(SceneComponent, Default, Clone)]
#[scene(RadialMenuProps)]
pub struct RadialMenuWidget;

pub struct RadialMenuProps {
    pub menu: RadialMenu,                       // authored data, already resolved
    pub highlighted: Option<usize>,
    pub sector_overlay: Box<dyn SceneList>,     // slot, spawned on the hub layer
}
impl Default for RadialMenuProps { /* at least MIN_SECTORS, empty overlay */ }

impl RadialMenuWidget {
    fn scene(props: RadialMenuProps) -> impl Scene { bsn! { /* ... */ } }
}
```

- Props are evaluated immediately and cannot be patched. Regular fields can be.
- The props type needs `Default`, and nothing else needs deriving.
- Components used inside `bsn!` need `Default + Clone` or `FromTemplate`. Use
  `FromTemplate` when a field is an `Entity` (so it can take a `#Name`
  reference) or a `Handle<T>`.
- A plain function returning `impl Scene` is enough when the widget has no
  marker component. Use a `SceneComponent` when consumers must query or
  observe the widget root.

## Pattern 2: slots for consumer content

These are ordered from most preferred to least:

1. **`Box<dyn SceneList>` prop.** Feathers uses this for `caption`
   (button, checkbox, radio) and `rows` (list view). The widget splices the
   slot where it belongs: `Children [ {props.caption} ]`. An empty slot is
   `Box::new(bsn_list! {})`.
2. **Caller-appended `Children [ ... ]`.** Relationship entries merge, so
   `@FeathersButton on(...) Children [ ... ]` adds siblings to the widget's own
   children. This works when the widget does not care where the children go.
3. **Per-item slot factory.** When each repeated item (sector, cell, row) needs
   its own slot content, pass a factory prop:
   `Arc<dyn Fn(SectorContext) -> Box<dyn SceneList> + Send + Sync>`. 0.20
   Feathers uses the same shape for `FeathersLazyMenu`. Pass geometry and index
   in the context so the consumer never has to recompute the widget's layout.
4. **Marker plus observer injection.** The consumer inserts a marker such as
   `EditableSector` or `RadialMenuEditing`. A global `On<Add<Sector>>` observer
   (or a `Changed<RadialMenuEditing>` system) then spawns the affordances as
   children. This keeps the widget scene unchanged, but the timing is deferred,
   so write a test for it.
5. **Declarative slot components (used by this repo's HUD).** The host scene
   only lists feature-owned slot markers, and each feature plugin fills its
   slot from an `On<Add<Slot>>` observer:

   ```rust
   // hud: composition only
   bsn! { WheelHudRoot Children [ @page::page(i) -- PageTabs -- EditMode ] }
   // page: a stack of component slots carrying identity
   bsn! { Page(i) Children [ PageBackground(i) -- RadialMenuSets(i) -- Buttons(i) -- PageSwitches(i) ] }

   #[derive(Component, Default, Clone, Copy)]
   #[require(Node = hud_layer(), Pickable = Pickable::IGNORE)]
   pub struct Buttons(pub usize);
   fn fill_buttons(add: On<Add<Buttons>>, slots: Query<&Buttons>, view: HudView, mut commands: Commands) { .. }
   pub(crate) fn plugin(app: &mut App) { app.add_observer(fill_buttons); }
   ```

   - Put identity (the page index) in the slot component, not in parent
     lookups. Scene spawn order between parent and child components is not a
     contract you should rely on.
   - Full-screen slot layers must use `Pickable::IGNORE`. UI nodes without
     `Pickable` block picking of the layers below them by default.
   - Bundle the shared read-only resources in one `#[derive(SystemParam)]`,
     such as `HudView`, so each observer signature stays short.
   - A headless test (`MinimalPlugins + AssetPlugin + ScenePlugin`, the slot
     plugins, and one `app.update()`) proves every slot is filled.
   - Hints and controls that must always be visible (for example the "Edit"
     toggle) belong to the host, not to a conditional child.
6. **Decorate after spawn with `Ready` (0.20, verified in this repo).** A
   widget returned as one `impl Scene` cannot hand back its inner entities,
   so it marks them instead (`SectorIndex(i)`, `RadialMenuHub`,
   `RadialMenuCenter`). The consumer patches identity and an entity observer
   onto the root:

   ```rust
   bsn! { @{radial_menu(&menu, highlighted)} HudRadialMenu(menu_ref) on(decorate_radial_menu) }
   fn decorate_radial_menu(ready: On<bevy::scene::Ready>, children: Query<&Children>, hubs: Query<(), With<RadialMenuHub>>, ..) {
       let hub = children.iter_descendants(ready.entity).find(|&e| hubs.contains(e));
   }
   ```

   - `Ready` fires per entity, children first, after the entity and all of its
     descendants have spawned. Anything spawned from the root's observer is
     appended after the widget's own children, so it draws on top.
   - Attach the observer with `on(...)` inside the scene. An `.observe()`
     queued after `spawn_scene` runs too late and misses `Ready`.
   - A single `impl Scene` is not a `SceneList`. Wrap it with
     `bevy::scene::EntityScene(scene)` before boxing it as `Box<dyn SceneList>`.
     `Option<L>` is a `SceneList`, which is handy for optional children.
   - Handlers resolve full identity from the item marker plus
     `iter_ancestors` to the root identity component. Don't copy the
     identity onto every item.

Avoid the "return entity handles and patch afterwards" shape. In that shape
the widget returns `struct SceneEntities { hub, center, sectors: Vec<Entity> }`,
and the consumer inserts identity, `Button`, or overlays into those entities.
It couples the consumer to the widget's internal hierarchy, it duplicates
widget state such as a cloned and re-themed config, and it breaks when the
widget restructures. Keep that shape only as a documented temporary boundary
while you migrate.

## Pattern 3: the widget exposes anchors, not raw geometry

If consumers place controls relative to parts of the widget (sector edges, the
inner arc, the outer arc, the hub), the widget module should export a typed
anchor API:

```rust
pub enum SectorAnchor { StartEdge, EndEdge, InnerArc, OuterArc, Center }
pub fn sector_anchor(menu: &RadialMenu, sector: usize, anchor: SectorAnchor) -> Vec2;
```

Consumers should never reimplement `slice_angles` or the midpoint math. Spawn
overlays on a non-rotated layer (the hub origin) at anchor positions. If you
parent them under a rotated sector panel with `UiTransform`, they rotate, get
clipped, or have to counter-rotate.

## Pattern 4: identity lives in one component, not in index tuples

Do not thread `(page, entry, menu, sector)` tuples through every builder and
action enum. Instead:

- Put the consumer identity on the widget root as one component, for example
  `HudRadialMenuRef { page, entry, menu }`.
- The widget puts its own local identity on each item (`SectorIndex(usize)`).
- A handler resolves the full identity by walking `ChildOf` ancestors, or
  through a custom relationship.
- Use a custom relationship to link overlay controls to their owner, instead
  of an owner enum that is recomputed from the action:

```rust
#[derive(Component)]
#[relationship(relationship_target = SectorAffordances)]
pub struct AffordanceOf(pub Entity);
#[derive(Component)]
#[relationship_target(relationship = AffordanceOf)]
pub struct SectorAffordances(Vec<Entity>);
```

## Pattern 5: event routing

- `Activate` (bevy_ui_widgets 0.19 and 0.20) is an `EntityEvent` **without**
  propagation. A root observer does not see child activations.
- `ui_widgets::Button` calls `propagate(false)` on pointer click, press, and
  release (`PointerClick`/`PointerPress`/`PointerRelease` in 0.20).
  Pointer events bubble through `ChildOf` only until a button consumes them.
- If a widget needs to report item events to its root or to a consumer,
  define its own bubbling event and re-emit it from the item observer:

```rust
#[derive(Clone, EntityEvent)]
#[entity_event(propagate, auto_propagate)]
pub struct SectorActivated { pub entity: Entity, pub sector: usize }
```

  The consumer observes it on the root with `on(|ev: On<SectorActivated>| ...)`.
  `ev.original_event_target()` returns the sector entity.
- Attach per-control behavior with `on(...)` inside `bsn!`. That replaces a
  giant `match` over a global action enum in a single
  `Changed<PickingInteraction>` system. If a shared command enum is still
  needed for undo or persistence, keep it data-only and dispatch it from the
  observers.

## Pattern 6: update in place; rebuild only on structural change

- Selection, hover, focus, and edit-mode visibility: toggle `Node::display`
  (`Display::None`/`Flex`), `Visibility`, `BackgroundColor`, or
  `InteractionDisabled` in place. Write only when the value differs, as
  Feathers does in `update_button_styles` and in the tree view's display
  toggle.
- Rebuild the widget subtree only when the authored document changes
  (sector count, config reload). Rebuild only the widget, never the whole HUD
  root.
- A highlighted-sector change should not despawn and respawn hundreds of
  geometry strips. Keep both the normal and highlighted panel variants, or
  patch the highlighted panel only.
- In 0.19 and 0.20, `despawn()` is already recursive through `ChildOf`
  (`linked_spawn`). Hand-written recursive despawn helpers are redundant.

## Pattern 7: dependency direction

The widget module must not read consumer resources such as HUD state,
editor state, or consumer action enums. Invert the dependency with one of:

- a component the consumer inserts (`RadialMenuInputBlocked`,
  `RadialMenuEditing`);
- a run condition the consumer supplies when it adds the widget systems;
- a system set the consumer orders against.

## Pattern 8: shared presentation is one struct, not two copies

When a group (for example a `RadialMenuSet`) shares presentation with its
members, extract the shared fields into one struct, such as
`RadialMenuPresentation`. Embed it in the member with `#[serde(flatten)]` for
file compatibility, and store it once on the group. Do not maintain a second
struct with 20+ mirrored fields and hand-written `From`/`apply_to` copies.
Resolve the effective presentation once, and pass it as a prop.

## Worked example: an editable radial menu (reference image)

Target behavior when edit mode is on and a sector is selected:

- A "+" at `StartEdge`, `EndEdge`, and `OuterArc` inserts a sector before,
  after, or outward.
- A trash icon at `InnerArc` deletes the sector. It is disabled when the menu
  is at `MIN_SECTORS`.
- The hub shows the selected sector's icon, its name with an edit pencil, and
  input hints ("Enter Apply", "Esc Back").
- A Feathers inspector window with "Sector settings" and "Sector control" is
  owned by the editor, not by the radial menu.

Composition (0.20 syntax sketch; confirm it compiles against the pinned version):

```rust
// radial_menu (knows nothing about HUD or editor)
bsn! {
    @RadialMenuWidget {
        @menu: resolved_menu,
        @highlighted: selected,
        @sector_overlay: overlay,    // Box<dyn SceneList>; empty outside edit mode
        @hub_footer: hints,          // Box<dyn SceneList>; hub slot
    }
}

// editor (consumer) builds the overlay from the widget's anchor API
fn sector_edit_overlay(menu: &RadialMenu, sector: usize) -> Box<dyn SceneList> {
    let at = |a| sector_anchor(menu, sector, a);
    Box::new(bsn_list! {
        @edit_badge(at(SectorAnchor::StartEdge), icons::PLUS) InsertSector(Side::Before)
        --
        @edit_badge(at(SectorAnchor::EndEdge), icons::PLUS) InsertSector(Side::After)
        --
        @edit_badge(at(SectorAnchor::OuterArc), icons::PLUS) InsertSector(Side::Outer)
        --
        @edit_badge(at(SectorAnchor::InnerArc), icons::TRASH) DeleteSector
    })
}
// A `Changed<RadialMenu>` system inserts/removes `InteractionDisabled` on
// `DeleteSector` when the sector count reaches `MIN_SECTORS`.
```

Each `edit_badge` is a small scene function wrapping `@FeathersButton` or a
bespoke round badge, with its own `on(|_: On<Activate>, ...|)`. It reads the
owning menu and sector from ancestors or from `AffordanceOf`, not from
captured index tuples.

## Review smells for widget composition

- An adapter function has more than 6 parameters, several of them
  `Option<(usize, usize, Option<usize>, usize)>`.
- The consumer re-clones widget config and reapplies theme or visuals the
  widget has already applied.
- Thin forwarding wrappers exist only to call another module's function
  with the same arguments.
- Geometry math is duplicated in the consumer to position overlays.
- Focus is encoded as magic integers (`edit_control_focus == Some(3)`).
  Use a `Focused` marker or `InputFocus` on the entity.
- A widget module imports consumer resources, such as `Option<Res<HudState>>`
  inside a core radial-menu system.
- The same helper is copied into both modules (`parse_hex_color`), or color
  constants are stored as `[f32; 4]` and converted at every use. Use
  `const Color = Color::srgba(..)`, which is a `const fn` in 0.19 and 0.20.
- A single "component" file of 1000+ lines mixes authored data, runtime
  components, constants, geometry, scene functions, and tests. Split it into
  `model`, `geometry`, `style`, `widget`/`scene`, and `systems`.
