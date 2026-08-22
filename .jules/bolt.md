## 2025-05-20 - Virtualized React components with framer-motion layout prop

**Learning:** Combining `@tanstack/react-virtual` with Framer Motion's `layout` prop on every item causes severe DOM layout thrashing. Because virtualized containers already manage item position via `translateY` transform styles, Framer Motion's `layout` measurements trigger continuous layout recalculations on every scroll event and state update across all visible items. Furthermore, passing non-memoized inline callbacks and `selectedIds` arrays into virtualized item renderers breaks React component memoization completely.

**Action:** Remove `layout` props from items inside `@tanstack/react-virtual` containers. Wrap item components (`FileCard`, `FileListItem`, `FileTypeIcon`, `SidebarItem`) in `React.memo`, pass boolean flags (like `isSelected`) instead of array collections, and ensure callbacks are stable function references.
