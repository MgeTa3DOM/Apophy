/**
 * Apophy Dashboard — Real-time monitoring UI
 * Built with Bun + Hono for maximum performance.
 *
 * Connects to api-server at APOPHY_API_URL (default: http://localhost:8080)
 */

import { Hono } from "hono";

const app = new Hono();
const apiUrl = process.env.APOPHY_API_URL ?? "http://localhost:8080";

interface HealthResponse {
  status: string;
  version: string;
}

interface RouterInfo {
  models: Array<{
    name: string;
    role: string;
    resource: string;
    maxTokens: number;
  }>;
  totalMemoryMb: number;
}

interface MemoryFragment {
  id: string;
  sessionId: string;
  content: string;
  createdAt: string;
}

// Dashboard home
app.get("/", (c) => {
  return c.html(`<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Apophy Dashboard</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body { font-family: system-ui, sans-serif; background: #0a0a0a; color: #e0e0e0; padding: 2rem; }
    h1 { font-size: 1.5rem; margin-bottom: 1rem; color: #fff; }
    .card { background: #1a1a1a; border: 1px solid #333; border-radius: 8px; padding: 1.5rem; margin-bottom: 1rem; }
    .card h2 { font-size: 1rem; color: #888; margin-bottom: 0.5rem; text-transform: uppercase; letter-spacing: 0.1em; }
    .status { display: inline-block; width: 8px; height: 8px; border-radius: 50%; margin-right: 0.5rem; }
    .status.ok { background: #4ade80; }
    .status.err { background: #f87171; }
    pre { background: #111; padding: 1rem; border-radius: 4px; overflow-x: auto; font-size: 0.875rem; }
    #data { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1rem; }
  </style>
</head>
<body>
  <h1>Apophy Dashboard</h1>
  <div id="data">
    <div class="card"><h2>Health</h2><pre id="health">Loading...</pre></div>
    <div class="card"><h2>Models</h2><pre id="models">Loading...</pre></div>
    <div class="card"><h2>Memory</h2><pre id="memory">Loading...</pre></div>
    <div class="card"><h2>Sessions</h2><pre id="sessions">Loading...</pre></div>
  </div>
  <script>
    const api = "${apiUrl}";
    async function load(id, url) {
      try {
        const res = await fetch(url);
        const data = await res.json();
        document.getElementById(id).textContent = JSON.stringify(data, null, 2);
      } catch (e) {
        document.getElementById(id).textContent = "Error: " + e.message;
      }
    }
    load("health", api + "/health");
    load("models", api + "/api/v1/router/models");
    load("memory", api + "/api/v1/memory/count");
    load("sessions", api + "/api/v1/memory/sessions");
    setInterval(() => {
      load("health", api + "/health");
      load("memory", api + "/api/v1/memory/count");
    }, 5000);
  </script>
</body>
</html>`);
});

// Proxy API calls (CORS-free)
app.get("/api/*", async (c) => {
  const path = c.req.path;
  const res = await fetch(`${apiUrl}${path}`);
  return c.json(await res.json());
});

const port = parseInt(process.env.DASHBOARD_PORT ?? "3000");
console.log(`Apophy Dashboard running at http://localhost:${port}`);

export default {
  port,
  fetch: app.fetch,
};
