# Build the Quark search extension — instructions for an AI agent

You are an AI agent. Your job is to create a public GitHub repository that
contains one file, `quark-sites.json`, and to return its raw HTTPS URL. This
extension restores the seven site-search adapters that Quark used to ship:
GameBounty, AnkerGames, AstralGames, Kryoto, Online-Fix, ZeiGames and scene.cat.

Do not modify the Quark source code. Do not add executable code to the
extension. The extension is data only.

## Preconditions

- `git` and either `gh` or the GitHub REST API are available.
- A GitHub token is available as `GITHUB_TOKEN` if `gh` is not authenticated.
- The repository must be **public**. A private repository cannot be fetched by
  Quark.
- The default branch must be `main`. If you use another branch, change the raw
  URL in step 6 accordingly.

## Step 1 — Create a clean work directory

```bash
export EXT_REPO="quark-search-sites"
export EXT_OWNER="${EXT_OWNER:-$(gh api user --jq .login)}"
rm -rf /tmp/quark-search-sites
mkdir -p /tmp/quark-search-sites
cd /tmp/quark-search-sites
git init -b main
```

If `gh` is not authenticated, set `EXT_OWNER` to the GitHub login that owns the
token, then authenticate `gh` with `echo "$GITHUB_TOKEN" | gh auth login --with-token`.

## Step 2 — Write `quark-sites.json` exactly

Create `/tmp/quark-search-sites/quark-sites.json` with this content:

```json
{
  "schemaVersion": 1,
  "id": "quark-sites",
  "name": "Sites Quark",
  "version": "1.0.0",
  "description": "Les sept sites de recherche fournis avec Quark.",
  "homepage": "https://github.com/Matheotaku4/Quark",
  "sites": [
    {
      "id": "gamebounty",
      "label": "GameBounty",
      "host": "gamebounty.st",
      "searchUrl": "https://gamebounty.st/search?q={query}",
      "driver": "gamebounty"
    },
    {
      "id": "ankergames",
      "label": "AnkerGames",
      "host": "ankergames.net",
      "searchUrl": "https://ankergames.net/search?q={query}",
      "driver": "ankergames"
    },
    {
      "id": "astralgames",
      "label": "AstralGames",
      "host": "astralgames.net",
      "searchUrl": "https://astralgames.net/search?q={query}",
      "driver": "astralgames"
    },
    {
      "id": "kryoto",
      "label": "Kryoto",
      "host": "kryo.to",
      "searchUrl": "https://kryo.to/catalog?q={query}",
      "driver": "kryoto"
    },
    {
      "id": "onlinefix",
      "label": "Online-Fix",
      "host": "online-fix.me",
      "searchUrl": "https://online-fix.me/index.php?do=search&subaction=search&story={query}",
      "driver": "onlinefix"
    },
    {
      "id": "zeigames",
      "label": "ZeiGames",
      "host": "zeigames.com",
      "searchUrl": "https://zeigames.com/search/?q={query}",
      "driver": "zeigames"
    },
    {
      "id": "scenecat",
      "label": "scene.cat",
      "host": "scene.cat",
      "searchUrl": "https://scene.cat/?q={query}",
      "driver": "scenecat"
    }
  ]
}
```

Do not rename the drivers. The valid built-in driver names are exactly:
`gamebounty`, `ankergames`, `astralgames`, `kryoto`, `onlinefix`, `zeigames`,
`scenecat`.

## Step 3 — Validate the manifest before pushing

Run this Python check from `/tmp/quark-search-sites`. It must print `OK`:

```bash
python3 - <<'PY'
import json, re, sys
from urllib.parse import urlparse

path = "quark-sites.json"
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
allowed = {"gamebounty", "ankergames", "astralgames", "kryoto",
           "onlinefix", "zeigames", "scenecat", "json", "html"}
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
    if site.get("driver") not in allowed:
        errors.append(f"unknown driver for {sid}: {site.get('driver')!r}")
if errors:
    print("\n".join(errors))
    sys.exit(1)
print("OK")
PY
```

