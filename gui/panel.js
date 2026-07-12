const { invoke } = window.__TAURI__.core;

const translations = {
  en: {
    navOverview: "Overview", navMedia: "Media", navOverlay: "Overlay", navModeration: "Moderation", navHistory: "History", navHelp: "Help", navAbout: "About",
    language: "Language", appearance: "Appearance", light: "Light", dark: "Dark", overlays: "overlays",
    system: "System", playback: "Playback", output: "Output", safety: "Safety", archive: "Archive", guide: "Guide", about: "About",
    overviewKicker: "Local broadcast", overviewTitle: "One channel. Every screen.",
    overviewCopy: "Connect Discord once, choose a channel, then keep the relay running quietly in the tray.",
    credentialsTitle: "Discord connection", credentialsCopy: "Credentials are encrypted by Windows and never shown again.",
    clientId: "Discord client ID", botToken: "Discord bot token", connectBot: "Encrypt and start bot",
    inviteUrl: "Bot invitation URL", copy: "Copy", copied: "Copied",
    routingTitle: "Input routing", routingCopy: "Choose one Discord channel for media and another for spoken messages.",
    mediaChannel: "Media channel", ttsChannel: "TTS message channel", localPort: "Local port", saveRouting: "Save routing",
    selectChannel: "Select a visible text channel", ttsDisabled: "TTS disabled", unavailableChannel: "Unavailable channel",
    refreshChannels: "Refresh channels", channelsRefreshed: "Channel list updated",
    mediaKicker: "Playback queue", mediaTitle: "Media, on your terms.",
    mediaCopy: "Images and GIFs use separate display timers. Video and audio continue naturally until they end.",
    transportLabel: "Live transport", transportReady: "Ready for the next item", skip: "Skip current item",
    playbackTitle: "Playback settings", imageDuration: "Image duration", gifDuration: "GIF duration",
    imageDurationHelp: "Used for static images only.", gifDurationHelp: "Animated GIFs loop for this duration.", seconds: "sec",
    mediaVolume: "Media volume", mediaVolumeHelp: "Applied to video, audio and spoken messages.",
    widgetSound: "Widget sound", widgetSoundHelp: "Play video and audio sound in the Windows widget. OBS sources keep their own audio.",
    ttsCharacterLimit: "TTS character limit", ttsCharacterLimitHelp: "Use 0 for unlimited message length.", characters: "chars",
    ttsQueueLimit: "TTS queue capacity", ttsQueueLimitHelp: "Maximum waiting messages, from 1 to 50.", items: "items",
    ttsSpeech: "TTS voice", ttsSpeechHelp: "When off, TTS messages show as silent notifications.",
    obsNotifications: "Show TTS notifications in OBS", obsNotificationsHelp: "Display the author and message while TTS is speaking.",
    obsNotificationOutput: "OBS TTS notification overlay", obsNotificationOutputHelp: "Independent Browser Source displayed only inside OBS.",
    enableObsNotifications: "Enable OBS overlay", enableObsNotificationsHelp: "Does not change the Windows widget.",
    windowsNotificationWidget: "Windows TTS notification widget", windowsNotificationWidgetHelp: "Independent desktop window that can be placed on any monitor.",
    notificationSound: "Notification sound", notificationSoundHelp: "Played by the Windows widget for each message. Any audio file up to 10 seconds.",
    chooseNotificationSound: "Choose audio file", resetNotificationSound: "Remove sound", noNotificationSound: "No file selected.",
    notificationSoundObs: "Notification sound in OBS", notificationSoundObsHelp: "The OBS notification overlay plays the same chosen sound, audible on stream.",
    showAuthor: "Show author", showAuthorHelp: "Display the Discord name and avatar over media.",
    supportedFormats: "Images, GIF, MP4/WebM and common audio formats are accepted.", savePlayback: "Save playback",
    overlayKicker: "Program output", overlayTitle: "What OBS receives.",
    overlayCopy: "The canvas stays transparent until media enters the queue.", livePreview: "Live preview",
    transparentCanvas: "Transparent canvas", browserSource: "OBS Browser Sources",
    browserSourceHelp: "Add each private URL as a separate OBS Browser Source.", visualSource: "Visual media", ttsSource: "TTS audio", notificationSource: "TTS notifications",
    audioSource: "Audio, music and voice messages", regenerateSecret: "Reconnect OBS sources",
    floatingWidget: "Media floating widget", widgetHelp: "Unlock, position it on any monitor, then lock it to pass clicks through.",
    notificationWidget: "TTS notification widget", notificationWidgetHelp: "Show it on Windows, move it anywhere, then lock it to pass clicks through.", showNotificationWidget: "Show on Windows",
    historyKicker: "Last 50 items", historyTitle: "Media history",
    historyCopy: "Replay a previous item or clear every connected overlay.", clearOverlay: "Clear overlay",
    historyEmpty: "Waiting for the first Discord media.", replay: "Replay",
    moderationKicker: "Broadcast safety", moderationTitle: "You decide what reaches OBS.",
    moderationCopy: "Optionally hold incoming media until you approve it locally.", moderationSettings: "Moderation settings",
    enableModeration: "Enable manual moderation", enableModerationHelp: "When disabled, media keeps flowing directly to OBS.",
    allowImages: "Images and GIFs", allowImagesHelp: "Allow these items to enter the approval queue.",
    allowVideos: "Videos", allowVideosHelp: "Allow video files to enter the approval queue.",
    allowAudio: "Audio", allowAudioHelp: "Allow audio files to enter the approval queue.",
    moderationLocalOnly: "Decisions stay local and never notify Discord users.", saveModeration: "Save moderation",
    pendingMedia: "Pending media", clearPending: "Reject all", moderationEmpty: "No media is waiting for approval.",
    moderationDisabled: "Manual moderation is disabled.", approve: "Approve", reject: "Reject",
    botOffline: "Bot offline", serverOnline: "Server online", serverOffline: "Server offline",
    notConfigured: "Not configured", savedVia: "Saved via", encrypting: "Encrypting…",
    encryptedStarting: "Encrypted; bot starting", saving: "Saving…", saved: "Saved",
    regenerating: "Reconnecting…", secretRegenerated: "Permanent links preserved", skipped: "Current item skipped",
    widgetHidden: "Widget hidden", widgetVisibleLocked: "Widget visible and locked",
    widgetVisibleMovable: "Widget visible and movable", showWidget: "Show widget", hideWidget: "Hide widget",
    unlockMove: "Unlock to move", lockDisplay: "Lock display", unknownAuthor: "Unknown author",
    helpKicker: "Setup guide", helpTitle: "From Discord to OBS.",
    helpCopy: "Follow these steps once, then keep Relay running quietly in the system tray.",
    helpStartTitle: "Recommended setup order",
    helpStartCopy: "Discord application, permissions, channels, OBS sources, widgets, then a live test.",
    helpDiscordTitle: "Create the Discord bot", helpDiscordSummary: "Application, bot token and client ID",
    helpDiscordStep1: "Open the Discord Developer Portal and create a New Application.",
    helpDiscordStep2: "Open Bot, create or reset the token, then copy it once.",
    helpDiscordStep3: "Copy the Application ID from General Information; this is the client ID.",
    helpDiscordStep4: "Paste both values into Relay. They are encrypted by Windows.",
    openDiscordPortal: "Open Discord Developer Portal",
    helpIntentTitle: "Enable permissions and intents", helpIntentSummary: "Required to read normal messages",
    helpIntentStep1: "In Bot → Privileged Gateway Intents, enable Message Content Intent.",
    helpIntentStep2: "Use Relay's invitation URL to add the bot to your Discord server.",
    helpIntentStep3: "Give the bot View Channel and Read Message History in both watched channels, Manage Roles for locking, and Manage Messages for clearing.",
    helpIntentNote: "“Disallowed gateway intents” means Message Content Intent is still disabled; the token does not need regeneration.",
    helpChannelsTitle: "Configure Discord channels", helpChannelsSummary: "Separate media and spoken messages",
    helpChannelsStep1: "Create one text channel for images, GIFs, videos and audio.",
    helpChannelsStep2: "Create a second text channel for plain TTS messages.",
    helpChannelsStep3: "Select both channels on Overview and save routing.",
    helpChannelsStep4: "French and English messages automatically use the matching Windows voice.",
    helpObsTitle: "Install the OBS Browser Sources", helpObsSummary: "Media, TTS audio and notifications",
    helpObsStep1: "In OBS, add a separate Browser Source for each URL shown on Overlay.",
    helpObsStep2: "Use the visual URL for media and keep its transparent background.",
    helpObsStep3: "Use the TTS URL as a dedicated audio Browser Source and enable OBS audio control.",
    helpObsStep4: "Use the notifications URL for the PS5-style message card.",
    helpObsStep5: "Keep the private URLs unchanged; Relay reconnects them after every restart.",
    openObsGuide: "Open OBS Browser Source guide",
    helpWidgetsTitle: "Place the Windows widgets", helpWidgetsSummary: "Move, lock and show on any monitor",
    helpWidgetsStep1: "Open Overlay and show the media or notification widget.",
    helpWidgetsStep2: "Leave it unlocked, drag it to the desired monitor and position.",
    helpWidgetsStep3: "Lock it so mouse clicks pass through to applications underneath.",
    helpWidgetsStep4: "Visibility, lock state and position are restored on restart.",
    helpTroubleshootingTitle: "Troubleshooting", helpTroubleshootingSummary: "Fast checks when nothing appears",
    helpTroubleshooting1: "No media: verify the selected media channel and the bot's channel permissions.",
    helpTroubleshooting2: "No TTS: verify the separate TTS channel and Windows speech packs.",
    helpTroubleshooting3: "No notification: enable Show TTS notifications in OBS, then send a new message.",
    helpTroubleshooting4: "Blank OBS source: refresh it once and confirm Relay is running on the displayed local port.",
    helpTroubleshooting5: "Never regenerate a Discord token for a gateway-intent error; enable the intent instead.",
    aboutKicker: "About Relay", aboutTitle: "Built locally. Made to disappear.",
    aboutCopy: "Relay connects Discord to OBS and Windows while keeping credentials and traffic on this computer.",
    aboutStatement: "A private broadcast utility for media, spoken messages and notifications—with permanent local OBS sources.",
    aboutCreatorLabel: "Creator", aboutCreatorCopy: "Explore the creator's projects and source work on GitHub.",
    aboutPrivacy: "Credentials encrypted by Windows", aboutNetwork: "Local server · 127.0.0.1",
    privacyCardTitle: "No data collection by Relay",
    privacyCardCopy: "No telemetry, analytics, advertising or developer-held user profile. Local settings stay on this computer; Discord remains an external service.",
    privacyCardLink: "Privacy and regional rights",
    privacyDetailsTitle: "Privacy and regional rights", privacyDetailsSummary: "GDPR and equivalent laws worldwide",
    privacyDetailsLocal: "Relay has no telemetry, analytics, advertising or developer-operated collection service. It does not create a remote Relay account or user profile.",
    privacyDetailsFlow: "Discord messages necessarily pass through Discord before reaching the bot. OBS sources and Windows widgets communicate locally through 127.0.0.1. Credentials are protected by Windows and preferences remain in the local application configuration.",
    privacyDetailsRights: "Your applicable rights depend on your place of residence and local law, including the GDPR in the EU/EEA and equivalent privacy laws in other regions. Relay does not infer or collect your location.",
    privacyDisclaimer: "Product information only—not legal advice. External services remain governed by their own privacy terms.",
    privacyGlobalReference: "View worldwide privacy legislation — UNCTAD",
  },
  fr: {
    navOverview: "Aperçu", navMedia: "Médias", navOverlay: "Overlay", navModeration: "Modération", navHistory: "Historique", navHelp: "Aide", navAbout: "À propos",
    language: "Langue", appearance: "Apparence", light: "Clair", dark: "Sombre", overlays: "overlays",
    system: "Système", playback: "Lecture", output: "Sortie", safety: "Sécurité", archive: "Archives", guide: "Guide", about: "À propos",
    overviewKicker: "Diffusion locale", overviewTitle: "Un canal. Tous vos écrans.",
    overviewCopy: "Connectez Discord une fois, choisissez un canal, puis laissez le relais fonctionner discrètement dans la zone de notification.",
    credentialsTitle: "Connexion Discord", credentialsCopy: "Les identifiants sont chiffrés par Windows et ne sont jamais réaffichés.",
    clientId: "ID client Discord", botToken: "Token du bot Discord", connectBot: "Chiffrer et démarrer le bot",
    inviteUrl: "URL d’invitation du bot", copy: "Copier", copied: "Copié",
    routingTitle: "Routage d’entrée", routingCopy: "Choisissez un canal Discord pour les médias et un autre pour les messages lus.",
    mediaChannel: "Canal des médias", ttsChannel: "Canal des messages TTS", localPort: "Port local", saveRouting: "Enregistrer le routage",
    selectChannel: "Sélectionner un canal texte visible", ttsDisabled: "TTS désactivé", unavailableChannel: "Canal indisponible",
    refreshChannels: "Actualiser les salons", channelsRefreshed: "Liste des salons mise à jour",
    mediaKicker: "File de lecture", mediaTitle: "Vos médias, à votre rythme.",
    mediaCopy: "Les images utilisent une durée d’affichage. Les vidéos et les audios continuent naturellement jusqu’à leur fin.",
    transportLabel: "Contrôle en direct", transportReady: "Prêt pour le prochain élément", skip: "Passer l’élément actuel",
    playbackTitle: "Réglages de lecture", imageDuration: "Durée des images",
    imageDurationHelp: "Utilisée uniquement pour les images et GIF animés.", seconds: "sec",
    mediaVolume: "Volume des médias", mediaVolumeHelp: "Appliqué aux vidéos, aux audios et aux messages lus.",
    widgetSound: "Son du widget", widgetSoundHelp: "Joue le son des vidéos et des audios dans le widget Windows. Les sources OBS gardent leur propre son.",
    ttsCharacterLimit: "Limite de caractères TTS", ttsCharacterLimitHelp: "Utilisez 0 pour une longueur de message illimitée.", characters: "car.",
    ttsQueueLimit: "Capacité de la file TTS", ttsQueueLimitHelp: "Nombre maximal de messages en attente, de 1 à 50.", items: "éléments",
    ttsSpeech: "Voix TTS", ttsSpeechHelp: "Désactivée, les messages TTS deviennent des notifications silencieuses.",
    obsNotifications: "Afficher les notifications TTS dans OBS", obsNotificationsHelp: "Affiche l’auteur et le message pendant la lecture TTS.",
    obsNotificationOutput: "Overlay de notifications TTS OBS", obsNotificationOutputHelp: "Source navigateur indépendante affichée uniquement dans OBS.",
    enableObsNotifications: "Activer l’overlay OBS", enableObsNotificationsHelp: "Ne modifie pas le widget Windows.",
    windowsNotificationWidget: "Widget de notifications TTS Windows", windowsNotificationWidgetHelp: "Fenêtre de bureau indépendante, positionnable sur n’importe quel écran.",
    notificationSound: "Son de notification", notificationSoundHelp: "Joué par le widget Windows à chaque message. N’importe quel fichier audio de 10 secondes maximum.",
    chooseNotificationSound: "Choisir un fichier audio", resetNotificationSound: "Retirer le son", noNotificationSound: "Aucun fichier sélectionné.",
    notificationSoundObs: "Son de notification dans OBS", notificationSoundObsHelp: "L’overlay de notifications OBS joue le même son choisi, audible sur le stream.",
    showAuthor: "Afficher l’auteur", showAuthorHelp: "Affiche le nom Discord et l’avatar sur le média.",
    supportedFormats: "Images, GIF, MP4/WebM et formats audio courants sont acceptés.", savePlayback: "Enregistrer la lecture",
    overlayKicker: "Sortie programme", overlayTitle: "Ce qu’OBS reçoit.",
    overlayCopy: "La zone reste transparente jusqu’à l’arrivée d’un média.", livePreview: "Aperçu en direct",
    transparentCanvas: "Zone transparente", browserSource: "Sources navigateur OBS",
    browserSourceHelp: "Ajoutez chaque URL privée comme source navigateur OBS séparée.", visualSource: "Médias visuels", ttsSource: "Audio TTS", notificationSource: "Notifications TTS",
    audioSource: "Audio, musique et messages vocaux", regenerateSecret: "Reconnecter les sources OBS",
    floatingWidget: "Widget flottant des médias", widgetHelp: "Déverrouillez, placez-le sur un écran, puis verrouillez-le pour laisser passer les clics.",
    notificationWidget: "Widget de notifications TTS", notificationWidgetHelp: "Affichez-le sous Windows, déplacez-le librement, puis verrouillez-le pour laisser passer les clics.", showNotificationWidget: "Afficher sous Windows",
    historyKicker: "50 derniers éléments", historyTitle: "Historique des médias",
    historyCopy: "Relancez un média précédent ou effacez tous les overlays connectés.", clearOverlay: "Effacer l’overlay",
    historyEmpty: "En attente du premier média Discord.", replay: "Relancer",
    moderationKicker: "Sécurité de diffusion", moderationTitle: "Vous décidez de ce qui atteint OBS.",
    moderationCopy: "Placez facultativement les médias en attente jusqu’à leur validation locale.", moderationSettings: "Réglages de modération",
    enableModeration: "Activer la modération manuelle", enableModerationHelp: "Si elle est désactivée, les médias continuent directement vers OBS.",
    allowImages: "Images et GIF", allowImagesHelp: "Autorise ces éléments à entrer dans la file de validation.",
    allowVideos: "Vidéos", allowVideosHelp: "Autorise les vidéos à entrer dans la file de validation.",
    allowAudio: "Sons", allowAudioHelp: "Autorise les fichiers audio à entrer dans la file de validation.",
    moderationLocalOnly: "Les décisions restent locales et ne notifient jamais les utilisateurs Discord.", saveModeration: "Enregistrer la modération",
    pendingMedia: "Médias en attente", clearPending: "Tout refuser", moderationEmpty: "Aucun média n’attend de validation.",
    moderationDisabled: "La modération manuelle est désactivée.", approve: "Approuver", reject: "Refuser",
    botOffline: "Bot hors ligne", serverOnline: "Serveur en ligne", serverOffline: "Serveur hors ligne",
    notConfigured: "Non configuré", savedVia: "Enregistré via", encrypting: "Chiffrement…",
    encryptedStarting: "Chiffré ; démarrage du bot", saving: "Enregistrement…", saved: "Enregistré",
    regenerating: "Reconnexion…", secretRegenerated: "Liens permanents conservés", skipped: "Élément actuel passé",
    widgetHidden: "Widget masqué", widgetVisibleLocked: "Widget visible et verrouillé",
    widgetVisibleMovable: "Widget visible et déplaçable", showWidget: "Afficher le widget", hideWidget: "Masquer le widget",
    unlockMove: "Déverrouiller", lockDisplay: "Verrouiller", unknownAuthor: "Auteur inconnu",
    helpKicker: "Guide d’installation", helpTitle: "De Discord vers OBS.",
    helpCopy: "Suivez ces étapes une fois, puis laissez Relay fonctionner discrètement dans la zone de notification.",
    helpStartTitle: "Ordre d’installation recommandé",
    helpStartCopy: "Application Discord, permissions, salons, sources OBS, widgets, puis test réel.",
    helpDiscordTitle: "Créer le bot Discord", helpDiscordSummary: "Application, token du bot et ID client",
    helpDiscordStep1: "Ouvrez le portail développeur Discord et créez une nouvelle application.",
    helpDiscordStep2: "Ouvrez Bot, créez ou réinitialisez le token, puis copiez-le une seule fois.",
    helpDiscordStep3: "Copiez l’Application ID dans General Information ; il s’agit de l’ID client.",
    helpDiscordStep4: "Collez les deux valeurs dans Relay. Elles sont chiffrées par Windows.",
    openDiscordPortal: "Ouvrir le portail développeur Discord",
    helpIntentTitle: "Activer les permissions et intents", helpIntentSummary: "Requis pour lire les messages normaux",
    helpIntentStep1: "Dans Bot → Privileged Gateway Intents, activez Message Content Intent.",
    helpIntentStep2: "Utilisez l’URL d’invitation de Relay pour ajouter le bot à votre serveur Discord.",
    helpIntentStep3: "Accordez au bot Voir le salon et Voir les anciens messages dans les deux salons surveillés, Gérer les rôles pour le verrouillage et Gérer les messages pour le nettoyage.",
    helpIntentNote: "« Disallowed gateway intents » signifie que Message Content Intent est encore désactivé ; inutile de régénérer le token.",
    helpChannelsTitle: "Configurer les salons Discord", helpChannelsSummary: "Séparer les médias et les messages lus",
    helpChannelsStep1: "Créez un salon texte pour les images, GIF, vidéos et fichiers audio.",
    helpChannelsStep2: "Créez un second salon texte pour les messages TTS simples.",
    helpChannelsStep3: "Sélectionnez les deux salons dans Aperçu et enregistrez le routage.",
    helpChannelsStep4: "Les messages français et anglais utilisent automatiquement la voix Windows correspondante.",
    helpObsTitle: "Installer les sources navigateur OBS", helpObsSummary: "Médias, audio TTS et notifications",
    helpObsStep1: "Dans OBS, ajoutez une source navigateur séparée pour chaque URL affichée dans Overlay.",
    helpObsStep2: "Utilisez l’URL visuelle pour les médias et conservez son fond transparent.",
    helpObsStep3: "Utilisez l’URL TTS comme source audio dédiée et activez le contrôle audio par OBS.",
    helpObsStep4: "Utilisez l’URL Notifications pour la carte de message de style PS5.",
    helpObsStep5: "Conservez les URL privées ; Relay les reconnecte après chaque redémarrage.",
    openObsGuide: "Ouvrir le guide des sources navigateur OBS",
    helpWidgetsTitle: "Placer les widgets Windows", helpWidgetsSummary: "Déplacer, verrouiller et afficher sur tout écran",
    helpWidgetsStep1: "Ouvrez Overlay et affichez le widget média ou notifications.",
    helpWidgetsStep2: "Laissez-le déverrouillé, puis déplacez-le sur l’écran et à la position voulus.",
    helpWidgetsStep3: "Verrouillez-le pour laisser passer les clics vers les applications situées dessous.",
    helpWidgetsStep4: "La visibilité, le verrouillage et la position sont restaurés au redémarrage.",
    helpTroubleshootingTitle: "Dépannage", helpTroubleshootingSummary: "Vérifications rapides si rien ne s’affiche",
    helpTroubleshooting1: "Aucun média : vérifiez le salon média sélectionné et les permissions du bot.",
    helpTroubleshooting2: "Aucun TTS : vérifiez le salon TTS séparé et les packs vocaux Windows.",
    helpTroubleshooting3: "Aucune notification : activez les notifications TTS dans OBS, puis envoyez un nouveau message.",
    helpTroubleshooting4: "Source OBS vide : actualisez-la une fois et confirmez que Relay tourne sur le port local affiché.",
    helpTroubleshooting5: "Ne régénérez jamais le token pour une erreur d’intent ; activez plutôt l’intent demandé.",
    aboutKicker: "À propos de Relay", aboutTitle: "Construit localement. Fait pour disparaître.",
    aboutCopy: "Relay connecte Discord à OBS et Windows tout en gardant les identifiants et le trafic sur cet ordinateur.",
    aboutStatement: "Un utilitaire de diffusion privé pour les médias, messages lus et notifications, avec des sources OBS locales permanentes.",
    aboutCreatorLabel: "Créateur", aboutCreatorCopy: "Découvrez les projets et le travail source du créateur sur GitHub.",
    aboutPrivacy: "Identifiants chiffrés par Windows", aboutNetwork: "Serveur local · 127.0.0.1",
    privacyCardTitle: "Aucune collecte de données par Relay",
    privacyCardCopy: "Aucune télémétrie, analyse, publicité ni profil utilisateur détenu par le développeur. Les réglages restent sur cet ordinateur ; Discord demeure un service externe.",
    privacyCardLink: "Confidentialité et droits régionaux",
    privacyDetailsTitle: "Confidentialité et droits régionaux", privacyDetailsSummary: "RGPD et législations équivalentes dans le monde",
    privacyDetailsLocal: "Relay ne contient aucun service de télémétrie, d’analyse, de publicité ou de collecte exploité par le développeur. Aucun compte Relay distant ni profil utilisateur n’est créé.",
    privacyDetailsFlow: "Les messages passent nécessairement par Discord avant d’atteindre le bot. Les sources OBS et widgets Windows communiquent localement via 127.0.0.1. Les identifiants sont protégés par Windows et les préférences restent dans la configuration locale.",
    privacyDetailsRights: "Vos droits applicables dépendent de votre lieu de résidence et de la loi locale, notamment le RGPD dans l’UE/EEE et les lois équivalentes dans les autres régions. Relay ne déduit ni ne collecte votre position.",
    privacyDisclaimer: "Information sur le produit uniquement — ceci ne constitue pas un conseil juridique. Les services externes restent soumis à leurs propres règles de confidentialité.",
    privacyGlobalReference: "Voir les législations mondiales sur la confidentialité — CNUCED",
  },
};

