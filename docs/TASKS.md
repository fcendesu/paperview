# PaperView — Implementation Tracker

This document tracks the progress of features across the three workspace crates.

## Status Legend
- ⬜ **To-do**
- 🏗️ **In Progress**
- ✅ **Done**
- ❌ **Blocked**

---

## Phase 1: MVP (v0.1)

| Feature | Core Logic | GUI (Iced) | TUI (Ratatui) |
| :--- | :---: | :---: | :---: |
| **Project Setup** | ✅ | ✅ | ✅ |
| **File Opening (CLI/Open)** | ✅ | ✅ | ✅ |
| **Basic Markdown Rendering** | ✅ | ✅ | ✅ |
| **Hybrid Theme (Dark/Cream)** | ⬜ | ✅ | ⬜ |
| **Live Reload (Watcher)** | ✅ | ✅ | ✅ |
| **Tabbed Interface** | ✅ | ✅ | ⬜ |
| **Split View (Side-by-Side)** | ⬜ | ⬜ | ⬜ |
| **History Sidebar** | ✅ | ✅ | ✅ |
| **Table of Contents (TOC)** | ✅ | ✅ | ✅ |
| **Scroll Synchronization** | ⬜ | ⬜ | ⬜ |
| **LaTeX Support** | ⬜ | ⬜ | ⬜ |
| **Mermaid Support** | ⬜ | ⬜ | ⬜ |
| **Zen Mode** | ⬜ | ✅ | ⬜ |
| **Drag & Drop** | ✅ | ✅ | ⬜ |

---

## MVP Notes

- History Sidebar includes shared persistence, GUI click-to-open behavior, and the TUI recent-files dashboard.
- Live Reload covers active documents in both GUI and TUI; debouncing and exact scroll restoration are deferred.
- Drag & Drop currently covers native GUI file drops; multi-file behavior waits for tabs.
- Zen Mode currently covers the GUI focused reader layout; TUI and persisted preferences are deferred.
- Tabbed Interface currently covers shared open-document state plus GUI tab activation and close controls; reorder/TUI tabs are deferred.

---

## CLI Toolkit (Headless)

| Command | Status | Notes |
| :--- | :---: | :--- |
| **paperview search** | ⬜ | Needs `ripgrep` integration |
| **paperview export** | ⬜ | Needs PDF/HTML backend |
| **paperview stats** | ⬜ | Needs AST analysis logic |
| **paperview config** | ⬜ | Needs TOML persistence |

---

## Quality & Performance

| Task | Status | Target |
| :--- | :---: | :--- |
| **Cold Startup Time** | ⬜ | < 500ms |
| **Scrolling 60 FPS** | ⬜ | Constant |
| **Memory Footprint** | ⬜ | < 100MB (Typical) |
| **Zero-Dependency Build**| ⬜ | Single Binary |
