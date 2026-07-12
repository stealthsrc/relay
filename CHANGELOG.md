# Changelog

All notable user-facing changes to Relay are documented in this file, in English and in French.
Toutes les évolutions notables de Relay sont documentées dans ce fichier, en anglais et en français.

## Versioning policy / Politique de version

- A major Relay update increments the middle number: `1.0.0` → `1.1.0`.
- A minor update, bug fix, or simple addition increments the patch number: `1.0.0` → `1.0.1`.
- Changes remain under `Unreleased` until the matching GitHub release is published.

## [Unreleased]

### English

#### Fixed

- Windows notification widgets now grow to the minimum height required by their content scale, preventing the card from disappearing at 135% and above.

### Français

#### Corrigé

- Les widgets de notifications Windows s’agrandissent désormais jusqu’à la hauteur minimale requise par leur échelle de contenu, ce qui empêche la carte de disparaître à partir de 135 %.

## [1.1.22] - 2026-07-12

### English

#### Added

- Added an Output readiness center showing connection status for visual, audio, TTS, notification, and sticker outputs.
- Added separate OBS, preview, and Windows widget connection tracking for every output.
- Added isolated local output tests that never post to Discord or add entries to Relay history.

#### Changed

- The top-bar OBS source count now excludes internal previews and connection probes.

### Français

#### Ajouté

- Ajout d’un centre d’état des sorties pour les médias visuels, l’audio, le TTS, les notifications et les stickers.
- Ajout d’un suivi distinct des connexions OBS, aperçu et widget Windows pour chaque sortie.
- Ajout de tests locaux isolés par sortie, sans publication Discord ni ajout à l’historique Relay.

#### Modifié

- Le compteur de sources OBS de la barre supérieure exclut désormais les aperçus internes et les sondes de connexion.

## [1.1.21] - 2026-07-12

### English

#### Added

- Added persistent live previews for media and notification output geometry in the Relay panel.
- Added a synchronized top-bar audio player with previous, pause/resume, and skip controls.

#### Fixed

- History now loads a first-frame thumbnail for videos and MP4 GIFs; the Relay logo is used only when loading fails.
- Discord stickers posted in the TTS channel now render in notification cards without speech synthesis.
- Notification content scaling keeps cards inside the viewport.

### Français

#### Ajouté

- Ajout d’aperçus persistants en direct pour régler la géométrie des sorties médias et notifications dans le panneau Relay.
- Ajout d’un mini-lecteur audio synchronisé dans la barre supérieure avec précédent, pause/reprise et suivant.

#### Corrigé

- L’historique charge désormais une miniature de la première image des vidéos et GIF MP4 ; le logo Relay n’est utilisé qu’en cas d’échec.
- Les stickers Discord publiés dans le salon TTS s’affichent désormais dans les cartes de notification sans synthèse vocale.
- L’échelle du contenu des notifications conserve les cartes dans le viewport.

## [1.1.1] - 2026-07-12

### English

#### Added

- Added persistent resizing for the media and notification Windows widgets, with monitor-aware limits and an optional 16:9 ratio for media.
- Added independent crop controls (0–40% per side) and content scaling (50–200%) for media and notifications in OBS and Windows widgets.
- Added configurable Discord bot online status and activity text for custom, playing, listening, watching, and competing activities.
- Added optional media sound in the Windows widget and configurable notification sounds for the widget and OBS.

#### Changed

- Output geometry, crop, scale, and bot presence changes now apply live without reloading overlays or interrupting playback.
- Audio cards, notification cards, and author details now scale cleanly while preserving readable, bounded text.
- Remote artwork downloads now accept only approved HTTPS media hosts and safe redirects.

### Français

#### Ajouté

- Ajout du redimensionnement persistant des widgets Windows médias et notifications, avec des limites adaptées à l’écran et un ratio 16:9 optionnel pour les médias.
- Ajout de contrôles indépendants de rognage (0–40 % par côté) et d’échelle du contenu (50–200 %) pour les médias et notifications dans OBS et les widgets Windows.
- Ajout de la configuration du statut en ligne et de l’activité du bot Discord : personnalisé, joue, écoute, regarde ou participe à une compétition.
- Ajout du son optionnel des médias dans le widget Windows et de sons de notification configurables pour le widget et OBS.

#### Modifié

- Les changements de géométrie, rognage, échelle et présence du bot s’appliquent désormais en direct, sans recharger les overlays ni interrompre la lecture.
- Les cartes audio, cartes de notification et informations d’auteur se redimensionnent proprement avec des textes lisibles et contenus.
- Les téléchargements de pochettes distantes sont désormais limités aux hôtes médias HTTPS approuvés et aux redirections sûres.

## [1.1.0] - 2026-07-12

### English

#### Added

- Added a dedicated Commands page with individual availability switches.
- Added `/relay clear` to delete messages from the configured Discord media and TTS channels without clearing Relay history.
- Added `/relay lock` as a reversible toggle for the configured Discord media channel.
- Added `/relay changelog <channel>` to post the latest release notes, fetched live from GitHub, into a chosen Discord channel.
- Preserved access for Discord administrators and moderation roles while a channel is locked.
- Stored channel permission snapshots locally so unlock restores the previous state.
- Added a dedicated `/stickers` OBS Browser Source with its own 50-item FIFO queue and configurable duration.
- Added Discord PNG, APNG, GIF, and Lottie sticker capture with bounded local caching and a safe visual fallback.
- Added visual rendering for Unicode, static custom, and animated custom emojis in TTS notifications.
- Added a "TTS voice" playback switch; when disabled, TTS messages become silent notifications.
- Added a configurable notification duration (1 to 60 seconds) controlling how long silent TTS notifications stay visible in OBS and the Windows widget.