Object.assign(translations.en, {
  stickerDuration: "Sticker duration", stickerDurationHelp: "Discord stickers stay visible for this duration.",
  notificationDuration: "Notification duration", notificationDurationHelp: "TTS notifications without audio stay visible for this duration.",
  durationsGroup: "Display durations", durationsGroupHelp: "How long images, stickers, notifications and GIFs stay visible.",
  audioTtsGroup: "Audio and TTS", audioTtsGroupHelp: "Volume, message length, queue and voice.",
  displayGroup: "Display", displayGroupHelp: "What appears over the media.",
  stickerSource: "Discord stickers",
  navPersonalization: "Personalization",
  personalizationKicker: "Interface",
  personalizationTitle: "Make Relay yours.",
  personalizationCopy: "Changes apply immediately to Relay, Windows widgets and OBS outputs.",
  themeLabel: "Theme", accentColor: "Accent color", fontSize: "Text size",
  previewTitle: "Relay interface preview", previewCopy: "Readable text with your selected accent color.",
  previewButton: "Example button", resetDefaults: "Restore defaults", personalizationSaved: "Preferences applied",
});

Object.assign(translations.fr, {
  stickerDuration: "Durée des stickers", stickerDurationHelp: "Les stickers Discord restent visibles pendant cette durée.",
  notificationDuration: "Durée des notifications", notificationDurationHelp: "Les notifications TTS sans audio restent visibles pendant cette durée.",
  durationsGroup: "Durées d’affichage", durationsGroupHelp: "Temps de visibilité des images, stickers, notifications et GIF.",
  audioTtsGroup: "Audio et TTS", audioTtsGroupHelp: "Volume, longueur des messages, file d’attente et voix.",
  displayGroup: "Affichage", displayGroupHelp: "Ce qui apparaît au-dessus des médias.",
  stickerSource: "Stickers Discord",
  mediaCopy: "Les images et les GIF utilisent des durées distinctes. Les vidéos et les audios continuent naturellement jusqu’à leur fin.",
  gifDuration: "Durée des GIF",
  imageDurationHelp: "Utilisée uniquement pour les images fixes.",
  gifDurationHelp: "Les GIF animés bouclent pendant cette durée.",
  navPersonalization: "Personnalisation",
  personalizationKicker: "Interface",
  personalizationTitle: "Personnalisez Relay.",
  personalizationCopy: "Les changements s’appliquent immédiatement à Relay, aux widgets Windows et aux sorties OBS.",
  themeLabel: "Thème", accentColor: "Couleur d’accent", fontSize: "Taille du texte",
  previewTitle: "Aperçu de l’interface Relay", previewCopy: "Un texte lisible avec la couleur choisie.",
  previewButton: "Bouton d’exemple", resetDefaults: "Restaurer les valeurs", personalizationSaved: "Préférences appliquées",
});

