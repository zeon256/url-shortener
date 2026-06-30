import { Show } from "solid-js";

export function DuplicateNotice(props: {
  shortUrl: string;
  copied: boolean;
  onCopy: () => void;
}) {
  return (
    <div
      class="rounded-control border-2 border-accent bg-accent-soft px-4 py-3"
      role="status"
    >
      <p class="font-mono text-xs text-ink-muted">
        Already shortened — same link.
      </p>
      <div class="mt-2 flex flex-wrap items-center gap-2 sm:gap-3">
        <a
          class="min-w-0 flex-1 break-all font-mono text-sm font-semibold text-accent hover:underline"
          href={props.shortUrl}
        >
          {props.shortUrl}
        </a>
        <button
          class="press shrink-0 rounded-control border border-ink bg-surface px-3 py-1.5 font-mono text-xs font-bold uppercase tracking-wider text-accent hover:bg-accent hover:text-white"
          onClick={props.onCopy}
          type="button"
        >
          <Show when={props.copied} fallback={"Copy"}>
            Copied
          </Show>
        </button>
      </div>
    </div>
  );
}
