# PaperView — Design Specification

This document details the visual identity and UI components of PaperView, based on the reference design.

## 1. Visual Language
The design follows a "Paper-on-Desktop" metaphor: dark, functional sidebars anchoring a high-contrast, physical-feeling document area.

### 1.1 Color Palette
#### **Sidebars & UI Shell (Dark Mode)**
- **Background:** `#111318` (Primary sidebar/header background)
- **Secondary Background:** `#161A22` (Active item highlights)
- **Text (Primary):** `#FFFFFF` (Titles, active items)
- **Text (Secondary):** `#8B949E` (Paths, inactive items, timestamps)
- **Accent:** `#58A6FF` (Links, active indicators, badges)

#### **Main Reader (The "Paper")**
- **Background:** `#FDF8EF` (Soft cream/off-white)
- **Text:** `#1F2328` (Deep charcoal for maximum readability)
- **Table Borders:** `#D0D7DE`
- **Table Header Shading:** `#F6F8FA`

---

## 2. Layout & Components

### 2.1 Header & Tab Bar
- **Header:** Top-most bar containing the "History" and "Navigation" toggle buttons and global search.
- **Tab Bar:** Located directly above the Reader area, spanning the width of the center column.
    - **Styling:** Dark background (`#111318`) with individual tabs.
    - **Active Tab:** Matches the Reader's cream background (`#FDF8EF`) or has a blue bottom border.
    - **Inactive Tab:** Dimmed text, matching the sidebar background.
    - **Close Button:** The current GUI has close controls on each tab.
- **Split View Toggle:** The current GUI exposes a header toggle and also uses
  `Cmd + \` on macOS and `Ctrl + \` elsewhere to toggle Split View.
- **Split View Resize:** The current GUI uses `Cmd + ]` / `Cmd + [` on macOS
  and `Ctrl + ]` / `Ctrl + [` elsewhere to grow or shrink the primary pane.
- **Secondary Pane Selector:** While Split View is on, non-active tabs expose a
  compact selector for choosing the right-side pane.

### 2.2 Left Sidebar: "History"
- **Structure:**
    - Top: "History" title with Search and Filter icons.
    - Content: Vertical list grouped by date (e.g., "Today", "Yesterday").
    - Item Meta: File icon + Filename (Bold) + Full path (Small/Dimmed).
    - Bottom Utility Bar: Sidebar toggle, History/Clock icon, Settings gear.

### 2.3 Main Reader & Split Layout
- **Single View:** Centered content with a max width of 720–860px.
- **Split View:** The current GUI foundation divides the reader area vertically
  into two panes: active tab on the left, secondary open tab on the right.
    - **Independent Scrolling:** Deferred.
    - **Secondary Selection:** Chosen from non-active tabs while Split View is on.
    - **Keyboard Resize:** The primary pane ratio is bounded from 30% to 70%
      and defaults to 50%.
    - **Comparison Mode:** Highlighting a section on one side can sync the other side (optional toggle; deferred).
- **Margins:** Generous white space (padding: ~40px - 60px).
- **Typography:**
    - **Headings:** Serif font (e.g., *Source Serif Pro* or *Georgia*). Bold, heavy weight.
    - **Body:** Sans-serif font (e.g., *Inter* or *SF Pro*). 1.6 line height for readability.
- **Table Design:** 
    - Rounded corners (4px - 6px).
    - Light grey borders.
    - Alternating row highlights or distinct header shading.
    - The current GUI renders table panels with light borders, equal-width
      cells, alignment-aware text, and shaded header cells.

### 2.4 Right Sidebar: "Navigation"
- **"On this page" (TOC):**
    - Hierarchical numbering (1., 1.1, etc.).
    - Active section highlighted in accent blue.
- **"Bookmarks":**
    - List of saved files.
    - Blue bookmark icon + Filename + Badge for page/count.

---

## 3. Typography Specs
| Element | Font Family | Size | Weight | Color |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | Serif | 32px | 700 | `#1F2328` |
| **H2** | Serif | 24px | 600 | `#1F2328` |
| **Body** | Sans-Serif | 16px | 400 | `#1F2328` |
| **Sidebar Title** | Sans-Serif | 14px | 600 | `#FFFFFF` |
| **Sidebar Path** | Sans-Serif | 11px | 400 | `#8B949E` |
| **Code Block** | Monospace | 14px | 400 | `#1F2328` |

Current source-preserving technical blocks use the same restrained panel shape
as code blocks. LaTeX math uses a warm accent border, and Mermaid diagrams use a
green accent border until native formula and diagram rendering are selected.

---

## 4. Interaction Details
- **Active State:** The current open file in the left sidebar should have a subtle background highlight (`#1B1F27`) and a thin blue vertical line on the left edge.
- **Hover State:** Sidebar items should transition to a slightly lighter grey.
- **Scroll Sync:** As the user scrolls, the corresponding entry in the "On this page" TOC turns accent blue. Clicking a TOC entry jumps the active reader to that section. The current GUI maps active reader scroll progress and TOC jumps to estimated reader heading anchors; exact Iced layout rectangles are deferred.
- **Drag & Drop Overlay:** When a file is dragged over the window, the GUI shows a subtle accent border around the shell and a header status prompt indicating the hovered path.

## 5. Zen Mode & Edit Mode

### 5.1 Zen Mode (Phase 1)
- **Concept:** Ultimate distraction-free reading.
- **Visuals:** Hides the left sidebar, right sidebar, and tab bar. The current implementation keeps the Header visible and lets the Main Reader fill the remaining space.
- **Trigger:** Platform command shortcut: `Cmd + Shift + F` on macOS and `Ctrl + Shift + F` elsewhere.

### 5.2 Edit Mode (Phase 2)
- **Editor Background:** Matches the Sidebar (`#111318`) or a slightly lighter grey to distinguish from the "Paper" preview.
- **Cursor:** Accent blue (`#58A6FF`) block or line cursor.
- **Syntax Highlighting:** Minimalist scheme following the UI colors (Greys, Blues, and Whites).
- **Split Pane:** Vertical divider between the raw Markdown text and the rendered "Paper" view.
