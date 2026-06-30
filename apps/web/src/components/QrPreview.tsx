export function QrPreview(props: { url: string; src: string; compact?: boolean }) {
  return (
    <div
      class={
        props.compact
          ? "mt-3 flex items-center gap-3 border border-ink bg-surface-2 p-3"
          : "mt-4 flex items-center gap-4 border-2 border-ink bg-surface p-4"
      }
    >
      <div class="shrink-0 border-2 border-accent bg-white p-1.5">
        <img
          alt={`QR code for ${props.url}`}
          class={props.compact ? "block h-20 w-20" : "block h-28 w-28 sm:h-36 sm:w-36"}
          height="144"
          src={props.src}
          width="144"
        />
      </div>
      <p class="font-mono text-xs leading-6 text-ink-muted">
        Scan to open this short link on another device.
      </p>
    </div>
  );
}