translations.es = {
  navOverview: "Resumen", navMedia: "Medios", navOverlay: "Overlay", navModeration: "Moderación", navHistory: "Historial", navHelp: "Ayuda", navPersonalization: "Personalización", navAbout: "Acerca de",
  language: "Idioma", appearance: "Apariencia", light: "Claro", dark: "Oscuro", overlays: "superposiciones",
  system: "Sistema", playback: "Reproducción", output: "Salida", safety: "Seguridad", archive: "Archivo", guide: "Guía", about: "Acerca de",
  overviewKicker: "Emisión local", overviewTitle: "Un canal. Todas tus pantallas.", overviewCopy: "Conecta Discord una vez, elige un canal y deja que Relay funcione discretamente en la bandeja del sistema.",
  credentialsTitle: "Conexión con Discord", credentialsCopy: "Las credenciales se cifran con Windows y no vuelven a mostrarse.",
  clientId: "ID de cliente de Discord", botToken: "Token del bot de Discord", connectBot: "Cifrar e iniciar el bot", inviteUrl: "URL de invitación del bot", copy: "Copiar", copied: "Copiado",
  routingTitle: "Enrutamiento de entrada", routingCopy: "Elige un canal de Discord para los medios y otro para los mensajes hablados.",
  mediaChannel: "Canal de medios", ttsChannel: "Canal de mensajes TTS", localPort: "Puerto local", saveRouting: "Guardar enrutamiento",
  selectChannel: "Selecciona un canal de texto visible", ttsDisabled: "TTS desactivado", unavailableChannel: "Canal no disponible",
  refreshChannels: "Actualizar canales", channelsRefreshed: "Lista de canales actualizada",
  mediaKicker: "Cola de reproducción", mediaTitle: "Tus medios, a tu manera.", mediaCopy: "Las imágenes y los GIF usan duraciones distintas. Los vídeos y audios continúan hasta terminar.",
  transportLabel: "Control en directo", transportReady: "Listo para el siguiente elemento", skip: "Omitir elemento actual",
  playbackTitle: "Ajustes de reproducción", imageDuration: "Duración de imágenes", gifDuration: "Duración de GIF", imageDurationHelp: "Solo para imágenes estáticas.", gifDurationHelp: "Los GIF animados se repiten durante este tiempo.", seconds: "s",
  mediaVolume: "Volumen de medios", mediaVolumeHelp: "Se aplica a vídeos, audio y mensajes hablados.",
  widgetSound: "Sonido del widget", widgetSoundHelp: "Reproduce el sonido de vídeos y audios en el widget de Windows. Las fuentes de OBS conservan su propio audio.",
  ttsCharacterLimit: "Límite de caracteres TTS", ttsCharacterLimitHelp: "Usa 0 para una longitud ilimitada.", characters: "car.", ttsQueueLimit: "Capacidad de la cola TTS", ttsQueueLimitHelp: "Máximo de mensajes en espera, de 1 a 50.", items: "elementos",
  ttsSpeech: "Voz TTS", ttsSpeechHelp: "Desactivada, los mensajes TTS se muestran como notificaciones silenciosas.",
  obsNotifications: "Mostrar notificaciones TTS en OBS", obsNotificationsHelp: "Muestra el autor y el mensaje mientras habla el TTS.", obsNotificationOutput: "Overlay de notificaciones TTS de OBS", obsNotificationOutputHelp: "Fuente de navegador independiente visible solo en OBS.",
  enableObsNotifications: "Activar overlay de OBS", enableObsNotificationsHelp: "No modifica el widget de Windows.", windowsNotificationWidget: "Widget de notificaciones TTS de Windows", windowsNotificationWidgetHelp: "Ventana independiente que puede colocarse en cualquier pantalla.",
  notificationSound: "Sonido de notificación", notificationSoundHelp: "Reproducido por el widget de Windows con cada mensaje. Cualquier archivo de audio de hasta 10 segundos.",
  chooseNotificationSound: "Elegir archivo de audio", resetNotificationSound: "Quitar sonido", noNotificationSound: "Ningún archivo seleccionado.",
  notificationSoundObs: "Sonido de notificación en OBS", notificationSoundObsHelp: "El overlay de notificaciones de OBS reproduce el mismo sonido elegido, audible en el stream.",
  showAuthor: "Mostrar autor", showAuthorHelp: "Muestra el nombre y avatar de Discord sobre el medio.", supportedFormats: "Se aceptan imágenes, GIF, MP4/WebM y formatos de audio habituales.", savePlayback: "Guardar reproducción",
  overlayKicker: "Salida del programa", overlayTitle: "Lo que recibe OBS.", overlayCopy: "El lienzo permanece transparente hasta que entra un medio en la cola.", livePreview: "Vista previa en directo", transparentCanvas: "Lienzo transparente", browserSource: "Fuentes de navegador de OBS", browserSourceHelp: "Añade cada URL privada como una fuente de navegador independiente.",
  visualSource: "Medios visuales", ttsSource: "Audio TTS", notificationSource: "Notificaciones TTS", audioSource: "Audio, música y mensajes de voz", regenerateSecret: "Reconectar fuentes OBS",
  floatingWidget: "Widget flotante de medios", widgetHelp: "Desbloquéalo, colócalo en cualquier pantalla y vuelve a bloquearlo para que los clics lo atraviesen.", notificationWidget: "Widget de notificaciones TTS", notificationWidgetHelp: "Muéstralo en Windows, muévelo y bloquéalo para que los clics lo atraviesen.", showNotificationWidget: "Mostrar en Windows",
  historyKicker: "Últimos 50 elementos", historyTitle: "Historial de medios", historyCopy: "Reproduce un elemento anterior o limpia todos los overlays conectados.", clearOverlay: "Limpiar overlay", historyEmpty: "Esperando el primer medio de Discord.", replay: "Repetir",
  moderationKicker: "Seguridad de emisión", moderationTitle: "Tú decides qué llega a OBS.", moderationCopy: "Opcionalmente, retén los medios entrantes hasta aprobarlos localmente.", moderationSettings: "Ajustes de moderación",
  enableModeration: "Activar moderación manual", enableModerationHelp: "Si está desactivada, los medios llegan directamente a OBS.", allowImages: "Imágenes y GIF", allowImagesHelp: "Permite que estos elementos entren en la cola de aprobación.", allowVideos: "Vídeos", allowVideosHelp: "Permite que los vídeos entren en la cola de aprobación.", allowAudio: "Audio", allowAudioHelp: "Permite que los audios entren en la cola de aprobación.",
  moderationLocalOnly: "Las decisiones permanecen locales y no notifican a los usuarios de Discord.", saveModeration: "Guardar moderación", pendingMedia: "Medios pendientes", clearPending: "Rechazar todo", moderationEmpty: "No hay medios esperando aprobación.", moderationDisabled: "La moderación manual está desactivada.", approve: "Aprobar", reject: "Rechazar",
  botOffline: "Bot desconectado", serverOnline: "Servidor en línea", serverOffline: "Servidor desconectado", notConfigured: "Sin configurar", savedVia: "Guardado mediante", encrypting: "Cifrando…", encryptedStarting: "Cifrado; iniciando bot", saving: "Guardando…", saved: "Guardado", regenerating: "Reconectando…", secretRegenerated: "Enlaces permanentes conservados", skipped: "Elemento actual omitido",
  widgetHidden: "Widget oculto", widgetVisibleLocked: "Widget visible y bloqueado", widgetVisibleMovable: "Widget visible y movible", showWidget: "Mostrar widget", hideWidget: "Ocultar widget", unlockMove: "Desbloquear para mover", lockDisplay: "Bloquear visualización", unknownAuthor: "Autor desconocido",
  helpKicker: "Guía de configuración", helpTitle: "De Discord a OBS.", helpCopy: "Sigue estos pasos una vez y deja Relay funcionando discretamente en la bandeja.", helpStartTitle: "Orden de configuración recomendado", helpStartCopy: "Aplicación de Discord, permisos, canales, fuentes OBS, widgets y una prueba real.",
  helpDiscordTitle: "Crear el bot de Discord", helpDiscordSummary: "Aplicación, token del bot e ID de cliente", helpDiscordStep1: "Abre el Portal de desarrolladores de Discord y crea una nueva aplicación.", helpDiscordStep2: "Abre Bot, crea o restablece el token y cópialo una vez.", helpDiscordStep3: "Copia el ID de aplicación de Información general; es el ID de cliente.", helpDiscordStep4: "Pega ambos valores en Relay. Windows los cifra.", openDiscordPortal: "Abrir Portal de desarrolladores de Discord",
  helpIntentTitle: "Activar permisos e intents", helpIntentSummary: "Necesarios para leer mensajes normales", helpIntentStep1: "En Bot → Privileged Gateway Intents, activa Message Content Intent.", helpIntentStep2: "Usa la URL de invitación de Relay para añadir el bot al servidor.", helpIntentStep3: "Concede Ver canal y Leer historial de mensajes, Gestionar roles para el bloqueo y Gestionar mensajes para la limpieza.", helpIntentNote: "“Disallowed gateway intents” significa que Message Content Intent sigue desactivado; no regeneres el token.",
  helpChannelsTitle: "Configurar canales de Discord", helpChannelsSummary: "Separar medios y mensajes hablados", helpChannelsStep1: "Crea un canal de texto para imágenes, GIF, vídeos y audio.", helpChannelsStep2: "Crea otro canal para mensajes TTS de texto.", helpChannelsStep3: "Selecciona ambos canales en Resumen y guarda el enrutamiento.", helpChannelsStep4: "Los mensajes en francés e inglés usan automáticamente la voz de Windows correspondiente.",
  helpObsTitle: "Instalar las fuentes de navegador de OBS", helpObsSummary: "Medios, audio TTS y notificaciones", helpObsStep1: "En OBS, añade una fuente de navegador distinta para cada URL de Overlay.", helpObsStep2: "Usa la URL visual para los medios y conserva el fondo transparente.", helpObsStep3: "Usa la URL TTS como fuente de audio dedicada y activa el control de audio de OBS.", helpObsStep4: "Usa la URL de notificaciones para la tarjeta de mensajes estilo PS5.", helpObsStep5: "No cambies las URL privadas; Relay las reconecta tras cada reinicio.", openObsGuide: "Abrir guía de fuentes de navegador OBS",
  helpWidgetsTitle: "Colocar los widgets de Windows", helpWidgetsSummary: "Mover, bloquear y mostrar en cualquier pantalla", helpWidgetsStep1: "Abre Overlay y muestra el widget de medios o notificaciones.", helpWidgetsStep2: "Déjalo desbloqueado y arrástralo a la pantalla y posición deseadas.", helpWidgetsStep3: "Bloquéalo para que los clics lleguen a las aplicaciones situadas debajo.", helpWidgetsStep4: "La visibilidad, el bloqueo y la posición se restauran al reiniciar.",
  helpTroubleshootingTitle: "Solución de problemas", helpTroubleshootingSummary: "Comprobaciones rápidas si no aparece nada", helpTroubleshooting1: "Sin medios: comprueba el canal seleccionado y los permisos del bot.", helpTroubleshooting2: "Sin TTS: comprueba el canal TTS separado y los paquetes de voz de Windows.", helpTroubleshooting3: "Sin notificación: activa las notificaciones TTS en OBS y envía otro mensaje.", helpTroubleshooting4: "Fuente OBS vacía: actualízala una vez y confirma que Relay usa el puerto mostrado.", helpTroubleshooting5: "No regeneres el token por un error de intent; activa el intent solicitado.",
  aboutKicker: "Acerca de Relay", aboutTitle: "Local por diseño. Hecho para desaparecer.", aboutCopy: "Relay conecta Discord con OBS y Windows manteniendo credenciales y tráfico en este equipo.", aboutStatement: "Una utilidad privada de emisión para medios, mensajes hablados y notificaciones, con fuentes OBS locales permanentes.", aboutCreatorLabel: "Creador", aboutCreatorCopy: "Explora los proyectos y el código del creador en GitHub.", aboutPrivacy: "Credenciales cifradas por Windows", aboutNetwork: "Servidor local · 127.0.0.1",
  privacyCardTitle: "Relay no recopila datos", privacyCardCopy: "Sin telemetría, analítica, publicidad ni perfil remoto. Los ajustes permanecen en este equipo; Discord sigue siendo un servicio externo.", privacyCardLink: "Privacidad y derechos regionales", privacyDetailsTitle: "Privacidad y derechos regionales", privacyDetailsSummary: "RGPD y leyes equivalentes en todo el mundo", privacyDetailsLocal: "Relay no contiene servicios de telemetría, analítica, publicidad ni recopilación operados por el desarrollador. No crea cuentas ni perfiles remotos.", privacyDetailsFlow: "Los mensajes pasan por Discord antes de llegar al bot. Las fuentes OBS y los widgets se comunican localmente mediante 127.0.0.1. Windows protege las credenciales y las preferencias permanecen en la configuración local.", privacyDetailsRights: "Tus derechos dependen de tu residencia y legislación local, incluido el RGPD en la UE/EEE y normas equivalentes. Relay no deduce ni recopila tu ubicación.", privacyDisclaimer: "Información del producto; no constituye asesoramiento jurídico. Los servicios externos se rigen por sus propias normas.", privacyGlobalReference: "Consultar legislación mundial de privacidad — UNCTAD",
  personalizationKicker: "Interfaz", personalizationTitle: "Haz que Relay sea tuyo.", personalizationCopy: "Los cambios se aplican inmediatamente a Relay, los widgets de Windows y las salidas OBS.",
  themeLabel: "Tema", accentColor: "Color de acento", fontSize: "Tamaño del texto", previewTitle: "Vista previa de Relay", previewCopy: "Texto legible con el color seleccionado.", previewButton: "Botón de ejemplo", resetDefaults: "Restaurar valores", personalizationSaved: "Preferencias aplicadas",
};

