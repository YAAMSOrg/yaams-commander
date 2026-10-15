---
name: YAAMS Commander
description: Dark, dense desktop operations client for YAAMS virtual airlines.
colors:
  ops-night: "#20242e"
  ops-sidebar: "#1a1e26"
  console-slate: "#282e3a"
  console-slate-raised: "#303745"
  console-slate-hover: "#343c4b"
  table-zebra: "#2b3240"
  ledger-line: "#384050"
  ledger-line-strong: "#4a5366"
  ledger-line-hover: "#5a6478"
  text-primary: "#f5f6f7"
  text-secondary: "#b6bac2"
  text-muted: "#7f8693"
  text-placeholder: "#596170"
  dispatch-amber: "#e8a53f"
  dispatch-amber-hover: "#f0b456"
  dispatch-amber-pressed: "#cf8f2e"
  dispatch-amber-link-hover: "#f3c27a"
  amber-bronze: "#865f31"
  amber-umber: "#574736"
  on-amber: "#1c2430"
  success: "#3a8f42"
  success-light: "#d9f4d9"
  danger: "#cc272e"
  danger-light: "#ffdfda"
  warning: "#ccac16"
  warning-light: "#f9efc9"
  info: "#2c94b0"
  info-light: "#d6f0f9"
  neutral-status: "#69737d"
  neutral-status-light: "#e2e5e8"
typography:
  display:
    fontFamily: "Space Grotesk"
    fontSize: "26px"
    fontWeight: 700
  headline:
    fontFamily: "Space Grotesk"
    fontSize: "22px"
    fontWeight: 700
  title:
    fontFamily: "Space Grotesk"
    fontSize: "20px"
    fontWeight: 700
  body:
    fontFamily: "Inter"
    fontSize: "13px"
    fontWeight: 400
  body-strong:
    fontFamily: "Inter"
    fontSize: "13px"
    fontWeight: 700
  body-small:
    fontFamily: "Inter"
    fontSize: "12px"
    fontWeight: 400
  label:
    fontFamily: "Inter"
    fontSize: "11px"
    fontWeight: 700
    letterSpacing: "normal"
  data:
    fontFamily: "JetBrains Mono"
    fontSize: "13px"
    fontWeight: 400
  data-strong:
    fontFamily: "JetBrains Mono"
    fontSize: "13px"
    fontWeight: 700
  clock:
    fontFamily: "JetBrains Mono"
    fontSize: "17px"
    fontWeight: 700
  chip:
    fontFamily: "JetBrains Mono"
    fontSize: "10px"
    fontWeight: 700
rounded:
  chip: "4px"
  control: "6px"
  panel: "8px"
  dialog: "10px"
  pill: "999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "24px"
components:
  button-primary:
    backgroundColor: "{colors.dispatch-amber}"
    textColor: "{colors.on-amber}"
    typography: "{typography.body-strong}"
    rounded: "{rounded.control}"
    padding: "8px 14px"
  button-primary-hover:
    backgroundColor: "{colors.dispatch-amber-hover}"
    textColor: "{colors.on-amber}"
  button-primary-pressed:
    backgroundColor: "{colors.dispatch-amber-pressed}"
    textColor: "{colors.on-amber}"
  button-primary-disabled:
    backgroundColor: "{colors.amber-umber}"
  button-secondary:
    backgroundColor: "{colors.console-slate-raised}"
    textColor: "{colors.text-primary}"
    typography: "{typography.body-strong}"
    rounded: "{rounded.control}"
    padding: "8px 14px"
  button-secondary-hover:
    backgroundColor: "{colors.console-slate-hover}"
  button-danger:
    textColor: "{colors.danger-light}"
    typography: "{typography.body-strong}"
    rounded: "{rounded.control}"
    padding: "8px 14px"
  button-danger-hover:
    backgroundColor: "{colors.danger}"
    textColor: "{colors.text-primary}"
  button-ghost:
    textColor: "{colors.text-secondary}"
    rounded: "{rounded.control}"
    padding: "7px"
  button-ghost-hover:
    backgroundColor: "{colors.console-slate-hover}"
    textColor: "{colors.text-primary}"
  input:
    backgroundColor: "{colors.ops-night}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.control}"
    padding: "7px 10px"
  nav-item:
    textColor: "{colors.text-secondary}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    padding: "9px 12px"
  nav-item-active:
    textColor: "{colors.dispatch-amber}"
    typography: "{typography.body-strong}"
  segment-active:
    backgroundColor: "{colors.dispatch-amber}"
    textColor: "{colors.on-amber}"
    rounded: "{rounded.chip}"
  segment:
    backgroundColor: "{colors.console-slate-raised}"
    textColor: "{colors.text-secondary}"
    rounded: "{rounded.chip}"
  status-chip:
    typography: "{typography.chip}"
    rounded: "{rounded.chip}"
    padding: "2px 7px"
  banner:
    textColor: "{colors.text-primary}"
    rounded: "{rounded.control}"
    padding: "10px 14px"
  panel:
    backgroundColor: "{colors.console-slate}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.panel}"
    padding: "18px"
  table-header:
    backgroundColor: "{colors.console-slate-raised}"
    textColor: "{colors.text-secondary}"
    typography: "{typography.label}"
    height: "36px"
  table-row:
    backgroundColor: "{colors.console-slate}"
    textColor: "{colors.text-primary}"
    typography: "{typography.data}"
    height: "38px"
  toast:
    backgroundColor: "{colors.console-slate-raised}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.control}"
    padding: "12px 14px"
    width: "360px"
