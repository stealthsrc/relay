# Changelog

All notable user-facing changes to Relay are documented in this file, in English and in French.
Toutes les évolutions notables de Relay sont documentées dans ce fichier, en anglais et en français.

## Versioning policy / Politique de version

- A major Relay update increments the middle number: `1.0.0` → `1.1.0`.
- A minor update, bug fix, or simple addition increments the patch number: `1.0.0` → `1.0.1`.
- Changes remain under `Unreleased` until the matching GitHub release is published.

## [Unreleased]

## [1.2.7] - 2026-08-14

### English

#### Added

- Added a local settings search bar with `Ctrl+K`, keyboard-accessible results, and page-aware Back and Forward controls.
- Added regional language choices with bundled SVG flags for English (US, UK, and India), French, German, Spanish, and Latin American Spanish while preserving the complete English, French, Spanish, and German dictionaries.
- Added eight locally bundled interface fonts: Bricolage Grotesque, DM Sans, Figtree, Inter, JetBrains Mono, Manrope, Poppins, and Space Grotesk. Font files never load from a remote service at runtime.

#### Changed

- Reorganized Moderation into three independent collapsible sections for automatic filtering, manual moderation, and anti-doxxing protection. The existing settings and save behavior are unchanged.
- Completed and corrected the Moderation translations in English, French, Spanish, and German, including protection profiles and input placeholders.

#### Fixed

- HEVC/H.265 Discord videos are now transcoded locally to an H.264-compatible cache when FFmpeg is available, allowing playback in Windows WebView2 widgets. Relay falls back to the original source if conversion cannot complete.

### Français

#### Ajouté

- Ajout d’une barre de recherche locale des réglages avec `Ctrl+K`, de résultats accessibles au clavier et de boutons Retour et Suivant tenant compte de l’historique des pages.
- Ajout de variantes régionales avec des drapeaux SVG embarqués pour l’anglais (États-Unis, Royaume-Uni et Inde), le français, l’allemand, l’espagnol et l’espagnol latino-américain, tout en conservant les dictionnaires complets anglais, français, espagnol et allemand.
- Ajout de huit polices d’interface embarquées localement : Bricolage Grotesque, DM Sans, Figtree, Inter, JetBrains Mono, Manrope, Poppins et Space Grotesk. Aucun fichier de police n’est chargé depuis un service distant à l’exécution.

#### Modifié

- Réorganisation de la page Modération en trois sections repliables indépendantes : filtrage automatique, modération manuelle et protection anti-doxxing. Les réglages existants et leur sauvegarde restent inchangés.
- Traductions de la page Modération complétées et corrigées en anglais, français, espagnol et allemand, notamment pour les profils de protection et les textes indicatifs des champs.

#### Corrigé

- Les vidéos Discord en HEVC/H.265 sont désormais transcodées localement vers un cache compatible H.264 lorsque FFmpeg est disponible, afin de permettre leur lecture dans les widgets Windows WebView2. Relay revient à la source d’origine si la conversion ne peut pas aboutir.

## [1.2.6] - 2026-08-14

### English

#### Fixed

- Discord messages blocked by an automatic filter word are now deleted when **Delete blocked Discord messages** is enabled and Relay has the **Manage Messages** permission, including when the local privacy scan is disabled.

### Français

#### Corrigé

- Les messages Discord bloqués par un mot de filtrage automatique sont désormais supprimés lorsque **Supprimer les messages Discord bloqués** est activé et que Relay possède la permission **Gérer les messages**, même si le scan local de confidentialité est désactivé.

## [1.2.5] - 2026-08-14

### English

#### Added