translations.de = {
  navOverview: "Übersicht", navMedia: "Medien", navOverlay: "Overlay", navModeration: "Moderation", navHistory: "Verlauf", navHelp: "Hilfe", navPersonalization: "Personalisierung", navAbout: "Info",
  language: "Sprache", appearance: "Darstellung", light: "Hell", dark: "Dunkel", overlays: "Overlays", system: "System", playback: "Wiedergabe", output: "Ausgabe", safety: "Sicherheit", archive: "Archiv", guide: "Anleitung", about: "Info",
  overviewKicker: "Lokale Übertragung", overviewTitle: "Ein Kanal. Alle Bildschirme.", overviewCopy: "Verbinde Discord einmal, wähle einen Kanal und lasse Relay unauffällig im Infobereich laufen.",
  credentialsTitle: "Discord-Verbindung", credentialsCopy: "Die Zugangsdaten werden von Windows verschlüsselt und nie erneut angezeigt.", clientId: "Discord-Client-ID", botToken: "Discord-Bot-Token", connectBot: "Verschlüsseln und Bot starten", inviteUrl: "Einladungs-URL des Bots", copy: "Kopieren", copied: "Kopiert",
  routingTitle: "Eingangszuordnung", routingCopy: "Wähle einen Discord-Kanal für Medien und einen weiteren für gesprochene Nachrichten.", mediaChannel: "Medienkanal", ttsChannel: "TTS-Nachrichtenkanal", localPort: "Lokaler Port", saveRouting: "Zuordnung speichern", selectChannel: "Sichtbaren Textkanal auswählen", ttsDisabled: "TTS deaktiviert", unavailableChannel: "Kanal nicht verfügbar", refreshChannels: "Kanäle aktualisieren", channelsRefreshed: "Kanalliste aktualisiert",
  mediaKicker: "Wiedergabewarteschlange", mediaTitle: "Medien nach deinen Regeln.", mediaCopy: "Bilder und GIFs verwenden getrennte Anzeigedauern. Videos und Audio laufen bis zum Ende.", transportLabel: "Live-Steuerung", transportReady: "Bereit für das nächste Element", skip: "Aktuelles Element überspringen",
  playbackTitle: "Wiedergabeeinstellungen", imageDuration: "Bilddauer", gifDuration: "GIF-Dauer", imageDurationHelp: "Nur für statische Bilder.", gifDurationHelp: "Animierte GIFs wiederholen sich für diese Dauer.", seconds: "Sek.", mediaVolume: "Medienlautstärke", mediaVolumeHelp: "Gilt für Video, Audio und gesprochene Nachrichten.", ttsCharacterLimit: "TTS-Zeichenlimit", ttsCharacterLimitHelp: "0 bedeutet unbegrenzte Nachrichtenlänge.", characters: "Zeichen", ttsQueueLimit: "TTS-Warteschlangengröße", ttsQueueLimitHelp: "Maximal 1 bis 50 wartende Nachrichten.", items: "Elemente",
  widgetSound: "Widget-Ton", widgetSoundHelp: "Gibt den Ton von Videos und Audio im Windows-Widget wieder. OBS-Quellen behalten ihren eigenen Ton.",
  ttsSpeech: "TTS-Stimme", ttsSpeechHelp: "Deaktiviert erscheinen TTS-Nachrichten als stille Benachrichtigungen.",
  obsNotifications: "TTS-Benachrichtigungen in OBS anzeigen", obsNotificationsHelp: "Zeigt Autor und Nachricht während der TTS-Ausgabe.", obsNotificationOutput: "OBS-TTS-Benachrichtigungs-Overlay", obsNotificationOutputHelp: "Unabhängige Browserquelle, die nur in OBS angezeigt wird.", enableObsNotifications: "OBS-Overlay aktivieren", enableObsNotificationsHelp: "Ändert das Windows-Widget nicht.", windowsNotificationWidget: "Windows-TTS-Benachrichtigungswidget", windowsNotificationWidgetHelp: "Unabhängiges Fenster, das auf jedem Bildschirm platziert werden kann.",
  notificationSound: "Benachrichtigungston", notificationSoundHelp: "Wird vom Windows-Widget bei jeder Nachricht abgespielt. Beliebige Audiodatei mit maximal 10 Sekunden.",
  chooseNotificationSound: "Audiodatei auswählen", resetNotificationSound: "Ton entfernen", noNotificationSound: "Keine Datei ausgewählt.",
  notificationSoundObs: "Benachrichtigungston in OBS", notificationSoundObsHelp: "Das OBS-Benachrichtigungs-Overlay spielt denselben gewählten Ton, hörbar im Stream.",
  showAuthor: "Autor anzeigen", showAuthorHelp: "Zeigt Discord-Name und Avatar über dem Medium.", supportedFormats: "Bilder, GIFs, MP4/WebM und gängige Audioformate werden unterstützt.", savePlayback: "Wiedergabe speichern",
  overlayKicker: "Programmausgabe", overlayTitle: "Was OBS empfängt.", overlayCopy: "Die Fläche bleibt transparent, bis ein Medium die Warteschlange erreicht.", livePreview: "Live-Vorschau", transparentCanvas: "Transparente Fläche", browserSource: "OBS-Browserquellen", browserSourceHelp: "Füge jede private URL als separate OBS-Browserquelle hinzu.", visualSource: "Visuelle Medien", ttsSource: "TTS-Audio", notificationSource: "TTS-Benachrichtigungen", audioSource: "Audio, Musik und Sprachnachrichten", regenerateSecret: "OBS-Quellen neu verbinden",
  floatingWidget: "Schwebendes Medien-Widget", widgetHelp: "Entsperre es, platziere es auf einem Bildschirm und sperre es, damit Klicks hindurchgehen.", notificationWidget: "TTS-Benachrichtigungswidget", notificationWidgetHelp: "Zeige es unter Windows, verschiebe und sperre es, damit Klicks hindurchgehen.", showNotificationWidget: "Unter Windows anzeigen",
  historyKicker: "Letzte 50 Elemente", historyTitle: "Medienverlauf", historyCopy: "Spiele ein früheres Element erneut ab oder leere alle verbundenen Overlays.", clearOverlay: "Overlay leeren", historyEmpty: "Warte auf das erste Discord-Medium.", replay: "Wiederholen",
  moderationKicker: "Übertragungssicherheit", moderationTitle: "Du entscheidest, was OBS erreicht.", moderationCopy: "Halte eingehende Medien optional zurück, bis du sie lokal freigibst.", moderationSettings: "Moderationseinstellungen", enableModeration: "Manuelle Moderation aktivieren", enableModerationHelp: "Wenn deaktiviert, gelangen Medien direkt zu OBS.", allowImages: "Bilder und GIFs", allowImagesHelp: "Erlaubt diesen Elementen, in die Freigabewarteschlange zu gelangen.", allowVideos: "Videos", allowVideosHelp: "Erlaubt Videos in der Freigabewarteschlange.", allowAudio: "Audio", allowAudioHelp: "Erlaubt Audiodateien in der Freigabewarteschlange.", moderationLocalOnly: "Entscheidungen bleiben lokal und benachrichtigen keine Discord-Nutzer.", saveModeration: "Moderation speichern", pendingMedia: "Ausstehende Medien", clearPending: "Alle ablehnen", moderationEmpty: "Keine Medien warten auf Freigabe.", moderationDisabled: "Manuelle Moderation ist deaktiviert.", approve: "Freigeben", reject: "Ablehnen",
  botOffline: "Bot offline", serverOnline: "Server online", serverOffline: "Server offline", notConfigured: "Nicht konfiguriert", savedVia: "Gespeichert über", encrypting: "Wird verschlüsselt…", encryptedStarting: "Verschlüsselt; Bot wird gestartet", saving: "Wird gespeichert…", saved: "Gespeichert", regenerating: "Wird neu verbunden…", secretRegenerated: "Dauerhafte Links beibehalten", skipped: "Aktuelles Element übersprungen", widgetHidden: "Widget ausgeblendet", widgetVisibleLocked: "Widget sichtbar und gesperrt", widgetVisibleMovable: "Widget sichtbar und verschiebbar", showWidget: "Widget anzeigen", hideWidget: "Widget ausblenden", unlockMove: "Zum Verschieben entsperren", lockDisplay: "Anzeige sperren", unknownAuthor: "Unbekannter Autor",
  helpKicker: "Einrichtungsanleitung", helpTitle: "Von Discord zu OBS.", helpCopy: "Führe diese Schritte einmal aus und lasse Relay anschließend im Infobereich laufen.", helpStartTitle: "Empfohlene Einrichtungsreihenfolge", helpStartCopy: "Discord-Anwendung, Berechtigungen, Kanäle, OBS-Quellen, Widgets und ein Live-Test.",
  helpDiscordTitle: "Discord-Bot erstellen", helpDiscordSummary: "Anwendung, Bot-Token und Client-ID", helpDiscordStep1: "Öffne das Discord Developer Portal und erstelle eine neue Anwendung.", helpDiscordStep2: "Öffne Bot, erstelle oder erneuere das Token und kopiere es einmal.", helpDiscordStep3: "Kopiere die Anwendungs-ID aus den allgemeinen Informationen; das ist die Client-ID.", helpDiscordStep4: "Füge beide Werte in Relay ein. Windows verschlüsselt sie.", openDiscordPortal: "Discord Developer Portal öffnen",
  helpIntentTitle: "Berechtigungen und Intents aktivieren", helpIntentSummary: "Zum Lesen normaler Nachrichten erforderlich", helpIntentStep1: "Aktiviere unter Bot → Privileged Gateway Intents den Message Content Intent.", helpIntentStep2: "Füge den Bot mit der Einladungs-URL von Relay deinem Server hinzu.", helpIntentStep3: "Gewähre Kanal anzeigen und Nachrichtenverlauf lesen sowie Rollen verwalten für die Sperre und Nachrichten verwalten für die Bereinigung.", helpIntentNote: "„Disallowed gateway intents“ bedeutet, dass Message Content Intent noch deaktiviert ist; das Token muss nicht erneuert werden.",
  helpChannelsTitle: "Discord-Kanäle konfigurieren", helpChannelsSummary: "Medien und gesprochene Nachrichten trennen", helpChannelsStep1: "Erstelle einen Textkanal für Bilder, GIFs, Videos und Audio.", helpChannelsStep2: "Erstelle einen zweiten Kanal für einfache TTS-Nachrichten.", helpChannelsStep3: "Wähle beide Kanäle in der Übersicht und speichere die Zuordnung.", helpChannelsStep4: "Französische und englische Nachrichten verwenden automatisch die passende Windows-Stimme.",
  helpObsTitle: "OBS-Browserquellen installieren", helpObsSummary: "Medien, TTS-Audio und Benachrichtigungen", helpObsStep1: "Füge in OBS für jede unter Overlay angezeigte URL eine eigene Browserquelle hinzu.", helpObsStep2: "Verwende die visuelle URL für Medien und behalte den transparenten Hintergrund.", helpObsStep3: "Verwende die TTS-URL als eigene Audio-Browserquelle und aktiviere die OBS-Audiosteuerung.", helpObsStep4: "Verwende die Benachrichtigungs-URL für die Nachrichtenkarte im PS5-Stil.", helpObsStep5: "Lasse die privaten URLs unverändert; Relay verbindet sie nach jedem Neustart erneut.", openObsGuide: "Anleitung für OBS-Browserquellen öffnen",
  helpWidgetsTitle: "Windows-Widgets platzieren", helpWidgetsSummary: "Auf jedem Bildschirm verschieben, sperren und anzeigen", helpWidgetsStep1: "Öffne Overlay und zeige das Medien- oder Benachrichtigungswidget.", helpWidgetsStep2: "Lasse es entsperrt und ziehe es auf den gewünschten Bildschirm und an die gewünschte Position.", helpWidgetsStep3: "Sperre es, damit Mausklicks an darunterliegende Anwendungen weitergeleitet werden.", helpWidgetsStep4: "Sichtbarkeit, Sperrstatus und Position werden beim Neustart wiederhergestellt.",
  helpTroubleshootingTitle: "Fehlerbehebung", helpTroubleshootingSummary: "Schnelle Prüfungen, wenn nichts erscheint", helpTroubleshooting1: "Keine Medien: Prüfe den ausgewählten Medienkanal und die Bot-Berechtigungen.", helpTroubleshooting2: "Kein TTS: Prüfe den separaten TTS-Kanal und die Windows-Sprachpakete.", helpTroubleshooting3: "Keine Benachrichtigung: Aktiviere TTS-Benachrichtigungen in OBS und sende eine neue Nachricht.", helpTroubleshooting4: "Leere OBS-Quelle: Aktualisiere sie einmal und prüfe, ob Relay den angezeigten lokalen Port verwendet.", helpTroubleshooting5: "Erneuere bei einem Intent-Fehler niemals das Token; aktiviere stattdessen den angeforderten Intent.",
  aboutKicker: "Über Relay", aboutTitle: "Lokal entwickelt. Zum Verschwinden gemacht.", aboutCopy: "Relay verbindet Discord mit OBS und Windows und hält Zugangsdaten und Datenverkehr auf diesem Computer.", aboutStatement: "Ein privates Übertragungswerkzeug für Medien, gesprochene Nachrichten und Benachrichtigungen mit dauerhaften lokalen OBS-Quellen.", aboutCreatorLabel: "Ersteller", aboutCreatorCopy: "Entdecke die Projekte und Quellarbeiten des Erstellers auf GitHub.", aboutPrivacy: "Von Windows verschlüsselte Zugangsdaten", aboutNetwork: "Lokaler Server · 127.0.0.1",
  privacyCardTitle: "Keine Datenerfassung durch Relay", privacyCardCopy: "Keine Telemetrie, Analysen, Werbung oder vom Entwickler geführte Nutzerprofile. Lokale Einstellungen bleiben auf diesem Computer; Discord ist ein externer Dienst.", privacyCardLink: "Datenschutz und regionale Rechte", privacyDetailsTitle: "Datenschutz und regionale Rechte", privacyDetailsSummary: "DSGVO und vergleichbare Gesetze weltweit", privacyDetailsLocal: "Relay enthält keine vom Entwickler betriebenen Dienste für Telemetrie, Analysen, Werbung oder Datenerfassung. Es erstellt weder ein entferntes Konto noch ein Nutzerprofil.", privacyDetailsFlow: "Discord-Nachrichten durchlaufen Discord, bevor sie den Bot erreichen. OBS-Quellen und Windows-Widgets kommunizieren lokal über 127.0.0.1. Zugangsdaten werden von Windows geschützt und Einstellungen bleiben in der lokalen Konfiguration.", privacyDetailsRights: "Deine Rechte hängen von Wohnort und örtlichem Recht ab, einschließlich der DSGVO in EU/EWR und vergleichbarer Gesetze. Relay leitet deinen Standort weder ab noch erfasst es ihn.", privacyDisclaimer: "Nur Produktinformation, keine Rechtsberatung. Externe Dienste unterliegen ihren eigenen Datenschutzregeln.", privacyGlobalReference: "Weltweite Datenschutzgesetze ansehen — UNCTAD",
  personalizationKicker: "Oberfläche", personalizationTitle: "Gestalte Relay nach deinen Wünschen.", personalizationCopy: "Änderungen gelten sofort für Relay, Windows-Widgets und OBS-Ausgaben.",
  themeLabel: "Design", accentColor: "Akzentfarbe", fontSize: "Textgröße", previewTitle: "Relay-Vorschau", previewCopy: "Lesbarer Text mit deiner Akzentfarbe.", previewButton: "Beispielschaltfläche", resetDefaults: "Standard wiederherstellen", personalizationSaved: "Einstellungen angewendet",
};

