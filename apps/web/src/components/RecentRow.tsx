import { Show } from "solid-js";
import { QrPreview } from "./QrPreview";
import { ScrambleCode } from "./ScrambleCode";
import type { RecentEntry } from "./RecentSection";

function displayOriginal(url: string) {
  return url.replace(/^https?:\/\//, "");
}

function shortLinkPrefix(shortUrl: string, shortCode: string) {
  const idx = shortCode ? shortUrl.lastIndexOf(shortCode) : -1;
  return idx > -1 ? shortUrl.slice(0, idx) : shortUrl;
}

export function RecentRow(props: {
  entry: RecentEntry;
  expanded: boolean;
  copied: boolean;
  onToggle: () => void;
  onCopy: () => void;
}) {
  const prefix = () => shortLinkPrefix(props.entry.shortUrl, props.entry.shortCode);

  return (
    <div
      class={
        props.expanded
          ? "border-2 border-accent-warm bg-accent-soft"
          : "border-b border-border"
      }
    >
      <div class="grid grid-cols-[auto_1fr_1fr_auto] items-center gap-2 px-3 py-2.5 font-mono text-xs sm:grid-cols-[auto_minmax(0,1.2fr)_minmax(0,1fr)_auto] sm:gap-3 sm:px-4 sm:text-sm">
        <button
          aria-expanded={props.expanded}
          aria-label={props.expanded ? "Collapse link details" : "Expand link details"}
          class="press grid h-6 w-6 shrink-0 place-items-center border border-ink bg-surface text-ink-muted"
          onClick={props.onToggle}
          type="button"
        >
          {props.expanded ? "▼" : "▶"}
        </button>

        <span class="truncate text-ink-muted" title={props.entry.originalUrl}>
          {displayOriginal(props.entry.originalUrl)}
        </span>

        <a
          class="truncate text-accent hover:underline"
          href={props.entry.shortUrl}
          title={props.entry.shortUrl}
        >
          {props.entry.shortUrl}
        </a>

        <button
          class="press shrink-0 border border-ink bg-surface px-2 py-1 text-[0.65rem] font-bold uppercase tracking-wider text-accent hover:bg-accent hover:text-white sm:px-2.5 sm:text-xs"
          onClick={(event) => {
            event.stopPropagation();
            props.onCopy();
          }}
          type="button"
        >
          <Show when={props.copied} fallback={"Copy"}>
            Copied
          </Show>
        </button>
      </div>

      <Show when={props.expanded}>
        <div class="border-t-2 border-ink bg-surface px-3 py-4 sm:px-4">
          <a
            class="block break-all font-mono text-base text-ink hover:text-accent sm:text-lg"
            href={props.entry.shortUrl}
          >
            <span class="text-ink-muted">{prefix()}</span>
            <Show
              when={props.entry.animate}
              fallback={
                <span class="font-bold text-accent">{props.entry.shortCode}</span>
              }
            >
              <ScrambleCode
                animate
                class="font-bold text-accent"
                code={props.entry.shortCode}
              />
            </Show>
          </a>

          <button
            class="press mt-3 w-full border-2 border-ink bg-surface-2 px-3 py-2 font-mono text-xs font-bold uppercase tracking-[0.14em] text-accent hover:bg-accent hover:text-white sm:text-sm"
            onClick={props.onCopy}
            type="button"
          >
            <Show when={props.copied} fallback={"Copy link"}>
              Copied ✓
            </Show>
          </button>

          <Show when={props.entry.qrCode}>
            {(qr) => (
              <QrPreview compact src={qr()} url={props.entry.shortUrl} />
            )}
          </Show>
        </div>
      </Show>
    </div>
  );
}