#### Changed

- Reorganized the playback settings into collapsible categories: display durations, audio and TTS, display.
- Updated the Discord invitation URL with the permissions required for media reading, channel permission overwrites, and message cleanup.
- Documented command permissions in English, French, Spanish, and German.
- Messages containing an emoji now skip speech synthesis while preserving the author and message in the notification output.

#### Fixed

- TTS notifications now appear immediately and stay visible even when audio playback fails in OBS or the widget.
- Fixed visual emoji notifications blocking the following spoken TTS message.
- Added a synthesis timeout so a stalled Windows voice cannot freeze the global TTS queue.
- Added automatic fallback to the default Windows voice and preserved notifications when synthesis fails.
- Fixed `/relay clear` by requiring one explicit Discord channel and a message count from 1 to 1000.
- Fixed delayed Discord GIF embeds that arrived through partial message updates.
- Fixed favorite GIFs represented by Discord as thumbnail-only image embeds.
- Added support for direct thumbnail GIFs without a known GIF provider.

### Français

#### Ajouté

- Ajout d’une page Commandes dédiée avec des interrupteurs de disponibilité individuels.
- Ajout de `/relay clear` pour supprimer des messages des salons Discord médias et TTS configurés sans effacer l’historique Relay.
- Ajout de `/relay lock`, un verrouillage réversible du salon média Discord configuré.
- Ajout de `/relay changelog <channel>` pour publier les dernières notes de version, récupérées en direct depuis GitHub, dans le salon Discord choisi.
- Préservation de l’accès des administrateurs Discord et des rôles de modération pendant le verrouillage d’un salon.
- Sauvegarde locale des instantanés de permissions du salon afin que le déverrouillage restaure l’état précédent.
- Ajout d’une source navigateur OBS `/stickers` dédiée avec sa propre file FIFO de 50 éléments et une durée configurable.
- Ajout de la capture des stickers Discord PNG, APNG, GIF et Lottie avec un cache local borné et un repli visuel sûr.
- Ajout du rendu visuel des emojis Unicode, personnalisés statiques et personnalisés animés dans les notifications TTS.
- Ajout d’un interrupteur « Voix TTS » ; désactivé, les messages TTS deviennent des notifications silencieuses.
- Ajout d’une durée de notification configurable (1 à 60 secondes) contrôlant la visibilité des notifications TTS silencieuses dans OBS et le widget Windows.

#### Modifié

- Réorganisation des réglages de lecture en catégories dépliables : durées d’affichage, audio et TTS, affichage.
- Mise à jour de l’URL d’invitation Discord avec les permissions requises pour la lecture des médias, la modification des permissions de salon et le nettoyage des messages.
- Documentation des permissions des commandes en anglais, français, espagnol et allemand.
- Les messages contenant un emoji sautent désormais la synthèse vocale tout en conservant l’auteur et le message dans la sortie de notification.

#### Corrigé

- Les notifications TTS apparaissent immédiatement et restent visibles même si la lecture audio échoue dans OBS ou le widget.
- Correction des notifications emoji visuelles qui bloquaient le message TTS parlé suivant.
- Ajout d’un délai de synthèse afin qu’une voix Windows bloquée ne puisse plus geler la file TTS globale.
- Ajout d’un repli automatique vers la voix Windows par défaut et préservation des notifications en cas d’échec de synthèse.
- Correction de `/relay clear` en exigeant un salon Discord explicite et un nombre de messages entre 1 et 1000.
- Correction des embeds GIF Discord retardés arrivant via des mises à jour partielles de message.
- Correction des GIF favoris représentés par Discord comme des embeds d’image miniature uniquement.
- Prise en charge des GIF miniatures directs sans fournisseur GIF connu.

## [1.0.0] - 2026-07-12

### English

#### Added

- First public release of Relay for Windows.
- Discord media relay for OBS Browser Sources and Windows widgets.
- Separate media, audio, TTS, and notification outputs.
- Local moderation, playback controls, history, personalization, and multilingual interface.

### Français

#### Ajouté

- Première version publique de Relay pour Windows.
- Relais de médias Discord vers les sources navigateur OBS et les widgets Windows.
- Sorties séparées pour les médias, l’audio, le TTS et les notifications.
- Modération locale, contrôles de lecture, historique, personnalisation et interface multilingue.

[Unreleased]: https://github.com/stealthsrc/relay/compare/v1.1.21...HEAD
[1.1.21]: https://github.com/stealthsrc/relay/releases/tag/v1.1.21
[1.1.1]: https://github.com/stealthsrc/relay/releases/tag/v1.1.1
[1.1.0]: https://github.com/stealthsrc/relay/releases/tag/v1.1.0
[1.0.0]: https://github.com/stealthsrc/relay/releases/tag/v1.0.0
