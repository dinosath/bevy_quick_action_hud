# Bevy 0.20 review checklist

Use this as a routing checklist, not as a requirement to comment on every item.

## Architecture and project layout

Look for a small app entrypoint that composes plugins, with feature modules owning their components, resources, messages, systems, and schedules. A typical production project may separate `app`, `gameplay`, `ui`, `rendering`, `assets`, `input`, `save`, `networking`, and `debug`, but recommend boundaries based on coupling rather than folder fashion. Identify modules that should become plugins and avoid a single `lib.rs` or `main.rs` containing unrelated feature logic.

Prefer a flow such as:

```text
App setup -> feature plugins -> state-scoped systems -> messages/events -> focused queries
                         \-> assets/loading -> authored data -> runtime components
```

## ECS review

Check that components are small data records and systems express behavior. Flag:

- resources containing per-entity collections that could be components or messages;
- components combining unrelated identity, presentation, input, and simulation data;
- systems that mutate many unrelated resources or query the whole world to perform one job;
- hidden ordering through shared mutable resources;
- content encoded as large `match` statements instead of serializable data;
- event/message types used as implicit global command buses without ownership or lifecycle.

Recommend events/messages for decoupled communication, resources only for global state, and marker components or state-scoped schedules for activation. In Bevy 0.20, verify whether the project uses the current message/event APIs expected by its exact dependency version rather than copying older examples.

## Schedules and states

Review `Startup`, `PostStartup`, `Update`, `FixedUpdate`, `OnEnter`, `OnExit`, and `OnTransition` usage. Simulation, physics, and deterministic movement often belong in `FixedUpdate`; input sampling, presentation, and UI generally belong in `Update`, with explicit boundaries where needed. Avoid adding ordering chains everywhere: order only actual dependencies and document why.

Recommend explicit Bevy `States` for loading, main menu, gameplay, pause, cutscenes, and overlays when those modes change which systems should run. Use state-run conditions instead of polling mode flags in every system. Keep persistent resources across transitions only when their ownership is intentional.

## Query and performance review

For each hot-path query ask:

1. Can a marker/filter narrow it?
2. Can `Changed<T>`, `Added<T>`, or `RemovedComponents<T>` avoid work?
3. Is `Single<Entity>` or a targeted query more truthful than iterating all entities?
4. Is a mutable query causing avoidable archetype/resource contention?
5. Is the work frame-rate-sensitive, fixed-step, or event-driven?

Discuss entity count, archetype fragmentation, allocations, command-buffer volume, asset churn, and frame pacing only with evidence or a suggested measurement. Recommend Tracy, Bevy diagnostics, render diagnostics, logging with throttling, or a focused benchmark where appropriate. Do not optimize based only on intuition.

## UI review

Bevy UI is an ECS hierarchy. Keep long-lived UI entities stable and update visual state in place. Pointer hover, focus, pressed state, animation, and selection should not rebuild or despawn an entire UI tree. Structural rebuilds are reasonable after a document/schema/layout change, but should be explicit and infrequent. Track ownership of dynamically spawned subtrees and recursively despawn owned descendants before parents, using the APIs supported by the exact Bevy version.

For reusable widgets, check the widget/consumer boundary: flag adapters with
long positional parameter lists or index tuples, consumers that clone and
re-theme widget data, consumers that reimplement widget geometry to place
overlays, forwarding wrappers with no logic, magic-integer focus indices,
duplicated helpers across modules, `[f32; 4]` color constants converted at
each use instead of `const Color`, hand-written recursive despawn
(`despawn()` is already recursive), and core widget systems that read consumer
resources. See [reusable-widgets.md](reusable-widgets.md).

Separate UI/editor state from gameplay state. For keyboard/gamepad UI navigation, use focused entities, semantic actions, and consistent picking-based interaction (`PickingInteraction`, or `Hovered` + `Pressed`, in 0.20; `Interaction` only on 0.19) and focus handling. Check hit targets, hidden controls, z-order, layout bounds, and whether contextual controls intercept input meant for their owner.

## Assets and persistence

Prefer `AssetServer` handles and a loading state or asset collection for startup dependencies. Keep loading, failure, and retry policy explicit. Avoid synchronous reads in frame systems. For save data, separate serialized authored/runtime data from transient ECS components, use schema defaults/migrations, and make writes atomic when data loss matters.

## Modern API compatibility

Inspect `Cargo.toml`, lockfile, feature flags, and imports before judging API usage. Cross-check release notes and migration guides for changes to schedules, messages/events, observers, UI, hierarchy, rendering, input, and asset APIs; for 0.19 → 0.20 use [migration-0.19-to-0.20.md](migration-0.19-to-0.20.md). Flag deprecated APIs (`#[deprecated]` warnings) as migration debt even when they still compile. Mark Cheatbook advice as conditional if it targets another version. Prefer compiler errors and official examples as the final authority.

## Plugin recommendations

Recommend an ecosystem plugin only after identifying the concrete capability and checking compatibility with the pinned Bevy release (ecosystem crates often lag behind 0.20 prereleases), maintenance, licensing, and integration cost. Examples include:

- `bevy_asset_loader` for declarative asset-loading states;
- `bevy_mod_picking` for pointer picking when native UI interaction is insufficient;
- `leafwing-input-manager` for action maps and rebinding;
- `bevy_egui` for tooling/debug panels rather than native game UI;
- `avian` or `bevy_rapier` for physics, after choosing one physics authority;
- `bevy_kira_audio` for richer audio buses and music control;
- `iyes_progress` for progress aggregation;
- networking, save, and hot-reload plugins only after defining authority, persistence, and version constraints.

Never add a plugin merely because it is on a list.

## Genre guidance

Apply only the relevant slice:

- **RPG/action RPG/soulslike/Skyrim-style:** feature plugins for combat, actors, inventory, quests, dialogue, world streaming, save/load, input, and UI; keep stats/effects data-driven and state transitions explicit.
- **RTS/city builder:** separate simulation and presentation, use fixed-step or tick-based simulation, spatial partitioning, command/event streams, selection/placement UI, and scalable queries.
- **Visual novel:** state-driven dialogue graph, localization assets, save checkpoints, presentation timeline, and minimal per-frame polling.
- **Roguelike:** seeded run state, deterministic generation, turn/tick boundaries, content assets, run persistence, and explicit meta-progression ownership.
- **Platformer:** fixed-step or carefully bounded movement/physics, input buffering, collision events, animation/presentation separation, and camera/UI state boundaries.

## Generation mode

When generating a project, produce a small root app, feature plugins, state definitions, asset-loading flow, tests for pure data/systems, CI for `fmt`, `clippy`, tests, and a compatible build. Add feature flags only for real optional capabilities. Prefer a thin vertical slice over empty folders and placeholder managers.

## Review output

Always include the requested score block, then evidence-backed strengths, issues with severity and location, incremental refactoring steps, and at least one concrete improved-code example when code is available. Distinguish correctness bugs, compatibility risks, performance risks, and maintainability suggestions.
