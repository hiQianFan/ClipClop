const DOWNLOADS = {
  "/download/macos": "macos",
  "/download/windows": "windows",
};

export async function handleDownload(request, fetchMetadata = fetch) {
  if (request.method !== "GET" && request.method !== "HEAD") {
    return new Response("Method Not Allowed", { status: 405 });
  }

  const url = new URL(request.url);
  const platform = DOWNLOADS[url.pathname];
  if (!platform) return new Response("Not Found", { status: 404 });

  try {
    const response = await fetchMetadata(new URL("/downloads.json", url));
    if (!response.ok) throw new Error(`metadata returned ${response.status}`);

    const target = (await response.json())[platform];
    if (typeof target !== "string" || !target.startsWith("/releases/")) {
      throw new Error("invalid download target");
    }

    return new Response(null, {
      status: 302,
      headers: {
        "Cache-Control": "no-cache",
        Location: new URL(target, url).toString(),
      },
    });
  } catch {
    return new Response("Download temporarily unavailable", { status: 503 });
  }
}

export default {
  fetch(request) {
    return handleDownload(request);
  },
};
