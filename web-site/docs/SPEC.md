# zkVM Benchmarks Website Specification

## Overview

The zkVM Benchmarks website (`zk-wars.blocksense.network`) presents performance data for various Zero-Knowledge Virtual Machines (zkVMs). It enables technical users (developers, engineers, researchers) to quickly determine the best-performing toolchain and hardware configuration for specific cryptographic benchmarks.

## Audience

* Primary: Developers, engineers, researchers in ZK technology.
* Secondary: Accessible to non-technical users interested in performance comparisons.

## User Goals

* Quickly identify the best-performing toolchain overall.
* Evaluate toolchain performance for predefined and custom cryptographic benchmarks.
* Determine optimal hardware configurations.

## Key Features

### Benchmark Data Display

For each toolchain and benchmark program, the website displays:

* Proof generation time
* Proof generation memory usage
* Proof verification time
* Proof verification memory usage
* Circuit compilation time
* Circuit compilation memory usage
* Proof size

Results are presented per hardware configuration. Cold/hot executions are reported separately when available.

### Data Visualization Components

* **Ranked Horizontal Bar Chart**: Best-performing toolchains listed clearly with visual bar representations.
* **Performance Metrics Table**: Numeric values with background-colored cells acting as proportional bars.
* **Line Charts**: Visualize performance trends based on input variables.

### Benchmark Definition

#### Predefined Benchmarks

Examples include:

* Merkle Tree generation
* Merkle proof verification
* ECDSA signature verification

#### Custom Comparison Builder

* Users select primitives (e.g., signature verification, hashing) and specify execution counts.
* UI is analogous to an online shopping cart:

  * Users select primitives via an auto-complete search box or hierarchical browsing.
  * Primitives organized in categories (e.g., Hashing, Signatures, Arithmetic Operations, Symmetric Encryption).

### Combined Comparison Calculation

* Metrics are calculated as a simple linear sum of individual primitives.
* Results displayed as aggregated metrics, with detailed breakdown accessible via tap-to-expand interaction.
* Detailed breakdown shows percentage contributions for each primitive.

## UI/UX Considerations

### Interaction Model

* Tap-to-expand interactions for detailed breakdowns (desktop and mobile).
* Dropdown menus for selecting comparison metrics (proof generation time, verification time, both).

### Tables and Charts

* Numeric tables use cell backgrounds to visually indicate performance proportionally.
* Combined metric charts use color-coded bars to indicate different contributions (generation vs verification).

### Hardware Configuration

* Predefined hardware presets (limited set).
* Detailed hardware specs shown upon selection.

### Navigation Structure

Hybrid approach:

* SPA-like dynamic interactions with deep-linked, SEO-friendly URLs.
* Descriptive path segments for benchmarks, toolchain-specific pages, hardware-specific pages.

### URL Structure

* Benchmark pages: `zk-wars.blocksense.network/benchmarks/<use-case>?hardware=<preset>&hash=<hash>&metric=<metric>`
* Custom benchmarks: `zk-wars.blocksense.network/benchmarks/custom?data=<encoded-state>`
* Toolchain pages: `zk-wars.blocksense.network/toolchains/<toolchain-name>?hardware=<preset>`
* Hardware pages: `zk-wars.blocksense.network/hardware/<preset-name>`
* Hardware comparisons: `zk-wars.blocksense.network/hardware/<preset-name>/vs/<second-preset-name>`

### URL State Management

* Dropdown selection changes use `replaceState` (no new history entry).
* Navigating between pages or comparisons uses `pushState`.

### Shared URLs

* Users can share/bookmark benchmarks via a generated URL encoding the UI state.
* Invalid URL parameters load defaults with subtle warning icons next to affected UI components.

### Responsive Design

* Fully responsive; all data/features available on both desktop and mobile.
* Tables adapt by stacking columns/rows vertically on smaller screens.

## Technical Implementation

### Frontend

* Next.js (React framework).
* Server-side rendering (SSR) or Static Site Generation (SSG) for SEO and performance.
* Client-side JSON fetching for dynamic data interactions.