---

# Design System: YAAMS Commander

## Overview

**Creative North Star: "The Dispatch Ledger"**

YAAMS Commander is the dispatcher's record book made digital. Every screen is a page of the ledger: flight numbers, ICAO codes, block times and fuel figures in neat monospaced columns, ruled by thin lines, read under a dim night-shift light. The interface exists to make the entries legible and the one pending decision obvious. Nothing is there for decoration.

The world is dark and dense. A near-black slate ground (`ops-night`) carries slightly lighter console panels, separated by tone and 1px rules instead of shadows. A single amber accent - the YAAMS brand colour - marks what is active, selected or actionable: the current nav entry, the selected row, the primary button, the flight number. Status colours (green, red, yellow, teal) appear only as small translucent chips and banners that annotate the ledger, never as large fills.

The app takes a hobby and plays it straight. It should read like real airline operations software: UTC everywhere with a `Z` suffix, uppercase captions, exact figures, no illustrations, no gamified flourishes.

**Key Characteristics:**
- Dark tonal ledger: three close slate steps plus 1px rules carry all structure.
- One amber voice for active, selected and primary.
- Monospaced data, sans-serif prose, geometric display headings.
- Status as annotation: translucent chips and banners, never solid blocks.
- Fixed desktop chrome: sidebar, top bar with switchers and UTC clock, status bar.

## Colors

A cool, blue-grey slate night palette with one warm amber signal and four muted status tones.

### Primary
- **Dispatch Amber** (`dispatch-amber`): the only accent. Logo, primary button, active nav entry (text on a 14% amber wash), selected table row (13% wash plus 55% amber outline), focused input border, active segment, active sort chevron, flight numbers in tables and the detail header. Hover lightens to `dispatch-amber-hover`, pressed darkens to `dispatch-amber-pressed`; disabled primary sinks to **Amber Umber** (`amber-umber`) with greyed text. **Amber Bronze** (`amber-bronze`) is the pressed border.
- **On-Amber Ink** (`on-amber`): text and icons placed on solid amber. Never use white on amber.

### Neutral
- **Ops Night** (`ops-night`): app background, top bar, and the recessed "well" inside panels (inputs, route strings, code boxes).
- **Sidebar Slate** (`ops-sidebar`): sidebar and status bar - the darkest step, framing the work area.
- **Console Slate** (`console-slate`): panels, cards, table body.
- **Raised Slate** (`console-slate-raised`): table header, secondary buttons, menus, tooltips, toasts.
- **Hover Slate** (`console-slate-hover`): hover fill for rows, ghost and secondary buttons. **Zebra Slate** (`table-zebra`) stripes alternate rows.
- **Ledger Line** (`ledger-line`): default 1px rules and panel borders. **Strong Line** (`ledger-line-strong`) borders controls at rest; **Hover Line** (`ledger-line-hover`) borders controls on hover.
- **Text ramp**: `text-primary` for values and titles, `text-secondary` for body copy and inactive nav, `text-muted` for captions and hints, `text-placeholder` for input placeholders.

### Status tones
Each tone is a pair: a strong colour and a light colour. On the dark UI the **light** tone is the foreground (text, icons); the **strong** tone is used at 18% alpha as fill and 55% alpha as outline.
- **Accepted / success** (`success`, `success-light`)
- **Rejected / danger / errors** (`danger`, `danger-light`)
- **Pending / warning / read-only** (`warning`, `warning-light`)
- **Informational notices** (`info`, `info-light`)
- **Neutral / unknown status** (`neutral-status`, `neutral-status-light`)

### Named Rules
**The One Amber Rule.** Amber means "this is active, selected, or the action that matters". At most one solid amber button per view; everything else amber is text, outline or a translucent wash.

**The Annotation Rule.** Status colour never fills a large surface. It appears as a chip, a banner, an icon disc or a field-error line, always translucent fill plus outline plus light-tone text.

