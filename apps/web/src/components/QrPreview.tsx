export function QrPreview(props: { url: string; src: string }) {
  return (
    <div class="mt-5 flex items-center gap-4 rounded-control bg-surface p-4">
      <img
        alt={`QR code for ${props.url}`}
        class="h-28 w-28 shrink-0 rounded-lg border border-border bg-sgds-white p-2 sm:h-36 sm:w-36"
        height="144"
        src={props.src}
        width="144"
      />
      <p class="text-sm leading-6 text-ink-muted">
        Scan this QR code to open the short link on another device.
      </p>
    </div>
  );
}