### Data Management

Benchmark data stored as structured JSON files with the following schema:

```json
{
  "benchmarking": [{
    "name": "<benchmark-program-name>",
    "compile": {
      "timeStarted": "<ISO-timestamp>",
      "runs": <number-of-runs>,
      "totalDuration": <total-duration-seconds>,
      "mean": <mean-duration-seconds>,
      "deviation": <standard-deviation-seconds>,
      "min": <min-duration-seconds>,
      "max": <max-duration-seconds>,
      "memory": <memory-usage-bytes>,
      "size": <proof-or-circuit-size-in-bytes>
    },
    "prove": { <same-as-compile> },
    "verify": { <same-as-compile> }
  }],
  "hardware": {
    "cpu": [{"model": "<cpu-model>", "cores": <cores-count>, "speed": <speed-GHz>}],
    "memory": {"model": "<memory-model>", "size": <size-bytes>, "speed": <speed-MHz>},
    "hardwareAcceleration": [{"model": "<gpu-model>", "cores": <cores-count>, "speed": <speed-MHz>}],
    "accelerated": false
  }
}
```

In the static build of the web-site. The client will have a pre-populated cache with all benchmark
data available without making requests to the server.

### Development Dependencies

* Managed via Nix, provided as a Nix flake.

### Hosting & Deployment

* Hosted statically via Cloudflare.
* Consider exporting the complete site as static HTML files for performance.

## Performance and Accessibility Goals

* Extremely fast initial page loads (target ≤1s).
* Compliance with WCAG 2.1 AA standards.
* Compatible with all major desktop/mobile browsers.

## Branding & Visual Design

* Matches existing BlockSense Network branding (see `https://blocksense.network`).
* Domain: `zk-wars.blocksense.network`.

## Scalability & Maintenance

* Adding new hardware configurations or benchmarks is straightforward (structured JSON updates).
* Initially manageable dataset size; potential infinite scrolling if datasets grow significantly.

---

# Detailed Page & Visual Design