- Added a fully local anti-doxxing scanner with `SAFE`, `LOW`, `MEDIUM`, `HIGH`, and `CRITICAL` risk levels for Discord text, attachment names, images, and metadata.
- Added configurable Balanced, Strict, and Paranoid protection levels, per-category controls, an automatic block threshold, a local review option, an allowlist, and a private-data protection list.
- Added detection for email addresses, phone numbers, IP addresses, GPS coordinates, postal addresses, IBANs, validated payment cards, license plates, sensitive URLs, administrative-document signals, and user-protected strings.
- Added local Windows OCR and EXIF/GPS inspection for supported images without sending detected content to an external service.
- Added automatic deletion of Discord messages blocked by the selected privacy threshold when the bot has the **Manage Messages** permission.
- Added automatic filter words and phrases with configurable aliases, bounded regular expressions, cautious obfuscation handling, and role-based exemptions.
- Split the Commands page into **Default Commands** and **Custom Commands**, with up to 16 locally configured `/relay <name>` subcommands synchronized for the Relay bot.
- Added predefined Ban, Unban, Kick, Timeout, Remove timeout, Clear messages, Add role, Remove role, and Reply actions with required, optional, or fixed parameters and user, role, channel, and permission restrictions.

#### Security

- Privacy checks now run before sensitive Discord content can enter visible history, WebSocket or OBS output, Windows widgets, media caches, replay, or moderation approval paths.
- Image inspection now enforces trusted Discord CDN hosts, bounded downloads, file-signature checks, size, pixel and frame limits, concurrency limits, and timeouts.
- Privacy logs contain only the risk level, detected category codes, and action; detected addresses, contact details, OCR text, metadata values, and protected strings are not copied into logs.
- Custom moderation actions derive non-disableable Discord permissions, require a one-time 60-second confirmation, recheck authorization and role hierarchy before execution, and suppress mentions in predefined replies.
- Custom-command synchronization validates Discord's candidate schema before local persistence, restores the previous schema if persistence fails, and logs only the command name, action code, and sanitized outcome.

#### Fixed

- Postal addresses are now recognized across punctuation, unusual separators, obfuscated street types, and multiline layouts, including probable addresses without a postcode.
- Intermediate-risk media now uses the existing local moderation queue even when general manual moderation is disabled.
- OCR, malformed metadata, and image-decoder failures no longer interrupt Relay or expose scanned private values in errors.
- Custom Ban and Timeout commands now accept the camelCase action fields sent by the desktop editor while retaining compatibility with previously serialized snake_case fields.
- Custom Ban commands now accept either a current member or a verified Discord user ID that is not yet in the server, allowing a preemptive ban without bypassing hierarchy checks for present members.

### Français

#### Ajouté

- Ajout d'un scanner anti-doxxing entièrement local avec les niveaux de risque `SAFE`, `LOW`, `MEDIUM`, `HIGH` et `CRITICAL` pour le texte Discord, les noms de pièces jointes, les images et leurs métadonnées.
- Ajout des niveaux de protection Balanced, Strict et Paranoid, de catégories configurables, d'un seuil de blocage automatique, d'une option de révision locale, d'une allowlist et d'une liste de données privées à protéger.
- Ajout de la détection des adresses e-mail, numéros de téléphone, adresses IP, coordonnées GPS, adresses postales, IBAN, cartes de paiement validées, plaques d'immatriculation, URL sensibles, indices de documents administratifs et chaînes protégées par l'utilisateur.
- Ajout de l'OCR Windows local et de l'analyse EXIF/GPS pour les images prises en charge, sans transmettre le contenu détecté à un service externe.
- Ajout de la suppression automatique des messages Discord bloqués par le seuil de confidentialité choisi lorsque le bot possède la permission **Gérer les messages**.
- Ajout de mots et expressions filtrés automatiquement avec alias configurables, expressions régulières limitées, gestion prudente de l'obfuscation et exemptions par rôle.
- Séparation de la page Commandes entre **Commandes par défaut** et **Commandes personnalisées**, avec jusqu'à 16 sous-commandes `/relay <nom>` configurées localement et synchronisées pour le bot Relay.
- Ajout des actions prédéfinies Bannir, Débannir, Expulser, Timeout, Retirer le timeout, Effacer des messages, Ajouter un rôle, Retirer un rôle et Réponse, avec paramètres requis, optionnels ou fixes et restrictions par utilisateur, rôle, salon et permission.

