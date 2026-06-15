const ENVIRONMENTS = ["production", "development"];

function json(status, body) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" }
  });
}

async function readManifest(env, environment) {
  const obj = await env.BUCKET.get(`${environment}/manifest.json`);
  if (!obj) return null;
  try {
    return JSON.parse(await obj.text());
  } catch {
    return null;
  }
}

async function serveObject(env, key, request) {
  const obj = await env.BUCKET.get(key);
  if (!obj) return json(404, { error: `not found: ${key}` });

  const headers = new Headers();
  obj.writeHttpMetadata(headers);
  headers.set("etag", obj.httpEtag);
  if (!headers.has("content-type")) {
    headers.set("content-type", "application/octet-stream");
  }
  headers.set("access-control-allow-origin", "*");
  headers.set("cache-control", "public, max-age=300");

  if (request.method === "HEAD") {
    return new Response(null, { headers });
  }
  return new Response(obj.body, { headers });
}

export default {
  async fetch(request, env) {
    if (request.method !== "GET" && request.method !== "HEAD") {
      return json(405, { error: "method not allowed" });
    }

    const url = new URL(request.url);
    const params = url.searchParams;
    const environment = params.get("env") || "production";
    if (!ENVIRONMENTS.includes(environment)) {
      return json(400, { error: `invalid env: ${environment}` });
    }

    // Latest-download by platform key: resolves the newest version from the
    // env manifest and redirects to the versioned object (used by README links).
    const key = params.get("key");
    if (key) {
      const manifest = await readManifest(env, environment);
      if (!manifest) return json(404, { error: "no manifest" });
      const filename = manifest.artifacts?.[key];
      if (!filename) return json(404, { error: `no artifact for key: ${key}` });
      const target = `/?artifact=${encodeURIComponent(filename)}&env=${environment}&version=${manifest.version}`;
      return Response.redirect(new URL(target, url).toString(), 302);
    }

    const artifact = params.get("artifact");
    if (!artifact) {
      return json(400, { error: "missing artifact or key" });
    }
    const version = params.get("version");

    let objectKey;
    if (artifact === "manifest.json" && !version) {
      objectKey = `${environment}/manifest.json`;
    } else if (version) {
      objectKey = `${environment}/${version}/${artifact}`;
    } else {
      const manifest = await readManifest(env, environment);
      if (!manifest) return json(404, { error: "no manifest" });
      objectKey = `${environment}/${manifest.version}/${artifact}`;
    }

    return serveObject(env, objectKey, request);
  }
};
