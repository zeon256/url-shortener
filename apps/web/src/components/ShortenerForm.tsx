import { Show } from "solid-js";

type Status = "idle" | "loading" | "success" | "error";

export function ShortenerForm(props: {
  url: string;
  status: Status;
  error: string;
  onInput: (value: string) => void;
  onSubmit: () => void;
}) {
  const isLoading = () => props.status === "loading";

  return (
    <form
      class="space-y-4"
      onSubmit={(event) => {
        event.preventDefault();
        props.onSubmit();
      }}
    >
      <div class="space-y-2">
        <label class="block space-y-2" for="long-url">
          <span class="block text-sm font-semibold text-ink">Original URL</span>

          <div
            class="
              group flex items-stretch overflow-hidden rounded-control border border-border
              bg-surface shadow-sm transition focus-within:border-primary
              focus-within:ring-4 focus-within:ring-sgds-primary-100
              has-[input:disabled]:bg-primary-muted
            "
          >
            <span
              class="grid select-none place-items-center bg-primary-muted px-3 text-sm text-primary"
              aria-hidden
            >
              https://
            </span>
            <input
              aria-describedby="url-help url-error"
              class="min-w-0 flex-1 bg-transparent px-4 py-3 text-base text-ink outline-none placeholder:text-ink-muted/70"
              disabled={isLoading()}
              id="long-url"
              inputMode="url"
              onInput={(event) => props.onInput(event.currentTarget.value)}
              placeholder="example.gov.sg/service"
              required
              type="url"
              value={props.url}
            />
          </div>
        </label>

        <p class="text-sm text-ink-muted" id="url-help">
          Use a full http:// or https:// URL.
        </p>
        <Show when={props.status === "error"}>
          <p
            class="text-sm font-medium text-danger"
            id="url-error"
            role="alert"
          >
            {props.error}
          </p>
        </Show>
      </div>

      <button
        class="press w-full rounded-control bg-primary px-5 py-3.5 font-semibold text-sgds-white transition hover:bg-primary-hover focus:outline-none focus:ring-4 focus:ring-sgds-primary-200 disabled:cursor-not-allowed disabled:bg-sgds-primary-200 disabled:text-sgds-primary-700"
        disabled={isLoading()}
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