#### Sécurité

- Les contrôles de confidentialité s'exécutent désormais avant qu'un contenu Discord sensible puisse atteindre l'historique visible, les sorties WebSocket ou OBS, les widgets Windows, les caches média, le replay ou l'approbation de modération.
- L'analyse des images impose désormais des hôtes CDN Discord approuvés, des téléchargements limités, une vérification de signature, des limites de taille, de pixels et d'images, ainsi que des limites de concurrence et de durée.
- Les journaux de confidentialité contiennent uniquement le niveau de risque, les codes des catégories détectées et l'action ; les adresses, coordonnées, textes OCR, valeurs de métadonnées et chaînes protégées ne sont jamais recopiés.
- Les actions de modération personnalisées imposent les permissions Discord minimales, une confirmation unique de 60 secondes et une nouvelle vérification des autorisations et de la hiérarchie avant exécution ; les mentions sont neutralisées dans les réponses prédéfinies.
- La synchronisation valide le schéma Discord candidat avant la sauvegarde locale, restaure l'ancien schéma si la sauvegarde échoue et ne journalise que le nom de commande, le code d'action et un résultat assaini.

#### Corrigé

- Les adresses postales sont désormais reconnues malgré la ponctuation, les séparateurs inhabituels, les types de voie obfusqués et les présentations multilignes, y compris les adresses probables sans code postal.
- Les médias présentant un risque intermédiaire utilisent désormais la file de modération locale existante même lorsque la modération manuelle générale est désactivée.
- Les échecs OCR, les métadonnées malformées et les erreurs du décodeur d'image n'interrompent plus Relay et n'exposent aucune valeur privée analysée dans les erreurs.
- Les commandes personnalisées Ban et Timeout acceptent désormais les champs camelCase envoyés par l'éditeur tout en restant compatibles avec les anciens champs snake_case sérialisés.
- Les commandes Ban acceptent désormais soit un membre présent, soit l'ID Discord vérifié d'un utilisateur absent du serveur, afin de permettre un bannissement préventif sans contourner la hiérarchie des membres présents.

## [1.2.1] - 2026-07-28

### English

#### Added

- Added `/relay status` to report the live Discord connection, local relay, OBS outputs, queues, and Windows widget state directly in Discord.
- Added `/relay test` for isolated local tests of media, audio, TTS, notifications, and stickers without posting to Discord or adding history entries.
- Added independent options to show up to 180 characters from the Discord media message in OBS and the Windows media widget.
- Added a **Start with Windows** toggle to the system tray; automatic launches now open Relay directly in the tray without showing the control panel.

#### Fixed

- Local media tests and incoming live media now wake the Windows widget before playback so their output is immediately visible.
- The Windows media widget now restores active media after hide/show and avoids WebView2 edge artifacts around videos.
- OBS media and audio outputs no longer overlap during playback.
- Discord invitation links now open reliably in the default web browser instead of File Explorer.

### Français

#### Ajouté

- Ajout de `/relay status` pour afficher directement dans Discord l’état de la connexion, du relais local, des sorties OBS, des files d’attente et des widgets Windows.
- Ajout de `/relay test` pour tester localement les médias, l’audio, le TTS, les notifications et les stickers sans publier dans Discord ni créer d’entrée dans l’historique.
- Ajout d’options indépendantes pour afficher jusqu’à 180 caractères du message Discord associé au média dans OBS et dans le widget média Windows.
- Ajout d’un bouton **Démarrer avec Windows** dans le system tray ; les lancements automatiques ouvrent désormais Relay directement dans le tray sans afficher le panneau de contrôle.

#### Corrigé

