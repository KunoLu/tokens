# Component Guidelines

> UI is shadcn v4 on `@base-ui/react` primitives with Tailwind v4 semantic
> tokens. Components are vendored, not imported from a package.

---

## Primitive pattern (`components/ui/`)

Follow `components/ui/button.tsx`:

- Wrap a `@base-ui/react` primitive (`import { Button as ButtonPrimitive } from "@base-ui/react/button"`).
- Variants via `cva` (`class-variance-authority`); merge classes with `cn()`
  from `@/lib/utils`.
- Set `data-slot` attributes for styling hooks.
- Export both the component and its variants (`export { Button, buttonVariants }`).

Same stack in `alert.tsx`, `table.tsx`, `card.tsx`, `dropdown-menu.tsx`.
Import from `@/components/ui/*` — never add another component library
(HeroUI was removed; styled-components is being phased out).

## Styling rules

Per `docs/upstream_policy.md`:

- **Semantic tokens only**: `bg-background`, `text-muted-foreground`,
  `border`, `bg-primary`, … No hardcoded hex, no manual `dark:` branches —
  both themes fall out of the tokens in `src/app/globals.css` (`@theme`).
  A missing token silently compiles to nothing (`bg-muted` broke this way
  after the HeroUI removal), so if a class does nothing, check the token
  exists.
- **Numbers use `.tabular`** so columns line up.
- Layout width comes from the shared `CONTAINER` constant
  (`max-w-[1200px] px-4 sm:px-6`) in `components/layout/Container.tsx`,
  reused by header, footer, and `app/error.tsx`.
- Page tops use `components/layout/PageHeader.tsx` (title + description);
  pages render `<main id="main-content">`.

## Feature components

- One folder per route/domain: `components/profile/`, `components/leaderboard/`,
  `components/shame/`, `components/docs/`, `components/legal/`.
- Large feature surfaces export a barrel (`components/profile/index.tsx`
  re-exports components + types).
- Big client orchestrators receive server-loaded data as props:
  `app/u/[username]/ProfilePageClient.tsx` takes `initialData` from the RSC
  page; `components/leaderboard/Leaderboard.tsx` syncs filters to the URL via
  `useRouter` + `useSearchParams`.
- Client components start with `"use client"` and stay as far down the tree
  as possible.

## `tw()`: legacy helper, not a pattern to extend

`lib/tw.tsx` (`tw('div', '…classes')`) is the migration leftover that replaced
styled-components in large client pages (used heavily in
`app/settings/SettingsClient.tsx`). Rules:

- Fine for simple repeated elements in an existing `tw()`-style file.
- Do **not** build variants by interpolating runtime strings into `tw()`
  classes — the comment in `tw.tsx` says to use a normal component with
  `cva`/`cn()` for anything conditional.
- New components default to plain function components + `cn()`; new
  styled-components usage is forbidden.
