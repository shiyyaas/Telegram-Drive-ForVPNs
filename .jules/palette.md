# Palette's UX & Accessibility Journal

## 2025-05-14 - Hover-only Quick Actions in Virtualized Item Lists
**Learning:** Virtualized item cards/rows (`FileCard`, `FileListItem`) using `group-hover:opacity-100` hide action buttons from keyboard users unless accompanied by `focus-within:opacity-100` and individual `focus-visible` ring indicators. Furthermore, custom overlay indicators (like selection bubbles) require explicit `<button role="checkbox" aria-checked="...">` semantics and visible focus states so screen readers and keyboard users can toggle item selections without relying on pointer clicks.
**Action:** Always combine `group-hover:opacity-100` with `focus-within:opacity-100` on action clusters and use semantic button elements with `role="checkbox"` and `aria-label`s for item-level selection controls.

## 2025-05-15 - Accessible Confirmation Dialog Semantics & Safe Keyboard Focus
**Learning:** Modal confirmation and destructive action dialogs must announce themselves via `role="alertdialog"`, `aria-modal="true"`, and corresponding `aria-labelledby`/`aria-describedby` associations so screen reader users understand the urgent context. Additionally, keyboard navigation should default focus to the safe "Cancel" action on destructive ('danger') prompts to prevent accidental irreversible deletions on Enter presses, while providing `Escape` key dismiss and backdrop-click cancellation.
**Action:** Always wrap confirm modals in `role="alertdialog"` with explicit title/description IDs, support Escape/backdrop cancellation, and autofocus the Cancel button when variant is 'danger'.