The file must also pass `python3 -m json.tool quark-sites.json >/dev/null`.

## Step 4 — Commit and publish

```bash
cd /tmp/quark-search-sites
git add quark-sites.json
git commit -m "add Quark search sites extension"
gh repo create "$EXT_OWNER/$EXT_REPO" --public --source=. --remote=origin --push
```

If the repository already exists, use:

```bash
git remote add origin "https://github.com/$EXT_OWNER/$EXT_REPO.git"
git push -u origin main
```

Do not force-push an existing repository that users may already reference.

## Step 5 — Produce the raw URL

```bash
export EXT_RAW="https://raw.githubusercontent.com/$EXT_OWNER/$EXT_REPO/main/quark-sites.json"
printf '%s\n' "$EXT_RAW"
curl -fsSL "$EXT_RAW" | python3 -m json.tool >/dev/null && echo "RAW OK"
```

`RAW OK` is mandatory. A browser URL such as
`https://github.com/.../blob/main/quark-sites.json` is not accepted by Quark;
use the `raw.githubusercontent.com` URL.

## Step 6 — Verify inside Quark

1. Quit Quark.
2. Open `~/.config/quark/config.json`.
3. Add `EXT_RAW` to `settings.extension_sources`, then save the file.
4. Start Quark. Open any game and click the download-sites button.
   Results must be grouped under GameBounty, AnkerGames, AstralGames, Kryoto,
   Online-Fix, ZeiGames and scene.cat.
5. To refresh, delete `~/.config/quark/extensions.json` and restart Quark.
6. To disable the extension, remove `EXT_RAW` from
   `settings.extension_sources` and restart Quark.

## Step 7 — Report

Return these values to the user:

```text
Repository: https://github.com/<owner>/<repo>
Raw URL:    https://raw.githubusercontent.com/<owner>/<repo>/main/quark-sites.json
Commit:     <git rev-parse HEAD>
Validation: OK
Quark test: 7 sites visible, search works
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

## Adding a site later

If the site is not one of the seven built-ins, use the generic `json` or `html`
driver. Keep the seven built-in entries unchanged unless the site itself
changes. The full schema and examples are in `README.md` next to this file.

Example JSON driver:

```json
{
  "id": "example",
  "label": "Example",
  "host": "example.com",
  "searchUrl": "https://example.com/search?q={query}",
  "driver": "json",
  "request": { "method": "GET", "url": "https://example.com/api?q={query}" },
  "list": "data.results",
  "title": "title",
  "url": "https://example.com/game/{slug}",
  "detail": ["{size}", "{version}"],
  "cover": ["cover"],
  "limit": 8
}
```

Example HTML driver:

```json
{
  "id": "example",
  "label": "Example",
  "host": "example.com",
  "searchUrl": "https://example.com/search?q={query}",
  "driver": "html",
  "request": { "method": "GET", "url": "https://example.com/search?q={query}" },
  "items": "div.result",
  "title": { "selector": "h2" },
  "url": { "selector": "a", "attr": "href" },
  "detail": [{ "selector": ".size" }],
  "cover": { "selector": "img", "attr": "src" },
  "limit": 8
}
```

## Troubleshooting

| Error or symptom | Fix |
| --- | --- |
| `HTTP 404` | Check the repository is public, the branch is `main`, and the filename is exact. |
| `manifeste illisible` | Run `python3 -m json.tool quark-sites.json` and fix the JSON. |
| `HTTPS obligatoire` | Replace `http://` with `https://`. |
| `hôte local ou privé interdit` | Use a public domain, not `localhost` or a private IP. |
| `site en double` | Make every `sites[].id` unique. |
| `driver inconnu` | Use one of the exact driver names listed above. |
| `l'extension ... est déjà fournie par ...` | Another installed extension already uses this manifest `id`. Change `id` or remove the other extension. |
| No results | Add the manifest to `settings.extension_sources` in `~/.config/quark/config.json`, restart Quark, and check that `extensions.json` contains 7 sites. |
| Search returns errors for every site | The site changed its HTML/API. Update that site's driver or remove it. |
