import { render } from "solid-js/web";
import { createSignal } from "solid-js";
import "./app.css";

import { AppBar } from "./components/AppBar";
import { Hero } from "./components/Hero";
import { ShortenerForm } from "./components/ShortenerForm";
import { ResultCard } from "./components/ResultCard";
import { shortenUrl } from "./lib/shorten";
import { createQrCode } from "./lib/qrcode";

type RequestStatus = "idle" | "loading" | "success" | "error";

function App() {
  const [url, setUrl] = createSignal("");
  const [status, setStatus] = createSignal<RequestStatus>("idle");
  const [shortUrl, setShortUrl] = createSignal("");
  const [qrCode, setQrCode] = createSignal("");
  const [error, setError] = createSignal("");
  const [copied, setCopied] = createSignal(false);

  const liveMessage = () => {
    if (status() === "loading") return "Creating a short URL.";
    if (status() === "success") return "Short URL created.";
    if (status() === "error") return error();
    return "";
  };

  async function handleSubmit() {
    setStatus("loading");
    setShortUrl("");
    setQrCode("");
    setError("");
    setCopied(false);

    try {
      const { shortUrl: nextShortUrl } = await shortenUrl(url());
      setShortUrl(nextShortUrl);
      setQrCode(await createQrCode(nextShortUrl));
      setStatus("success");
    } catch (caughtError) {
      setStatus("error");
      setError(
        caughtError instanceof Error
          ? caughtError.message
          : "Unable to shorten this URL."
      );
    }
  }

  async function copyShortUrl() {
    const value = shortUrl();
    if (value === "") return;

    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2200);
    } catch {
      setStatus("error");
      setError("Copy failed. Select the short URL and copy it manually.");
    }
  }

  return (
    <div class="dot-grid flex min-h-svh flex-col">
      <AppBar status={status()} />

      <main class="flex-1">
        <section class="mx-auto flex w-full max-w-2xl flex-col items-center gap-8 px-4 pt-12 pb-16 sm:px-6 sm:pt-20 lg:px-8">
          <Hero />

          <div class="w-full rounded-card border border-border bg-surface p-5 shadow-card sm:p-7">
              <ShortenerForm
                url={url()}
                status={status()}
                error={error()}
                onInput={setUrl}
                onSubmit={handleSubmit}
              />

              <p aria-live="polite" class="sr-only">
                {liveMessage()}
              </p>

              <ResultCard
                shortUrl={shortUrl()}
                qrCode={qrCode()}
                copied={copied()}
                onCopy={copyShortUrl}
              />
            </div>
        </section>
      </main>

      <footer class="border-t border-border bg-surface/60 px-4 py-5 sm:px-6 lg:px-8">
        <div class="mx-auto flex w-full max-w-3xl items-center justify-center">
          <a
            class="press inline-flex items-center gap-1.5 text-xs text-ink-muted transition hover:text-primary"
            href="https://github.com/zeon256/url-shortener"
            rel="noreferrer"
            target="_blank"
          >
            <svg
              aria-hidden
              class="h-4 w-4"
              fill="currentColor"
              viewBox="0 0 16 16"
            >
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z" />
            </svg>
            zeon256/url-shortener
          </a>
        </div>
      </footer>
    </div>
  );
}

const root = document.getElementById("root");
if (root === null) throw new Error("root element not found");
render(() => <App />, root);