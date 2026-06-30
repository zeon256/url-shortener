import {
  defineRailway,
  project,
  service,
  postgres,
  image,
  github,
} from "railway/iac";

export default defineRailway(() => {
  const githubBranch = process.env.RAILWAY_GITHUB_BRANCH?.trim();
  const shortenerApiTag = process.env.SHORTENER_API_TAG?.trim() || "0.1.2";
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
      HOST: apiDomain,
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
    source: github(
      "zeon256/url-shortener",
      githubBranch ? { branch: githubBranch } : undefined,
    ),
    build: {
      builder: "DOCKERFILE",
      dockerfilePath: "apps/web/Dockerfile",
    },
    healthcheck: "/",
    healthcheckTimeout: 30,
    domains: [{ domain: webDomain, port: 8080 }],
    env: {
      VITE_BACKEND_URL: `https://${apiDomain}`,
    },
  });

  return project("url-shortener", { resources: [api, db, web] });
});
