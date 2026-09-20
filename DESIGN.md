# dsh launcher design system

## Intent

A compact desktop launcher rather than a dashboard. The interface uses a dark split layout: configuration on the left, product identity and runtime controls on the right. Every visible state reflects a real local runtime condition.

## Color

- Page: `#0a0a0a`
- Right canvas: `#0b0e14`
- Primary text: `#ffffff`
- Secondary text: translucent white
- Brand / ready / restart: `#6799fe`
- Destructive emphasis: `#e8666b`
- Borders: translucent white

## Typography

Use the native sans-serif stack for interface copy and the native monospace stack for the compact product badge. Headings are restrained and use tight tracking; labels remain readable at the 780×500 default window size.

## Layout

- Default window: 780×500, minimum 720×460.
- Custom 54px frameless toolbar.
- 338px configuration column and flexible atmospheric launch area.
- Runtime status stays at the lower-left edge.
- Primary runtime actions stay at the lower-right edge.
- Below 700px, sections stack vertically.

## Components

- The language control is one transparent toolbar icon matching the window controls.
- Version fields use compact MUI selects; DSH version management opens a focused dialog.
- The idle runtime uses one white launch pill.
- After launch, the pill splits into a white stop button and a blue restart button.
- The blue ready indicator always includes status text and never relies on color alone.

## Motion

- Window entry: short opacity and scale transition.
- Language switch: icon rotates 360 degrees while translatable content fades, blurs, swaps, and returns.
- Runtime actions: the single launch pill expands and separates into two controls without moving its right-side anchor.
- Loading and stopping states preserve layout to prevent button jumps.
- `prefers-reduced-motion` collapses animations to near-instant transitions.

## Accessibility

- All icon-only controls have localized accessible names.
- Focus rings remain visible on keyboard navigation.
- Controls are disabled while an incompatible runtime transition is active.
- Status messages use `aria-live`.
