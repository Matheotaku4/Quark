# Extensions de recherche Quark

Quark n'installe **aucune extension par défaut**. La recherche de sites reste
vide tant que l'utilisateur n'a pas ajouté un manifeste dans
**Réglages → Extensions**.

Une extension est un simple fichier JSON hébergé en HTTPS. Elle ne contient
aucun code exécutable : Quark lit le manifeste, affiche ses sites et ouvre les
résultats dans le navigateur.

## Ajouter une extension

1. Publie un fichier JSON sur un dépôt ou un serveur HTTPS public.
2. Ouvre le fichier brut (par exemple l'URL `raw.githubusercontent.com`).
3. Dans **Réglages → Extensions**, colle l'URL et clique sur **Ajouter**.
4. Les sites de l'extension apparaissent dans la recherche multi-sites. Le même
   onglet permet de l'actualiser ou de la retirer.

## Sites inconnus de Quark

Une extension peut déclarer n'importe quel site public. Si Quark ne connaît pas
le site, utilise le driver générique `json` ou `html` : l'extension décrit
elle-même la requête et les sélecteurs à lire. Aucun identifiant de site n'est
imposé par le launcher.

## Format du manifeste

Exemple avec le driver HTML générique :

```json
{
  "schemaVersion": 1,
  "id": "my-sites",
  "name": "My sites",
  "version": "1.0.0",
  "description": "Mes sites de recherche",
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
      "detail": [{ "selector": ".size" }],
      "cover": { "selector": "img", "attr": "src" },
      "limit": 8
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
| `driver` | oui | `json` ou `html` pour un site arbitraire |

### Driver JSON

Pour une API JSON. `list`, `title`, `cover` sont des chemins à points ; les
accolades sont remplacées par le champ correspondant.

```json
{
  "id": "example-api",
  "label": "Example API",
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

Chaque champ accepte `selector`, `attr`, `template`, `stripPrefix` et
`stripSuffix`. Sans `attr`, le texte de l'élément est lu. Les URL relatives sont
résolues par rapport à l'adresse de la requête.

## Sécurité

- HTTPS obligatoire, hôte public uniquement.
- Aucun cookie, aucun `Authorization`, aucun code exécuté.
- Un seul site invalide rejette tout le manifeste.
- 256 Kio maximum, 64 sites maximum, 8 résultats par site par défaut.
- En-têtes autorisés : `Accept`, `Accept-Language`, `Content-Type`,
  `User-Agent`, `Referer`, `Origin`, `Next-Action`, `X-Requested-With`.

## Dépannage

| Symptôme | Cause probable |
| --- | --- |
| `HTTP 404` | mauvaise branche, fichier renommé ou dépôt privé |
| `manifeste illisible` | JSON invalide |
| `HTTPS obligatoire` | adresse en `http://` |
| `hôte local ou privé interdit` | `localhost`, `127.0.0.1`, `192.168.x.x` |
| `site en double` | deux sites avec le même `id` |
| `driver inconnu` | driver absent de `json` ou `html` |
| aucun résultat | extension absente de **Réglages → Extensions**, ou cache vide |

Le guide destiné à un agent IA est dans
[`AGENT_BUILD_EXTENSION.md`](AGENT_BUILD_EXTENSION.md).
