# syntax=docker/dockerfile:1

# Production image for the web app. Build context MUST be the repo root —
# the web build script copies install.sh / pre-install-tokens.* /
# .github/assets/client-* into web/public before `next build`.
#
#   docker build -f docs/deploy/docker/prod/Dockerfile .
#
# No DATABASE_URL (or any secret) is needed at build time: every page that
# touches the database renders on demand, and the leaderboard tolerates a
# missing DATABASE_URL (verified in the self-host cutover).

# ---- deps: frozen workspace install ----------------------------------------
FROM oven/bun:1 AS deps
WORKDIR /app
COPY package.json bun.lock ./
COPY web/package.json web/
# Workspace manifests are needed for `bun install --frozen-lockfile` to
# resolve the whole workspace graph. The CLI packages are npm distribution
# artifacts; their sources are never built or shipped in this image.
COPY packages/cli/package.json packages/cli/
COPY packages/cli-darwin-arm64/package.json packages/cli-darwin-arm64/
COPY packages/cli-darwin-x64/package.json packages/cli-darwin-x64/
COPY packages/cli-linux-arm64-gnu/package.json packages/cli-linux-arm64-gnu/
COPY packages/cli-linux-arm64-musl/package.json packages/cli-linux-arm64-musl/
COPY packages/cli-linux-x64-gnu/package.json packages/cli-linux-x64-gnu/
COPY packages/cli-linux-x64-musl/package.json packages/cli-linux-x64-musl/
COPY packages/cli-win32-arm64-msvc/package.json packages/cli-win32-arm64-msvc/
COPY packages/cli-win32-x64-msvc/package.json packages/cli-win32-x64-msvc/
RUN bun install --frozen-lockfile

# ---- build: next build (also stages install.sh / client assets into public/)
FROM deps AS build
COPY . .
RUN bun run --cwd web build

# ---- runner: minimal runtime surface ----------------------------------------
FROM oven/bun:1 AS runner
WORKDIR /app
ENV NODE_ENV=production
# Root workspace node_modules (bun hoists there) + the whole web/ tree
# (.next, public, node_modules, next.config.ts, drizzle.config.ts + the
# migration SQL, so `bun run --cwd web db:migrate` works inside the
# container for first-boot and upgrade migrations).
COPY --from=build --chown=bun:bun /app/node_modules ./node_modules
COPY --from=build --chown=bun:bun /app/web ./web
USER bun
EXPOSE 3000
CMD ["bun", "run", "--cwd", "web", "start", "--", "-p", "3000"]
