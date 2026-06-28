const alphabet = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

function wait(milliseconds: number) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

export function makeShortCode(input: string) {
  let hash = 2166136261;

  for (let index = 0; index < input.length; index += 1) {
    hash ^= input.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }

  let value = hash >>> 0;
  let code = "";

  do {
    code = alphabet[value % alphabet.length] + code;
    value = Math.floor(value / alphabet.length);
  } while (value > 0);

  return code.padStart(6, "0").slice(0, 7);
}

export interface ShortenResult {
  shortUrl: string;
  shortCode: string;
}

// Client seam for the URL shortener. The signature and result shape match the
// assumed `POST /api/v1/shorten` contract (see docs/decision.md), so wiring the
// real API later (#12b) only swaps this body, not the call sites.
//
// Current implementation is a local mock (no backend dependency) — see #12.
export async function shortenUrl(input: string): Promise<ShortenResult> {
  await wait(650);

  let parsedUrl: URL;
  try {
    parsedUrl = new URL(input.trim());
  } catch {
    throw new Error("Enter a full URL, including https:// or http://.");
  }

  if (parsedUrl.protocol !== "https:" && parsedUrl.protocol !== "http:") {
    throw new Error("Only http:// and https:// URLs can be shortened.");
  }

  const shortCode = makeShortCode(parsedUrl.href);
  return {
    shortUrl: `${window.location.origin}/${shortCode}`,
    shortCode,
  };
}