Object.assign(translations.en, {
  navCommands: "Commands", commandsKicker: "Discord controls", commandsTitle: "Commands, under your control.",
  commandsCopy: "Enable only the Relay commands you want available in Discord.", commandsSettings: "Command availability",
  commandChannelHelp: "Choose the Discord media channel.", commandUrlHelp: "Show local Relay and OBS URLs ephemerally.",
  commandShowHelp: "Show the active Relay configuration.", commandRegenerateHelp: "Reconnect local outputs without changing their URLs.",
  commandClearHelp: "Delete the requested number of messages from one Discord channel selected in the command.", commandLockHelp: "Toggle the configured media channel lock.",
  commandChangelogHelp: "Post the latest release notes from GitHub into a chosen channel.",
  commandLockInactive: "The media channel is currently unlocked.", commandLockActive: "The media channel is locked. /relay lock remains available for unlocking.",
  saveCommands: "Save commands", commandsSaved: "Command availability saved",
  commandsPermission: "Channel locking requires Manage Roles; clearing requires Manage Messages. Commands are restricted to Discord administrators.",
});
Object.assign(translations.fr, {
  navCommands: "Commandes", commandsKicker: "Contrôles Discord", commandsTitle: "Vos commandes, vos règles.",
  commandsCopy: "Activez uniquement les commandes Relay que vous souhaitez rendre disponibles dans Discord.", commandsSettings: "Disponibilité des commandes",
  commandChannelHelp: "Choisit le salon Discord des médias.", commandUrlHelp: "Affiche les URL locales Relay et OBS de façon éphémère.",
  commandShowHelp: "Affiche la configuration Relay active.", commandRegenerateHelp: "Reconnecte les sorties locales sans modifier leurs URL.",
  commandClearHelp: "Supprime le nombre demandé de messages dans le salon Discord choisi dans la commande.", commandLockHelp: "Verrouille ou déverrouille le salon média configuré.",
  commandChangelogHelp: "Publie les dernières notes de version depuis GitHub dans le salon choisi.",
  commandLockInactive: "Le salon média est actuellement déverrouillé.", commandLockActive: "Le salon média est verrouillé. /relay lock reste disponible pour le déverrouiller.",
  saveCommands: "Enregistrer les commandes", commandsSaved: "Disponibilité des commandes enregistrée",
  commandsPermission: "Le verrouillage nécessite Gérer les rôles ; le nettoyage nécessite Gérer les messages. Les commandes sont réservées aux administrateurs Discord.",
});
Object.assign(translations.es, {
  stickerDuration: "Duración de stickers", stickerDurationHelp: "Los stickers de Discord permanecen visibles durante este tiempo.",
  notificationDuration: "Duración de notificaciones", notificationDurationHelp: "Las notificaciones TTS sin audio permanecen visibles durante este tiempo.",
  durationsGroup: "Duraciones de visualización", durationsGroupHelp: "Tiempo de visibilidad de imágenes, stickers, notificaciones y GIF.",
  audioTtsGroup: "Audio y TTS", audioTtsGroupHelp: "Volumen, longitud de mensajes, cola y voz.",
  displayGroup: "Visualización", displayGroupHelp: "Lo que aparece sobre los medios.",
  stickerSource: "Stickers de Discord",
  navCommands: "Comandos", commandsKicker: "Controles de Discord", commandsTitle: "Tus comandos, tus reglas.",
  commandsCopy: "Activa solo los comandos de Relay que quieras usar en Discord.", commandsSettings: "Disponibilidad de comandos",
  commandChannelHelp: "Elige el canal de medios de Discord.", commandUrlHelp: "Muestra de forma efímera las URL locales de Relay y OBS.",
  commandShowHelp: "Muestra la configuración activa de Relay.", commandRegenerateHelp: "Reconecta las salidas locales sin cambiar sus URL.",
  commandClearHelp: "Elimina el número solicitado de mensajes del canal Discord elegido en el comando.", commandLockHelp: "Bloquea o desbloquea el canal de medios configurado.",
  commandChangelogHelp: "Publica las últimas notas de versión desde GitHub en el canal elegido.",
  commandLockInactive: "El canal de medios está desbloqueado.", commandLockActive: "El canal de medios está bloqueado. /relay lock sigue disponible para desbloquearlo.",
  saveCommands: "Guardar comandos", commandsSaved: "Disponibilidad de comandos guardada",
  commandsPermission: "El bloqueo requiere Gestionar roles; la limpieza requiere Gestionar mensajes. Los comandos están restringidos a administradores de Discord.",
});
Object.assign(translations.de, {
  stickerDuration: "Sticker-Dauer", stickerDurationHelp: "Discord-Sticker bleiben für diese Dauer sichtbar.",
  notificationDuration: "Benachrichtigungsdauer", notificationDurationHelp: "TTS-Benachrichtigungen ohne Audio bleiben für diese Dauer sichtbar.",
  durationsGroup: "Anzeigedauern", durationsGroupHelp: "Sichtbarkeitsdauer von Bildern, Stickern, Benachrichtigungen und GIFs.",
  audioTtsGroup: "Audio und TTS", audioTtsGroupHelp: "Lautstärke, Nachrichtenlänge, Warteschlange und Stimme.",
  displayGroup: "Anzeige", displayGroupHelp: "Was über den Medien erscheint.",
  stickerSource: "Discord-Sticker",
  navCommands: "Befehle", commandsKicker: "Discord-Steuerung", commandsTitle: "Deine Befehle, deine Regeln.",
  commandsCopy: "Aktiviere nur die Relay-Befehle, die in Discord verfügbar sein sollen.", commandsSettings: "Befehlsverfügbarkeit",
  commandChannelHelp: "Wählt den Discord-Medienkanal.", commandUrlHelp: "Zeigt lokale Relay- und OBS-URLs ephemer an.",
  commandShowHelp: "Zeigt die aktive Relay-Konfiguration.", commandRegenerateHelp: "Verbindet lokale Ausgaben neu, ohne ihre URLs zu ändern.",
  commandClearHelp: "Löscht die angegebene Anzahl Nachrichten aus dem im Befehl gewählten Discord-Kanal.", commandLockHelp: "Sperrt oder entsperrt den konfigurierten Medienkanal.",
  commandChangelogHelp: "Veröffentlicht die neuesten Versionshinweise von GitHub im gewählten Kanal.",
  commandLockInactive: "Der Medienkanal ist derzeit entsperrt.", commandLockActive: "Der Medienkanal ist gesperrt. /relay lock bleibt zum Entsperren verfügbar.",
  saveCommands: "Befehle speichern", commandsSaved: "Befehlsverfügbarkeit gespeichert",
  commandsPermission: "Die Sperre erfordert Rollen verwalten; die Bereinigung erfordert Nachrichten verwalten. Befehle sind auf Discord-Administratoren beschränkt.",
});

Object.assign(translations.en, {
  sizeAndCrop: "Size and crop", sizeAndCropHelp: "Adjust each output independently. OBS keeps control of the Browser Source canvas size.",
  mediaObsOutput: "Media in OBS", mediaWidgetOutput: "Media Windows widget", notificationObsOutput: "Notifications in OBS", notificationWidgetOutput: "Notifications Windows widget",
  contentScale: "Content scale", cropTop: "Crop top", cropRight: "Crop right", cropBottom: "Crop bottom", cropLeft: "Crop left",
  outputWidth: "Width", outputHeight: "Height", keepAspectRatio: "Keep 16:9 ratio", resetOutput: "Reset", geometrySaved: "Saved", geometryPreview: "Live preview",
});
Object.assign(translations.fr, {
  sizeAndCrop: "Taille et rognage", sizeAndCropHelp: "Réglez chaque sortie indépendamment. OBS conserve le contrôle de la taille du canevas de la source navigateur.",
  mediaObsOutput: "Médias dans OBS", mediaWidgetOutput: "Widget médias Windows", notificationObsOutput: "Notifications dans OBS", notificationWidgetOutput: "Widget notifications Windows",
  contentScale: "Échelle du contenu", cropTop: "Rognage haut", cropRight: "Rognage droite", cropBottom: "Rognage bas", cropLeft: "Rognage gauche",
  outputWidth: "Largeur", outputHeight: "Hauteur", keepAspectRatio: "Conserver le ratio 16:9", resetOutput: "Réinitialiser", geometrySaved: "Enregistré", geometryPreview: "Aperçu en direct",
});
Object.assign(translations.es, {
  sizeAndCrop: "Tamaño y recorte", sizeAndCropHelp: "Ajusta cada salida por separado. OBS mantiene el control del tamaño del lienzo de la fuente del navegador.",
  mediaObsOutput: "Medios en OBS", mediaWidgetOutput: "Widget multimedia de Windows", notificationObsOutput: "Notificaciones en OBS", notificationWidgetOutput: "Widget de notificaciones de Windows",
  contentScale: "Escala del contenido", cropTop: "Recorte superior", cropRight: "Recorte derecho", cropBottom: "Recorte inferior", cropLeft: "Recorte izquierdo",
  outputWidth: "Ancho", outputHeight: "Alto", keepAspectRatio: "Mantener proporción 16:9", resetOutput: "Restablecer", geometrySaved: "Guardado", geometryPreview: "Vista previa en directo",
});
Object.assign(translations.de, {
  sizeAndCrop: "Größe und Zuschnitt", sizeAndCropHelp: "Passe jede Ausgabe separat an. OBS steuert weiterhin die Canvas-Größe der Browserquelle.",
  mediaObsOutput: "Medien in OBS", mediaWidgetOutput: "Windows-Medienwidget", notificationObsOutput: "Benachrichtigungen in OBS", notificationWidgetOutput: "Windows-Benachrichtigungswidget",
  contentScale: "Inhaltsskalierung", cropTop: "Oben zuschneiden", cropRight: "Rechts zuschneiden", cropBottom: "Unten zuschneiden", cropLeft: "Links zuschneiden",
  outputWidth: "Breite", outputHeight: "Höhe", keepAspectRatio: "16:9-Verhältnis beibehalten", resetOutput: "Zurücksetzen", geometrySaved: "Gespeichert", geometryPreview: "Live-Vorschau",
});

Object.assign(translations.en, {
  botPresence: "Bot status", botPresenceHelp: "Choose how Relay appears in Discord.", onlineStatus: "Online status",
  statusOnline: "Online", statusIdle: "Idle", statusDnd: "Do not disturb", statusInvisible: "Invisible",
  activityType: "Activity type", activityNone: "No activity", activityCustom: "Custom status", activityPlaying: "Playing",
  activityListening: "Listening", activityWatching: "Watching", activityCompeting: "Competing", activityText: "Status text",
  activityTextHelp: "Shown on the bot profile and member list.", saveBotPresence: "Save bot status", botPresenceSaved: "Bot status saved",
});
Object.assign(translations.fr, {
  botPresence: "Statut du bot", botPresenceHelp: "Choisissez comment Relay apparaît dans Discord.", onlineStatus: "État de connexion",
  statusOnline: "En ligne", statusIdle: "Inactif", statusDnd: "Ne pas déranger", statusInvisible: "Invisible",
  activityType: "Type d’activité", activityNone: "Aucune activité", activityCustom: "Statut personnalisé", activityPlaying: "Joue à",
  activityListening: "Écoute", activityWatching: "Regarde", activityCompeting: "Participe à", activityText: "Texte du statut",
  activityTextHelp: "Affiché sur le profil du bot et dans la liste des membres.", saveBotPresence: "Enregistrer le statut", botPresenceSaved: "Statut du bot enregistré",
});
Object.assign(translations.es, {
  botPresence: "Estado del bot", botPresenceHelp: "Elige cómo aparece Relay en Discord.", onlineStatus: "Estado de conexión",
  statusOnline: "En línea", statusIdle: "Ausente", statusDnd: "No molestar", statusInvisible: "Invisible",
  activityType: "Tipo de actividad", activityNone: "Sin actividad", activityCustom: "Estado personalizado", activityPlaying: "Jugando",
  activityListening: "Escuchando", activityWatching: "Viendo", activityCompeting: "Compitiendo", activityText: "Texto del estado",
  activityTextHelp: "Se muestra en el perfil del bot y en la lista de miembros.", saveBotPresence: "Guardar estado", botPresenceSaved: "Estado del bot guardado",
});
Object.assign(translations.de, {
  botPresence: "Bot-Status", botPresenceHelp: "Lege fest, wie Relay in Discord erscheint.", onlineStatus: "Online-Status",
  statusOnline: "Online", statusIdle: "Abwesend", statusDnd: "Nicht stören", statusInvisible: "Unsichtbar",
  activityType: "Aktivitätstyp", activityNone: "Keine Aktivität", activityCustom: "Benutzerdefinierter Status", activityPlaying: "Spielt",
  activityListening: "Hört", activityWatching: "Schaut", activityCompeting: "Tritt an", activityText: "Statustext",
  activityTextHelp: "Wird im Bot-Profil und in der Mitgliederliste angezeigt.", saveBotPresence: "Bot-Status speichern", botPresenceSaved: "Bot-Status gespeichert",
});

const pageMetadata = {
  overview: { title: "navOverview", kicker: "system" },
  media: { title: "navMedia", kicker: "playback" },
  overlay: { title: "navOverlay", kicker: "output" },
  moderation: { title: "navModeration", kicker: "safety" },
  commands: { title: "navCommands", kicker: "commandsKicker" },
  history: { title: "navHistory", kicker: "archive" },
  help: { title: "navHelp", kicker: "guide" },
  personalization: { title: "navPersonalization", kicker: "personalizationKicker" },
  about: { title: "navAbout", kicker: "about" },
};

const $ = (selector) => document.querySelector(selector);
const $$ = (selector, root = document) => [...root.querySelectorAll(selector)];

