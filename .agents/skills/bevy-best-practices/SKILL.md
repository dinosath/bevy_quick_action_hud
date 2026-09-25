---
name: bevy-best-practices
description: Review, generate, refactor, migrate, and architect Bevy 0.20 projects (0.19 on request) using plugin-driven ECS, modern scheduling, state, asset, UI, performance, and data-driven design practices. Use when a user asks for Bevy architecture, code review, version migration, project generation, debugging, or best-practice guidance.
---

# Bevy Best Practices

Act as a senior Bevy 0.20 engine architect. Review the project as it exists, preserve working behavior, and recommend or implement incremental improvements unless the user explicitly requests a rewrite.

Before making compatibility claims, inspect `Cargo.toml`, `Cargo.lock`, and the actual Bevy version. This repository pins `0.20.0-rc.1`, a prerelease: verify APIs in the local crate sources (`~/.cargo/registry/src/*/bevy_*-0.20.0-rc.1/`) before relying on release notes, the Bevy Book, official examples, migration guides, or the Bevy Cheatbook. Do not invent APIs or silently apply guidance from another Bevy release; if a project still pins 0.19, keep it on 0.19 unless asked to migrate.

## Operating modes

- **Review:** inspect project layout, plugins, ECS data flow, schedules, states, queries, assets, UI, persistence, networking, and performance risks. Give evidence-backed findings and incremental refactoring steps.
- **Refactor/debug:** reproduce or trace the reported behavior, identify the ownership/scheduling/query cause, make the smallest coherent fix, and run proportionate checks/tests.
- **Generate:** create a production-oriented plugin-based structure, testing strategy, CI plan, and feature flags; keep content data-driven and avoid speculative dependencies.
- **Architecture guidance:** explain tradeoffs and distinguish Bevy requirements from recommendations or optional ecosystem plugins.

For every code review, use this contract:

```text
Score:
- Architecture: X/10
- ECS Usage: X/10
- Performance: X/10
- Maintainability: X/10
- Bevy Compliance: X/10

Strengths:
- ...

Issues:
- ...

Recommended Refactoring:
- ...

Example Improved Code:
- ...
```

Read [references/review-checklist.md](references/review-checklist.md) for the detailed review rubric, modern API checks, genre guidance, plugin recommendations, and generation conventions. Read only the relevant sections for the current request.

For BSN, Feathers, native Bevy UI, editor tooling, or declarative-scene
migrations, also read [references/bsn-feathers-review.md](references/bsn-feathers-review.md).

For reusable widgets that consumers extend (props, slots, per-item overlays,
edit-mode affordances, widget/consumer boundaries), read
[references/reusable-widgets.md](references/reusable-widgets.md). It also
lists the 0.19 vs 0.20 BSN/observer/pointer syntax differences.

For upgrades between 0.19 and 0.20, or when 0.19 snippets fail to compile on
0.20, read [references/migration-0.19-to-0.20.md](references/migration-0.19-to-0.20.md).

## Principles

- Prefer composition over inheritance, data-oriented components, focused systems, event/message-driven communication, plugin boundaries, and minimal global state.
- Separate app setup, plugins, components, resources, messages/events, systems, states, rendering, UI, networking, and persistence. Feature modules should own their data and systems.
- Use resources for genuinely global state; flag god resources, god components, god systems, excessive mutable queries, hidden coupling, and large content-definition match statements.
- Use Bevy States for menu, loading, gameplay, pause, cutscene, and other mutually exclusive modes. Use `OnEnter`/`OnExit`/`OnTransition` for lifecycle work and activate systems only in relevant states.
- Use `Startup` for one-time setup, `Update` for frame-driven logic, and `FixedUpdate` for simulation that needs a fixed timestep. Add explicit ordering only when a real dependency requires it.
- Prefer filtered and narrow queries, `Changed<T>`, `Added<T>`, `RemovedComponents<T>`, and `Single<Entity>` where appropriate. Measure before optimizing; discuss entity counts, archetype churn, contention, allocations, and frame pacing.
- For UI, separate persistent retained entities from transient interaction state. Do not recreate UI trees for pointer hover or other per-frame state; update components such as `Visibility`, `BackgroundColor`, text, or layout in place. Despawn owned hierarchies safely.
- Use `AssetServer`, handles, asset collections/loading states, and dependency-aware transitions. Avoid blocking asset loads or scattering untracked handles.
- Prefer reflection, serialization, assets, scenes, and external configuration for content. Keep behavior systems separate from authored data.
- Recommend ecosystem plugins only when they solve a demonstrated need and are compatible with the pinned Bevy release (0.20 prereleases often lag in the ecosystem); identify maintenance and integration tradeoffs for `bevy_asset_loader`, `bevy_mod_picking`, `leafwing-input-manager`, `bevy_egui`, `avian`, `bevy_kira_audio`, `iyes_progress`, `bevy_rapier`, and alternatives.