## Typography

**Display Font:** Space Grotesk Bold (headings, wordmark)
**Body Font:** Inter, regular and bold
**Data Font:** JetBrains Mono, regular and bold

**Character:** Space Grotesk gives page titles an engineered, slightly technical edge; Inter stays neutral for prose and controls; JetBrains Mono turns every code, time and figure into an aligned ledger entry. All three are embedded as static files - never variable fonts, which cosmic-text cannot instance.

### Hierarchy
- **Display** (Space Grotesk 700, 26px): wizard step headings only.
- **Headline** (Space Grotesk 700, 22px): page title in the top bar, under a breadcrumb caption.
- **Title** (Space Grotesk 700, 20px): empty-state headings.
- **Body** (Inter 400, 13px, `text-secondary`): descriptions, nav labels, banner messages. 14px for empty-state messages.
- **Body strong** (Inter 700, 13px): button labels, banner and toast titles, active nav label, user name.
- **Body small** (Inter 400, 12px): hints, field errors, toast messages, tooltips.
- **Label** (Inter 700, 11px, uppercase, `text-muted`): field labels, section captions, table headers, breadcrumbs.
- **Data** (JetBrains Mono 400/700, 13px): ICAO codes, flight numbers, callsigns, times, fuel, route strings, pagination counts. Larger mono (up to ~30px) for the flight-strip and detail-panel identifiers.
- **Clock** (JetBrains Mono 700, 17px) over a 10px mono uppercase date: the UTC clock.
- **Chip** (JetBrains Mono 700, 10px, uppercase): status chips, role chips, "SOON" markers.

### Named Rules
**The Ledger Column Rule.** If it is a code, a time, an ID or a number, it is set in JetBrains Mono. Prose is never set in mono; data is never set in Inter.

**The Zulu Rule.** Every displayed time is UTC and ends in `Z` (`10:00Z`, `2026-07-11 11:35Z`). Durations (block time) carry no suffix.

## Layout

A fixed desktop shell, default window 1440x900, minimum 1180x720. No responsive breakpoints; the content area stretches.

- **Sidebar** (236px fixed, `ops-sidebar`): brand lockup on top, nav grouped under uppercase captions (Operations, Airline, Instance) with 22px between groups and 2px between items, Settings and the user card pinned to the bottom.
- **Top bar** (`ops-night`, 12px x 24px padding): breadcrumb caption + page headline on the left; airline switcher, role chip, profile switcher, UTC clock and refresh on the right, controls 40px tall.
- **Content** (20px x 24px padding, 16px vertical rhythm): stat cards, filter toolbar, then a table with an optional detail panel on the right, or a form column with a sticky summary column.
- **Status bar** (`ops-sidebar`, 5px x 14px): connection dot, instance, versions, latency, sync time, URL - all in 11px mono separated by `|` rules.
- **Spacing scale:** 4, 8, 12, 16, 24px. Inside panels 16-18px padding, between fields 6px (label to input), between controls 8-10px.

**The Density Rule.** Rows are 38px, headers 36px, controls ~32-36px tall. Prefer more rows on screen over more whitespace.

## Elevation & Depth

The system is flat. Surfaces at rest are separated by tonal steps (sidebar darker than background, panels lighter, table header lighter still) and 1px `ledger-line` borders. Recessed wells reuse `ops-night` inside a panel to read as "sunk" without any shadow.

### Shadow Vocabulary
- **Dialog** (`0 12px 32px rgba(0,0,0,0.45)`): the wizard card floating on the background.
- **Toast** (`0 8px 24px rgba(0,0,0,0.5)`): transient notifications.
- **Menu** (`0 8px 20px rgba(0,0,0,0.45)`): pick-list dropdowns.
- **Tooltip** (`0 4px 12px rgba(0,0,0,0.4)`): icon-button tooltips.

### Named Rules
**The Floating-Only Shadow Rule.** Only things that float above the app (dialogs, toasts, menus, tooltips) cast a shadow. Panels, cards, tables and buttons never do.

## Shapes

Small, consistent radii and 1px strokes everywhere - the look of ruled paper and instrument bezels, not soft consumer cards.

- **Chip corner** (4px): status and role chips, segments, checkboxes, tooltips, scrollbar thumbs.
- **Control corner** (6px): buttons, inputs, pick lists, menus, nav items, banners, toasts, wells.
- **Panel corner** (8px): panels, cards and the table header's top edge.
- **Dialog corner** (10px): the floating wizard card.
- **Pill** (999px): icon discs, avatar, scroll-to-end button, status dot.
- Table rows are square; a selected row gains a 1px amber outline, no rounding.

## Components

