import { render } from "solid-js/web";
import { Show, createEffect, createSignal, onMount } from "solid-js";
import "./app.css";

import { AppBar, type NavStatus } from "./components/AppBar";
import { Hero } from "./components/Hero";
import { ShortenerForm } from "./components/ShortenerForm";
import { DuplicateNotice } from "./components/DuplicateNotice";
import { PanelHeader } from "./components/PanelHeader";
import { RecentSection, type RecentEntry } from "./components/RecentSection";
import { shortenUrl } from "./lib/shorten";
import { createQrCode } from "./lib/qrcode";
import { checkHealth } from "./lib/health";
import {
  readCachedHealth,
  readCachedHistory,
  toHistoryRecords,
  writeCachedHealth,
  writeCachedHistory,
} from "./lib/session-cache";
import {
  bumpHistoryRecord,
  createHistoryId,
  findByOriginalUrl,
  loadHistory,
  saveHistoryRecord,
  type HistoryRecord,
  HISTORY_MAX_ENTRIES,
} from "./lib/history";

type RequestStatus = "idle" | "loading" | "success" | "error";

function withQrCodes(records: HistoryRecord[]): RecentEntry[] {
  return records.map((record) => ({ ...record, qrCode: "" }));
}

async function attachQrCodes(entries: RecentEntry[]) {
  const withCodes = await Promise.all(
    entries.map(async (entry) => ({
      ...entry,
      qrCode: await createQrCode(entry.shortUrl),
    }))
  );
  return withCodes;
}

