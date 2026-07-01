import { Show } from "solid-js";

export type NavStatus = "checking" | "online" | "offline" | "working" | "error";

export function AppBar(props: { status: NavStatus }) {
  const label = () => {
    switch (props.status) {
      case "checking":
        return "checking";
      case "online":
        return "online";
      case "offline":
        return "offline";
      case "working":
        return "working";
      case "error":
        return "error";
    }
  };

  const ledClass = () => {
    switch (props.status) {
      case "checking":
        return "bg-border-bright";
      case "online":
        return "bg-accent";
      case "offline":
        return "bg-danger";
      case "working":
        return "bg-accent animate-pulse";
      case "error":
        return "bg-danger";
    }
  };

  const textClass = () => {
    if (props.status === "error" || props.status === "offline") return "text-danger";
    if (props.status === "online") return "text-accent";
    return "text-ink-muted";
  };

  return (
    <header class="sticky top-0 z-30 border-b-2 border-border bg-surface/95 backdrop-blur supports-[backdrop-filter]:bg-surface/90">
      <div class="mx-auto flex w-full max-w-2xl items-center justify-between gap-3 px-4 py-2.5 sm:px-6 lg:px-8">
        <a class="press block min-w-0 shrink" href="/">
          <img
            alt="url-shortener"
            class="logo-nav"
            decoding="async"
            height="341"
            src="/logo.webp"
            width="512"
          />
        </a>

        <output
          class="inline-flex items-center gap-2 border-2 border-ink bg-surface px-2.5 py-1 font-mono text-[0.7rem] uppercase tracking-[0.14em] text-ink-muted"
        >
          <span class={`h-1.5 w-1.5 ${ledClass()}`} aria-hidden />
          <Show when={props.status === "working"}>
            <span class="spinner text-accent" aria-hidden />
          </Show>
          <span class={textClass()}>{label()}</span>
        </output>
      </div>
    </header>
  );
}
