import { drizzle, type PostgresJsDatabase } from "drizzle-orm/postgres-js";
import * as schema from "./schema";

/** The app's Drizzle client over the shared schema. */
export type DbClient = PostgresJsDatabase<typeof schema>;

/**
 * Self-hosted Node runtime: the Postgres URL always comes from DATABASE_URL.
 * (The Workers/Hyperdrive layer was removed in the self-host cutover.)
 */

function getConnectionString(): string {
  const connectionString = process.env.DATABASE_URL;

  if (!connectionString) {
    throw new Error("DATABASE_URL environment variable is not set");
  }

  return connectionString;
}

// TLS: off for a plain local Postgres, "require" for providers that need it;
// DATABASE_SSL toggles explicitly. Production defaults to require.
function resolveSsl(): "require" | false {
  const mode = process.env.DATABASE_SSL?.toLowerCase();
  if (mode === "disable" || mode === "false" || mode === "off") return false;
  if (mode === "require" || mode === "true" || mode === "on") return "require";
  return process.env.NODE_ENV === "production" ? "require" : false;
}

// One long-lived server process, so a single pool serves every request.
// Default 5 covers the widest page (/u/[username] fires three queries in
// parallel) with headroom; DATABASE_POOL_MAX overrides (clamped 1..5).
function poolMax(): number {
  const n = Number(process.env.DATABASE_POOL_MAX);
  if (Number.isInteger(n) && n >= 1 && n <= 5) return n;
  return 5;
}

// Use drizzle's config-based API to create the postgres client internally.
// Passing a `postgres` Sql instance directly causes type errors in the monorepo
// due to duplicate package resolution (two copies of postgres with incompatible
// branded types).
function createDb(): DbClient {

  return drizzle({
    connection: {
      url: getConnectionString(),
      ssl: resolveSsl(),
      max: poolMax(),

      // Close idle connections after 20 s so they don't linger between
      // infrequent invocations.
      idle_timeout: 20,

      // Hard cap: recycle every connection after 5 minutes regardless of
      // activity. Prevents stale connections after deploys / DB restarts.
      max_lifetime: 60 * 5,

      // Fail fast when the DB is unreachable instead of hanging the request.
      connect_timeout: 10,

      // Prepared statements are connection-scoped, and max_lifetime recycles
      // connections — keeping prepared statements across that boundary would
      // surface as "prepared statement does not exist".
      prepare: false,
    },
    schema,
  });
}



// Singleton: one pool per process. globalThis survives Next dev HMR reloads.
const globalForDb = globalThis as unknown as {
  _db: DbClient | undefined;
};

export function getDb(): DbClient {
  if (!globalForDb._db) {
    globalForDb._db = createDb();
  }
  return globalForDb._db;
}

export const db: DbClient = new Proxy({} as DbClient, {
  get(_target, prop) {
    const value = Reflect.get(getDb(), prop);
    return typeof value === "function" ? value.bind(getDb()) : value;
  },
});

export * from "./schema";
