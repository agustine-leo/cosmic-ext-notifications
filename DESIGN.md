---
name: cosmic-ext-notifications
description: COSMIC notification banners set as a Nordic railway timetable, with time first, then who, then what.
colors:
  timetable-navy: "#121c28"
  paper: "#f2f0ea"
  paper-dim: "rgba(242, 240, 234, 0.74)"
  slate: "#8c98a6"
  timetable-red: "#c8243a"
  column-rule: "#2a3647"
  lifted-navy: "#2a3647"
  pressed-navy: "#36455a"
typography:
  time-figures:
    fontFamily: "Cantarell, sans-serif"
    fontSize: "20px"
    fontWeight: 700
  title:
    fontFamily: "Cantarell, sans-serif"
    fontSize: "15px"
    fontWeight: 700
  body:
    fontFamily: "Cantarell, sans-serif"
    fontSize: "14px"
  action:
    fontFamily: "Cantarell, sans-serif"
    fontSize: "14px"
    fontWeight: 700
  app-label:
    fontFamily: "Cantarell, sans-serif"
    fontSize: "11px"
rounded:
  button: "4px"
  card: "6px"
spacing:
  xxs: "2px"
  xs: "4px"
  sm: "6px"
  md: "8px"
  icon-gap: "10px"
  card: "12px"
components:
  banner-card:
    backgroundColor: "{colors.timetable-navy}"
    textColor: "{colors.paper}"
    rounded: "{rounded.card}"
    width: "420px"
  time-column:
    textColor: "{colors.paper}"
    typography: "{typography.time-figures}"
    padding: "12px 0 12px 12px"
    width: "72px"
  time-column-low:
    textColor: "{colors.slate}"
  time-column-critical:
    backgroundColor: "{colors.timetable-red}"
    textColor: "{colors.paper}"
  button-action:
    textColor: "{colors.paper}"
    typography: "{typography.action}"
    rounded: "{rounded.button}"
    padding: "4px 8px"
  button-action-hover:
    backgroundColor: "{colors.lifted-navy}"
    textColor: "{colors.paper}"
  button-action-pressed:
    backgroundColor: "{colors.pressed-navy}"
    textColor: "{colors.paper}"
  button-dismiss:
    textColor: "{colors.slate}"
    rounded: "{rounded.button}"
    padding: "4px"
    size: "16px"
  button-dismiss-hover:
    backgroundColor: "{colors.lifted-navy}"
    textColor: "{colors.paper}"
---

# Design System: cosmic-ext-notifications

## Overview

**Creative North Star: "The Nordic Timetable"**

Each banner is one row of a printed Nordic departure board: a deep navy strip with paper-white type, read left to right as when, then who, then what. A fixed-width time column on the left carries the arrival time in bold figures and the sending app beneath it; a hairline rule divides it from the content, where the sender picture or app icon sits beside a bold title and a two-line message. Because the time column sits at the same x on every banner, a stack of banners reads as a single table rather than a pile of cards.

The system is flat, dense and quiet. There is one neutral surface, two text tones and one alarm colour; urgency changes the time column only, never the content. It deliberately departs from the rounded icon-tile notification card of macOS and GNOME, which this desktop's owner rejected as generic.

The surface is a native COSMIC layer-shell banner drawn with libcosmic/iced. Tokens are Rust constants in `src/banner.rs`; type is the desktop face (Cantarell) through `cosmic::font::bold()` and the theme default.

**Key Characteristics:**
- A 72px time column at a fixed x on every banner, so stacked banners align as one table.
- Time figures set in fixed cells (12px per digit, 6px for the colon) so columns of times line up although the face's figures are proportional.
- Navy strip, paper type, slate secondary, one red reserved for critical urgency.
- Flat: no shadow, no border, a 6px corner and a 1px column rule.
- Actions are bold text, never filled pills.

## Colors

A near-black navy ground with warm paper type, a cool slate for secondary information, and a single timetable red held back for critical notices.

### Primary
- **Timetable Red** (`timetable-red`): fills the whole time column of a critical-urgency banner, with paper figures on it. It appears nowhere else.

### Neutral
- **Timetable Navy** (`timetable-navy`): the banner surface. Every banner, every urgency.
- **Paper** (`paper`): title, time figures (normal and critical), action labels, the hovered dismiss glyph, and the default icon tint.
- **Paper Dim** (`paper-dim`, paper at 74% opacity): message body text.
- **Slate** (`slate`): app name under the time, time figures of low-urgency banners, the dismiss glyph at rest, disabled buttons.
- **Column Rule** (`column-rule`): the 1px vertical rule between time column and content. Hidden on critical banners, where the red fill makes the division.
- **Lifted Navy** (`lifted-navy`): hover fill of flat buttons. Same value as the column rule.
- **Pressed Navy** (`pressed-navy`): pressed fill of flat buttons.

### Named Rules
**The One Red Rule.** Timetable red marks critical urgency and nothing else. It never tints text, icons, buttons or borders.

