import { Show } from "solid-js";

type Status = "idle" | "loading" | "success" | "error";

export function AppBar(props: { status: Status }) {
  const label = () => {
    switch (props.status) {
      case "loading":
        return "working";
      case "success":
        return "ready";
      case "error":
        return "error";
      default:
        return "idle";
    }
  };

  const dotClass = () => {
    switch (props.status) {
      case "loading":
        return "bg-sgds-cyan";
      case "success":
        return "bg-primary";
      case "error":
        return "bg-danger";
      default:
        return "bg-sgds-primary-500";
    }
  };

  return (
    <header class="sticky top-0 z-30 border-b border-border bg-surface/80 backdrop-blur supports-[backdrop-filter]:bg-surface/70">
      <div class="mx-auto flex w-full max-w-3xl items-center justify-between gap-3 px-4 py-3 sm:px-6 lg:px-8">
        <a class="flex items-center gap-2.5 press" href="/">
          <span
            aria-hidden
            class="grid h-8 w-8 place-items-center rounded-lg border border-sgds-primary-200 bg-primary-muted text-sm font-semibold text-primary"
          >
            u/
          </span>
          <span class="text-sm font-semibold tracking-tight text-sgds-primary-900">
            url.shortener
          </span>
        </a>

        <Show when={props.status !== "idle"}>
          <span class="inline-flex items-center gap-2 rounded-full border border-border bg-primary-muted px-3 py-1.5 text-xs font-medium text-primary">
            <span class={`h-1.5 w-1.5 rounded-full ${dotClass()}`} aria-hidden />
            <Show when={props.status === "loading"}>
              <span class="spinner" aria-hidden />
            </Show>
            {label()}
          </span>
        </Show>
      </div>
    </header>
  );
}