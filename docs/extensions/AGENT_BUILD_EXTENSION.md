# Build a Quark search extension — instructions for an AI agent

You are an AI agent. Your job is to create a public GitHub repository that
contains one manifest file, `quark-extension.json`, and to return its raw HTTPS
URL. Quark does not ship any extension by default: the user adds your URL in
**Settings → Extensions**.

Do not modify the Quark source code. Do not add executable code to the
extension. The extension is data only.

The manifest may describe any public HTTPS site, including sites Quark has
never seen. Use the generic `json` or `html` driver and describe how to read
the site yourself.

## Preconditions

- `git` and either `gh` or the GitHub REST API are available.
- A GitHub token is available as `GITHUB_TOKEN` if `gh` is not authenticated.
- The repository must be **public**. A private repository cannot be fetched by
  Quark.
- The default branch must be `main`. Change the raw URL later if you use
  another branch.

## Step 1 — Create a clean work directory

```bash
export EXT_REPO="quark-search-extension"
export EXT_OWNER="${EXT_OWNER:-$(gh api user --jq .login)}"
rm -rf /tmp/quark-search-extension
mkdir -p /tmp/quark-search-extension
cd /tmp/quark-search-extension
git init -b main
```

## Step 2 — Write the manifest

Create `/tmp/quark-search-extension/quark-extension.json`. Replace the example
site with the site the user asked for. If the site is unknown to Quark, keep
the generic `html` or `json` driver.

```json
{
  "schemaVersion": 1,
  "id": "my-sites",
  "name": "My search sites",
  "version": "1.0.0",
  "description": "Sites added by this extension.",
  "sites": [
    {
      "id": "example",
      "label": "Example",
      "host": "example.com",
      "searchUrl": "https://example.com/search?q={query}",
      "driver": "html",
      "request": {
        "method": "GET",
        "url": "https://example.com/search?q={query}"
      },
      "items": "article.result",
      "title": { "selector": "h2" },
      "url": { "selector": "a", "attr": "href" },
      "limit": 8
    }
  ]
}
```

The generic drivers are the only ones you need for a new site:

- `json` for an API: provide `request`, `list`, `title`, `url`, and optionally
  `detail`, `cover`, `limit`.
- `html` for a page: provide `request`, `items`, `title`, `url`, and optionally
  `detail`, `cover`, `limit`.

Full field examples are in [`README.md`](README.md).

## Step 3 — Validate the manifest before pushing

Run this Python check from `/tmp/quark-search-extension`. It must print `OK`:

```bash
python3 - <<'PY'
import json, re, sys
from urllib.parse import urlparse

path = "quark-extension.json"
data = json.load(open(path, encoding="utf-8"))
errors = []
if data.get("schemaVersion") != 1:
    errors.append("schemaVersion must be 1")
if not re.fullmatch(r"[a-z0-9._-]{1,64}", data.get("id", "")):
    errors.append("invalid extension id")
if not data.get("name") or len(data["name"]) > 80:
    errors.append("invalid name")
if not data.get("version") or len(data["version"]) > 32:
    errors.append("invalid version")
sites = data.get("sites")
if not isinstance(sites, list) or not 1 <= len(sites) <= 64:
    errors.append("sites must contain 1..64 entries")
    sites = []
seen = set()
for site in sites:
    sid = site.get("id", "")
    if not re.fullmatch(r"[a-z0-9._-]{1,64}", sid):
        errors.append(f"invalid site id: {sid!r}")
    if sid in seen:
        errors.append(f"duplicate site id: {sid}")
    seen.add(sid)
    if not site.get("label") or len(site["label"]) > 80:
        errors.append(f"invalid label for {sid}")
    if not site.get("host") or "/" in site["host"]:
        errors.append(f"invalid host for {sid}")
    url = site.get("searchUrl", "")
    if "{query}" not in url:
        errors.append(f"searchUrl without {{query}} for {sid}")
    parsed = urlparse(url.replace("{query}", "test"))
    if parsed.scheme != "https":
        errors.append(f"searchUrl must use HTTPS for {sid}")
    if parsed.hostname in {"localhost", "127.0.0.1", "::1"}:
        errors.append(f"private host for {sid}")
    if site.get("driver") not in {"json", "html"}:
        errors.append(f"driver must be json or html for {sid}")
if errors:
    print("\n".join(errors))
    sys.exit(1)
print("OK")
PY
```

The file must also pass `python3 -m json.tool quark-extension.json >/dev/null`.

## Step 4 — Commit and publish

```bash
cd /tmp/quark-search-extension
git add quark-extension.json
git commit -m "add Quark search extension"
gh repo create "$EXT_OWNER/$EXT_REPO" --public --source=. --remote=origin --push
```

If the repository already exists:

```bash
git remote add origin "https://github.com/$EXT_OWNER/$EXT_REPO.git"
git push -u origin main
```

Do not force-push a repository that users may already reference.

## Step 5 — Produce the raw URL

```bash
export EXT_RAW="https://raw.githubusercontent.com/$EXT_OWNER/$EXT_REPO/main/quark-extension.json"
printf '%s\n' "$EXT_RAW"
curl -fsSL "$EXT_RAW" | python3 -m json.tool >/dev/null && echo "RAW OK"
```

`RAW OK` is mandatory. A browser URL such as
`https://github.com/.../blob/main/quark-extension.json` is not accepted by
Quark; use the `raw.githubusercontent.com` URL.

## Step 6 — Verify inside Quark

1. Start Quark. The site search must be empty before any extension is added.
2. Open **Settings → Extensions**, paste `EXT_RAW`, then click **Add**.
3. Open a game and click the download-sites button. The new site must appear
   with its own label and host.
4. Run a search. Results must be grouped under the extension's site.
5. Use **Refresh** to update the manifest, or **Remove** to hide its sites.

## Step 7 — Report

Return these values to the user:

```text
Repository: https://github.com/<owner>/<repo>
Raw URL:    https://raw.githubusercontent.com/<owner>/<repo>/main/quark-extension.json
Commit:     <git rev-parse HEAD>
Validation: OK
Quark test: site visible, search works
```

## Rules and limits

- `schemaVersion` is `1`.
- One extension manifest contains 1 to 64 sites.
- Site IDs and the extension ID use `[a-z0-9._-]`, maximum 64 characters.
- `searchUrl` must be HTTPS and contain `{query}`.
- A manifest is rejected entirely if one site is invalid.
- Two installed extensions cannot share the same `id`.
- The manifest is capped at 256 KiB.
- Headers allowed in a `request`: `Accept`, `Accept-Language`, `Content-Type`,
  `User-Agent`, `Referer`, `Origin`, `Next-Action`, `X-Requested-With`.
- Cookies, `Authorization` and arbitrary headers are rejected.
- Do not point a request at `localhost`, a private IP, or a non-HTTPS URL.
- Do not add JavaScript, shell scripts, binaries or install hooks to the
  extension repository. Quark only reads JSON.
