/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Base URL of the backend shortener API. Defaults to http://localhost:4002. */
  readonly VITE_BACKEND_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
