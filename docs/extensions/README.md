# Extensions de recherche Quark

La recherche de sites est **désactivée par défaut**. Quark ne contacte aucun
site de téléchargement tant qu'aucune extension n'est installée. Une extension
est un simple fichier JSON hébergé en HTTPS : elle ne contient aucun code
exécutable.

## Démarrage rapide

1. Copie `quark-sites.json` dans un dépôt GitHub public.
2. Ouvre le fichier sur GitHub puis clique sur **Raw**.
3. Dans `~/.config/quark/config.json`, ajoute l'adresse brute au tableau
   `settings.extension_sources`, puis redémarre Quark :

   ```json
   "settings": {
     "extension_sources": ["https://raw.githubusercontent.com/.../quark-sites.json"]
   }
   ```

4. La recherche de la fiche de jeu, de la boutique, de l'ajout d'un jeu et de
   la palette utilise alors les sites du manifeste. Sans URL configurée, aucun
   site n'est contacté.

Le fichier `quark-sites.json` de ce dossier reproduit les sept sites intégrés :
GameBounty, AnkerGames, AstralGames, Kryoto, Online-Fix, ZeiGames et scene.cat.

## Sécurité

- HTTPS obligatoire, hôte public uniquement (pas de `localhost`, pas d'adresse privée).
- Aucun cookie, aucun `Authorization`, aucun code exécuté.
- Un seul site invalide rejette tout le manifeste.
- 256 Kio maximum, 64 sites maximum, 8 résultats par site par défaut.
- Le réseau n'est utilisé que pour charger un manifeste absent du cache, et lors d'une recherche demandée.

## Format du manifeste

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
    }
  ]
}
```

| Champ | Obligatoire | Règle |
| --- | --- | --- |
| `schemaVersion` | oui | `1` |
| `id` | oui | `[a-z0-9._-]{1,64}`, unique parmi les extensions installées |
| `name` | oui | 1 à 80 caractères |
| `version` | oui | 1 à 32 caractères |
| `description` | non | texte libre |
| `homepage` | non | URL HTTPS publique |
| `sites` | oui | 1 à 64 sites, identifiants uniques |
| `searchUrl` | oui | HTTPS public, contient `{query}` |

## Drivers

### Drivers intégrés

Ces drivers réutilisent les adaptateurs écrits dans Quark. Ils n'ont pas de
configuration supplémentaire.

`gamebounty`, `ankergames`, `astralgames`, `kryoto`, `onlinefix`, `zeigames`,
`scenecat`.

### Driver JSON

Pour une API JSON. `list`, `title`, `cover` sont des chemins à points ;
`{slug}` et les autres accolades sont remplacées par le champ correspondant.

```json
{
  "id": "example",
  "label": "Example",
  "host": "example.com",
  "searchUrl": "https://example.com/search?q={query}",
  "driver": "json",
  "request": {
    "method": "GET",
    "url": "https://example.com/api/search?q={query}",
    "headers": { "Accept": "application/json" }
  },
  "list": "data.results",
  "title": "title",
  "url": "https://example.com/game/{slug}",
  "detail": ["{size}", "{version}"],
  "cover": ["cover", "poster"],
  "limit": 8
}
```

### Driver HTML

Pour une page HTML. Chaque champ accepte `selector`, `attr`, `template`,
`stripPrefix` et `stripSuffix`. Sans `attr`, le texte de l'élément est lu.
Les URL relatives sont résolues par rapport à l'adresse de la requête.

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

En-têtes autorisés : `Accept`, `Accept-Language`, `Content-Type`, `User-Agent`,
`Referer`, `Origin`, `Next-Action`, `X-Requested-With`.

## Dépannage

| Symptôme | Cause probable |
| --- | --- |
| `HTTP 404` | mauvaise branche, fichier renommé ou dépôt privé |
| `manifeste illisible` | JSON invalide |
| `HTTPS obligatoire` | adresse en `http://` |
| `hôte local ou privé interdit` | `localhost`, `127.0.0.1`, `192.168.x.x` |
| `site en double` | deux sites avec le même `id` |
| `driver inconnu` | nom de driver absent de la liste |
| aucun résultat | URL absente de `settings.extension_sources`, ou cache vide |

Le guide destiné à un agent IA est dans
[`AGENT_BUILD_EXTENSION.md`](AGENT_BUILD_EXTENSION.md).
