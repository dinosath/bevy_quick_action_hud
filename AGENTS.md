# Project development rules

## HUD document model

Use this ownership model in code, persisted data, editor UI, tests, and
documentation:

- The HUD contains at least one page.
- A page may contain zero or more `RadialMenuSet` entries and zero or more
  buttons. Page-switch controls remain page-level controls when enabled.
- A page must not contain a standalone `RadialMenu` entry.
- A `RadialMenuSet` contains one or more radial menus in its private
  `radial_menus` collection.
- Every radial menu contains at least two sectors.
- A radial menu set owns the navigation binding and previous/next menu
  shortcuts. Sectors do not have input bindings.
- Radial menus in a set share the set's presentation settings; sector content
  remains owned by each radial menu.

## Implementation enforcement

- Keep `SetEntry` limited to page-level components such as buttons,
  `RadialMenuSet`, and page switches. Do not reintroduce a standalone radial
  menu variant.
- Keep `RadialMenuSet::radial_menus` internal to the crate. Expose intentional
  accessors or commands instead of exposing the collection as HUD-owned state.
- Constructors and defaults must create at least two sectors.
- Configuration normalization must repair legacy or hand-authored menus to the
  minimum sector count and must ensure every set contains one radial menu.
- Editor actions may add/remove radial-menu sets and menus inside a set, but
  may not add a standalone radial menu to a page or capture a sector binding.
- Validation must report a set with no radial menus and a radial menu with fewer
  than two sectors.
- New code uses `page`, `RadialMenuSet`, `RadialMenu`, `sector`, and `button` as
  canonical terms. Avoid adding new wheel, wheelset, segment, or slice names.

When changing this model, update the README, persisted configuration examples,
normalization, editor actions, validation, and tests together.
