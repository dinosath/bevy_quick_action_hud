# BSN and Feathers review guide for Bevy 0.20

Use this reference for scene-notation migrations, editor/tooling UI reviews,
Feathers adoption, or UI bugs involving flicker, selection, hover, or input
capture.

## First establish the version boundary

Check the exact Bevy and `bevy_feathers` versions in `Cargo.toml` and the
lockfile. This repository pins `0.20.0-rc.1`. Treat the local crate docs
(`bevy_scene-<version>/src/lib.rs` contains the BSN syntax table), release
notes, migration guide, and official examples at the matching tag as the
authority. Both 0.19 and 0.20 support Rust-authored BSN composition through
`bsn!`, scene functions, templates, scene patches, `Children[]`, named entity
references, `SceneList`, and `spawn_scene`, but the syntax differs (0.20:
`--` separators, `@scene()`, `@{expr}`). See
[migration-0.19-to-0.20.md](migration-0.19-to-0.20.md).

Do not describe `.bsn` files as a completed asset workflow without verifying
the release notes: the `.bsn` file format is still unreleased in 0.20.0-rc.1.
A project may use BSN for reusable Rust scene definitions while keeping
serialized gameplay/editor configuration in its existing data format.

## Classify every hierarchy before changing it

For each root, panel, row, widget, and dynamic leaf, classify it as:

1. **Static declarative structure:** use a reusable scene function or
   `SceneComponent`; compose with `Children[]`, named references, and field
   patches.
2. **Data-dependent but structurally reusable:** use a scene function with
   props, a template, or a `SceneList`; keep the data-to-entity boundary
   explicit and event-driven.
3. **Runtime behavior/rendering:** keep ECS components and systems for input,
   selection, hover, hold progress, procedural geometry, asset readiness,
   capture, and state transitions.
4. **Entity-reference/deferred-command boundary:** use named references when
   the complete hierarchy is known during scene resolution. Otherwise retain a
   small imperative boundary and explain why it returns or injects an entity.

The correct migration target is not zero `spawn` calls. It is declarative
structure with focused runtime behavior and clear ownership.

## Feathers decision test

Evaluate buttons, tool buttons, toggles, checkboxes, text inputs, number inputs,
sliders, menus, lists, scrollbars, focus indicators, and theme tokens against
these questions:

- Does the stock widget provide the required value and activation messages?
- Does it work with the project’s keyboard/gamepad focus and pointer model?
- Can it coexist with rebinding/capture without leaking the triggering input to
  global application shortcuts?
- Does it preserve accessibility labels, disabled/selected state, and theme
  semantics?
- Does it avoid replacing a whole `Node` component after a Feathers scene has
  installed one?

Use Feathers scene functions and field-level BSN patches where possible. Be
careful with runtime entity targets: `FeathersScrollbar::scene` can use a
named `#inner` reference inside one scene, but a helper that must return a
content entity immediately may still need to spawn that content first and
spawn the Feathers widget with the entity as a runtime prop.

Retain custom widgets for radial/procedural geometry, game-specific hit
testing, or interaction whose semantics are not represented by Feathers.
Document the exception and migrate the surrounding shell where useful.

## UI flicker and accidental activation investigation

When all controls appear/disappear or a config panel opens unexpectedly:

1. Log only transitions, including frame-independent entity IDs, selected
   owner, hovered owner, editor-open state, config-window owner, and the reason
   for each rebuild or visibility write.
2. Find every system that writes `Visibility`, `Display`, selection ownership,
   `Interaction`, `Hovered`, or despawns the owning hierarchy.
3. Check whether hover on a child changes the parent’s ownership and causes a
   rebuild that invalidates the current hit target.
4. Check whether a rebuild happens before pointer/hover propagation completes,
   creating a feedback loop: target disappears, hover clears, target returns,
   hover is reacquired.
5. Check z-order, overlapping hit targets, and whether contextual toolbar
   buttons intercept input intended for the selected component.
6. Gate structural rebuilds on messages or a dirty flag. Update visual state in
   place for hover/focus/pressed changes. Use `Changed<T>`, `Added<T>`, and
   narrow queries where appropriate.

Input capture must run before global shortcuts. The capture system should mark
the event consumed, and application-level input systems must honor that
consumed state in the same update.

## Review deliverable

For a BSN/Feathers review report, include:

- fully migratable, partially migratable, not currently possible, Feathers
  candidates, and custom-widget exceptions;
- Before, After, Migration Notes, BSN coverage, and Feathers coverage for each
  module;
- the exact Bevy source basis (for example `0.20.0-rc.1`) and known API limitations;
- deferred-command and entity-reference tradeoffs;
- verification commands and behavioral tests, not only compilation;
- a prioritized incremental plan rather than a blanket rewrite.

Coverage percentages must state what they measure (for example, static
hierarchy lines or widget surface), because runtime entity counts and source
line counts give very different results.

## General architecture guidance

- Keep core domain logic, runtime presentation, and editor registration in
  clear feature boundaries, even when a feature needs later decomposition.
- Use messages/observers for selection, lifecycle, action, and value-change
  communication; transient presentation markers should not become god
  resources.
- Rebuild a data-dependent editor only for explicit selection or configuration
  changes, never for pointer hover or every frame.
- Custom procedural widgets may retain their rendering, materials, and hit
  testing in ECS while BSN owns surrounding overlays, panels, labels, icons,
  and reusable layout primitives.
- Prefer Feathers controls when its theme and focus behavior fit. Keep truly
  domain-specific components beside them instead of reimplementing stock
  controls.