const botStatusElement = $("#bot-status");
const botAvatarElement = $("#bot-avatar");
const botLabelElement = $("#bot-label");
const serverStatusElement = $("#server-status");
const serverLabelElement = $("#server-label");
const clientCountElement = $("#client-count");
const credentialForm = $("#credential-form");
const botPresenceForm = $("#bot-presence-form");
const routingForm = $("#routing-form");
const mediaForm = $("#media-form");
const moderationForm = $("#moderation-form");
const commandsForm = $("#commands-form");
const dirtyForms = new Set();
for (const form of [botPresenceForm, routingForm, mediaForm, moderationForm, commandsForm]) {
  form.addEventListener("input", () => dirtyForms.add(form));
}
const commandsSaveStateElement = $("#commands-save-state");
const channelLockStateElement = $("#channel-lock-state");
const commandInputs = {
  channel: $("#command-channel"), url: $("#command-url"), show: $("#command-show"),
  regenerate: $("#command-regenerate"), clear: $("#command-clear"), lock: $("#command-lock"),
  changelog: $("#command-changelog"),
};
const clientIdElement = $("#client-id");
const tokenElement = $("#discord-token");
const credentialStateElement = $("#credential-state");
const botOnlineStatusElement = $("#bot-online-status");
const botActivityTypeElement = $("#bot-activity-type");
const botActivityTextElement = $("#bot-activity-text");
const botPresenceSaveStateElement = $("#bot-presence-save-state");
const inviteRowElement = $("#invite-row");
const inviteUrlElement = $("#invite-url");
const copyInviteButton = $("#copy-invite");
const channelElement = $("#channel");
const refreshChannelsButton = $("#refresh-channels");
const ttsChannelElement = $("#tts-channel");
const durationElement = $("#duration");
const gifDurationElement = $("#gif-duration");
const stickerDurationElement = $("#sticker-duration");
const portElement = $("#port");
const mediaVolumeElement = $("#media-volume");
const mediaVolumeValueElement = $("#media-volume-value");
const widgetSoundEnabledElement = $("#widget-sound-enabled");
const ttsCharacterLimitElement = $("#tts-character-limit");
const ttsQueueLimitElement = $("#tts-queue-limit");
const notificationDurationElement = $("#notification-duration");
const ttsSpeechEnabledElement = $("#tts-speech-enabled");
const ttsNotificationsObsElement = $("#tts-notifications-obs");
const showAuthorElement = $("#show-author");
const moderationEnabledElement = $("#moderation-enabled");
const moderationAllowImagesElement = $("#moderation-allow-images");
const moderationAllowVideosElement = $("#moderation-allow-videos");
const moderationAllowAudioElement = $("#moderation-allow-audio");
const moderationSaveStateElement = $("#moderation-save-state");
const moderationCountElement = $("#moderation-count");
const moderationListElement = $("#moderation-list");
const moderationEmptyElement = $("#moderation-empty");
const moderationItemTemplate = $("#moderation-item-template");
const clearPendingMediaButton = $("#clear-pending-media");
const saveStateElement = $("#save-state");
const mediaSaveStateElement = $("#media-save-state");
const obsNotificationSaveStateElement = $("#obs-notification-save-state");
const overlayUrlElement = $("#overlay-url");
const copyUrlButton = $("#copy-url");
const audioUrlElement = $("#audio-url");
const copyAudioUrlButton = $("#copy-audio-url");
const ttsUrlElement = $("#tts-url");
const copyTtsUrlButton = $("#copy-tts-url");
const notificationUrlElement = $("#notification-url");
const copyNotificationUrlButton = $("#copy-notification-url");
const stickerUrlElement = $("#sticker-url");
const copyStickerUrlButton = $("#copy-sticker-url");
const regenerateSecretButton = $("#regenerate-secret");
const widgetStateElement = $("#widget-state");
const toggleWidgetButton = $("#toggle-widget");
const lockWidgetButton = $("#lock-widget");
const notificationWidgetStateElement = $("#notification-widget-state");
const notificationWidgetEnabledElement = $("#notification-widget-enabled");
const lockNotificationWidgetButton = $("#lock-notification-widget");
const notificationSoundEnabledElement = $("#notification-sound-enabled");
const notificationSoundObsElement = $("#notification-sound-obs");
const pickNotificationSoundButton = $("#pick-notification-sound");
const clearNotificationSoundButton = $("#clear-notification-sound");
const notificationSoundStateElement = $("#notification-sound-state");
const previewElement = $("#preview");
const outputGeometryGridElement = $("#output-geometry-grid");
const interfaceLanguageElement = $("#interface-language");
const interfaceThemeElement = $("#interface-theme");
const accentInputs = [$("#accent-r"), $("#accent-g"), $("#accent-b")];
const accentPickerElement = $("#accent-picker");
const fontScaleElement = $("#font-scale");
const fontScaleValueElement = $("#font-scale-value");
const personalizationStateElement = $("#personalization-state");
const resetPersonalizationButton = $("#reset-personalization");
const historyListElement = $("#history-list");
const historyEmptyElement = $("#history-empty");
const historyItemTemplate = $("#history-item-template");
const clearOverlayButton = $("#clear-overlay");
const skipMediaButton = $("#skip-media");
const languageToggleButton = $("#language-toggle");
const languageValueElement = $("#language-value");
const themeToggleButton = $("#theme-toggle");
const themeValueElement = $("#theme-value");
const pageTitleElement = $("#page-title");
const pageKickerElement = $("#page-kicker");

