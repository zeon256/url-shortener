import { For, Show, createEffect } from "solid-js";
import { PanelHeader } from "./PanelHeader";
import { RecentRow } from "./RecentRow";

export interface RecentEntry {
  id: string;
  originalUrl: string;
  shortUrl: string;
  shortCode: string;
  qrCode: string;
  createdAt: number;
  animate?: boolean;
}

export function RecentSection(props: {
  entries: RecentEntry[];
  expandedId: string | null;
  copiedId: string | null;
  onToggle: (id: string) => void;
  onCopy: (entry: RecentEntry) => void;
}) {
  let listEl: HTMLDivElement | undefined;

  createEffect(() => {
    const id = props.expandedId;
    if (!id || !listEl) return;
    const entry = props.entries.find((item) => item.id === id);
    if (entry?.animate) {
      listEl.scrollTop = 0;
    }
  });

  return (
    <Show when={props.entries.length > 0}>
      <section class="sticker w-full" aria-label="Recently shortened URLs">
        <PanelHeader title="Recent" />

        <div class="hidden grid-cols-[auto_minmax(0,1.2fr)_minmax(0,1fr)_auto] gap-3 border-b-2 border-ink bg-surface px-4 py-2 font-mono text-[0.65rem] uppercase tracking-[0.16em] text-ink-muted sm:grid sm:px-4">
          <span aria-hidden class="w-6" />
          <span>Original</span>
          <span>Short</span>
          <span class="text-right">Copy</span>
        </div>

        <div
          ref={(el) => {
            listEl = el;
          }}
          class={
            props.expandedId
              ? "h-72 overflow-y-auto overscroll-contain bg-surface"
              : "max-h-72 overflow-y-auto overscroll-contain bg-surface"
          }
        >
          <For each={props.entries}>
            {(entry) => (
              <RecentRow
                copied={props.copiedId === entry.id}
                entry={entry}
                expanded={props.expandedId === entry.id}
                onCopy={() => props.onCopy(entry)}
                onToggle={() => props.onToggle(entry.id)}
              />
            )}
          </For>
        </div>
      </section>
    </Show>
  );
}
