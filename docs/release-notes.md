## Quark 1.6.1 — Overlay DXGI/DirectX 12 et compteur FPS fiabilisé

### Nouveautés

- **Overlay DirectX 11 et 12** : l'option **Afficher les FPS dans le jeu** installe désormais un proxy `d3d11.dll` **et** un proxy `dxgi.dll` à côté de l'exécutable Windows. Ils relaient Direct3D 11 et DXGI au système, accrochent `Present`/`Present1` et dessinent un compteur `N FPS` discret dans le coin supérieur gauche, façon Steam. Ils se cachent dès que le jeu perd le premier plan.
- **Compteur partagé** : les deux proxies utilisent la même mémoire nommée pour les frames et le FPS. La fenêtre créée par l'un affiche donc les frames présentées par l'autre, y compris quand les deux DLL sont chargées.
- **Hooks coordonnés par vtable** : un registre partagé évite d'installer deux hooks `Present` sur la même classe de swap chain, tout en laissant chaque module couvrir une classe distincte. Le compteur ne peut plus rester à zéro ni compter deux fois la même présentation.
- **Patchs sérialisés** : des mutex nommés protègent le registre et la séquence `VirtualProtect` + lecture + écriture + restauration. Un échec d'acquisition abandonne le patch au lieu de modifier la vtable sans protection.
- **`F12` avec son pour toutes les boutiques** : la capture joue le carillon et affiche la notification, Steam inclus.
- **Installation sûre** : Quark ne remplace jamais un `d3d11.dll` ou un `dxgi.dll` étranger. Une DLL portant le marqueur Quark est mise à jour, et `WINEDLLOVERRIDES=d3d11=n,b;dxgi=n,b` demande à Wine/Proton de charger les proxies.
- **Release reproductible** : le workflow reconstruit la DLL Windows avec MinGW avant de compiler Quark et de l'embarquer dans le binaire Linux.

### Correctifs

- La DLL accroche le bon emplacement de `Present` dans la vtable de `IDXGISwapChain` (slot 8, pas 17).
- `CreateSwapChain` est patché sur la vtable de base `IDXGIFactory`, et les variantes `ForHwnd`/`ForComposition` uniquement après un `QueryInterface` réussi vers `IDXGIFactory2`.
- Les réservations de vtables sont atomiques et libérées si l'installation échoue, pour ne pas bloquer une classe ultérieure.
- L'overlay ne reste plus au-dessus de Quark ou du bureau quand le jeu n'est pas au premier plan.
- Un DLL étrangère, un dossier en lecture seule ou un conflit de fichier produit un message explicite au lieu d'un échec silencieux.

### Vérifications

- `cargo fmt --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- `cargo test --locked` : 299 tests réussis.
- `cargo build --release --locked` : réussi.
- DLL compilée en croisé avec MinGW ; exports `CreateDXGIFactory`, `CreateDXGIFactory1`, `CreateDXGIFactory2`, `D3D11CreateDevice`, `D3D11CreateDeviceAndSwapChain`, `D3D11On12CreateDevice`, `DXGIDeclareAdapterRemovalSupport`, `DXGIGetDebugInterface1` et `DllMain` vérifiés.
- CI GitHub verte sur la PR #39 ; revue CodeRabbit finale sans commentaire actionnable.

### Limites connues

- L'overlay n'a pas été exécuté sous Windows, Wine ou Proton avec un GPU réel : le build croisé, les exports, les tests et la revue statique sont vérifiés, pas le rendu à l'écran ni le compteur en jeu.
- Le chemin DirectX 12 via DXGI est couvert par le code et les tests de compilation, mais n'a pas été validé avec un jeu DirectX 12 réel.
- Vulkan et les jeux qui remplacent eux-mêmes `d3d11.dll` ou `dxgi.dll` ne sont pas couverts.
- La connexion Epic directe n'a pas été testée avec un vrai compte ; la bibliothèque et l'installation restent portées par Legendary.
- Les traductions espagnole, allemande et portugaise sont des traductions initiales, à relire par des locuteurs natifs.