- Les tests médias locaux et les nouveaux médias reçus réveillent désormais le widget Windows avant la lecture afin que leur sortie soit immédiatement visible.
- Le widget média Windows restaure le média actif après avoir été masqué puis affiché et évite les artefacts de bordure WebView2 autour des vidéos.
- Les sorties média et audio OBS ne se chevauchent plus pendant la lecture.
- Les liens d’invitation Discord s’ouvrent désormais correctement dans le navigateur web par défaut plutôt que dans l’Explorateur de fichiers.

## [1.2.0] - 2026-07-21

### English

#### Added

- Added three selectable interface styles in Personalization: OpenAI, Anthropic, and Playful Neo-Brutalism, each with light and warm dark variants, keyboard focus, reduced-motion support, and narrow layouts.
- Added Discord guild tags and badges beside author names in TTS notifications when the user enables their primary guild identity.

#### Changed

- The system tray now follows the interface language, theme, and selected design, including translated live status and widget controls.

#### Fixed

- Interface text scaling now applies the selected factor exactly once, preventing oversized or overflowing OpenAI text above 100%.
- Narrow layouts now remain constrained to the viewport when using the Neo-Brutalism design.

### Français

#### Ajouté

- Ajout de trois styles d’interface dans Personnalisation : OpenAI, Anthropic et Playful Neo-Brutalism, avec variantes claire et sombre chaude, focus clavier, réduction des animations et dispositions étroites.
- Ajout des tags et badges de serveur Discord à côté du nom de l’auteur dans les notifications TTS lorsque l’identité du serveur principal est activée par l’utilisateur.

#### Modifié

- Le system tray suit désormais la langue, le thème et le design sélectionné dans l’interface, y compris pour les états en direct et les contrôles des widgets.

#### Corrigé

- La mise à l’échelle du texte applique désormais le facteur sélectionné une seule fois, empêchant les textes OpenAI surdimensionnés ou débordants au-dessus de 100 %.
- Les dispositions étroites restent désormais contenues dans le viewport avec le design Neo-Brutalism.

## [1.1.23] - 2026-07-14

### English

#### Added

- Added an in-app update menu that can check for a new Relay release, then download and install it on confirmation.

#### Fixed

- Audio and video outputs now use an exclusive playback lease so music cannot start while a video is still playing.
- Relay now verifies every downloaded updater installer with an independently stored, pinned signing key before execution.
- Windows notification widgets now grow to the minimum height required by their content scale, preventing the card from disappearing at 135% and above.

### Français

#### Ajouté

- Ajout d’un menu de mise à jour intégré permettant de rechercher une nouvelle version de Relay, puis de la télécharger et de l’installer après confirmation.

#### Corrigé

- Les sorties audio et vidéo utilisent désormais un verrou de lecture exclusif afin que la musique ne démarre pas pendant qu’une vidéo est encore en cours.
- Relay vérifie désormais chaque installateur téléchargé avec une clé de signature épinglée et stockée indépendamment avant toute exécution.
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

[Unreleased]: https://github.com/stealthsrc/relay/compare/v1.2.7...HEAD
[1.2.7]: https://github.com/stealthsrc/relay/compare/v1.2.6...v1.2.7
[1.2.6]: https://github.com/stealthsrc/relay/compare/v1.2.5...v1.2.6
[1.2.5]: https://github.com/stealthsrc/relay/compare/v1.2.1...v1.2.5
[1.2.1]: https://github.com/stealthsrc/relay/compare/v1.2.0...v1.2.1
[1.2.0]: https://github.com/stealthsrc/relay/compare/v1.1.23...v1.2.0
[1.1.23]: https://github.com/stealthsrc/relay/compare/v1.1.22...v1.1.23
[1.1.22]: https://github.com/stealthsrc/relay/compare/v1.1.21...v1.1.22
[1.1.21]: https://github.com/stealthsrc/relay/releases/tag/v1.1.21
[1.1.1]: https://github.com/stealthsrc/relay/releases/tag/v1.1.1
[1.1.0]: https://github.com/stealthsrc/relay/releases/tag/v1.1.0
[1.0.0]: https://github.com/stealthsrc/relay/releases/tag/v1.0.0