function App() {
  const [url, setUrl] = createSignal("");
  const [status, setStatus] = createSignal<RequestStatus>("idle");
  const [history, setHistory] = createSignal(withQrCodes(readCachedHistory()));
  const [error, setError] = createSignal("");
  const [copiedId, setCopiedId] = createSignal<string | null>(null);
  const [expandedId, setExpandedId] = createSignal<string | null>(null);
  const [duplicateNotice, setDuplicateNotice] = createSignal<RecentEntry | null>(
    null
  );
  const [apiHealth, setApiHealth] = createSignal(readCachedHealth() ?? true);

  createEffect(() => {
    writeCachedHistory(toHistoryRecords(history()));
  });

  const navStatus = (): NavStatus => {
    if (status() === "loading") return "working";
    if (status() === "error") return "error";
    return apiHealth() ? "online" : "offline";
  };

  onMount(async () => {
    const online = await checkHealth();
    setApiHealth(online);
    writeCachedHealth(online);

    try {
      const records = await loadHistory();
      const entries = withQrCodes(records);
      setHistory(entries);

      attachQrCodes(entries)
        .then((withCodes) => {
          const qrById = new Map(withCodes.map((entry) => [entry.id, entry.qrCode]));
          setHistory(
            history().map((entry) =>
              entry.qrCode === "" && qrById.has(entry.id)
                ? { ...entry, qrCode: qrById.get(entry.id)! }
                : entry
            )
          );
        })
        .catch(() => {
          // Rows still work without QR previews.
        });
    } catch {
      // Keep session cache snapshot if IndexedDB is unavailable.
    }
  });

  const liveMessage = () => {
    if (duplicateNotice()) return "This URL was already shortened.";
    if (status() === "loading") return "Creating a short URL.";
    if (status() === "success") return "Short URL ready.";
    if (status() === "error") return error();
    return "";
  };

  async function promoteExisting(existing: RecentEntry) {
    const record: HistoryRecord = {
      id: existing.id,
      originalUrl: existing.originalUrl,
      shortUrl: existing.shortUrl,
      shortCode: existing.shortCode,
      createdAt: Date.now(),
    };

    try {
      await bumpHistoryRecord(record);
    } catch {
      // Keep working in memory if storage fails.
    }

    setCopiedId(null);
    setExpandedId(null);
    setDuplicateNotice(existing);
    setHistory([
      { ...existing, createdAt: record.createdAt, animate: false },
      ...history().filter((entry) => entry.id !== existing.id),
    ].slice(0, HISTORY_MAX_ENTRIES));
    setStatus("success");
  }

  async function handleSubmit() {
    const fullUrl = `https://${url()}`;

    const inMemory = history().find((entry) => entry.originalUrl === fullUrl);
    if (inMemory) {
      await promoteExisting(inMemory);
      return;
    }

    setStatus("loading");
    setError("");
    setCopiedId(null);
    setDuplicateNotice(null);

    try {
      let record = await findByOriginalUrl(fullUrl);

      if (record) {
        const existing: RecentEntry = {
          ...record,
          qrCode: await createQrCode(record.shortUrl),
        };
        await promoteExisting(existing);
        return;
      }

      const result = await shortenUrl(fullUrl);
      const qrCode = await createQrCode(result.shortUrl);
      record = {
        id: createHistoryId(),
        originalUrl: fullUrl,
        shortUrl: result.shortUrl,
        shortCode: result.shortCode,
        createdAt: Date.now(),
      };

      try {
        await saveHistoryRecord(record);
      } catch {
        // Still show the result if storage fails.
      }

      const entry: RecentEntry = { ...record, qrCode, animate: true };
      setExpandedId(entry.id);
      setHistory([entry, ...history()].slice(0, HISTORY_MAX_ENTRIES));
      setStatus("success");

      const entryId = entry.id;
      window.setTimeout(() => {
        setHistory(
          history().map((item) =>
            item.id === entryId ? { ...item, animate: false } : item
          )
        );
      }, entry.shortCode.length * 70 + 250);
    } catch (caughtError) {
      setStatus("error");
      setError(
        caughtError instanceof Error
          ? caughtError.message
          : "Unable to shorten this URL."
      );
    }
  }

  function handleInput(value: string) {
    setUrl(value);
    setDuplicateNotice(null);
    if (status() === "error") {
      setStatus("idle");
      setError("");
    }
  }

  function toggleExpanded(id: string) {
    setExpandedId((current) => (current === id ? null : id));
  }

  async function copyShortUrl(entry: RecentEntry) {
    try {
      await navigator.clipboard.writeText(entry.shortUrl);
      setCopiedId(entry.id);
      window.setTimeout(() => setCopiedId(null), 2200);
    } catch {
      setStatus("error");
      setError("Copy failed. Select the short URL and copy it manually.");
    }
  }

  return (
    <div class="grid-atmos flex min-h-svh flex-col">
      <AppBar status={navStatus()} />

      <main class="flex-1">
        <section class="mx-auto flex w-full max-w-2xl flex-col items-center gap-8 px-4 pt-6 pb-20 sm:px-6 sm:pt-10 lg:px-8">
          <Hero />

          <div class="sticker w-full">
            <PanelHeader title="Input" />

            <div class="p-4 sm:p-6">
              <ShortenerForm
                url={url()}
                status={status()}
                error={error()}
                onInput={handleInput}
                onSubmit={handleSubmit}
              />

              <Show when={duplicateNotice()}>
                {(entry) => (
                  <div class="mt-4">
                    <DuplicateNotice
                      copied={copiedId() === entry().id}
                      shortUrl={entry().shortUrl}
                      onCopy={() => copyShortUrl(entry())}
                    />
                  </div>
                )}
              </Show>

              <p aria-live="polite" class="sr-only">
                {liveMessage()}
              </p>
            </div>
          </div>

          <RecentSection
            copiedId={copiedId()}
            entries={history()}
            expandedId={expandedId()}
            onCopy={copyShortUrl}
            onToggle={toggleExpanded}
          />
        </section>
      </main>

      <footer class="border-t-2 border-border bg-surface/50 px-4 py-5 sm:px-6 lg:px-8">
        <div class="mx-auto flex w-full max-w-3xl items-center justify-center">
          <a
            class="press inline-flex items-center gap-1.5 font-mono text-xs text-ink-muted transition hover:text-accent"
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
