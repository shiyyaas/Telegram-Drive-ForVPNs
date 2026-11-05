# Palette's UX & Accessibility Journal

## 2025-05-14 - Hover-only Quick Actions in Virtualized Item Lists
**Learning:** Virtualized item cards/rows (`FileCard`, `FileListItem`) using `group-hover:opacity-100` hide action buttons from keyboard users unless accompanied by `focus-within:opacity-100` and individual `focus-visible` ring indicators. Furthermore, custom overlay indicators (like selection bubbles) require explicit `<button role="checkbox" aria-checked="...">` semantics and visible focus states so screen readers and keyboard users can toggle item selections without relying on pointer clicks.
**Action:** Always combine `group-hover:opacity-100` with `focus-within:opacity-100` on action clusters and use semantic button elements with `role="checkbox"` and `aria-label`s for item-level selection controls.