> **Branding Note** – All visual examples assume the BlockSense design language (see [https://blocksense.network](https://blocksense.network)). **Primary font is `Geist`** for all headings and body copy, backed by `sans-serif`. Example palette (confirm with latest brand assets): primary accent `#2F80ED`, secondary accent `#9747FF`, dark text `#1B1F23`, light text `#6B7280`, background `#F9FAFB`.

## Shared Design Tokens

| Token        | Usage                 | Example                                    |
| ------------ | --------------------- | ------------------------------------------ |
| Font family  | Primary text          | `Geist`, 1.2 rem base, 1.5 rem line‑height |
| Radius‑md    | Card & button corners | `0.75 rem`                                 |
| Elevation‑1  | Card shadow           | `0 2px 6px rgba(0,0,0,.06)`                |
| Spacing‑unit | Base grid             | `0.5 rem`                                  |

All pages use a **12‑column fluid grid** (`max‑width: 1440px`) with generous whitespace. Cards always span full width on ≤ 768 px screens and collapse to an 8‑column grid on tablets.

---

## 1. Landing / "Benchmarks Overview" Page

**Purpose** – Entry point; lets users pick a predefined benchmark or jump to custom builder.

### Layout

1. **Hero bar** (full‑width, gradient accent): title *"Blocksense ZK Wars"*, subtitle "Your performance guide to the ZK world", CTA buttons:

   * "Browse Standard Benchmarks" → scrolls to benchmark list
   * "Build Custom Comparison" → `/benchmarks/custom`
2. **Benchmark cards grid** – 8 predefined benchmarks in a `Card` component (image/emoji, title, short description). 3‑column desktop ▸ 1‑column mobile. A card named "More" that leads to `/benchmarks/`
3. **Featured Rankings** – horizontal bar chart comparing top 5 toolchains for the default hardware preset & an aggregated set of benchmarks; embedded in a card with dropdowns (metric & hardware). Two charts will be displayed (best on CPU and best on GPU).
4. **Footer** – links, GitHub repo badge, BlockSense branding.

### Interactions

* Benchmark card click → `/benchmarks/<slug>`
* Dropdown changes update chart via `replaceState`.

### Visual Notes

* Cards use Elevation‑1 shadow; on hover desk‑top card lifts (`translate‑y‑1`, shadow‑lg). Mobile: subtle scale‑in.

---

## 2. Predefined Benchmark Detail Page `/benchmarks/<slug>`

### Header

* Breadcrumb: Home ▸ Benchmark ▸ *Merkle Proof Verification*
* Title; dropdowns for **hardware** + **metric** (proof‑gen / verify / both).

### Main Section

| Column          | Content                                                                                                                                                                                                                      |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 8‑col (desktop) | **Performance Table** – toolchain in first column; proof times in second; cell background renders proportional bar (best = 100%). Third narrow column optional \:info: icon opens side drawer with raw JSON snippet for that row. |
| 4‑col (desktop) | **Benchmark Info Card** – text summary, primitive list & counts, link "Re‑mix in Builder".                                                                                                                                    |

### Expanded Row

* Clicking a toolchain row accordion reveals a horizontal stacked bar (gen vs verify) + memory & proof size chips.

### Mobile

* Table transforms: each toolchain becomes a `Disclosure` card; metrics stack vertically; bars shrink to 100 % width of card.

---

## 3. Custom Comparison Builder `/benchmarks/custom`

### Builder Panel (left on desktop, top on mobile)

* **Primitive Search** (command‑palette style): type to filter, ↵ to add.
* **Cart List** – each primitive row: name, qty stepper, subtotal metric preview badge.
* **Hardware & Metric dropdowns** at bottom.
* Sticky "Update Results" button triggers recalculation & URL `replaceState`.

### Results Panel (right on desktop, below on mobile)

* Aggregated result card: big number for total proof time; stacked bar showing breakdown (hover/tap segments → tooltip with % and ms).
* Full results table identical to Benchmark Detail.

### UX Notes

* Builder uses slide‑in side sheet on ≤ 1024 px to maximise space.

---

## 4. Toolchain Detail Page `/toolchains/<name>`

### Header

* Avatar/logo, repository link, commit SHA, hardware dropdown.

### Body

1. **Benchmark List Table** – each benchmark row: benchmark icon, metric bar cell, comparison badge "+12 % vs best".
2. **Filter chip bar** – filter by primitive category (hashing, signatures …).

### Expansion

* Row expand shows line chart of performance vs input size (if data). Chart built with Recharts; accessible via keyboard.

---

## 5. Hardware Detail Page `/hardware/<preset>`

### Masthead Card

* Hardware name, specs grid (CPU, RAM, optional GPU). Button "Compare with…" → opens modal to pick second preset (`pushState`).

### Aggregated Rankings Section

* Horizontal bar ranking of toolchain totals across all benchmarks.

### Benchmark Tabs

* Tab list of benchmark categories; contents: same table pattern.

---

## 6. Hardware Comparison Page `/hardware/<a>/vs/<b>`

### Split‑view Layout

* Sticky header showing A vs B specs side‑by‑side.
* **Comparison Table**: rows = benchmark categories; two metric cells with background bars; difference column coloured subtle gray. Expandable accordions show per‑benchmark table (bars again) identical to row pattern. Multiple sections can be expanded concurrently.

---

## 7. Global Components

* **Warning Icon** (invalid URL param): outline ⚠ icon next to dropdown; tooltip reveals issue.
* **Share Button**: Icon button copies current canonical URL; toast “Link copied!”.
* **Loading Skeletons**: shimmer rectangles mimic bar charts & table cells.

---

# Implementation Checklist (Visual)

* [ ] Design tokens defined in Tailwind config.
* [ ] Components scaffolded with shadcn/ui.
* [ ] Charts implemented with Recharts + accessible labels.
* [ ] Mobile breakpoints tested ≤ 360 px wide.
* [ ] Colour contrast verified (WCAG 2.1 AA).

---

*End of visual design section.*
