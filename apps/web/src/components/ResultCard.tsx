import { Show, createEffect, on } from "solid-js";
import { QrPreview } from "./QrPreview";

export function ResultCard(props: {
  shortUrl: string;
  qrCode: string;
  copied: boolean;
  onCopy: () => void;
}) {
  let sectionEl: HTMLDivElement | undefined;

  createEffect(
    on(
      () => props.shortUrl,
      (value) => {
        if (value && sectionEl) {
          sectionEl.scrollIntoView({ behavior: "smooth", block: "nearest" });
        }
      }
    )
  );

  return (
    <Show when={props.shortUrl}>
      {(current) => (
        <section
          ref={sectionEl}
          class="reveal-card mt-5 scroll-mt-20 rounded-card border border-sgds-primary-200 bg-primary-muted p-4 sm:p-6"
          aria-label="Shortened URL"
        >
          <div class="space-y-2.5">
            <a
              class="block break-all rounded-control bg-surface px-4 py-3 text-base font-semibold text-sgds-primary-800 underline-offset-4 transition hover:underline focus:outline-none focus:ring-4 focus:ring-sgds-primary-200 sm:text-lg"
              href={current()}
            >
              {current()}
            </a>
          </div>

          <div class="mt-4">
            <button
              class="press w-full rounded-control border border-primary bg-surface px-4 py-3 font-semibold text-primary transition hover:bg-sgds-primary-100 focus:outline-none focus:ring-4 focus:ring-sgds-primary-200"
              onClick={props.onCopy}
              type="button"
            >
              <Show when={props.copied} fallback={"Copy"}>
                <span class="inline-flex items-center justify-center gap-1.5">
                  Copied{" "}
                  <span class="text-primary" aria-hidden>
                    ✓
                  </span>
                </span>
              </Show>
            </button>
          </div>

          <Show when={props.qrCode}>
            {(qr) => <QrPreview url={current()} src={qr()} />}
          </Show>
        </section>
      )}
    </Show>
  );
}