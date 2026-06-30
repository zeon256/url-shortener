import { Show } from "solid-js";

type Status = "idle" | "loading" | "success" | "error";

const PROTOCOL = /^https?:\/\//i;

// The prefix chip shows https://, so we strip a pasted/typed http(s)://
// from the value before storing it; the API receives the full URL rebuilt
// from the chip + input.
export function stripProtocol(value: string) {
  return value.replace(PROTOCOL, "");
}

export function isValidUrl(value: string) {
  if (value.trim() === "") return false;
  try {
    // The chip supplies https://, so the bare value must be a valid host/path.
    const parsed = new URL(`https://${value}`);
    return Boolean(parsed.hostname);
  } catch {
    return false;
  }
}

export function ShortenerForm(props: {
  url: string;
  status: Status;
  error: string;
  onInput: (value: string) => void;
  onSubmit: () => void;
}) {
  const isLoading = () => props.status === "loading";
  const canSubmit = () => !isLoading() && isValidUrl(props.url);

  return (
    <form
      class="space-y-4"
      onSubmit={(event) => {
        event.preventDefault();
        if (canSubmit()) props.onSubmit();
      }}
    >
      <div class="space-y-2">
        <label class="block space-y-2" for="long-url">
          <span class="block font-mono text-[0.7rem] uppercase tracking-[0.18em] text-ink-muted">
            Original URL
          </span>

          <div
            class="
              group flex items-stretch overflow-hidden border-2 border-ink
              bg-surface transition focus-within:border-accent
              focus-within:shadow-[0_0_0_2px_var(--color-accent)]
              has-[input:disabled]:opacity-60
            "
          >
            <span
              class="grid select-none place-items-center border-r-2 border-ink bg-accent-soft px-3 font-mono text-sm text-accent"
              aria-hidden
            >
              https://
            </span>
            <input
              aria-describedby="url-help url-error"
              class="min-w-0 flex-1 bg-transparent px-4 py-3 font-mono text-base text-ink outline-none placeholder:text-ink-muted/50"
              disabled={isLoading()}
              id="long-url"
              inputMode="url"
              onInput={(event) =>
                props.onInput(stripProtocol(event.currentTarget.value))
              }
              onPaste={(event) => {
                const data = event.clipboardData;
                const text = data?.getData("text") ?? "";
                if (PROTOCOL.test(text)) {
                  event.preventDefault();
                  props.onInput(stripProtocol(text));
                }
              }}
              placeholder="example.gov.sg/service"
              required
              type="text"
              value={props.url}
            />
          </div>
        </label>

        <p class="font-mono text-xs text-ink-muted" id="url-help">
          Enter the address. We add https:// for you.
        </p>
        <Show when={props.status === "error"}>
          <p
            class="border border-danger/40 bg-danger/10 px-3 py-2 font-mono text-xs text-danger"
            id="url-error"
            role="alert"
          >
            {props.error}
          </p>
        </Show>
      </div>

      <button
        class="press w-full border-2 border-ink bg-accent px-5 py-3.5 font-mono text-sm font-bold uppercase tracking-[0.16em] text-white transition hover:bg-transparent hover:text-accent focus:outline-none focus-visible:shadow-[0_0_0_2px_var(--color-accent)] disabled:border-border disabled:bg-surface-2 disabled:text-ink-muted"
        disabled={!canSubmit()}
        type="submit"
      >
        <span class="inline-flex items-center justify-center gap-2">
          <Show when={isLoading()}>
            <span class="spinner" aria-hidden />
          </Show>
          {isLoading() ? "Shortening\u2026" : "Shorten URL"}
        </span>
      </button>
    </form>
  );
}
