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
| **File Opening (CLI/Open)** | ✅ | 🏗️ | ✅ |
| **Basic Markdown Rendering** | ✅ | 🏗️ | ✅ |
| **Hybrid Theme (Dark/Cream)** | ⬜ | ⬜ | ⬜ |
| **Live Reload (Watcher)** | ⬜ | ⬜ | ⬜ |
| **Tabbed Interface** | ⬜ | ⬜ | ⬜ |
| **Split View (Side-by-Side)** | ⬜ | ⬜ | ⬜ |
| **History Sidebar** | ⬜ | ⬜ | ⬜ |
| **Table of Contents (TOC)** | ⬜ | ⬜ | ⬜ |
| **Scroll Synchronization** | ⬜ | ⬜ | ⬜ |
| **LaTeX Support** | ⬜ | ⬜ | ⬜ |
| **Mermaid Support** | ⬜ | ⬜ | ⬜ |
| **Zen Mode** | ⬜ | ⬜ | ⬜ |
| **Drag & Drop** | ⬜ | ⬜ | ⬜ |

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