const history = [];
let bootstrap;
let socket;
let reconnectTimer;
let reconnectDelayMs = 1000;
let statusTimer;
let isUnloading = false;
let currentPage = "overview";
const supportedLanguages = ["en", "fr", "es", "de"];
let language = localStorage.getItem("relay-language") || "en";
if (!supportedLanguages.includes(language)) language = "en";
let theme = localStorage.getItem("relay-theme")
  || (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
let accentRgb = parseStoredAccent();
let fontScale = clamp(Number(localStorage.getItem("relay-font-scale")) || 100, 80, 140);
let personalizationTimer;

function t(key) {
  return translations[language][key] || translations.en[key] || key;
}

function applyTranslations(root = document) {
  for (const element of $$("[data-i18n]", root)) {
    element.textContent = t(element.dataset.i18n);
  }
}

function applyLanguage() {
  document.documentElement.lang = language;
  localStorage.setItem("relay-language", language);
  languageValueElement.textContent = language.toUpperCase();
  applyTranslations();
  updatePageHeading();
  if (bootstrap) {
    setBotStatus(bootstrap.bot);
    setServerStatus(bootstrap.server);
    setCredentials(bootstrap.credentials);
    setWidgetState(bootstrap.widget);
    setNotificationWidgetState(bootstrap.notificationWidget);
    populateChannels(channelElement, bootstrap.channels, channelElement.value, t("selectChannel"));
    populateChannels(ttsChannelElement, bootstrap.channels, ttsChannelElement.value, t("ttsDisabled"));
    renderHistory();
    renderModeration();
  }
}

function applyTheme() {
  document.documentElement.dataset.theme = theme;
  localStorage.setItem("relay-theme", theme);
  themeValueElement.textContent = t(theme);
  invoke("set_window_theme", { theme }).catch(() => {});
}

function clamp(value, minimum, maximum) {
  return Math.min(maximum, Math.max(minimum, Number(value) || 0));
}

const outputGeometryTargets = {
  mediaObs: { configKey: "mediaObsGeometry", titleKey: "mediaObsOutput", previewKey: "overlayUrl" },
  mediaWidget: { configKey: "mediaWidgetGeometry", titleKey: "mediaWidgetOutput", previewKey: "overlayUrl", widget: "media" },
  notificationObs: { configKey: "notificationObsGeometry", titleKey: "notificationObsOutput", previewKey: "notificationUrl" },
  notificationWidget: { configKey: "notificationWidgetGeometry", titleKey: "notificationWidgetOutput", previewKey: "notificationUrl", widget: "notification" },
};
const outputGeometryTimers = new Map();

function geometryControl(field, labelKey, minimum, maximum) {
  return `
    <label class="geometry-control">
      <span data-i18n="${labelKey}"></span>
      <input data-geometry-field="${field}" data-geometry-kind="range" type="range" min="${minimum}" max="${maximum}" step="1">
      <input data-geometry-field="${field}" data-geometry-kind="number" type="number" min="${minimum}" max="${maximum}" step="1">
    </label>`;
}

function initializeOutputGeometryControls() {
  outputGeometryGridElement.innerHTML = Object.entries(outputGeometryTargets).map(([target, metadata]) => {
    const sizeControls = metadata.widget ? `
      <div class="geometry-size-row">
        <label><span data-i18n="outputWidth"></span><input data-size-field="width" type="number" min="160" max="16384" step="1"></label>
        <label><span data-i18n="outputHeight"></span><input data-size-field="height" type="number" min="90" max="16384" step="1"></label>
      </div>` : "";
    const ratioControl = metadata.widget === "media" ? `
      <label class="inline-switch geometry-ratio">
        <span data-i18n="keepAspectRatio"></span>
        <span class="switch"><input data-keep-aspect-ratio type="checkbox"><span class="switch__track" aria-hidden="true"></span></span>
      </label>` : "";
    return `
      <article class="output-geometry-card" data-geometry-target="${target}">
        <header><h4 data-i18n="${metadata.titleKey}"></h4><span class="save-state" data-geometry-state role="status"></span></header>
        <div class="geometry-preview">
          <span data-i18n="geometryPreview"></span>
          <iframe data-geometry-preview title="Relay output preview"></iframe>
        </div>
        ${sizeControls}
        ${ratioControl}
        <div class="geometry-controls">
          ${geometryControl("contentScale", "contentScale", 50, 200)}
          ${geometryControl("cropTop", "cropTop", 0, 40)}
          ${geometryControl("cropRight", "cropRight", 0, 40)}
          ${geometryControl("cropBottom", "cropBottom", 0, 40)}
          ${geometryControl("cropLeft", "cropLeft", 0, 40)}
        </div>
        <button class="button button--quiet" data-reset-geometry type="button" data-i18n="resetOutput"></button>
      </article>`;
  }).join("");

  for (const card of $$("[data-geometry-target]", outputGeometryGridElement)) {
    const target = card.dataset.geometryTarget;
    for (const input of $$('[data-geometry-field]', card)) {
      input.addEventListener("input", () => {
        const peer = card.querySelector(
          `[data-geometry-field="${input.dataset.geometryField}"][data-geometry-kind="${input.dataset.geometryKind === "range" ? "number" : "range"}"]`,
        );
        peer.value = input.value;
        queueOutputGeometrySave(target);
      });
    }
    for (const input of $$('[data-size-field], [data-keep-aspect-ratio]', card)) {
      input.addEventListener("input", () => queueOutputGeometrySave(target));
    }
    card.querySelector("[data-reset-geometry]").addEventListener("click", () => {
      setOutputGeometryDefaults(target);
      persistOutputGeometry(target);
    });
  }
}

function applyOutputGeometryTarget(config, target, force = false) {
  const card = outputGeometryGridElement.querySelector(`[data-geometry-target="${target}"]`);
  if (!card || (!force && card.contains(document.activeElement))) return;
  const geometry = config[outputGeometryTargets[target].configKey] || {};
  const values = {
    contentScale: geometry.contentScale ?? 100,
    cropTop: geometry.cropTop ?? 0,
    cropRight: geometry.cropRight ?? 0,
    cropBottom: geometry.cropBottom ?? 0,
    cropLeft: geometry.cropLeft ?? 0,
  };
  for (const input of $$('[data-geometry-field]', card)) {
    input.value = String(values[input.dataset.geometryField]);
  }
  if (target === "mediaWidget") {
    card.querySelector('[data-size-field="width"]').value = String(Math.round(config.widgetWidth ?? 640));
    card.querySelector('[data-size-field="height"]').value = String(Math.round(config.widgetHeight ?? 360));
    card.querySelector("[data-keep-aspect-ratio]").checked = config.widgetKeepAspectRatio !== false;
  } else if (target === "notificationWidget") {
    card.querySelector('[data-size-field="width"]').value = String(Math.round(config.notificationWidgetWidth ?? 510));
    card.querySelector('[data-size-field="height"]').value = String(Math.round(config.notificationWidgetHeight ?? 130));
  }
}

function applyOutputGeometryConfig(config, force = false) {
  for (const target of Object.keys(outputGeometryTargets)) {
    applyOutputGeometryTarget(config, target, force);
  }
}

function outputGeometryPayload(target) {
  const card = outputGeometryGridElement.querySelector(`[data-geometry-target="${target}"]`);
  const value = (field, minimum, maximum) => clamp(
    card.querySelector(`[data-geometry-field="${field}"][data-geometry-kind="number"]`).value,
    minimum,
    maximum,
  );
  const payload = {
    target,
    geometry: {
      contentScale: value("contentScale", 50, 200),
      cropTop: value("cropTop", 0, 40),
      cropRight: value("cropRight", 0, 40),
      cropBottom: value("cropBottom", 0, 40),
      cropLeft: value("cropLeft", 0, 40),
    },
  };
  if (outputGeometryTargets[target].widget) {
    payload.width = clamp(card.querySelector('[data-size-field="width"]').value, 160, 16384);
    payload.height = clamp(card.querySelector('[data-size-field="height"]').value, 90, 16384);
  }
  if (target === "mediaWidget") {
    payload.keepAspectRatio = card.querySelector("[data-keep-aspect-ratio]").checked;
  }
  return payload;
}

function queueOutputGeometrySave(target) {
  outputGeometryGridElement.querySelector(`[data-geometry-target="${target}"] [data-geometry-state]`).textContent = t("saving");
  if (outputGeometryTimers.has(target)) return;
  outputGeometryTimers.set(target, window.setTimeout(() => {
    outputGeometryTimers.delete(target);
    persistOutputGeometry(target);
  }, 80));
}

async function persistOutputGeometry(target) {
  if (outputGeometryTimers.has(target)) {
    window.clearTimeout(outputGeometryTimers.get(target));
    outputGeometryTimers.delete(target);
  }
  const state = outputGeometryGridElement.querySelector(`[data-geometry-target="${target}"] [data-geometry-state]`);
  state.textContent = t("saving");
  try {
    const config = await invoke("set_output_geometry", outputGeometryPayload(target));
    bootstrap.config = config;
    applyOutputGeometryTarget(config, target);
    state.textContent = t("geometrySaved");
  } catch (error) {
    state.textContent = String(error);
  }
}

function setOutputGeometryDefaults(target) {
  const card = outputGeometryGridElement.querySelector(`[data-geometry-target="${target}"]`);
  for (const input of $$('[data-geometry-field]', card)) {
    input.value = input.dataset.geometryField === "contentScale" ? "100" : "0";
  }
  if (target === "mediaWidget") {
    card.querySelector('[data-size-field="width"]').value = "640";
    card.querySelector('[data-size-field="height"]').value = "360";
    card.querySelector("[data-keep-aspect-ratio]").checked = true;
  } else if (target === "notificationWidget") {
    card.querySelector('[data-size-field="width"]').value = "510";
    card.querySelector('[data-size-field="height"]').value = "130";
  }
}

function setOutputGeometryPreviewUrls() {
  for (const [target, metadata] of Object.entries(outputGeometryTargets)) {
    const iframe = outputGeometryGridElement.querySelector(`[data-geometry-target="${target}"] [data-geometry-preview]`);
    const url = new URL(bootstrap[metadata.previewKey]);
    if (metadata.previewKey === "notificationUrl") url.searchParams.set("preview", "1");
    if (metadata.widget === "media") {
      url.searchParams.set("widget", "1");
      url.searchParams.set("locked", "1");
    } else if (metadata.widget === "notification") {
      url.searchParams.set("target", "widget");
      url.searchParams.set("locked", "1");
    }
    if (iframe.src !== url.href) iframe.src = url.href;
  }
}

function parseStoredAccent() {
  try {
    const value = JSON.parse(localStorage.getItem("relay-accent-rgb") || "null");
    if (Array.isArray(value) && value.length === 3) {
      return value.map((channel) => clamp(channel, 0, 255));
    }
  } catch {}
  return [88, 185, 137];
}

function accentInk([red, green, blue]) {
  return (red * 299 + green * 587 + blue * 114) / 1000 > 145 ? "#07110b" : "#ffffff";
}

function rgbToHex(rgb) {
  return `#${rgb.map((channel) => clamp(channel, 0, 255).toString(16).padStart(2, "0")).join("")}`;
}

function hexToRgb(hex) {
  return [1, 3, 5].map((index) => Number.parseInt(hex.slice(index, index + 2), 16));
}

function scaleInterfaceText() {
  for (const element of $$('body *:not(svg):not(path)')) {
    if (!element.dataset.relayBaseFontSize) {
      const size = Number.parseFloat(window.getComputedStyle(element).fontSize);
      if (Number.isFinite(size) && size > 0) element.dataset.relayBaseFontSize = String(size);
    }
    if (element.dataset.relayBaseFontSize) {
      element.style.fontSize = `${Number(element.dataset.relayBaseFontSize) * fontScale / 100}px`;
    }
  }
}

function syncInterfacePreferences() {
  window.clearTimeout(personalizationTimer);
  personalizationTimer = window.setTimeout(async () => {
    try {
      await invoke("set_interface_preferences", {
        language, theme, accentRgb, fontScale,
      });
      personalizationStateElement.textContent = t("personalizationSaved");
    } catch (error) {
      personalizationStateElement.textContent = String(error);
    }
  }, 120);
}

function applyPersonalization(sync = true) {
  const color = `rgb(${accentRgb.join(" ")})`;
  document.documentElement.style.setProperty("--accent", color);
  document.documentElement.style.setProperty("--accent-ink", accentInk(accentRgb));
  localStorage.setItem("relay-accent-rgb", JSON.stringify(accentRgb));
  localStorage.setItem("relay-font-scale", String(fontScale));
  interfaceLanguageElement.value = language;
  interfaceThemeElement.value = theme;
  accentInputs.forEach((input, index) => { input.value = String(accentRgb[index]); });
  accentPickerElement.value = rgbToHex(accentRgb);
  fontScaleElement.value = String(fontScale);
  fontScaleValueElement.textContent = `${fontScale}%`;
  scaleInterfaceText();
  if (sync) syncInterfacePreferences();
}

function updatePageHeading() {
  const metadata = pageMetadata[currentPage];
  pageTitleElement.textContent = t(metadata.title);
  pageKickerElement.textContent = t(metadata.kicker);
}

function showPage(page) {
  currentPage = page;
  for (const element of $$("[data-page]")) {
    const active = element.dataset.page === page;
    element.hidden = !active;
    element.classList.toggle("is-active", active);
  }
  for (const button of $$("[data-page-target]")) {
    button.classList.toggle("is-active", button.dataset.pageTarget === page);
  }
  if (page === "history") {
    window.requestAnimationFrame(() => {
      for (const video of historyListElement.querySelectorAll("video")) {
        video.play().catch(() => {});
      }
    });
  }
  updatePageHeading();
}

function setBotStatus(status) {
  botStatusElement.classList.toggle("is-online", status.connected);
  botLabelElement.textContent = status.connected ? status.username : status.error || t("botOffline");
  botAvatarElement.hidden = !status.displayAvatarUrl;
  if (status.displayAvatarUrl) {
    botAvatarElement.src = status.displayAvatarUrl;
  } else {
    botAvatarElement.removeAttribute("src");
  }
}

function setServerStatus(status) {
  serverStatusElement.classList.toggle("is-online", status.connected);
  serverLabelElement.textContent = status.connected ? t("serverOnline") : status.error || t("serverOffline");
  clientCountElement.textContent = String(status.overlayClients || 0);
}

function applyConfig(config) {
  durationElement.value = String(config.displayDurationMs / 1000);
  gifDurationElement.value = String((config.gifDurationMs ?? config.displayDurationMs) / 1000);
  stickerDurationElement.value = String((config.stickerDurationMs ?? 8000) / 1000);
  portElement.value = String(config.port);
  mediaVolumeElement.value = String(config.mediaVolume ?? 50);
  mediaVolumeValueElement.value = `${mediaVolumeElement.value}%`;
  mediaVolumeValueElement.textContent = `${mediaVolumeElement.value}%`;
  ttsCharacterLimitElement.value = String(config.ttsCharacterLimit ?? 0);
  ttsQueueLimitElement.value = String(config.ttsQueueLimit ?? 50);
  notificationDurationElement.value = String((config.notificationDurationMs ?? 8000) / 1000);
  widgetSoundEnabledElement.checked = Boolean(config.widgetSoundEnabled);
  applyNotificationSoundConfig(config);
  ttsSpeechEnabledElement.checked = config.ttsSpeechEnabled !== false;
  ttsNotificationsObsElement.checked = Boolean(config.ttsNotificationsObsEnabled);
  botOnlineStatusElement.value = config.botOnlineStatus || "online";
  botActivityTypeElement.value = config.botActivityType || "custom";
  botActivityTextElement.value = config.botActivityText || "";
  updateBotActivityAvailability();
  showAuthorElement.checked = config.showAuthor;
  moderationEnabledElement.checked = Boolean(config.moderationEnabled);
  moderationAllowImagesElement.checked = config.moderationAllowImages !== false;
  moderationAllowVideosElement.checked = config.moderationAllowVideos !== false;
  moderationAllowAudioElement.checked = config.moderationAllowAudio !== false;
  channelElement.value = config.watchedChannelId;
  if (channelElement.value !== config.watchedChannelId) {
    populateChannels(channelElement, bootstrap?.channels || [], config.watchedChannelId, t("selectChannel"));
  }
  ttsChannelElement.value = config.ttsChannelId || "";
  if (ttsChannelElement.value !== (config.ttsChannelId || "")) {
    populateChannels(ttsChannelElement, bootstrap?.channels || [], config.ttsChannelId, t("ttsDisabled"));
  }
  commandInputs.channel.checked = config.commandChannelEnabled !== false;
  commandInputs.url.checked = config.commandUrlEnabled !== false;
  commandInputs.show.checked = config.commandShowEnabled !== false;
  commandInputs.regenerate.checked = config.commandRegenerateEnabled !== false;
  commandInputs.clear.checked = config.commandClearEnabled !== false;
  commandInputs.lock.checked = config.commandLockEnabled !== false;
  commandInputs.changelog.checked = config.commandChangelogEnabled !== false;
  commandInputs.lock.disabled = Boolean(config.channelLock);
  channelLockStateElement.dataset.i18n = config.channelLock ? "commandLockActive" : "commandLockInactive";
  channelLockStateElement.textContent = t(channelLockStateElement.dataset.i18n);
  applyOutputGeometryConfig(config);
}

function setCredentials(status) {
  credentialStateElement.textContent = status.configured
    ? `${t("savedVia")} ${status.source}`
    : t("notConfigured");
  clientIdElement.value = status.clientId || "";
  tokenElement.value = "";
}

function updateBotActivityAvailability() {
  botActivityTextElement.disabled = botActivityTypeElement.value === "none";
}

function setWidgetState(state) {
  if (bootstrap) {
    bootstrap.widget = state;
  }
  widgetStateElement.textContent = state.visible
    ? state.locked ? t("widgetVisibleLocked") : t("widgetVisibleMovable")
    : t("widgetHidden");
  toggleWidgetButton.textContent = state.visible ? t("hideWidget") : t("showWidget");
  lockWidgetButton.textContent = state.locked ? t("unlockMove") : t("lockDisplay");
}

function setNotificationWidgetState(state) {
  if (bootstrap) {
    bootstrap.notificationWidget = state;
  }
  notificationWidgetEnabledElement.checked = state.visible;
  notificationWidgetStateElement.textContent = state.visible
    ? state.locked ? t("widgetVisibleLocked") : t("widgetVisibleMovable")
    : t("widgetHidden");
  lockNotificationWidgetButton.textContent = state.locked ? t("unlockMove") : t("lockDisplay");
  lockNotificationWidgetButton.disabled = !state.visible;
}

function renderHistory() {
  historyListElement.replaceChildren();
  historyEmptyElement.hidden = history.length > 0;
  for (const mediaEvent of history) {
    const item = historyItemTemplate.content.cloneNode(true);
    const kind = mediaEvent.kind || "image";
    setMediaThumbnail(item, mediaEvent, kind);
    item.querySelector(".history-item__type").textContent = kind.toUpperCase();
    item.querySelector(".history-item__filename").textContent = mediaEvent.filename || kind;
    item.querySelector(".history-item__author").textContent = mediaEvent.author?.username || t("unknownAuthor");
    const time = item.querySelector(".history-item__time");
    time.dateTime = new Date(mediaEvent.timestamp).toISOString();
    time.textContent = new Date(mediaEvent.timestamp).toLocaleTimeString(language, {
      hour: "2-digit", minute: "2-digit", second: "2-digit",
    });
    const replayButton = item.querySelector(".history-item__replay");
    replayButton.textContent = t("replay");
    replayButton.addEventListener("click", async () => {
      try {
        await invoke("replay_media", { messageId: mediaEvent.messageId });
      } catch (error) {
        saveStateElement.textContent = String(error);
      }
    });
    historyListElement.append(item);
  }
}

function setMediaThumbnail(item, mediaEvent, kind) {
  const thumbnail = item.querySelector(".history-item__thumb");
  const panelToken = bootstrap?.wsUrl
    ? new URL(bootstrap.wsUrl).searchParams.get("token")
    : "";
  const source = mediaEvent.cachedMediaId
    ? `http://127.0.0.1:${bootstrap.config.port}/media-cache/${encodeURIComponent(mediaEvent.cachedMediaId)}?token=${encodeURIComponent(panelToken)}`
    : mediaEvent.url || mediaEvent.proxyUrl;
  if (kind === "gif" && mediaEvent.contentType?.startsWith("video/") && source) {
    const video = document.createElement("video");
    video.className = thumbnail.className;
    video.src = source;
    video.muted = true;
    video.autoplay = true;
    video.loop = true;
    video.playsInline = true;
    thumbnail.replaceWith(video);
    return;
  }
  thumbnail.src = kind === "image" || kind === "gif"
    ? source
    : "./assets/relay-radar.png";
  thumbnail.alt = mediaEvent.filename || kind;
}

function replaceHistory(mediaEvents) {
  history.splice(0, history.length, ...mediaEvents.slice(0, 50));
  renderHistory();
}

function rememberMedia(mediaEvent) {
  history.unshift(mediaEvent);
  history.length = Math.min(history.length, 50);
  renderHistory();
}

function renderModeration() {
  const pending = bootstrap?.pendingMedia || [];
  moderationListElement.replaceChildren();
  moderationCountElement.textContent = `${pending.length} / 50`;
  moderationEmptyElement.hidden = pending.length > 0;
  moderationEmptyElement.textContent = t(
    bootstrap?.config?.moderationEnabled ? "moderationEmpty" : "moderationDisabled",
  );
  clearPendingMediaButton.disabled = pending.length === 0;

  for (const pendingItem of pending) {
    const mediaEvent = pendingItem.media;
    const item = moderationItemTemplate.content.cloneNode(true);
    const kind = mediaEvent.kind || "image";
    setMediaThumbnail(item, mediaEvent, kind);
    item.querySelector(".history-item__type").textContent = kind.toUpperCase();
    item.querySelector(".history-item__filename").textContent = mediaEvent.filename || kind;
    item.querySelector(".history-item__author").textContent = mediaEvent.author?.username || t("unknownAuthor");
    const time = item.querySelector(".history-item__time");
    time.dateTime = new Date(mediaEvent.timestamp).toISOString();
    time.textContent = new Date(mediaEvent.timestamp).toLocaleTimeString(language, {
      hour: "2-digit", minute: "2-digit", second: "2-digit",
    });
    const approveButton = item.querySelector(".moderation-item__approve");
    const rejectButton = item.querySelector(".moderation-item__reject");
    approveButton.textContent = t("approve");
    rejectButton.textContent = t("reject");
    const decide = async (command) => {
      approveButton.disabled = true;
      rejectButton.disabled = true;
      try {
        await invoke(command, { id: pendingItem.id });
        bootstrap.pendingMedia = bootstrap.pendingMedia.filter(({ id }) => id !== pendingItem.id);
        renderModeration();
      } catch (error) {
        moderationSaveStateElement.textContent = String(error);
        approveButton.disabled = false;
        rejectButton.disabled = false;
      }
    };
    approveButton.addEventListener("click", () => decide("approve_pending_media"));
    rejectButton.addEventListener("click", () => decide("reject_pending_media"));
    moderationListElement.append(item);
  }
}

function populateChannels(element, channels, selectedChannelId, placeholderText) {
  element.replaceChildren();
  const placeholder = document.createElement("option");
  placeholder.value = "";
  placeholder.textContent = placeholderText;
  element.append(placeholder);
  const groups = new Map();
  for (const channel of channels) {
    if (!groups.has(channel.guildName)) {
      groups.set(channel.guildName, []);
    }
    groups.get(channel.guildName).push(channel);
  }
  for (const [guildName, guildChannels] of groups) {
    const group = document.createElement("optgroup");
    group.label = guildName;
    for (const channel of guildChannels) {
      const option = document.createElement("option");
      option.value = channel.id;
      option.textContent = `# ${channel.name}`;
      group.append(option);
    }
    element.append(group);
  }
  if (selectedChannelId && !channels.some((channel) => channel.id === selectedChannelId)) {
    const option = document.createElement("option");
    option.value = selectedChannelId;
    option.textContent = `${t("unavailableChannel")} (${selectedChannelId})`;
    element.append(option);
  }
  element.value = selectedChannelId;
}

function handleServerMessage(event) {
  let message;
  try {
    message = JSON.parse(event.data);
  } catch {
    return;
  }
  if (message.type === "config") {
    bootstrap.config = message.payload;
    if (dirtyForms.size === 0) {
      applyConfig(message.payload);
    }
  } else if (message.type === "history") {
    replaceHistory(message.payload);
  } else if (message.type === "media") {
    if (message.payload) rememberMedia(message.payload);
  } else if (message.type === "clear") {
    history.length = 0;
    renderHistory();
  }
}

function scheduleReconnect() {
  if (isUnloading || reconnectTimer) {
    return;
  }
  reconnectTimer = window.setTimeout(() => {
    reconnectTimer = undefined;
    connectPanelSocket();
  }, reconnectDelayMs);
  reconnectDelayMs = Math.min(reconnectDelayMs * 2, 10000);
}

function connectPanelSocket() {
  window.clearTimeout(reconnectTimer);
  reconnectTimer = undefined;
  const previousSocket = socket;
  const nextSocket = new WebSocket(bootstrap.wsUrl);
  socket = nextSocket;
  previousSocket?.close();
  nextSocket.addEventListener("open", () => {
    reconnectDelayMs = 1000;
  });
  nextSocket.addEventListener("message", handleServerMessage);
  nextSocket.addEventListener("close", () => {
    if (socket === nextSocket) {
      scheduleReconnect();
    }
  });
  nextSocket.addEventListener("error", () => nextSocket.close());
}

function applyBootstrap(nextBootstrap, reconnect = false) {
  bootstrap = nextBootstrap;
  dirtyForms.clear();
  setBotStatus(bootstrap.bot);
  setServerStatus(bootstrap.server);
  setCredentials(bootstrap.credentials);
  setWidgetState(bootstrap.widget);
  setNotificationWidgetState(bootstrap.notificationWidget);
  populateChannels(channelElement, bootstrap.channels, bootstrap.config.watchedChannelId, t("selectChannel"));
  populateChannels(ttsChannelElement, bootstrap.channels, bootstrap.config.ttsChannelId, t("ttsDisabled"));
  applyConfig(bootstrap.config);
  replaceHistory(bootstrap.history);
  renderModeration();
  overlayUrlElement.value = bootstrap.overlayUrl;
  audioUrlElement.value = bootstrap.audioUrl;
  ttsUrlElement.value = bootstrap.ttsUrl;
  notificationUrlElement.value = bootstrap.notificationUrl;
  stickerUrlElement.value = bootstrap.stickerUrl;
  inviteRowElement.hidden = !bootstrap.inviteUrl;
  inviteUrlElement.value = bootstrap.inviteUrl || "";
  if (previewElement.src !== bootstrap.overlayUrl) {
    previewElement.src = bootstrap.overlayUrl;
  }
  setOutputGeometryPreviewUrls();
  if (reconnect) {
    connectPanelSocket();
  }
}

async function saveConfig(stateElement) {
  stateElement.textContent = t("saving");
  const previousPort = bootstrap.config.port;
  try {
    const nextBootstrap = await invoke("apply_config", {
      config: {
        watchedChannelId: channelElement.value,
        ttsChannelId: ttsChannelElement.value,
        displayDurationMs: Number(durationElement.value) * 1000,
        gifDurationMs: Number(gifDurationElement.value) * 1000,
        stickerDurationMs: Number(stickerDurationElement.value) * 1000,
        mediaVolume: Number(mediaVolumeElement.value),
        ttsCharacterLimit: Number(ttsCharacterLimitElement.value),
        ttsQueueLimit: Number(ttsQueueLimitElement.value),
        notificationDurationMs: Number(notificationDurationElement.value) * 1000,
        ttsSpeechEnabled: ttsSpeechEnabledElement.checked,
        ttsNotificationsObsEnabled: ttsNotificationsObsElement.checked,
        botOnlineStatus: botOnlineStatusElement.value,
        botActivityType: botActivityTypeElement.value,
        botActivityText: botActivityTextElement.value,
        port: Number(portElement.value),
        showAuthor: showAuthorElement.checked,
        widgetSoundEnabled: widgetSoundEnabledElement.checked,
        moderationEnabled: moderationEnabledElement.checked,
        moderationAllowImages: moderationAllowImagesElement.checked,
        moderationAllowVideos: moderationAllowVideosElement.checked,
        moderationAllowAudio: moderationAllowAudioElement.checked,
      },
    });
    applyBootstrap(nextBootstrap, previousPort !== nextBootstrap.config.port);
    stateElement.textContent = t("saved");
    return true;
  } catch (error) {
    stateElement.textContent = String(error);
    return false;
  }
}

let statusRefreshInFlight = false;
let lastChannelsSignature;
let lastPendingSignature;

function applyNotificationSoundConfig(config) {
  notificationSoundEnabledElement.checked = Boolean(config.notificationSoundEnabled);
  notificationSoundObsElement.checked = Boolean(config.notificationSoundObsEnabled);
  const soundPath = config.notificationSoundPath || "";
  notificationSoundStateElement.textContent = soundPath
    ? soundPath.split(/[\\/]/).pop()
    : t("noNotificationSound");
}

async function refreshRuntimeStatus() {
  if (statusRefreshInFlight) {
    return;
  }
  statusRefreshInFlight = true;
  try {
    const status = await invoke("get_runtime_status");
    bootstrap.bot = status.bot;
    bootstrap.server = status.server;
    bootstrap.widget = status.widget;
    bootstrap.notificationWidget = status.notificationWidget;
    bootstrap.channels = status.channels;
    bootstrap.pendingMedia = status.pendingMedia;
    setBotStatus(status.bot);
    setServerStatus(status.server);
    setWidgetState(status.widget);
    setNotificationWidgetState(status.notificationWidget);
    const pendingSignature = JSON.stringify(status.pendingMedia.map((item) => item.id));
    if (pendingSignature !== lastPendingSignature) {
      lastPendingSignature = pendingSignature;
      renderModeration();
    }
    const channelsSignature = JSON.stringify(status.channels);
    const selectingChannel = document.activeElement === channelElement
      || document.activeElement === ttsChannelElement;
    if (channelsSignature !== lastChannelsSignature && !selectingChannel) {
      lastChannelsSignature = channelsSignature;
      populateChannels(channelElement, status.channels, channelElement.value, t("selectChannel"));
      populateChannels(ttsChannelElement, status.channels, ttsChannelElement.value, t("ttsDisabled"));
    }
  } catch {
    setServerStatus({ connected: false, overlayClients: 0 });
    setBotStatus({ connected: false });
  } finally {
    statusRefreshInFlight = false;
  }
}

for (const button of $$("[data-page-target]")) {
  button.addEventListener("click", () => showPage(button.dataset.pageTarget));
}

for (const button of $$("[data-help-link]")) {
  button.addEventListener("click", () => invoke("open_help_link", { link: button.dataset.helpLink }));
}

$("#privacy-reference").addEventListener("click", () => {
  showPage("help");
  const privacyDetails = $("#privacy-details");
  privacyDetails.open = true;
  window.requestAnimationFrame(() => privacyDetails.scrollIntoView({ behavior: "smooth", block: "start" }));
});

languageToggleButton.addEventListener("click", () => {
  language = supportedLanguages[(supportedLanguages.indexOf(language) + 1) % supportedLanguages.length];
  applyLanguage();
  applyTheme();
  applyPersonalization();
});

themeToggleButton.addEventListener("click", () => {
  theme = theme === "light" ? "dark" : "light";
  applyTheme();
  applyPersonalization();
});

interfaceLanguageElement.addEventListener("change", () => {
  language = interfaceLanguageElement.value;
  applyLanguage();
  applyTheme();
  applyPersonalization();
});

interfaceThemeElement.addEventListener("change", () => {
  theme = interfaceThemeElement.value;
  applyTheme();
  applyPersonalization();
});

for (const [index, input] of accentInputs.entries()) {
  input.addEventListener("input", () => {
    accentRgb[index] = clamp(input.value, 0, 255);
    applyPersonalization();
  });
}

accentPickerElement.addEventListener("input", () => {
  accentRgb = hexToRgb(accentPickerElement.value);
  applyPersonalization();
});

fontScaleElement.addEventListener("input", () => {
  fontScale = clamp(fontScaleElement.value, 80, 140);
  applyPersonalization();
});

resetPersonalizationButton.addEventListener("click", () => {
  language = "en";
  theme = "dark";
  accentRgb = [88, 185, 137];
  fontScale = 100;
  applyLanguage();
  applyTheme();
  applyPersonalization();
});

mediaVolumeElement.addEventListener("input", () => {
  mediaVolumeValueElement.value = `${mediaVolumeElement.value}%`;
  mediaVolumeValueElement.textContent = `${mediaVolumeElement.value}%`;
});

credentialForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  credentialStateElement.textContent = t("encrypting");
  try {
    applyBootstrap(await invoke("save_credentials", {
      clientId: clientIdElement.value.trim(),
      token: tokenElement.value.trim(),
    }));
    credentialStateElement.textContent = t("encryptedStarting");
  } catch (error) {
    credentialStateElement.textContent = String(error);
  } finally {
    tokenElement.value = "";
  }
});

