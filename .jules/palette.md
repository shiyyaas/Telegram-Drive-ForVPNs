# Palette's UX & Accessibility Journal

## 2025-05-14 - Hover-only Quick Actions in Virtualized Item Lists
**Learning:** Virtualized item cards/rows (`FileCard`, `FileListItem`) using `group-hover:opacity-100` hide action buttons from keyboard users unless accompanied by `focus-within:opacity-100` and individual `focus-visible` ring indicators. Furthermore, custom overlay indicators (like selection bubbles) require explicit `<button role="checkbox" aria-checked="...">` semantics and visible focus states so screen readers and keyboard users can toggle item selections without relying on pointer clicks.
**Action:** Always combine `group-hover:opacity-100` with `focus-within:opacity-100` on action clusters and use semantic button elements with `role="checkbox"` and `aria-label`s for item-level selection controls.

## 2025-05-15 - Accessible Confirmation Dialog Semantics & Safe Keyboard Focus
**Learning:** Modal confirmation and destructive action dialogs must announce themselves via `role="alertdialog"`, `aria-modal="true"`, and corresponding `aria-labelledby`/`aria-describedby` associations so screen reader users understand the urgent context. Additionally, keyboard navigation should default focus to the safe "Cancel" action on destructive ('danger') prompts to prevent accidental irreversible deletions on Enter presses, while providing `Escape` key dismiss and backdrop-click cancellation.
**Action:** Always wrap confirm modals in `role="alertdialog"` with explicit title/description IDs, support Escape/backdrop cancellation, and autofocus the Cancel button when variant is 'danger'.

## 2025-05-16 - Sibling Action Triggers in Navigation Tree Items
**Learning:** Placing nested clickable elements (like delete icons) inside `<button>` elements creates invalid HTML (interactive content cannot be nested inside a button) and breaks keyboard navigation because screen readers and tab navigation cannot reliably target child buttons. Instead, wrap the item in a container (`div.relative.group`), keep the primary navigation button and the secondary action button as siblings, and expose the secondary action on `:focus-visible` / `group-focus-within` with an explicit `aria-label`.
**Action:** In sidebar and list item rows, avoid nesting interactive elements inside `<button>`. Render secondary actions as sibling `<button type="button">` controls revealed via `group-hover:opacity-100 group-focus-within:opacity-100` and clear `aria-label`s.

## 2025-05-17 - Persistent Floating Queue Landmarks & Modal Escape Consistency
**Learning:** Floating background operational widgets (such as upload and download transfer queues) rendered outside the main landmark require `role="region"` with distinct `aria-label`s so assistive technology can locate and differentiate them from the document flow. Furthermore, full-screen image/media preview modal overlays must consistently register top-level window `Escape` event listeners alongside dialog ARIA attributes (`role="dialog"`, `aria-modal="true"`), matching confirmation and sub-dialog dismissal paradigms.
**Action:** Always identify floating task widgets with `role="region"` and an explicit label, and ensure every modal dialog overlay provides `Escape` key dismissal with accessible close button controls.
