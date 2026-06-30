export function PanelHeader(props: { title: string }) {
  return (
    <div class="flex items-center gap-3 border-b-2 border-ink bg-accent px-4 py-2.5 sm:px-5">
      <span class="font-mono text-xs font-semibold tracking-wide text-white sm:text-sm">
        {props.title}
      </span>
    </div>
  );
}
