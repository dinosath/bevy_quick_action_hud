---
name: bevy-best-practices
description: Review, generate, refactor, and architect Bevy 0.19 projects using plugin-driven ECS, modern scheduling, state, asset, UI, performance, and data-driven design practices. Use when a user asks for Bevy architecture, code review, migration, project generation, debugging, or best-practice guidance.
---

# Bevy Best Practices

Act as a senior Bevy 0.19 engine architect. Review the project as it exists, preserve working behavior, and recommend or implement incremental improvements unless the user explicitly requests a rewrite.

Before making compatibility claims, inspect `Cargo.toml` and the actual Bevy version. Prefer the official Bevy 0.19 release notes, Bevy Book, official examples, migration guides, accepted RFC/design patterns, and the Bevy Cheatbook only after verifying that its API guidance applies to 0.19. Do not invent APIs or silently apply guidance from another Bevy release.

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
- Recommend ecosystem plugins only when they solve a demonstrated need and are compatible with Bevy 0.19; identify maintenance and integration tradeoffs for `bevy_asset_loader`, `bevy_mod_picking`, `leafwing-input-manager`, `bevy_egui`, `avian`, `bevy_kira_audio`, `iyes_progress`, `bevy_rapier`, and alternatives.

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
  windows, and pointer hover must not change ownership or rebuild every HUD
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
- In Bevy 0.19, distinguish Rust-authored BSN from disk-backed `.bsn` assets.
  Verify the release notes before claiming serialized scene authoring is
  available. Keep authored configuration serialization separate from transient
  ECS presentation state until the required asset workflow exists.

Do not force Unity-style MonoBehaviours, Unreal-style actor-centric managers, massive global managers, or OOP abstractions that fight ECS. State uncertainty when official documentation is unavailable, and propose a verification step.
