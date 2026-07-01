import { defineRailway, project, service, postgres, image } from "railway/iac";

const requireEnv = (name: string): string => {
  const value = process.env[name]?.trim();
  if (!value) {
    throw new Error(`${name} is required`);
  }
  return value;
};

export default defineRailway(() => {
  const shortenerApiTag = requireEnv("SHORTENER_API_TAG");
  const shortenerWebTag = requireEnv("SHORTENER_WEB_TAG");
  const apiDomain = process.env.SHORTENER_API_DOMAIN?.trim() || "s.inve.rs";
  const webDomain =
    process.env.SHORTENER_WEB_DOMAIN?.trim() || "shorter.inve.rs";

  const db = postgres("Postgres");

  const api = service("shortener-api", {
    source: image(`ghcr.io/zeon256/shortener-api:${shortenerApiTag}`),
    healthcheck: "/healthz",
    healthcheckTimeout: 300,
    domains: [{ domain: apiDomain, port: 8000 }],
    env: {
      PORT: "8000",
      DISALLOWED_HOSTS: `${apiDomain},${webDomain}`,
      CORS_ALLOWED_ORIGINS: `https://${webDomain}`,
      POSTGRES_HOST: db.env.PGHOST,
      POSTGRES_PORT: db.env.PGPORT,
      POSTGRES_USER: db.env.PGUSER,
      POSTGRES_PASSWORD: db.env.PGPASSWORD,
      POSTGRES_DB: db.env.PGDATABASE,
      POSTGRES_ACQUIRE_TIMEOUT: "30",
    },
  });

  const web = service("web", {
    source: image(`ghcr.io/zeon256/shortener-web:${shortenerWebTag}`),
    healthcheck: "/",
    healthcheckTimeout: 30,
    domains: [{ domain: webDomain, port: 8080 }],
    env: {
      VITE_BACKEND_URL: `https://${apiDomain}`,
    },
  });

  return project("url-shortener", { resources: [api, db, web] });
});
