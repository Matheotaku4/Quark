## Quark 1.6.0 — Overlay FPS en DLL, connexion Epic directe et cinq langues

### Nouveautés

- **Overlay FPS en DLL** : l'option **Afficher les FPS dans le jeu** installe un proxy `d3d11.dll` à côté de l'exécutable Windows. La DLL relaie Direct3D 11 au système, accroche `Present` et dessine un compteur `N FPS` discret dans le coin supérieur gauche, façon Steam. Elle se cache dès que le jeu perd le premier plan.
- **Installation sûre** : Quark ne remplace jamais un `d3d11.dll` étranger. Une DLL portant le marqueur Quark est mise à jour lors des releases suivantes, et `WINEDLLOVERRIDES=d3d11=n,b` demande à Wine/Proton de charger le proxy.
- **Connexion Epic directe** : Quark échange lui-même le code d'autorisation avec Epic, puis écrit la session dans le `user.json` de Legendary avec des droits privés. `legendary auth` n'est plus lancé pour se connecter. La bibliothèque, l'installation et le lancement restent assurés par Legendary.
- **Cinq langues** : l'espagnol, l'allemand et le portugais rejoignent le français et l'anglais. Les catalogues comptent 685 clés chacun et `--lang` accepte `fr`, `en`, `es`, `de` et `pt`.
- **Qualité** : `cargo fmt --check` et `cargo clippy --all-targets --locked -- -D warnings` font désormais partie de la CI, aux côtés du build et des tests.
- **Release reproductible** : le workflow reconstruit la DLL Windows avec MinGW avant de compiler Quark et de l'embarquer dans le binaire Linux.

### Correctifs

- La DLL accroche le bon emplacement de `Present` dans la vtable de `IDXGISwapChain`.
- Les jeux qui créent leur swap chain via `IDXGIFactory` après `D3D11CreateDevice` sont couverts, pas seulement `D3D11CreateDeviceAndSwapChain`.
- L'overlay ne reste plus au-dessus de Quark ou du bureau quand le jeu n'est pas au premier plan.
- Un `d3d11.dll` étranger, un dossier en lecture seule ou un conflit de fichier produit un message explicite au lieu d'un échec silencieux.

### Vérifications

- `cargo fmt --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- `cargo test --locked` : 294 tests réussis.
- `cargo build --release --locked` : réussi.
- DLL compilée en croisé avec MinGW ; exports `D3D11CreateDevice`, `D3D11CreateDeviceAndSwapChain` et `D3D11On12CreateDevice` vérifiés avec `objdump`.
- Endpoint OAuth Epic testé avec un faux code : `400 invalid_grant`, donc la requête et le client sont acceptés.
- Rendus hors écran des réglages **En jeu** en espagnol, allemand et portugais.

### Limites connues

- L'overlay n'a pas été exécuté sous Windows, Wine ou Proton avec un GPU réel : la DLL cross-compilée et ses exports sont vérifiés, pas son rendu à l'écran.
- DirectX 11 uniquement. Les jeux qui n'importent que `dxgi.dll`, ainsi que DirectX 12 et Vulkan, ne sont pas couverts.
- La connexion Epic directe n'a pas été testée avec un vrai compte. La bibliothèque et l'installation restent portées par Legendary.
- Les traductions espagnole, allemande et portugaise sont des traductions initiales, à relire par des locuteurs natifs.