**The Tier Lives In The Time Column Rule.** Urgency is shown only in the time column: low dims the figures to slate, normal sets them in paper, critical fills the column red. The content side looks the same at every tier.

## Typography

**Display Font:** Cantarell (the COSMIC desktop face), bold, for time figures
**Body Font:** Cantarell, theme default weight
**Label Font:** Cantarell, same family

**Character:** One face throughout, as a timetable uses one face; hierarchy comes from size, weight and the two paper tones.

### Hierarchy
- **Time figures** (700, 20px): `HH:MM`, 24-hour, zero-padded, at the top of the time column. Each digit is centred in a 12px cell and the colon in a 6px cell.
- **Title** (700, 15px): the notification summary, first line only, one line ellipsized, in paper.
- **Body** (theme default weight, 14px): the message, markup stripped and whitespace collapsed, at most two lines ellipsized, in paper dim. A zero-width space follows every space so wrapped lines keep a clean left edge.
- **Action** (700, 14px): action button labels, one line ellipsized.
- **App label** (theme default weight, 11px): app name under the time, one line ellipsized, in slate. Omitted when empty or when it would repeat the title.

### Named Rules
**The Fixed Cell Rule.** Times are always set in fixed per-character cells, never as a free text run, so every banner's figures and colon fall on the same verticals.

**The Nothing Twice Rule.** A label that repeats the title is dropped.

## Layout

Banners are 420px wide and stack top-centre, 8px apart. Each is one row of three parts: the time column (72px fixed, padded 12px top, left and bottom), the 1px column rule, and the content column (padded 12px, with 8px on the right so the dismiss button sits near the edge). In the content, the 32px icon sits 10px from the text block; the title and body are 2px apart; the activate row and the action row are 6px apart; the dismiss button sits 4px from the text. The action row is indented 34px (icon 32 + gap 10 − button padding 8) so the first action label lines up with the title. The banner's height follows its content; the time column fills that height.

Sender pictures sent as raster data are drawn at no more than their native size divided by the output scale, capped at 32px and centred in the 32px slot, so they never upscale and blur.

## Elevation & Depth

Flat. No shadows, no borders, no gradients. The navy against the desktop gives separation, and the column rule organises the inside. Depth shows only as state: flat buttons gain a lighter navy fill on hover and a lighter one again on press.

### Named Rules
**The Printed Sheet Rule.** A banner is a printed sheet: it never casts a shadow or carries an outline.

## Shapes

Every banner corner is 6px, fixed by the system rather than the theme so the compositor clip matches the drawn strip; the critical red fill takes the same 6px on its left corners only. Buttons round to 4px and show that shape only when hovered or pressed. The column rule is a straight 1px line from top to bottom of the banner.

## Components

### Buttons
Flat text on navy: nothing at rest, a lifted navy on hover, deeper on press.
- **Shape:** gently rounded (4px), visible only in hover and press states.
- **Action:** bold 14px paper label, padded 4px by 8px, up to three per banner, in the app's order, in a row with 4px gaps under the message. The default action is never shown as a button; clicking the picture or text triggers it.
- **Dismiss (×):** 16px close glyph drawn from an embedded SVG and tinted symbolically, slate at rest, padded 4px, at the top right of the content.
- **Hover / Pressed:** label goes paper, fill goes lifted navy, then pressed navy.
- **Disabled:** slate label, no fill.

### Banner Card (signature component)
- **Corner Style:** 6px.
- **Background:** timetable navy; icon and text default to paper.
- **Shadow Strategy:** none (see Elevation & Depth).
- **Structure:** time column, column rule, content column. Only the picture and text activate the notification; dismiss and actions sit outside that area.
- **Icon:** 32px. An app-sent raster wins over the app icon; an icon name the theme cannot resolve falls back to `dialog-information-symbolic` so the slot is never empty.

### Time Column (signature component)
- **Width:** 72px fixed, on every banner.
- **Content:** time figures, then the app label 2px below.
- **Tiers:** low has slate figures; normal has paper figures and slate label; critical has a red fill, paper figures, a paper label, and no column rule.

## Do's and Don'ts

### Do:
- **Do** keep the time column at 72px and at the same x on every banner; the stack must read as one table.
- **Do** set times as 24-hour `HH:MM` in fixed cells (12px digits, 6px colon).
- **Do** show urgency only in the time column: slate figures for low, paper for normal, a red fill for critical.
- **Do** keep actions as bold flat text buttons (4px radius, 4px by 8px padding) that fill only on hover and press.
- **Do** keep banners flat on timetable navy with a 6px corner and a 1px column rule.

### Don't:
- **Don't** use timetable red for anything except the critical time column.
- **Don't** add shadows, outlines or gradients to banners.
- **Don't** return to the rounded icon-tile notification card (macOS/GNOME style); it was rejected as generic.
- **Don't** fill action buttons at rest or style them as pills.
- **Don't** label anything twice: drop the app name when it equals the title.