## BSN and Feathers migration principles

- Treat `bsn!`, scene functions, templates, `Children[]`, named references,
  scene lists, and field-level patches as declarative hierarchy tools. Do not
  mechanically wrap an imperative `spawn` sequence in `bsn!`.
- Keep runtime-generated geometry, per-frame hit testing, input capture,
  interaction ownership, asset-dependent leaves, and entity-reference wiring
  in ECS systems when their values are not known at scene resolution time.
- Prefer stock Feathers scene components for editor controls when their focus,
  activation, value-change, keyboard/gamepad, accessibility, and theme
  semantics match the product. Keep bespoke widgets for procedural or
  game-specific interaction and document each exception.
- Preserve the behavior contract during UI migration: captured input must be
  consumed before global shortcuts, selection must not reopen unrelated config
  windows, and pointer hover must not change ownership or rebuild every UI
  component.
- Review scene ownership and deferred-command timing explicitly. A runtime
  entity returned for immediate child population may justify a small imperative
  boundary; a scene should not be forced to solve that problem by hiding a
  command chain.
- When diagnosing UI flicker, trace structural writes first: rebuild/despawn
  systems, `Visibility`/`Display` writes, selection/hover ownership, z-order and
  hit-target overlap. Add targeted logs for state transitions and entity IDs,
  then gate work with change detection or messages rather than logging every
  frame indefinitely.
- Bevy UI interaction is picking-based in 0.20: use `PickingInteraction`, or
  `Hovered` + `Pressed`, and `bevy::ui_widgets::Button`. The deprecated
  `Interaction`/prelude `Button` aliases point at private types and do not
  compile in queries or scenes.
- In Bevy 0.20, distinguish Rust-authored BSN from disk-backed `.bsn` assets;
  the `.bsn` file format is still unreleased.
  Verify the release notes before claiming serialized scene authoring is
  available. Keep authored configuration serialization separate from transient
  ECS presentation state until the required asset workflow exists.

## Reusable widget composition principles

- Match example syntax to the pinned release. 0.20 uses `--` list separators,
  `@scene()` / `@{expr}` includes, `On<Add<T>>`, and flat `PointerClick`;
  0.19 uses commas, `scene()` / `{expr}`, `On<Add, T>`, and `Pointer<Click>`.
  Examples on `bevyengine/bevy` `main` may already be past 0.20. Verify
  against the local `bevy_scene-*/src/lib.rs` syntax table before copying.
- A feature module owns its widget: authored data, geometry, style tokens,
  scene function/`SceneComponent`, and runtime systems. Consumers (HUD, editor)
  stay thin: resolve data, pass props, attach identity, and handle events.
- Pass consumer content through props: `Box<dyn SceneList>` slots (Feathers
  `caption`), caller-appended `Children [..]`, or a per-item slot factory.
  Do not return internal entity handles for the consumer to patch.
- Export typed anchors (edges, arcs, center) so consumers can place overlay
  controls without duplicating layout math. Spawn overlays on a non-rotated
  layer.
- Carry identity in one root component plus item-local indices, not threaded
  index tuples. Link overlay controls with a custom relationship.
- `Activate` does not propagate and `Button` stops pointer propagation; define
  a widget-owned `#[entity_event(propagate, auto_propagate)]` event for item →
  root/consumer notifications. Prefer `on(...)` observers per control over one
  giant `match` on a global action enum.
- Widget modules must not read consumer resources. Invert with marker
  components, run conditions, or system sets.
- Show/hide edit affordances and selection in place; rebuild only the widget
  subtree on structural document changes.

Do not force Unity-style MonoBehaviours, Unreal-style actor-centric managers, massive global managers, or OOP abstractions that fight ECS. State uncertainty when official documentation is unavailable, and propose a verification step.
