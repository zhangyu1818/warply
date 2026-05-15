# APP-3632: Code Review Header UI Refactor

## Problem
The updated Figma designs for the code review panel introduce git operation buttons (commit, push, create PR) in the inner header. The existing header layout doesn't have room for these — it already contains the branch name, diff stats, discard button, add-context button, and diff mode dropdown. This PR clears space by relocating contextual info upward and consolidating actions into the overflow menu.

## Fork State
The code review header is retained local UI. This fork keeps the simplified header and git-operation affordances directly, without a rollout flag, disabled branch, or old render-path compatibility.

## Overview of Current Behavior
The code review header is structured into two layers:

1. **Right panel header** (top-level): shows repo context — repo path, branch name, and diff stats
2. **Inner code review header**: simplified to just the diff mode selector, file nav button, and an overflow menu

Actions that were previously standalone buttons (discard all, add diff set as context) are consolidated into the overflow menu. The file navigation toggle button is now an `ActionButton` with `PaneHeaderTheme` that appears in both wide and compact layouts.

## File-by-File Changes

### `app/src/workspace/view/right_panel.rs`
**Purpose**: Redesign the panel header to show contextual git info.

- The retained layout replaces the static "Code review" title with:
  - **Repo path** — tilde-shortened (e.g. `~/Repos/warp-internal:`), rendered in semibold sub-text color
  - **Branch name** — read from `DiffStateModel` via `get_diff_state_model()`
  - **Diff stats** — read from `CodeReviewView::loaded_diff_stats()`
- Uses shared `CONTENT_LEFT_MARGIN` / `CONTENT_RIGHT_MARGIN` constants so the header aligns with the content area below

### `app/src/code_review/code_review_header.rs`
**Purpose**: Simplify the inner header to only layout concerns.

- The retained renderer displays diff mode dropdown (left) + git operations button + file nav button + overflow menu (right). Compact layout is a single row.
- `render_header` in `CodeReviewView` calls the retained renderer directly.

### `app/src/code_review/code_review_view.rs`
**Purpose**: Restyle the dropdown, relocate buttons, consolidate menu items.

- **File navigation button**: `ViewHandle<ActionButton>` with `PaneHeaderTheme`, created once in `new()`. Passed to the header via `CodeReviewHeaderFields.file_nav_button` so both wide and compact layouts can render it. Tooltip updates dynamically when sidebar state changes.
- **Diff mode dropdown**: retained styling uses `ButtonVariant::Text` with semibold larger font.
- **`header_menu_items()`**: builds the retained overflow menu directly, including "Discard all" and the current `AISettings` check.
- **`loaded_diff_stats()`**: new public accessor for the right panel header
- **Shared margin constants**: `CONTENT_LEFT_MARGIN` (16px) and `CONTENT_RIGHT_MARGIN` (4px) exported as `pub(crate)`
- **`render_header`**: takes `state` and `app` params and renders the retained header.

### `app/src/pane_group/working_directories.rs`
**Purpose**: Expose diff state for the panel header to read.

- Adds `get_diff_state_model(&self, repo_path: &Path) -> Option<ModelHandle<DiffStateModel>>`

## Design Decisions

- **Single retained render path**: the fork keeps the current code review header directly. Future upstream merges should not restore `GitOperationsInCodeReview`, disabled branches, or old header render paths.
- **File nav button as `ActionButton`**: uses `PaneHeaderTheme` to match the three-dots and maximize buttons. Created as a `ViewHandle` in `new()` (not inline during render) so it can appear in both wide and compact layouts via `ChildView`.
- **Branch name reads from `DiffStateModel` directly** rather than being passed through `CodeReviewView`, because the panel header renders independently of whether diffs have loaded.
- **Diff stats still read from `CodeReviewView::loaded_diff_stats()`** because they depend on the loaded diff state, which only `CodeReviewView` owns.
- **Overflow menu is always rendered** (no longer gated on `FileAndDiffSetComments`). Individual items are independently gated, so the menu gracefully degrades to empty when all flags are off.
