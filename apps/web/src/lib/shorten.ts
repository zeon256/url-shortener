const BACKEND_URL = (
  import.meta.env.VITE_BACKEND_URL || "http://localhost:4002"
).replace(/\/+$/, "");

export interface ShortenResult {
  shortUrl: string;
  shortCode: string;
}

interface ShortenSuccessBody {
  short_code?: string;
  original_url?: string;
}

interface ShortenErrorBody {
  error?: { message?: string };
}

// Calls the backend shortener API at `${VITE_BACKEND_URL}/api/v1/shorten`
// (base defaults to http://localhost:4002 for local dev). The request/response
// shape is documented in docs/decision.md. The response carries `short_code`
// and `original_url`; the displayable short link is the API host itself serving
// `GET /:code` → 301, so we build `shortUrl` as `${BACKEND_URL}/${short_code}`.
export async function shortenUrl(input: string): Promise<ShortenResult> {
  let response: Response;
  try {
    response = await fetch(`${BACKEND_URL}/api/v1/shorten`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ url: input.trim() }),
    });
  } catch {
    throw new Error(
      "Could not reach the server. Check your connection and try again.",
    );
  }

  if (!response.ok) {
    throw new Error(await readErrorMessage(response));
  }

  const body = (await response.json()) as ShortenSuccessBody;
  if (!body.short_code || !body.original_url) {
    throw new Error("The server returned an unexpected response.");
  }

  return {
    shortUrl: `${BACKEND_URL}/${body.short_code}`,
    shortCode: body.short_code,
  };
}

async function readErrorMessage(response: Response): Promise<string> {
  try {
    const body = (await response.json()) as ShortenErrorBody;
    if (body.error?.message) return body.error.message;
  } catch {
    // Response body was not JSON; fall through to the generic message.
  }
  return "Unable to shorten this URL. Please try again.";
}
