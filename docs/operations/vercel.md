# Deploying docs on Vercel

Feather's public documentation lives in [`apps/docs`](../../apps/docs) (Fumadocs + Next.js). Only this app is deployed to Vercel — not the dashboard, server, or SDK UI.

## Create a Vercel project

1. Import the GitHub repo `Arbtrage/Feather`
2. Set **Root Directory** to `apps/docs`
3. Framework preset: **Next.js**
4. Build command: `npm run build` (default)
5. Install command: `npm install`

## Custom domain

In Vercel → Project → Settings → Domains, add e.g. `docs.feather.dev`.

## Environment variables (optional)

| Variable | Example | Purpose |
|----------|---------|---------|
| `NEXT_PUBLIC_GITHUB_URL` | `https://github.com/Arbtrage/Feather` | Navbar link override |

No secrets are required — the docs site is fully static.

## Local preview

```bash
cd apps/docs
npm install
npm run dev
# → http://localhost:3002/docs
```

## Content source

Markdown lives in repo-root [`docs/`](../../docs/). Fumadocs reads it via `source.config.ts` (`dir: "../../docs"`). Edit markdown + `meta.json` sidebar files; no GitBook sync.

## Preview deployments

Vercel automatically builds preview URLs for pull requests when the project is connected to GitHub.
