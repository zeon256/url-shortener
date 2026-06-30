const BACKEND_URL = (
  import.meta.env.VITE_BACKEND_URL || "http://localhost:4002"
).replace(/\/+$/, "");

export async function checkHealth(): Promise<boolean> {
  try {
    const response = await fetch(`${BACKEND_URL}/healthz`, { method: "GET" });
    return response.ok;
  } catch {
    return false;
  }
}
