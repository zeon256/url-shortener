import { Show, createEffect, createSignal, on, onCleanup } from "solid-js";

const SCRAMBLE_GLYPHS =
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

function prefersReducedMotion() {
  return (
    typeof window !== "undefined" &&
    window.matchMedia?.("(prefers-reduced-motion: reduce)").matches
  );
}

function randomGlyph() {
  return SCRAMBLE_GLYPHS[Math.floor(Math.random() * SCRAMBLE_GLYPHS.length)];
}

export function ScrambleCode(props: {
  code: string;
  animate?: boolean;
  class?: string;
}) {
  const [displayed, setDisplayed] = createSignal("");

  createEffect(
    on(
      () => [props.code, props.animate] as const,
      ([code, animate]) => {
        if (!code) return;

        if (!animate || prefersReducedMotion()) {
          setDisplayed(code);
          return;
        }

        let revealed = 0;
        let timer = 0;
        const frame = () => {
          if (revealed > code.length) {
            setDisplayed(code);
            return;
          }
          const fixed = code.slice(0, revealed);
          const scrambling = code
            .slice(revealed)
            .split("")
            .map((ch) => (ch === " " ? " " : randomGlyph()))
            .join("");
          setDisplayed(fixed + scrambling);
          revealed += 1;
          timer = window.setTimeout(frame, 70);
        };

        frame();
        onCleanup(() => window.clearTimeout(timer));
      }
    )
  );

  const decoding = () => props.code !== "" && displayed() !== props.code;

  return (
    <span class={props.class}>
      {displayed() || props.code}
      <Show when={decoding()}>
        <span class="caret" aria-hidden>
          _
        </span>
      </Show>
    </span>
  );
}