### Buttons
Instrument-precise: compact, 1px bordered, bold 13px label with an optional 14px leading Lucide icon (8px gap), 8px x 14px padding.
- **Primary:** solid Dispatch Amber with On-Amber Ink. Lighter on hover, darker on press, umber when disabled.
- **Secondary:** Raised Slate with a Strong Line border and primary text; Hover Slate on hover.
- **Danger:** translucent red (15%) with a 60% red border and light-red text; turns solid red with white text on hover.
- **Ghost:** borderless icon button (16px icon, 7px padding) in `text-secondary`; Hover Slate fill on hover; always paired with a tooltip.
- **Link:** amber text, no chrome, lighter amber on hover.

### Segmented control
Separate 4px-radius segments (4px apart, 7px x 12px padding, 12px bold label) for status filters. Active segment is solid amber with ink text; inactive segments are Raised Slate with secondary text.

### Chips
Uppercase 10px bold mono, 2px x 7px padding, 4px radius, translucent tone fill + outline + light-tone text. Status mapping: Pending = warning, Accepted = success, Rejected = danger, other = neutral. The amber variant marks roles and highlights.

### Cards / Containers
- **Panel:** Console Slate, 1px Ledger Line, 8px radius, 18px padding, no shadow. A titled card leads with a 14px amber icon and an uppercase caption.
- **Stat card:** panel with a 40px icon disc tinted by its tone, uppercase caption and a large figure; the active filter's card gets an amber border.
- **Well:** Ops Night with a 1px Ledger Line, 6px radius - for route strings, departure/arrival blocks and code boxes inside panels.

### Inputs / Fields
- **Style:** Ops Night fill, 1px Strong Line border, 6px radius, 13px text, `text-placeholder` placeholders. Code-like inputs (ICAO, times, routes) use mono.
- **Focus:** border turns amber; selection is 35% amber.
- **Hover:** border lightens to Hover Line.
- **Error:** border turns danger red; a 12px alert icon plus light-red message sits under the field in place of the hint.
- **Field anatomy:** uppercase label, 6px gap, input, then hint (muted) or error.

### Navigation
Sidebar items: 16px icon + 13px label, 9px x 12px padding, 6px radius. Inactive is secondary text; hover fills Console Slate; active is amber text and icon (bold label) on a 14% amber wash. Unimplemented areas show a muted mono "SOON" marker. Entries the user lacks abilities for are hidden.

### Data table
The signature component. Raised Slate header (36px) with uppercase 11px captions and sort chevrons (amber when active), 38px rows alternating Console Slate and Zebra Slate, 12px horizontal cell padding, numbers right-aligned. Flight numbers in bold amber mono, other data in mono. Hover fills Hover Slate; selection is a 13% amber wash with a 1px amber outline. A footer carries the range ("1-3 of 3"), rows-per-page pick list and first/prev/page/next/last ghost buttons.

### Banners and toasts
- **Banner:** full-width tone box (translucent fill, 1px tone outline, 6px radius, 10px x 14px padding) with a 16px tone icon, bold title and 12px secondary message, optional trailing action.
- **Toast:** 360px, Raised Slate tinted 12% toward the tone, tone outline, toast shadow, stacked bottom-right (max 4, 10px apart). Success and info expire after 6 seconds; errors stay until dismissed.

### Flight strip
The PIREP form's live summary: a panel showing the flight number in large amber mono, callsign, departure and arrival codes with a plane glyph between them, and a key-value ledger of aircraft, cruise, block times and fuel. Empty values render as dash placeholders (`----`, `--:--Z`) so the strip keeps its shape while the form fills in.

### Empty states
Centered column, max 460px: 72px tinted icon disc, 20px title, 14px secondary message, optional actions. Used for empty, error, forbidden and not-yet-implemented screens.

## Do's and Don'ts

### Do:
- **Do** take every colour, font and style from `src/theme.rs`; no widget falls back to the default Iced look.
- **Do** set codes, times, IDs and figures in JetBrains Mono, and end every time with `Z`.
- **Do** separate surfaces with the slate tonal steps and 1px `ledger-line` borders.
- **Do** keep a single solid amber primary action per view.
- **Do** show status as translucent chips and banners using the light tone as foreground.
- **Do** give every nav item and primary action a Lucide icon.
- **Do** use uppercase 11px bold captions for labels and section titles.

### Don't:
- **Don't** put shadows on panels, cards, tables or buttons; shadows are for floating layers only.
- **Don't** introduce a second accent colour or use status colours for decoration.
- **Don't** use white text on amber; use On-Amber Ink.
- **Don't** fill large areas with a status colour.
- **Don't** use variable font files or add font families beyond the three embedded ones.
- **Don't** show local time anywhere.
- **Don't** add illustrations, gradients or playful ornament; the ledger plays it straight.