botActivityTypeElement.addEventListener("change", updateBotActivityAvailability);

botPresenceForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (await saveConfig(botPresenceSaveStateElement)) {
    botPresenceSaveStateElement.textContent = t("botPresenceSaved");
  }
});

routingForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  await saveConfig(saveStateElement);
});

refreshChannelsButton.addEventListener("click", async () => {
  refreshChannelsButton.disabled = true;
  saveStateElement.textContent = "";
  try {
    const channels = await invoke("refresh_channels");
    bootstrap.channels = channels;
    populateChannels(channelElement, channels, channelElement.value, t("selectChannel"));
    populateChannels(ttsChannelElement, channels, ttsChannelElement.value, t("ttsDisabled"));
    saveStateElement.textContent = t("channelsRefreshed");
  } catch (error) {
    saveStateElement.textContent = String(error);
  } finally {
    refreshChannelsButton.disabled = false;
  }
});

mediaForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  await saveConfig(mediaSaveStateElement);
});

moderationForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  await saveConfig(moderationSaveStateElement);
});

commandsForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  commandsSaveStateElement.textContent = t("saving");
  try {
    const config = await invoke("save_command_settings", {
      settings: Object.fromEntries(Object.entries(commandInputs).map(([name, input]) => [name, input.checked])),
    });
    bootstrap.config = config;
    dirtyForms.clear();
    applyConfig(config);
    commandsSaveStateElement.textContent = t("commandsSaved");
  } catch (error) {
    commandsSaveStateElement.textContent = String(error);
  }
});

clearPendingMediaButton.addEventListener("click", async () => {
  try {
    await invoke("clear_pending_media");
    bootstrap.pendingMedia = [];
    renderModeration();
  } catch (error) {
    moderationSaveStateElement.textContent = String(error);
  }
});

ttsNotificationsObsElement.addEventListener("change", async () => {
  const requestedState = ttsNotificationsObsElement.checked;
  // saveConfig commits routing and media fields too; refuse invalid ones
  // instead of bypassing the forms' HTML validation.
  if (!routingForm.reportValidity() || !mediaForm.reportValidity()) {
    ttsNotificationsObsElement.checked = !requestedState;
    return;
  }
  if (!await saveConfig(obsNotificationSaveStateElement)) {
    ttsNotificationsObsElement.checked = !requestedState;
  }
});

async function copyValue(button, value) {
  await navigator.clipboard.writeText(value);
  button.textContent = t("copied");
  window.setTimeout(() => {
    button.textContent = t("copy");
  }, 1200);
}

copyUrlButton.addEventListener("click", () => copyValue(copyUrlButton, overlayUrlElement.value));
copyAudioUrlButton.addEventListener("click", () => copyValue(copyAudioUrlButton, audioUrlElement.value));
copyTtsUrlButton.addEventListener("click", () => copyValue(copyTtsUrlButton, ttsUrlElement.value));
copyNotificationUrlButton.addEventListener("click", () => copyValue(copyNotificationUrlButton, notificationUrlElement.value));
copyStickerUrlButton.addEventListener("click", () => copyValue(copyStickerUrlButton, stickerUrlElement.value));
copyInviteButton.addEventListener("click", () => copyValue(copyInviteButton, inviteUrlElement.value));

regenerateSecretButton.addEventListener("click", async () => {
  saveStateElement.textContent = t("regenerating");
  try {
    applyBootstrap(await invoke("regenerate_secret"), true);
    saveStateElement.textContent = t("secretRegenerated");
  } catch (error) {
    saveStateElement.textContent = String(error);
  }
});

toggleWidgetButton.addEventListener("click", async () => {
  try {
    setWidgetState(await invoke("toggle_widget"));
  } catch (error) {
    saveStateElement.textContent = String(error);
  }
});

lockWidgetButton.addEventListener("click", async () => {
  try {
    setWidgetState(await invoke("set_widget_locked", { locked: !bootstrap.widget.locked }));
  } catch (error) {
    saveStateElement.textContent = String(error);
  }
});

notificationWidgetEnabledElement.addEventListener("change", async () => {
  try {
    setNotificationWidgetState(await invoke("set_notification_widget_visible", {
      visible: notificationWidgetEnabledElement.checked,
    }));
  } catch (error) {
    notificationWidgetEnabledElement.checked = !notificationWidgetEnabledElement.checked;
    saveStateElement.textContent = String(error);
  }
});

lockNotificationWidgetButton.addEventListener("click", async () => {
  try {
    setNotificationWidgetState(await invoke("set_notification_widget_locked", {
      locked: !bootstrap.notificationWidget.locked,
    }));
  } catch (error) {
    saveStateElement.textContent = String(error);
  }
});

notificationSoundEnabledElement.addEventListener("change", async () => {
  try {
    const config = await invoke("set_notification_sound_enabled", {
      enabled: notificationSoundEnabledElement.checked,
    });
    bootstrap.config = config;
    applyNotificationSoundConfig(config);
  } catch (error) {
    notificationSoundEnabledElement.checked = !notificationSoundEnabledElement.checked;
    notificationSoundStateElement.textContent = String(error);
  }
});

notificationSoundObsElement.addEventListener("change", async () => {
  try {
    const config = await invoke("set_notification_sound_obs_enabled", {
      enabled: notificationSoundObsElement.checked,
    });
    bootstrap.config = config;
    applyNotificationSoundConfig(config);
  } catch (error) {
    notificationSoundObsElement.checked = !notificationSoundObsElement.checked;
    notificationSoundStateElement.textContent = String(error);
  }
});

pickNotificationSoundButton.addEventListener("click", async () => {
  pickNotificationSoundButton.disabled = true;
  try {
    const config = await invoke("pick_notification_sound");
    if (config) {
      bootstrap.config = config;
      applyNotificationSoundConfig(config);
    }
  } catch (error) {
    notificationSoundStateElement.textContent = String(error);
  } finally {
    pickNotificationSoundButton.disabled = false;
  }
});

clearNotificationSoundButton.addEventListener("click", async () => {
  try {
    const config = await invoke("clear_notification_sound");
    bootstrap.config = config;
    applyNotificationSoundConfig(config);
  } catch (error) {
    notificationSoundStateElement.textContent = String(error);
  }
});

skipMediaButton.addEventListener("click", async () => {
  try {
    await invoke("skip_media");
    mediaSaveStateElement.textContent = t("skipped");
  } catch (error) {
    mediaSaveStateElement.textContent = String(error);
  }
});

clearOverlayButton.addEventListener("click", async () => {
  try {
    await invoke("clear_overlay");
  } catch (error) {
    mediaSaveStateElement.textContent = String(error);
  }
});

window.addEventListener("beforeunload", () => {
  isUnloading = true;
  window.clearTimeout(reconnectTimer);
  window.clearInterval(statusTimer);
  socket?.close();
});

initializeOutputGeometryControls();
applyLanguage();
applyTheme();
applyPersonalization();
showPage(currentPage);

try {
  applyBootstrap(await invoke("get_bootstrap"));
  connectPanelSocket();
  statusTimer = window.setInterval(refreshRuntimeStatus, 1500);
} catch (error) {
  saveStateElement.textContent = String(error);
  credentialStateElement.textContent = String(error);
}
