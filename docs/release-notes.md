# Quark 1.8.8 — secours audio et adoption GOG multi-playTasks

### Correctifs

- **Secours audio automatique.** Si `pulsesink` n'ouvre pas le flux avant la première image, le lecteur relance une fois GStreamer avec `autoaudiosink` : PipeWire, ALSA ou une autre sortie disponible prend le relais au lieu d'afficher une erreur. Le secours ne s'applique jamais quand `QUARK_GST_AUDIO_SINK` impose une sortie, et le curseur de volume est masqué si la sortie de secours ne permet pas de régler le flux. Une vidéo déjà commencée n'est pas redémarrée.
- **Adoption GOG alignée sur les manifestes réels.** L'adoption exige l'exécutable de la tâche primaire et tous les exécutables de catégorie `game`, y compris les tâches cachées derrière un lanceur. Les manuels, outils de configuration et `URLTask` optionnels n'empêchent plus d'adopter une installation complète ; un `setup.exe` ou un fichier de `__redist` ne suffit toujours pas. Les chemins Windows, les doubles séparateurs et les différences de casse restent résolus.
- **Manifestes multi-playTasks couverts.** Quatre manifestes GOG réalistes (Beneath a Steel Sky, Trails in the Sky SC, Prison Architect et The Witcher 3) vérifient l'adoption, les tâches optionnelles, les tâches cachées et les lanceurs primaires.

### Vérifications

- `cargo fmt --all -- --check` : OK.
- `cargo test --locked --offline` : 362 tests réussis, 4 tests réseau ignorés.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
