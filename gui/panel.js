const { invoke } = window.__TAURI__.core;

const translations = {
  en: {
    navOverview: "Overview", navMedia: "Media", navOverlay: "Overlay", navModeration: "Moderation", navHistory: "History", navHelp: "Help", navAbout: "About",
    language: "Language", appearance: "Appearance", light: "Light", dark: "Dark", overlays: "OBS sources",
    system: "System", playback: "Playback", output: "Output", safety: "Safety", archive: "Archive", guide: "Guide", about: "About",
    overviewKicker: "Local broadcast", overviewTitle: "One channel. Every screen.",
    overviewCopy: "Connect Discord once, choose a channel, then keep the relay running quietly in the tray.",
    credentialsTitle: "Discord connection", credentialsCopy: "Credentials are encrypted by Windows and never shown again.",
    clientId: "Discord client ID", botToken: "Discord bot token", youtubeApiKey: "YouTube API key", youtubeApiKeyHelp: "Stored in Windows Credential Manager and never shown again.", connectBot: "Encrypt and start bot",
    inviteUrl: "Bot invitation URL", openInvite: "Open", copy: "Copy", copied: "Copied",
    routingTitle: "Input routing", routingCopy: "Choose one Discord channel for media and another for spoken messages.",
    mediaChannel: "Media channel", ttsChannel: "TTS message channel", musicChannel: "Music channel", localPort: "Local port", saveRouting: "Save routing",
    selectChannel: "Select a visible text channel", ttsDisabled: "TTS disabled", musicDisabled: "Music disabled", unavailableChannel: "Unavailable channel",
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
    language: "Langue", appearance: "Apparence", light: "Clair", dark: "Sombre", overlays: "sources OBS",
    system: "Système", playback: "Lecture", output: "Sortie", safety: "Sécurité", archive: "Archives", guide: "Guide", about: "À propos",
    overviewKicker: "Diffusion locale", overviewTitle: "Un canal. Tous vos écrans.",
    overviewCopy: "Connectez Discord une fois, choisissez un canal, puis laissez le relais fonctionner discrètement dans la zone de notification.",
    credentialsTitle: "Connexion Discord", credentialsCopy: "Les identifiants sont chiffrés par Windows et ne sont jamais réaffichés.",
    clientId: "ID client Discord", botToken: "Token du bot Discord", youtubeApiKey: "Clé API YouTube", youtubeApiKeyHelp: "Stockée dans le coffre Windows et jamais réaffichée.", connectBot: "Chiffrer et démarrer le bot",
    inviteUrl: "URL d’invitation du bot", openInvite: "Ouvrir", copy: "Copier", copied: "Copié",
    routingTitle: "Routage d’entrée", routingCopy: "Choisissez un canal Discord pour les médias et un autre pour les messages lus.",
    mediaChannel: "Canal des médias", ttsChannel: "Canal des messages TTS", musicChannel: "Canal musique", localPort: "Port local", saveRouting: "Enregistrer le routage",
    selectChannel: "Sélectionner un canal texte visible", ttsDisabled: "TTS désactivé", musicDisabled: "Musique désactivée", unavailableChannel: "Canal indisponible",
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
    allowAudio: "Audio", allowAudioHelp: "Autorise les fichiers audio à entrer dans la file de validation.",
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
  language: "Idioma", appearance: "Apariencia", light: "Claro", dark: "Oscuro", overlays: "fuentes OBS",
  system: "Sistema", playback: "Reproducción", output: "Salida", safety: "Seguridad", archive: "Archivo", guide: "Guía", about: "Acerca de",
  overviewKicker: "Emisión local", overviewTitle: "Un canal. Todas tus pantallas.", overviewCopy: "Conecta Discord una vez, elige un canal y deja que Relay funcione discretamente en la bandeja del sistema.",
  credentialsTitle: "Conexión con Discord", credentialsCopy: "Las credenciales se cifran con Windows y no vuelven a mostrarse.",
  clientId: "ID de cliente de Discord", botToken: "Token del bot de Discord", youtubeApiKey: "Clave de API de YouTube", youtubeApiKeyHelp: "Se guarda en el almacén de credenciales de Windows y nunca se vuelve a mostrar.", connectBot: "Cifrar e iniciar el bot", inviteUrl: "URL de invitación del bot", openInvite: "Abrir", copy: "Copiar", copied: "Copiado",
  routingTitle: "Enrutamiento de entrada", routingCopy: "Elige un canal de Discord para los medios y otro para los mensajes hablados.",
  mediaChannel: "Canal de medios", ttsChannel: "Canal de mensajes TTS", musicChannel: "Canal de música", localPort: "Puerto local", saveRouting: "Guardar enrutamiento",
  selectChannel: "Selecciona un canal de texto visible", ttsDisabled: "TTS desactivado", musicDisabled: "Música desactivada", unavailableChannel: "Canal no disponible",
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
  language: "Sprache", appearance: "Darstellung", light: "Hell", dark: "Dunkel", overlays: "OBS-Quellen", system: "System", playback: "Wiedergabe", output: "Ausgabe", safety: "Sicherheit", archive: "Archiv", guide: "Anleitung", about: "Info",
  overviewKicker: "Lokale Übertragung", overviewTitle: "Ein Kanal. Alle Bildschirme.", overviewCopy: "Verbinde Discord einmal, wähle einen Kanal und lasse Relay unauffällig im Infobereich laufen.",
  credentialsTitle: "Discord-Verbindung", credentialsCopy: "Die Zugangsdaten werden von Windows verschlüsselt und nie erneut angezeigt.", clientId: "Discord-Client-ID", botToken: "Discord-Bot-Token", youtubeApiKey: "YouTube-API-Schlüssel", youtubeApiKeyHelp: "Wird im Windows-Anmeldeinformations-Manager gespeichert und nie erneut angezeigt.", connectBot: "Verschlüsseln und Bot starten", inviteUrl: "Einladungs-URL des Bots", openInvite: "Öffnen", copy: "Kopieren", copied: "Kopiert",
  routingTitle: "Eingangszuordnung", routingCopy: "Wähle einen Discord-Kanal für Medien und einen weiteren für gesprochene Nachrichten.", mediaChannel: "Medienkanal", ttsChannel: "TTS-Nachrichtenkanal", musicChannel: "Musikkanal", localPort: "Lokaler Port", saveRouting: "Zuordnung speichern", selectChannel: "Sichtbaren Textkanal auswählen", ttsDisabled: "TTS deaktiviert", musicDisabled: "Musik deaktiviert", unavailableChannel: "Kanal nicht verfügbar", refreshChannels: "Kanäle aktualisieren", channelsRefreshed: "Kanalliste aktualisiert",
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
  showMediaTextObs: "Show media message in OBS", showMediaTextObsHelp: "Display up to 180 characters from the Discord message.",
  showMediaTextWidget: "Show media message in the Windows widget", showMediaTextWidgetHelp: "Independent from the OBS media message.",
  navCommands: "Commands", commandsKicker: "Discord controls", commandsTitle: "Commands, under your control.",
  commandsCopy: "Enable only the Relay commands you want available in Discord.", commandsSettings: "Command availability",
  commandChannelHelp: "Choose the Discord media channel.", commandUrlHelp: "Show local Relay and OBS URLs ephemerally.",
  commandShowHelp: "Show the active Relay configuration.", commandStatusHelp: "Show live output, queue, and widget status.", commandTestHelp: "Send an isolated local test to a connected output.", commandRegenerateHelp: "Reconnect local outputs without changing their URLs.",
  commandClearHelp: "Delete the requested number of messages from one Discord channel selected in the command.", commandLockHelp: "Toggle the configured media channel lock.",
  commandChangelogHelp: "Post the latest release notes from GitHub into a chosen channel.",
  commandLockInactive: "The media channel is currently unlocked.", commandLockActive: "The media channel is locked. /relay lock remains available for unlocking.",
  saveCommands: "Save commands", commandsSaved: "Command availability saved",
  commandsPermission: "Channel locking requires Manage Roles; clearing requires Manage Messages. Commands are restricted to Discord administrators.",
});
Object.assign(translations.fr, {
  showMediaTextObs: "Afficher le message du média dans OBS", showMediaTextObsHelp: "Affiche jusqu’à 180 caractères du message Discord.",
  showMediaTextWidget: "Afficher le message du média dans le widget Windows", showMediaTextWidgetHelp: "Indépendant du message affiché dans OBS.",
  navCommands: "Commandes", commandsKicker: "Contrôles Discord", commandsTitle: "Vos commandes, vos règles.",
  commandsCopy: "Activez uniquement les commandes Relay que vous souhaitez rendre disponibles dans Discord.", commandsSettings: "Disponibilité des commandes",
  commandChannelHelp: "Choisit le salon Discord des médias.", commandUrlHelp: "Affiche les URL locales Relay et OBS de façon éphémère.",
  commandShowHelp: "Affiche la configuration Relay active.", commandStatusHelp: "Affiche l’état en direct des sorties, files et widgets.", commandTestHelp: "Envoie un test local isolé vers une sortie connectée.", commandRegenerateHelp: "Reconnecte les sorties locales sans modifier leurs URL.",
  commandClearHelp: "Supprime le nombre demandé de messages dans le salon Discord choisi dans la commande.", commandLockHelp: "Verrouille ou déverrouille le salon média configuré.",
  commandChangelogHelp: "Publie les dernières notes de version depuis GitHub dans le salon choisi.",
  commandLockInactive: "Le salon média est actuellement déverrouillé.", commandLockActive: "Le salon média est verrouillé. /relay lock reste disponible pour le déverrouiller.",
  saveCommands: "Enregistrer les commandes", commandsSaved: "Disponibilité des commandes enregistrée",
  commandsPermission: "Le verrouillage nécessite Gérer les rôles ; le nettoyage nécessite Gérer les messages. Les commandes sont réservées aux administrateurs Discord.",
});
Object.assign(translations.es, {
  showMediaTextObs: "Mostrar el mensaje del medio en OBS", showMediaTextObsHelp: "Muestra hasta 180 caracteres del mensaje de Discord.",
  showMediaTextWidget: "Mostrar el mensaje del medio en el widget de Windows", showMediaTextWidgetHelp: "Independiente del mensaje mostrado en OBS.",
  stickerDuration: "Duración de stickers", stickerDurationHelp: "Los stickers de Discord permanecen visibles durante este tiempo.",
  notificationDuration: "Duración de notificaciones", notificationDurationHelp: "Las notificaciones TTS sin audio permanecen visibles durante este tiempo.",
  durationsGroup: "Duraciones de visualización", durationsGroupHelp: "Tiempo de visibilidad de imágenes, stickers, notificaciones y GIF.",
  audioTtsGroup: "Audio y TTS", audioTtsGroupHelp: "Volumen, longitud de mensajes, cola y voz.",
  displayGroup: "Visualización", displayGroupHelp: "Lo que aparece sobre los medios.",
  stickerSource: "Stickers de Discord",
  navCommands: "Comandos", commandsKicker: "Controles de Discord", commandsTitle: "Tus comandos, tus reglas.",
  commandsCopy: "Activa solo los comandos de Relay que quieras usar en Discord.", commandsSettings: "Disponibilidad de comandos",
  commandChannelHelp: "Elige el canal de medios de Discord.", commandUrlHelp: "Muestra de forma efímera las URL locales de Relay y OBS.",
  commandShowHelp: "Muestra la configuración activa de Relay.", commandStatusHelp: "Muestra el estado en directo de salidas, colas y widgets.", commandTestHelp: "Envía una prueba local aislada a una salida conectada.", commandRegenerateHelp: "Reconecta las salidas locales sin cambiar sus URL.",
  commandClearHelp: "Elimina el número solicitado de mensajes del canal Discord elegido en el comando.", commandLockHelp: "Bloquea o desbloquea el canal de medios configurado.",
  commandChangelogHelp: "Publica las últimas notas de versión desde GitHub en el canal elegido.",
  commandLockInactive: "El canal de medios está desbloqueado.", commandLockActive: "El canal de medios está bloqueado. /relay lock sigue disponible para desbloquearlo.",
  saveCommands: "Guardar comandos", commandsSaved: "Disponibilidad de comandos guardada",
  commandsPermission: "El bloqueo requiere Gestionar roles; la limpieza requiere Gestionar mensajes. Los comandos están restringidos a administradores de Discord.",
});
Object.assign(translations.de, {
  showMediaTextObs: "Mediennachricht in OBS anzeigen", showMediaTextObsHelp: "Zeigt bis zu 180 Zeichen der Discord-Nachricht.",
  showMediaTextWidget: "Mediennachricht im Windows-Widget anzeigen", showMediaTextWidgetHelp: "Unabhängig von der Anzeige in OBS.",
  stickerDuration: "Sticker-Dauer", stickerDurationHelp: "Discord-Sticker bleiben für diese Dauer sichtbar.",
  notificationDuration: "Benachrichtigungsdauer", notificationDurationHelp: "TTS-Benachrichtigungen ohne Audio bleiben für diese Dauer sichtbar.",
  durationsGroup: "Anzeigedauern", durationsGroupHelp: "Sichtbarkeitsdauer von Bildern, Stickern, Benachrichtigungen und GIFs.",
  audioTtsGroup: "Audio und TTS", audioTtsGroupHelp: "Lautstärke, Nachrichtenlänge, Warteschlange und Stimme.",
  displayGroup: "Anzeige", displayGroupHelp: "Was über den Medien erscheint.",
  stickerSource: "Discord-Sticker",
  navCommands: "Befehle", commandsKicker: "Discord-Steuerung", commandsTitle: "Deine Befehle, deine Regeln.",
  commandsCopy: "Aktiviere nur die Relay-Befehle, die in Discord verfügbar sein sollen.", commandsSettings: "Befehlsverfügbarkeit",
  commandChannelHelp: "Wählt den Discord-Medienkanal.", commandUrlHelp: "Zeigt lokale Relay- und OBS-URLs ephemer an.",
  commandShowHelp: "Zeigt die aktive Relay-Konfiguration.", commandStatusHelp: "Zeigt den Live-Status von Ausgaben, Warteschlangen und Widgets.", commandTestHelp: "Sendet einen isolierten lokalen Test an eine verbundene Ausgabe.", commandRegenerateHelp: "Verbindet lokale Ausgaben neu, ohne ihre URLs zu ändern.",
  commandClearHelp: "Löscht die angegebene Anzahl Nachrichten aus dem im Befehl gewählten Discord-Kanal.", commandLockHelp: "Sperrt oder entsperrt den konfigurierten Medienkanal.",
  commandChangelogHelp: "Veröffentlicht die neuesten Versionshinweise von GitHub im gewählten Kanal.",
  commandLockInactive: "Der Medienkanal ist derzeit entsperrt.", commandLockActive: "Der Medienkanal ist gesperrt. /relay lock bleibt zum Entsperren verfügbar.",
  saveCommands: "Befehle speichern", commandsSaved: "Befehlsverfügbarkeit gespeichert",
  commandsPermission: "Die Sperre erfordert Rollen verwalten; die Bereinigung erfordert Nachrichten verwalten. Befehle sind auf Discord-Administratoren beschränkt.",
});

Object.assign(translations.en, {
  defaultCommands: "Default Commands", customCommands: "Custom Commands",
  customCommandsHelp: "Create local /relay subcommands backed by one predefined Discord action. The same list is used on every server where this bot is installed.",
  customCommandsEmpty: "No custom commands configured.", addCustomCommand: "Add command", customCommandEditor: "Command editor",
  customCommandName: "Command name", customCommandAction: "Predefined action", customCommandDescription: "Discord description",
  customCommandEnabled: "Register this command in Discord", customActionParameters: "Action parameters", customAccessRestrictions: "Access restrictions",
  customAdminOnly: "Require Discord Administrator in addition to the action permission", customExtraPermissions: "Additional required permissions",
  permissionManageGuild: "Manage Server", permissionManageMessages: "Manage Messages", permissionManageRoles: "Manage Roles",
  permissionBanMembers: "Ban Members", permissionKickMembers: "Kick Members", permissionModerateMembers: "Moderate Members",
  customAllowedUsers: "Allowed user IDs", customAllowedRoles: "Allowed role IDs", customAllowedChannels: "Allowed invocation channel IDs",
  discordIdsPlaceholder: "One Discord ID or mention per line", cancel: "Cancel", saveCommandDraft: "Save command",
  syncCustomCommands: "Save and sync with Discord", edit: "Edit", delete: "Delete", active: "Active", disabled: "Disabled",
  customActionBan: "Ban member or user ID", customActionUnban: "Unban user", customActionKick: "Kick member",
  customActionTimeout: "Timeout member", customActionRemoveTimeout: "Remove timeout", customActionClearMessages: "Clear messages",
  customActionAddRole: "Add role", customActionRemoveRole: "Remove role", customActionReply: "Predefined reply",
  customParameterMode: "Mode", customParameterValue: "Fallback or fixed value", parameterRequired: "Required", parameterOptional: "Optional", parameterFixed: "Fixed locally",
  customReason: "Audit log reason", customDeleteDays: "Recent message deletion (days)", customDurationMinutes: "Timeout duration (minutes)",
  customChannelId: "Discord channel ID", customMessageCount: "Message count", customRoleId: "Discord role ID",
  customReplyText: "Reply text", customReplyVisibility: "Reply visibility", customReplyEphemeral: "Ephemeral", customReplyPublic: "Public",
  customRequiredPermission: "Minimum enforced permission: {permission}. Destructive actions always require a one-time confirmation.",
  customUnsaved: "Unsaved", customValidating: "Validating", customSyncing: "Syncing with Discord", customActive: "Active in Discord",
  customMaxReached: "Relay supports at most 16 custom commands.", customDuplicateName: "Command names must be unique and cannot use a default Relay command name.",
  customInvalidIds: "Use one valid Discord ID or mention per line.", customDraftSaved: "Command saved locally. Sync to activate it.",
});
Object.assign(translations.fr, {
  defaultCommands: "Commandes par défaut", customCommands: "Commandes personnalisées",
  customCommandsHelp: "Créez des sous-commandes /relay locales associées à une seule action Discord prédéfinie. La même liste est utilisée sur tous les serveurs où ce bot est installé.",
  customCommandsEmpty: "Aucune commande personnalisée configurée.", addCustomCommand: "Ajouter une commande", customCommandEditor: "Éditeur de commande",
  customCommandName: "Nom de la commande", customCommandAction: "Action prédéfinie", customCommandDescription: "Description Discord",
  customCommandEnabled: "Enregistrer cette commande dans Discord", customActionParameters: "Paramètres de l’action", customAccessRestrictions: "Restrictions d’accès",
  customAdminOnly: "Exiger Administrateur Discord en plus de la permission de l’action", customExtraPermissions: "Permissions supplémentaires requises",
  permissionManageGuild: "Gérer le serveur", permissionManageMessages: "Gérer les messages", permissionManageRoles: "Gérer les rôles",
  permissionBanMembers: "Bannir des membres", permissionKickMembers: "Expulser des membres", permissionModerateMembers: "Exclure temporairement des membres",
  customAllowedUsers: "ID utilisateurs autorisés", customAllowedRoles: "ID rôles autorisés", customAllowedChannels: "ID salons autorisés pour l’appel",
  discordIdsPlaceholder: "Un ID ou une mention Discord par ligne", cancel: "Annuler", saveCommandDraft: "Enregistrer la commande",
  syncCustomCommands: "Enregistrer et synchroniser avec Discord", edit: "Modifier", delete: "Supprimer", active: "Active", disabled: "Désactivée",
  customActionBan: "Bannir un membre ou un ID", customActionUnban: "Débannir un utilisateur", customActionKick: "Expulser un membre",
  customActionTimeout: "Exclure temporairement", customActionRemoveTimeout: "Retirer l’exclusion", customActionClearMessages: "Supprimer des messages",
  customActionAddRole: "Ajouter un rôle", customActionRemoveRole: "Retirer un rôle", customActionReply: "Réponse prédéfinie",
  customParameterMode: "Mode", customParameterValue: "Valeur de repli ou fixe", parameterRequired: "Obligatoire", parameterOptional: "Facultatif", parameterFixed: "Fixe localement",
  customReason: "Motif du journal d’audit", customDeleteDays: "Suppression des messages récents (jours)", customDurationMinutes: "Durée de l’exclusion (minutes)",
  customChannelId: "ID du salon Discord", customMessageCount: "Nombre de messages", customRoleId: "ID du rôle Discord",
  customReplyText: "Texte de réponse", customReplyVisibility: "Visibilité de la réponse", customReplyEphemeral: "Éphémère", customReplyPublic: "Publique",
  customRequiredPermission: "Permission minimale imposée : {permission}. Les actions destructrices exigent toujours une confirmation à usage unique.",
  customUnsaved: "Non synchronisé", customValidating: "Validation", customSyncing: "Synchronisation avec Discord", customActive: "Active dans Discord",
  customMaxReached: "Relay prend en charge au maximum 16 commandes personnalisées.", customDuplicateName: "Les noms doivent être uniques et ne peuvent pas reprendre une commande Relay par défaut.",
  customInvalidIds: "Utilisez un ID ou une mention Discord valide par ligne.", customDraftSaved: "Commande enregistrée localement. Synchronisez-la pour l’activer.",
});
Object.assign(translations.es, {
  defaultCommands: "Comandos predeterminados", customCommands: "Comandos personalizados",
  customCommandsHelp: "Crea subcomandos /relay locales asociados a una única acción predefinida de Discord. La misma lista se usa en todos los servidores donde está instalado este bot.",
  customCommandsEmpty: "No hay comandos personalizados configurados.", addCustomCommand: "Añadir comando", customCommandEditor: "Editor de comandos",
  customCommandName: "Nombre del comando", customCommandAction: "Acción predefinida", customCommandDescription: "Descripción de Discord",
  customCommandEnabled: "Registrar este comando en Discord", customActionParameters: "Parámetros de la acción", customAccessRestrictions: "Restricciones de acceso",
  customAdminOnly: "Exigir Administrador de Discord además del permiso de la acción", customExtraPermissions: "Permisos adicionales requeridos",
  permissionManageGuild: "Gestionar servidor", permissionManageMessages: "Gestionar mensajes", permissionManageRoles: "Gestionar roles",
  permissionBanMembers: "Banear miembros", permissionKickMembers: "Expulsar miembros", permissionModerateMembers: "Moderar miembros",
  customAllowedUsers: "ID de usuarios permitidos", customAllowedRoles: "ID de roles permitidos", customAllowedChannels: "ID de canales permitidos para invocar",
  discordIdsPlaceholder: "Un ID o mención de Discord por línea", cancel: "Cancelar", saveCommandDraft: "Guardar comando",
  syncCustomCommands: "Guardar y sincronizar con Discord", edit: "Editar", delete: "Eliminar", active: "Activo", disabled: "Desactivado",
  customActionBan: "Banear miembro o ID", customActionUnban: "Desbanear usuario", customActionKick: "Expulsar miembro",
  customActionTimeout: "Silenciar temporalmente", customActionRemoveTimeout: "Quitar silencio", customActionClearMessages: "Borrar mensajes",
  customActionAddRole: "Añadir rol", customActionRemoveRole: "Quitar rol", customActionReply: "Respuesta predefinida",
  customParameterMode: "Modo", customParameterValue: "Valor alternativo o fijo", parameterRequired: "Obligatorio", parameterOptional: "Opcional", parameterFixed: "Fijo localmente",
  customReason: "Motivo del registro de auditoría", customDeleteDays: "Borrado de mensajes recientes (días)", customDurationMinutes: "Duración del silencio (minutos)",
  customChannelId: "ID del canal de Discord", customMessageCount: "Número de mensajes", customRoleId: "ID del rol de Discord",
  customReplyText: "Texto de respuesta", customReplyVisibility: "Visibilidad de la respuesta", customReplyEphemeral: "Efímera", customReplyPublic: "Pública",
  customRequiredPermission: "Permiso mínimo aplicado: {permission}. Las acciones destructivas siempre requieren una confirmación de un solo uso.",
  customUnsaved: "Sin sincronizar", customValidating: "Validando", customSyncing: "Sincronizando con Discord", customActive: "Activo en Discord",
  customMaxReached: "Relay admite como máximo 16 comandos personalizados.", customDuplicateName: "Los nombres deben ser únicos y no pueden usar un comando predeterminado de Relay.",
  customInvalidIds: "Usa un ID o mención de Discord válido por línea.", customDraftSaved: "Comando guardado localmente. Sincronízalo para activarlo.",
});
Object.assign(translations.de, {
  defaultCommands: "Standardbefehle", customCommands: "Benutzerdefinierte Befehle",
  customCommandsHelp: "Erstelle lokale /relay-Unterbefehle mit genau einer vordefinierten Discord-Aktion. Dieselbe Liste gilt auf allen Servern, auf denen dieser Bot installiert ist.",
  customCommandsEmpty: "Keine benutzerdefinierten Befehle konfiguriert.", addCustomCommand: "Befehl hinzufügen", customCommandEditor: "Befehlseditor",
  customCommandName: "Befehlsname", customCommandAction: "Vordefinierte Aktion", customCommandDescription: "Discord-Beschreibung",
  customCommandEnabled: "Diesen Befehl in Discord registrieren", customActionParameters: "Aktionsparameter", customAccessRestrictions: "Zugriffsbeschränkungen",
  customAdminOnly: "Zusätzlich zur Aktionsberechtigung Discord-Administrator verlangen", customExtraPermissions: "Zusätzlich erforderliche Berechtigungen",
  permissionManageGuild: "Server verwalten", permissionManageMessages: "Nachrichten verwalten", permissionManageRoles: "Rollen verwalten",
  permissionBanMembers: "Mitglieder bannen", permissionKickMembers: "Mitglieder kicken", permissionModerateMembers: "Mitglieder moderieren",
  customAllowedUsers: "Erlaubte Benutzer-IDs", customAllowedRoles: "Erlaubte Rollen-IDs", customAllowedChannels: "Erlaubte Kanal-IDs für Aufrufe",
  discordIdsPlaceholder: "Eine Discord-ID oder Erwähnung pro Zeile", cancel: "Abbrechen", saveCommandDraft: "Befehl speichern",
  syncCustomCommands: "Speichern und mit Discord synchronisieren", edit: "Bearbeiten", delete: "Löschen", active: "Aktiv", disabled: "Deaktiviert",
  customActionBan: "Mitglied oder Benutzer-ID bannen", customActionUnban: "Benutzer entbannen", customActionKick: "Mitglied kicken",
  customActionTimeout: "Mitglied aussetzen", customActionRemoveTimeout: "Aussetzung entfernen", customActionClearMessages: "Nachrichten löschen",
  customActionAddRole: "Rolle hinzufügen", customActionRemoveRole: "Rolle entfernen", customActionReply: "Vordefinierte Antwort",
  customParameterMode: "Modus", customParameterValue: "Ersatz- oder fester Wert", parameterRequired: "Erforderlich", parameterOptional: "Optional", parameterFixed: "Lokal festgelegt",
  customReason: "Grund für das Audit-Protokoll", customDeleteDays: "Löschen neuer Nachrichten (Tage)", customDurationMinutes: "Dauer der Aussetzung (Minuten)",
  customChannelId: "Discord-Kanal-ID", customMessageCount: "Nachrichtenanzahl", customRoleId: "Discord-Rollen-ID",
  customReplyText: "Antworttext", customReplyVisibility: "Sichtbarkeit der Antwort", customReplyEphemeral: "Ephemer", customReplyPublic: "Öffentlich",
  customRequiredPermission: "Erzwungene Mindestberechtigung: {permission}. Destruktive Aktionen benötigen immer eine einmalige Bestätigung.",
  customUnsaved: "Nicht synchronisiert", customValidating: "Wird geprüft", customSyncing: "Wird mit Discord synchronisiert", customActive: "In Discord aktiv",
  customMaxReached: "Relay unterstützt höchstens 16 benutzerdefinierte Befehle.", customDuplicateName: "Namen müssen eindeutig sein und dürfen keinen Relay-Standardbefehl verwenden.",
  customInvalidIds: "Verwende pro Zeile eine gültige Discord-ID oder Erwähnung.", customDraftSaved: "Befehl lokal gespeichert. Zum Aktivieren synchronisieren.",
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

Object.assign(translations.en, {
  nowPlaying: "Now playing", previousAudio: "Previous audio", pauseAudio: "Pause audio",
  resumeAudio: "Resume audio", skipAudio: "Skip audio",
  controlsGroup: "Keyboard controls", controlsGroupHelp: "Choose the global shortcut used to skip the current media.",
  skipShortcut: "Skip current media", skipShortcutHelp: "Click Capture, then press the key combination you want to use everywhere.",
  captureShortcut: "Capture", pressShortcut: "Press a key combination…", shortcutSaved: "Shortcut saved", shortcutCanceled: "Shortcut capture canceled",
  shortcutInvalid: "That shortcut could not be registered.", download: "Download", downloading: "Downloading…", downloaded: "Saved", downloadCanceled: "Download canceled",
});
Object.assign(translations.fr, {
  nowPlaying: "Lecture en cours", previousAudio: "Audio précédent", pauseAudio: "Mettre en pause",
  resumeAudio: "Reprendre la lecture", skipAudio: "Passer l’audio",
  controlsGroup: "Raccourcis clavier", controlsGroupHelp: "Choisissez le raccourci global qui passe le média actuel.",
  skipShortcut: "Passer le média actuel", skipShortcutHelp: "Cliquez sur Capturer, puis appuyez sur la combinaison à utiliser partout.",
  captureShortcut: "Capturer", pressShortcut: "Appuyez sur une combinaison…", shortcutSaved: "Raccourci enregistré", shortcutCanceled: "Capture annulée",
  shortcutInvalid: "Ce raccourci ne peut pas être enregistré.", download: "Télécharger", downloading: "Téléchargement…", downloaded: "Enregistré", downloadCanceled: "Téléchargement annulé",
});
Object.assign(translations.es, {
  nowPlaying: "Reproduciendo", previousAudio: "Audio anterior", pauseAudio: "Pausar audio",
  resumeAudio: "Reanudar audio", skipAudio: "Omitir audio",
  controlsGroup: "Atajos de teclado", controlsGroupHelp: "Elige el atajo global para omitir el medio actual.",
  skipShortcut: "Omitir medio actual", skipShortcutHelp: "Pulsa Capturar y después la combinación que quieras usar.",
  captureShortcut: "Capturar", pressShortcut: "Pulsa una combinación…", shortcutSaved: "Atajo guardado", shortcutCanceled: "Captura cancelada",
  shortcutInvalid: "No se pudo registrar ese atajo.", download: "Descargar", downloading: "Descargando…", downloaded: "Guardado", downloadCanceled: "Descarga cancelada",
});
Object.assign(translations.de, {
  nowPlaying: "Aktuelle Wiedergabe", previousAudio: "Vorheriges Audio", pauseAudio: "Audio pausieren",
  resumeAudio: "Audio fortsetzen", skipAudio: "Audio überspringen",
  controlsGroup: "Tastenkürzel", controlsGroupHelp: "Wähle das globale Kürzel zum Überspringen des aktuellen Mediums.",
  skipShortcut: "Aktuelles Medium überspringen", skipShortcutHelp: "Klicke auf Aufnehmen und drücke die gewünschte Tastenkombination.",
  captureShortcut: "Aufnehmen", pressShortcut: "Tastenkombination drücken…", shortcutSaved: "Kürzel gespeichert", shortcutCanceled: "Aufnahme abgebrochen",
  shortcutInvalid: "Dieses Kürzel konnte nicht registriert werden.", download: "Herunterladen", downloading: "Wird heruntergeladen…", downloaded: "Gespeichert", downloadCanceled: "Download abgebrochen",
});

Object.assign(translations.en, {
  outputReadiness: "Output readiness",
  outputReadinessHelp: "See which local outputs are connected. Tests stay local and never post to Discord.",
  outputObs: "OBS", outputPreview: "Preview", outputWidget: "Widget",
  outputDisconnected: "Not connected", outputLastConnected: "Last connected", outputNeverConnected: "Never connected",
  testOutput: "Test output", outputTestSent: "Test sent", outputTestFailed: "Test failed",
  outputTestNeedsLiveOutput: "Connect OBS or a widget before testing.",
});
Object.assign(translations.fr, {
  outputReadiness: "État des sorties",
  outputReadinessHelp: "Vérifiez les sorties locales connectées. Les tests restent locaux et ne publient jamais sur Discord.",
  outputObs: "OBS", outputPreview: "Aperçu", outputWidget: "Widget",
  outputDisconnected: "Non connecté", outputLastConnected: "Dernière connexion", outputNeverConnected: "Jamais connecté",
  testOutput: "Tester la sortie", outputTestSent: "Test envoyé", outputTestFailed: "Test échoué",
  outputTestNeedsLiveOutput: "Connectez OBS ou un widget avant le test.",
});
Object.assign(translations.es, {
  outputReadiness: "Estado de las salidas",
  outputReadinessHelp: "Comprueba qué salidas locales están conectadas. Las pruebas son locales y nunca publican en Discord.",
  outputObs: "OBS", outputPreview: "Vista previa", outputWidget: "Widget",
  outputDisconnected: "Sin conexión", outputLastConnected: "Última conexión", outputNeverConnected: "Nunca conectado",
  testOutput: "Probar salida", outputTestSent: "Prueba enviada", outputTestFailed: "Error en la prueba",
  outputTestNeedsLiveOutput: "Conecta OBS o un widget antes de probar.",
});
Object.assign(translations.de, {
  outputReadiness: "Ausgabestatus",
  outputReadinessHelp: "Sieh, welche lokalen Ausgaben verbunden sind. Tests bleiben lokal und posten nie in Discord.",
  outputObs: "OBS", outputPreview: "Vorschau", outputWidget: "Widget",
  outputDisconnected: "Nicht verbunden", outputLastConnected: "Letzte Verbindung", outputNeverConnected: "Noch nie verbunden",
  testOutput: "Ausgabe testen", outputTestSent: "Test gesendet", outputTestFailed: "Test fehlgeschlagen",
  outputTestNeedsLiveOutput: "Verbinde vor dem Test OBS oder ein Widget.",
});

Object.assign(translations.en, {
  updatesTitle: "Relay updates", checkUpdates: "Check for updates",
  checkUpdatesPrompt: "Check GitHub for a newer Relay release.",
  checkingUpdates: "Checking the latest official release…",
  updateAvailable: "Relay v{version} is available.", upToDate: "Relay v{version} is up to date.",
  downloadAndInstall: "Download and install", downloadingUpdate: "Downloading and verifying v{version}…",
  openReleases: "View releases", closeUpdateMenu: "Close update menu",
  updateCheckFailed: "Update check failed:", updateInstallFailed: "Update failed:",
});
Object.assign(translations.fr, {
  updatesTitle: "Mises à jour Relay", checkUpdates: "Rechercher des mises à jour",
  checkUpdatesPrompt: "Vérifier si une version plus récente de Relay existe sur GitHub.",
  checkingUpdates: "Vérification de la dernière version officielle…",
  updateAvailable: "Relay v{version} est disponible.", upToDate: "Relay v{version} est à jour.",
  downloadAndInstall: "Télécharger et installer", downloadingUpdate: "Téléchargement et vérification de v{version}…",
  openReleases: "Voir les versions", closeUpdateMenu: "Fermer le menu des mises à jour",
  updateCheckFailed: "Échec de la vérification :", updateInstallFailed: "Échec de la mise à jour :",
});
Object.assign(translations.es, {
  updatesTitle: "Actualizaciones de Relay", checkUpdates: "Buscar actualizaciones",
  checkUpdatesPrompt: "Comprueba en GitHub si existe una versión más reciente de Relay.",
  checkingUpdates: "Comprobando la última versión oficial…",
  updateAvailable: "Relay v{version} está disponible.", upToDate: "Relay v{version} está actualizado.",
  downloadAndInstall: "Descargar e instalar", downloadingUpdate: "Descargando y verificando v{version}…",
  openReleases: "Ver versiones", closeUpdateMenu: "Cerrar el menú de actualizaciones",
  updateCheckFailed: "Error al buscar actualizaciones:", updateInstallFailed: "Error de actualización:",
});
Object.assign(translations.de, {
  updatesTitle: "Relay-Updates", checkUpdates: "Nach Updates suchen",
  checkUpdatesPrompt: "Auf GitHub nach einer neueren Relay-Version suchen.",
  checkingUpdates: "Neueste offizielle Version wird geprüft…",
  updateAvailable: "Relay v{version} ist verfügbar.", upToDate: "Relay v{version} ist aktuell.",
  downloadAndInstall: "Herunterladen und installieren", downloadingUpdate: "v{version} wird geladen und geprüft…",
  openReleases: "Versionen anzeigen", closeUpdateMenu: "Update-Menü schließen",
  updateCheckFailed: "Update-Prüfung fehlgeschlagen:", updateInstallFailed: "Update fehlgeschlagen:",
});

Object.assign(translations.en, {
  designLabel: "Design", openaiDesignCopy: "Precise, calm and utilitarian.",
  anthropicDesignCopy: "Warm, literary and human.", neoDesignCopy: "Bold, editorial and playful.",
});
Object.assign(translations.fr, {
  designLabel: "Design", openaiDesignCopy: "Précis, calme et utilitaire.",
  anthropicDesignCopy: "Chaleureux, littéraire et humain.", neoDesignCopy: "Audacieux, éditorial et ludique.",
});
Object.assign(translations.es, {
  designLabel: "Diseño", openaiDesignCopy: "Preciso, sereno y funcional.",
  anthropicDesignCopy: "Cálido, literario y humano.", neoDesignCopy: "Audaz, editorial y divertido.",
});
Object.assign(translations.de, {
  designLabel: "Designstil", openaiDesignCopy: "Präzise, ruhig und funktional.",
  anthropicDesignCopy: "Warm, literarisch und menschlich.", neoDesignCopy: "Mutig, redaktionell und verspielt.",
});

Object.assign(translations.en, {
  automaticFilterWords: "Automatic filtering",
  automaticFilterWordsHelp: "Filter words are saved automatically after you stop typing. They run even when manual moderation and the local image scan are off. Exact matches and existing regexes block immediately and never enter the manual moderation queue.",
  manualModeration: "Manual moderation",
  manualModerationHelp: "Hold media for approval independently from automatic filtering.",
  privacyScanEnabled: "Enable local privacy scan",
  privacyScanEnabledHelp: "Inspect image metadata and local OCR before history or OBS.",
  privacySuspiciousPolicy: "Suspicious media policy",
  privacySuspiciousPolicyHelp: "Choose whether weak signals are allowed, reviewed, or blocked.",
  privacyPolicyAllow: "Allow",
  privacyPolicyReview: "Review",
  privacyPolicyBlock: "Block",
  privacySuspiciousThreshold: "Review threshold",
  privacySensitiveThreshold: "Sensitive threshold",
  privacySimilarityBoost: "Similarity boost",
  privacyConcepts: "Filter words or phrases",
  privacyExemptRoles: "Exempt Discord roles",
  privacyExemptRolesHelp: "Enter role IDs or Discord role mentions separated by commas or new lines. These roles bypass filter words only; local privacy signals and manual moderation still apply.",
  unsaved: "Unsaved changes",
  privacyConceptsHelp: "Enter comma-separated words or phrases, for example: fdp, hitler. Relay handles case, punctuation and separators, leetspeak, supported homoglyphs, repeated letters, and cautious similarity. Existing aliases and regexes are preserved when the same canonical form remains.",
  privacyReviewQueueEmpty: "Filter matches using Review appear here even when manual moderation is off.",
  privacyPendingManual: "Manual review",
  privacyProtection: "Anti-doxxing protection",
  privacyProtectionHelp: "Scan locally before Discord text or media can reach history, WebSocket, Windows widgets, or OBS. Detected values are never copied into logs.",
  privacyProtectionLevel: "Protection level",
  privacyProtectionLevelHelp: "Balanced reduces false positives. Strict and Paranoid escalate weak signals sooner.",
  privacyProfileBalanced: "Balanced",
  privacyProfileStrict: "Strict",
  privacyProfileParanoid: "Paranoid",
  privacyBlockThreshold: "Automatic block threshold",
  privacyBlockThresholdHelp: "HIGH blocks by default. CRITICAL sends HIGH cases to local review instead.",
  privacyReviewIntermediate: "Review MEDIUM-risk cases",
  privacyReviewIntermediateHelp: "Put intermediate-risk media in the existing local moderation queue.",
  privacyAutoDeleteBlockedMessages: "Delete blocked Discord messages",
  privacyAutoDeleteBlockedMessagesHelp: "Delete messages blocked by the privacy threshold or an automatic filter word. Requires Manage Messages.",
  privacyCategories: "Enabled detection categories",
  privacyCategoryEmail: "Email", privacyCategoryPhone: "Phone", privacyCategoryIp: "IP addresses",
  privacyCategoryGps: "GPS and coordinates", privacyCategoryAddress: "Postal addresses",
  privacyCategoryFinancial: "IBAN and payment cards", privacyCategoryPlate: "License plates",
  privacyCategoryUrl: "Sensitive URLs", privacyCategoryCustom: "Protected private strings",
  privacyCategoryMetadata: "EXIF metadata", privacyCategoryOcr: "Local OCR",
  privacyCategoryDocument: "Administrative documents",
  privacyCustomPatterns: "Private data to protect",
  privacyCustomPatternsHelp: "Add names, old usernames, address variants, streets, cities, emails, phone numbers or other private strings. Values stay in the local Relay configuration.",
  privacyCustomPatternsPlaceholder: "One value per line",
  privacyAllowlist: "Allowlist",
  privacyAllowlistHelp: "Exact public values listed here are masked before automatic detection.",
  privacyAllowlistPlaceholder: "One public value per line",
});
Object.assign(translations.fr, {
  automaticFilterWords: "Filtrage automatique",
  automaticFilterWordsHelp: "Les mots et expressions filtrés sont enregistrés automatiquement après la saisie. Ils s’appliquent même lorsque la modération manuelle et l’analyse locale sont désactivées. Les correspondances exactes et les expressions régulières existantes sont bloquées immédiatement sans passer par la file de modération.",
  manualModeration: "Modération manuelle",
  manualModerationHelp: "Place les médias en attente de validation indépendamment du filtrage automatique.",
  privacyScanEnabled: "Activer l’analyse locale de confidentialité",
  privacyScanEnabledHelp: "Analyse les métadonnées des images et exécute l’OCR local avant l’historique ou OBS.",
  privacySuspiciousPolicy: "Politique des médias suspects",
  privacySuspiciousPolicyHelp: "Choisissez si les signaux faibles sont autorisés, vérifiés ou bloqués.",
  privacyPolicyAllow: "Autoriser",
  privacyPolicyReview: "Vérifier",
  privacyPolicyBlock: "Bloquer",
  privacySuspiciousThreshold: "Seuil de vérification",
  privacySensitiveThreshold: "Seuil sensible",
  privacySimilarityBoost: "Renforcement de similarité",
  privacyConcepts: "Mots ou phrases de filtrage",
  privacyExemptRoles: "Rôles Discord exemptés",
  privacyExemptRolesHelp: "Saisissez des identifiants de rôle ou des mentions de rôle Discord séparés par des virgules ou des retours à la ligne. Ces rôles contournent uniquement les mots filtrés ; les signaux de confidentialité et la modération manuelle restent actifs.",
  unsaved: "Modifications non enregistrées",
  privacyConceptsHelp: "Saisissez des mots ou expressions séparés par des virgules, par exemple : fdp, hitler. Relay gère la casse, la ponctuation, les séparateurs, le leetspeak, les homoglyphes pris en charge, les lettres répétées et une similarité prudente. Les alias et expressions régulières existants sont conservés tant que la même forme canonique reste présente.",
  privacyReviewQueueEmpty: "Les correspondances à vérifier apparaissent ici même lorsque la modération manuelle est désactivée.",
  privacyPendingManual: "Vérification manuelle",
  privacyProtection: "Protection anti-doxxing",
  privacyProtectionHelp: "Analyse localement les textes et médias Discord avant qu’ils n’atteignent l’historique, WebSocket, les widgets Windows ou OBS. Les valeurs détectées ne sont jamais copiées dans les journaux.",
  privacyProtectionLevel: "Niveau de protection",
  privacyProtectionLevelHelp: "Équilibré réduit les faux positifs. Strict et Paranoïaque font remonter plus rapidement les signaux faibles.",
  privacyProfileBalanced: "Équilibré",
  privacyProfileStrict: "Strict",
  privacyProfileParanoid: "Paranoïaque",
  privacyBlockThreshold: "Seuil de blocage automatique",
  privacyBlockThresholdHelp: "HIGH bloque par défaut. CRITICAL envoie les cas HIGH en vérification locale.",
  privacyReviewIntermediate: "Vérifier les cas à risque MEDIUM",
  privacyReviewIntermediateHelp: "Place les médias à risque intermédiaire dans la file de modération locale existante.",
  privacyAutoDeleteBlockedMessages: "Supprimer automatiquement les messages Discord bloqués",
  privacyAutoDeleteBlockedMessagesHelp: "Supprime les messages bloqués par le seuil de confidentialité ou un mot de filtrage automatique. Nécessite Gérer les messages.",
  privacyCategories: "Catégories de détection actives",
  privacyCategoryEmail: "E-mail", privacyCategoryPhone: "Téléphone", privacyCategoryIp: "Adresses IP",
  privacyCategoryGps: "GPS et coordonnées", privacyCategoryAddress: "Adresses postales",
  privacyCategoryFinancial: "IBAN et cartes bancaires", privacyCategoryPlate: "Plaques d’immatriculation",
  privacyCategoryUrl: "URL sensibles", privacyCategoryCustom: "Chaînes privées protégées",
  privacyCategoryMetadata: "Métadonnées EXIF", privacyCategoryOcr: "OCR local",
  privacyCategoryDocument: "Documents administratifs",
  privacyCustomPatterns: "Données privées à protéger",
  privacyCustomPatternsHelp: "Ajoutez des noms, anciens pseudos, variantes d’adresse, rues, villes, e-mails, numéros de téléphone ou autres chaînes privées. Ces valeurs restent dans la configuration locale de Relay.",
  privacyCustomPatternsPlaceholder: "Une valeur par ligne",
  privacyAllowlist: "Liste d’autorisation",
  privacyAllowlistHelp: "Les valeurs publiques exactes de cette liste sont masquées avant la détection automatique.",
  privacyAllowlistPlaceholder: "Une valeur publique par ligne",
});
Object.assign(translations.es, {
  automaticFilterWords: "Filtrado automático",
  automaticFilterWordsHelp: "Las palabras y frases filtradas se guardan automáticamente al terminar de escribir. Se aplican aunque la moderación manual y el análisis local estén desactivados. Las coincidencias exactas y las expresiones regulares existentes se bloquean de inmediato sin entrar en la cola de moderación.",
  manualModeration: "Moderación manual",
  manualModerationHelp: "Retiene los medios para su aprobación independientemente del filtrado automático.",
  privacyScanEnabled: "Activar el análisis local de privacidad",
  privacyScanEnabledHelp: "Analiza los metadatos de las imágenes y ejecuta el OCR local antes del historial o de OBS.",
  privacySuspiciousPolicy: "Política de medios sospechosos",
  privacySuspiciousPolicyHelp: "Elige si las señales débiles se permiten, revisan o bloquean.",
  privacyPolicyAllow: "Permitir",
  privacyPolicyReview: "Revisar",
  privacyPolicyBlock: "Bloquear",
  privacySuspiciousThreshold: "Umbral de revisión",
  privacySensitiveThreshold: "Umbral sensible",
  privacySimilarityBoost: "Refuerzo de similitud",
  privacyConcepts: "Palabras o frases filtradas",
  privacyConceptsHelp: "Introduce palabras o frases separadas por comas, por ejemplo: fdp, hitler. Relay gestiona mayúsculas y minúsculas, puntuación, separadores, leetspeak, homógrafos compatibles, letras repetidas y similitud prudente. Los alias y las expresiones regulares existentes se conservan mientras permanezca la misma forma canónica.",
  privacyExemptRoles: "Roles de Discord exentos",
  privacyExemptRolesHelp: "Introduce identificadores o menciones de roles de Discord separados por comas o saltos de línea. Estos roles solo omiten las palabras filtradas; las señales de privacidad y la moderación manual siguen activas.",
  privacyReviewQueueEmpty: "Las coincidencias pendientes de revisión aparecen aquí aunque la moderación manual esté desactivada.",
  privacyPendingManual: "Revisión manual",
  privacyProtection: "Protección contra doxxing",
  privacyProtectionHelp: "Analiza localmente los textos y medios de Discord antes de que lleguen al historial, WebSocket, los widgets de Windows u OBS. Los valores detectados nunca se copian en los registros.",
  privacyProtectionLevel: "Nivel de protección",
  privacyProtectionLevelHelp: "Equilibrado reduce los falsos positivos. Estricto y Paranoico elevan antes las señales débiles.",
  privacyProfileBalanced: "Equilibrado",
  privacyProfileStrict: "Estricto",
  privacyProfileParanoid: "Paranoico",
  privacyBlockThreshold: "Umbral de bloqueo automático",
  privacyBlockThresholdHelp: "HIGH bloquea de forma predeterminada. CRITICAL envía los casos HIGH a revisión local.",
  privacyReviewIntermediate: "Revisar casos de riesgo MEDIUM",
  privacyReviewIntermediateHelp: "Coloca los medios de riesgo intermedio en la cola de moderación local existente.",
  privacyAutoDeleteBlockedMessages: "Eliminar automáticamente los mensajes de Discord bloqueados",
  privacyAutoDeleteBlockedMessagesHelp: "Elimina los mensajes bloqueados por el umbral de privacidad o por una palabra del filtro automático. Requiere Gestionar mensajes.",
  privacyCategories: "Categorías de detección activas",
  privacyCategoryEmail: "Correo electrónico", privacyCategoryPhone: "Teléfono", privacyCategoryIp: "Direcciones IP",
  privacyCategoryGps: "GPS y coordenadas", privacyCategoryAddress: "Direcciones postales",
  privacyCategoryFinancial: "IBAN y tarjetas bancarias", privacyCategoryPlate: "Matrículas",
  privacyCategoryUrl: "URL sensibles", privacyCategoryCustom: "Cadenas privadas protegidas",
  privacyCategoryMetadata: "Metadatos EXIF", privacyCategoryOcr: "OCR local",
  privacyCategoryDocument: "Documentos administrativos",
  privacyCustomPatterns: "Datos privados que se deben proteger",
  privacyCustomPatternsHelp: "Añade nombres, nombres de usuario anteriores, variantes de dirección, calles, ciudades, correos electrónicos, teléfonos u otras cadenas privadas. Estos valores permanecen en la configuración local de Relay.",
  privacyCustomPatternsPlaceholder: "Un valor por línea",
  privacyAllowlist: "Lista de permitidos",
  privacyAllowlistHelp: "Los valores públicos exactos de esta lista se ocultan antes de la detección automática.",
  privacyAllowlistPlaceholder: "Un valor público por línea",
});
Object.assign(translations.de, {
  automaticFilterWords: "Automatische Filterung",
  automaticFilterWordsHelp: "Gefilterte Wörter und Ausdrücke werden nach der Eingabe automatisch gespeichert. Sie gelten auch bei deaktivierter manueller Moderation und lokalem Scan. Exakte Treffer und vorhandene reguläre Ausdrücke werden sofort blockiert und gelangen nicht in die Moderationswarteschlange.",
  manualModeration: "Manuelle Moderation",
  manualModerationHelp: "Hält Medien unabhängig von der automatischen Filterung zur Freigabe zurück.",
  privacyScanEnabled: "Lokalen Datenschutzscan aktivieren",
  privacyScanEnabledHelp: "Prüft Bildmetadaten und führt lokale OCR aus, bevor Inhalte im Verlauf oder in OBS erscheinen.",
  privacySuspiciousPolicy: "Richtlinie für verdächtige Medien",
  privacySuspiciousPolicyHelp: "Lege fest, ob schwache Signale zugelassen, geprüft oder blockiert werden.",
  privacyPolicyAllow: "Zulassen",
  privacyPolicyReview: "Prüfen",
  privacyPolicyBlock: "Blockieren",
  privacySuspiciousThreshold: "Prüfschwelle",
  privacySensitiveThreshold: "Schwelle für sensible Inhalte",
  privacySimilarityBoost: "Ähnlichkeitsverstärkung",
  privacyConcepts: "Zu filternde Wörter oder Ausdrücke",
  privacyConceptsHelp: "Gib durch Kommas getrennte Wörter oder Ausdrücke ein, zum Beispiel: fdp, hitler. Relay berücksichtigt Groß- und Kleinschreibung, Satzzeichen, Trennzeichen, Leetspeak, unterstützte Homoglyphen, wiederholte Buchstaben und vorsichtige Ähnlichkeit. Vorhandene Aliasse und reguläre Ausdrücke bleiben erhalten, solange dieselbe kanonische Form vorhanden ist.",
  privacyExemptRoles: "Ausgenommene Discord-Rollen",
  privacyExemptRolesHelp: "Gib durch Kommas oder Zeilenumbrüche getrennte Rollen-IDs oder Discord-Rollenerwähnungen ein. Diese Rollen umgehen nur gefilterte Wörter; Datenschutzsignale und manuelle Moderation bleiben aktiv.",
  privacyReviewQueueEmpty: "Zu prüfende Filtertreffer erscheinen hier auch bei deaktivierter manueller Moderation.",
  privacyPendingManual: "Manuelle Prüfung",
  privacyProtection: "Doxxing-Schutz",
  privacyProtectionHelp: "Prüft Discord-Texte und -Medien lokal, bevor sie Verlauf, WebSocket, Windows-Widgets oder OBS erreichen. Erkannte Werte werden nie in Protokolle kopiert.",
  privacyProtectionLevel: "Schutzstufe",
  privacyProtectionLevelHelp: "Ausgewogen reduziert Fehlalarme. Streng und Paranoid stufen schwache Signale früher hoch.",
  privacyProfileBalanced: "Ausgewogen",
  privacyProfileStrict: "Streng",
  privacyProfileParanoid: "Paranoid",
  privacyBlockThreshold: "Automatischer Blockierschwellenwert",
  privacyBlockThresholdHelp: "HIGH blockiert standardmäßig. CRITICAL leitet HIGH-Fälle stattdessen zur lokalen Prüfung weiter.",
  privacyReviewIntermediate: "MEDIUM-Risikofälle prüfen",
  privacyReviewIntermediateHelp: "Legt Medien mit mittlerem Risiko in die vorhandene lokale Moderationswarteschlange.",
  privacyAutoDeleteBlockedMessages: "Blockierte Discord-Nachrichten automatisch löschen",
  privacyAutoDeleteBlockedMessagesHelp: "Löscht Nachrichten, die durch den Datenschutzschwellenwert oder ein automatisch gefiltertes Wort blockiert wurden. Erfordert Nachrichten verwalten.",
  privacyCategories: "Aktive Erkennungskategorien",
  privacyCategoryEmail: "E-Mail", privacyCategoryPhone: "Telefon", privacyCategoryIp: "IP-Adressen",
  privacyCategoryGps: "GPS und Koordinaten", privacyCategoryAddress: "Postanschriften",
  privacyCategoryFinancial: "IBAN und Zahlungskarten", privacyCategoryPlate: "Kfz-Kennzeichen",
  privacyCategoryUrl: "Sensible URLs", privacyCategoryCustom: "Geschützte private Zeichenfolgen",
  privacyCategoryMetadata: "EXIF-Metadaten", privacyCategoryOcr: "Lokale OCR",
  privacyCategoryDocument: "Behördliche Dokumente",
  privacyCustomPatterns: "Zu schützende private Daten",
  privacyCustomPatternsHelp: "Füge Namen, frühere Benutzernamen, Adressvarianten, Straßen, Städte, E-Mail-Adressen, Telefonnummern oder andere private Zeichenfolgen hinzu. Diese Werte bleiben in der lokalen Relay-Konfiguration.",
  privacyCustomPatternsPlaceholder: "Ein Wert pro Zeile",
  privacyAllowlist: "Zulassungsliste",
  privacyAllowlistHelp: "Exakte öffentliche Werte aus dieser Liste werden vor der automatischen Erkennung ausgeblendet.",
  privacyAllowlistPlaceholder: "Ein öffentlicher Wert pro Zeile",
});
Object.assign(translations.en, {
  navigationBack: "Go back",
  navigationForward: "Go forward",
  searchLabel: "Search Relay settings",
  searchPlaceholder: "Search settings",
  searchNoResults: "No matching setting",
  clearSearch: "Clear search",
  fontFamily: "Interface font",
  fontDesignDefault: "Match selected design",
  fontFamilyHelp: "Applies to Relay and its tray menu. OBS output typography remains unchanged.",
});
Object.assign(translations.fr, {
  navigationBack: "Retour",
  navigationForward: "Suivant",
  searchLabel: "Rechercher dans les réglages Relay",
  searchPlaceholder: "Rechercher un réglage",
  searchNoResults: "Aucun réglage correspondant",
  clearSearch: "Effacer la recherche",
  fontFamily: "Police de l’interface",
  fontDesignDefault: "Suivre le design sélectionné",
  fontFamilyHelp: "S’applique à Relay et à son menu de zone de notification. La typographie des sorties OBS reste inchangée.",
});
Object.assign(translations.es, {
  navigationBack: "Atrás",
  navigationForward: "Adelante",
  searchLabel: "Buscar en los ajustes de Relay",
  searchPlaceholder: "Buscar un ajuste",
  searchNoResults: "No hay ningún ajuste coincidente",
  clearSearch: "Borrar búsqueda",
  unsaved: "Cambios sin guardar",
  fontFamily: "Fuente de la interfaz",
  fontDesignDefault: "Usar la fuente del diseño seleccionado",
  fontFamilyHelp: "Se aplica a Relay y a su menú de la bandeja. La tipografía de las salidas de OBS no cambia.",
});
Object.assign(translations.de, {
  navigationBack: "Zurück",
  navigationForward: "Vorwärts",
  searchLabel: "Relay-Einstellungen durchsuchen",
  searchPlaceholder: "Einstellung suchen",
  searchNoResults: "Keine passende Einstellung",
  clearSearch: "Suche löschen",
  unsaved: "Nicht gespeicherte Änderungen",
  fontFamily: "Oberflächenschrift",
  fontDesignDefault: "Schrift des ausgewählten Designs verwenden",
  fontFamilyHelp: "Gilt für Relay und sein Infobereichsmenü. Die Typografie der OBS-Ausgaben bleibt unverändert.",
});
translations.ru = {};
translations.zh = {};
translations.ko = {};
translations.ja = {};
translations.id = {};
Object.assign(translations.ru, {
  navOverview: "Обзор", navMedia: "Медиа", navOverlay: "Оверлей", navModeration: "Модерация", navHistory: "История", navHelp: "Справка", navPersonalization: "Персонализация", navCommands: "Команды", navAbout: "О Relay",
  language: "Язык", appearance: "Внешний вид", light: "Светлая", dark: "Тёмная", overlays: "источники OBS", system: "Система", playback: "Воспроизведение", output: "Вывод", safety: "Безопасность", archive: "Архив", guide: "Руководство", about: "О Relay",
  overviewKicker: "Локальная трансляция", overviewTitle: "Один канал. Каждый экран.", overviewCopy: "Подключите Discord один раз, выберите канал и оставьте Relay незаметно работающим в области уведомлений.",
  credentialsTitle: "Подключение Discord", credentialsCopy: "Учётные данные шифруются Windows и больше не отображаются.", clientId: "ID клиента Discord", botToken: "Токен бота Discord", connectBot: "Зашифровать и запустить бота", inviteUrl: "URL приглашения бота", openInvite: "Открыть", copy: "Копировать", copied: "Скопировано",
  routingTitle: "Маршрутизация входа", routingCopy: "Выберите один канал Discord для медиа и другой для озвучиваемых сообщений.", mediaChannel: "Канал медиа", ttsChannel: "Канал сообщений TTS", localPort: "Локальный порт", saveRouting: "Сохранить маршрутизацию", selectChannel: "Выберите доступный текстовый канал", ttsDisabled: "TTS отключён", unavailableChannel: "Канал недоступен", refreshChannels: "Обновить каналы", channelsRefreshed: "Список каналов обновлён",
  mediaKicker: "Очередь воспроизведения", mediaTitle: "Медиа на ваших условиях.", mediaCopy: "Изображения и GIF используют отдельные таймеры. Видео и аудио воспроизводятся до конца.", transportLabel: "Управление в реальном времени", transportReady: "Готово к следующему элементу", skip: "Пропустить текущий элемент",
  playbackTitle: "Настройки воспроизведения", imageDuration: "Длительность изображения", gifDuration: "Длительность GIF", imageDurationHelp: "Только для статических изображений.", gifDurationHelp: "Анимированные GIF повторяются указанное время.", seconds: "с", mediaVolume: "Громкость медиа", mediaVolumeHelp: "Применяется к видео, аудио и озвучиваемым сообщениям.", widgetSound: "Звук виджета", widgetSoundHelp: "Воспроизводит звук видео и аудио в виджете Windows. Источники OBS используют собственный звук.", ttsCharacterLimit: "Лимит символов TTS", ttsCharacterLimitHelp: "Используйте 0 для неограниченной длины сообщения.", characters: "симв.", ttsQueueLimit: "Размер очереди TTS", ttsQueueLimitHelp: "Максимум ожидающих сообщений: от 1 до 50.", items: "элементов",
  ttsSpeech: "Голос TTS", ttsSpeechHelp: "При отключении сообщения TTS показываются как тихие уведомления.", obsNotifications: "Показывать уведомления TTS в OBS", obsNotificationsHelp: "Показывает автора и сообщение во время озвучивания.", obsNotificationOutput: "Оверлей уведомлений TTS для OBS", obsNotificationOutputHelp: "Независимый источник браузера, отображаемый только в OBS.", enableObsNotifications: "Включить оверлей OBS", enableObsNotificationsHelp: "Не изменяет виджет Windows.", windowsNotificationWidget: "Виджет уведомлений TTS Windows", windowsNotificationWidgetHelp: "Независимое окно рабочего стола, которое можно разместить на любом мониторе.", notificationSound: "Звук уведомления", notificationSoundHelp: "Воспроизводится виджетом Windows для каждого сообщения. Поддерживается аудиофайл до 10 секунд.", chooseNotificationSound: "Выбрать аудиофайл", resetNotificationSound: "Удалить звук", noNotificationSound: "Файл не выбран.", notificationSoundObs: "Звук уведомления в OBS", notificationSoundObsHelp: "Оверлей уведомлений OBS воспроизводит тот же выбранный звук, слышимый в трансляции.", showAuthor: "Показывать автора", showAuthorHelp: "Показывает имя Discord и аватар поверх медиа.", supportedFormats: "Поддерживаются изображения, GIF, MP4/WebM и распространённые аудиоформаты.", savePlayback: "Сохранить воспроизведение",
  overlayKicker: "Вывод программы", overlayTitle: "Что получает OBS.", overlayCopy: "Холст остаётся прозрачным, пока медиа не попадёт в очередь.", livePreview: "Предпросмотр", transparentCanvas: "Прозрачный холст", browserSource: "Источники браузера OBS", browserSourceHelp: "Добавьте каждый приватный URL как отдельный источник браузера OBS.", visualSource: "Визуальные медиа", ttsSource: "Аудио TTS", notificationSource: "Уведомления TTS", audioSource: "Аудио, музыка и голосовые сообщения", regenerateSecret: "Переподключить источники OBS", floatingWidget: "Плавающий медиа-виджет", widgetHelp: "Разблокируйте, разместите на любом мониторе, затем заблокируйте для пропуска кликов.", notificationWidget: "Виджет уведомлений TTS", notificationWidgetHelp: "Покажите его в Windows, переместите и заблокируйте для пропуска кликов.", showNotificationWidget: "Показать в Windows",
  historyKicker: "Последние 50 элементов", historyTitle: "История медиа", historyCopy: "Повторите предыдущий элемент или очистите все подключённые оверлеи.", clearOverlay: "Очистить оверлей", historyEmpty: "Ожидание первого медиа из Discord.", replay: "Повторить",
  moderationKicker: "Безопасность трансляции", moderationTitle: "Вы решаете, что попадёт в OBS.", moderationCopy: "При необходимости удерживайте входящие медиа до локального одобрения.", moderationSettings: "Настройки модерации", enableModeration: "Включить ручную модерацию", enableModerationHelp: "Если выключено, медиа сразу поступают в OBS.", allowImages: "Изображения и GIF", allowImagesHelp: "Разрешить этим элементам попасть в очередь одобрения.", allowVideos: "Видео", allowVideosHelp: "Разрешить видеофайлам попасть в очередь одобрения.", allowAudio: "Аудио", allowAudioHelp: "Разрешить аудиофайлам попасть в очередь одобрения.", moderationLocalOnly: "Решения остаются локальными и не уведомляют пользователей Discord.", saveModeration: "Сохранить модерацию", pendingMedia: "Ожидающие медиа", clearPending: "Отклонить все", moderationEmpty: "Нет медиа, ожидающих одобрения.", moderationDisabled: "Ручная модерация отключена.", approve: "Одобрить", reject: "Отклонить",
  botOffline: "Бот не в сети", serverOnline: "Сервер в сети", serverOffline: "Сервер не в сети", notConfigured: "Не настроено", savedVia: "Сохранено через", encrypting: "Шифрование…", encryptedStarting: "Зашифровано; бот запускается", saving: "Сохранение…", saved: "Сохранено", regenerating: "Переподключение…", secretRegenerated: "Постоянные ссылки сохранены", skipped: "Текущий элемент пропущен", widgetHidden: "Виджет скрыт", widgetVisibleLocked: "Виджет видим и заблокирован", widgetVisibleMovable: "Виджет видим и перемещаем", showWidget: "Показать виджет", hideWidget: "Скрыть виджет", unlockMove: "Разблокировать для перемещения", lockDisplay: "Заблокировать отображение", unknownAuthor: "Неизвестный автор",
  helpKicker: "Руководство по настройке", helpTitle: "От Discord до OBS.", helpCopy: "Выполните эти шаги один раз, затем оставьте Relay работающим в области уведомлений.",
  aboutKicker: "О Relay", aboutTitle: "Создан локально. Сделан, чтобы исчезать.", aboutCopy: "Relay соединяет Discord с OBS и Windows, сохраняя учётные данные и трафик на этом компьютере.", aboutStatement: "Приватная утилита трансляции для медиа, озвучиваемых сообщений и уведомлений с постоянными локальными источниками OBS.", aboutCreatorLabel: "Автор", aboutCreatorCopy: "Посмотрите проекты и исходный код автора на GitHub.", aboutPrivacy: "Учётные данные зашифрованы Windows", aboutNetwork: "Локальный сервер · 127.0.0.1",
  privacyCardTitle: "Relay не собирает данные", privacyCardCopy: "Нет телеметрии, аналитики, рекламы или удалённого профиля пользователя. Настройки остаются на этом компьютере; Discord остаётся внешним сервисом.", privacyCardLink: "Конфиденциальность и региональные права", privacyDetailsTitle: "Конфиденциальность и региональные права", privacyDetailsSummary: "GDPR и аналогичные законы во всём мире",
  personalizationKicker: "Интерфейс", personalizationTitle: "Настройте Relay под себя.", personalizationCopy: "Изменения сразу применяются к Relay, виджетам Windows и выходам OBS.", themeLabel: "Тема", accentColor: "Цвет акцента", fontSize: "Размер текста", previewTitle: "Предпросмотр интерфейса Relay", previewCopy: "Читаемый текст с выбранным цветом.", previewButton: "Пример кнопки", resetDefaults: "Сбросить настройки", personalizationSaved: "Настройки применены",
  showMediaTextObs: "Показывать текст медиа в OBS", showMediaTextObsHelp: "Показывает до 180 символов сообщения Discord.", showMediaTextWidget: "Показывать текст медиа в виджете Windows", showMediaTextWidgetHelp: "Не зависит от текста, показанного в OBS.", stickerDuration: "Длительность стикеров", stickerDurationHelp: "Стикеры Discord остаются видимыми указанное время.", notificationDuration: "Длительность уведомлений", notificationDurationHelp: "Уведомления TTS без аудио остаются видимыми указанное время.", durationsGroup: "Длительность отображения", durationsGroupHelp: "Время видимости изображений, стикеров, уведомлений и GIF.", audioTtsGroup: "Аудио и TTS", audioTtsGroupHelp: "Громкость, длина сообщений, очередь и голос.", displayGroup: "Отображение", displayGroupHelp: "Что показывается поверх медиа.", stickerSource: "Стикеры Discord",
  commandsKicker: "Управление Discord", commandsTitle: "Ваши команды, ваши правила.", commandsCopy: "Включите только те команды Relay, которые нужны в Discord.", commandsSettings: "Доступность команд", commandChannelHelp: "Выберите канал медиа Discord.", commandUrlHelp: "Кратко показывает локальные URL Relay и OBS.", commandShowHelp: "Показывает активную конфигурацию Relay.", commandStatusHelp: "Показывает состояние выходов, очередей и виджетов.", commandTestHelp: "Отправляет изолированный локальный тест на подключённый выход.", commandRegenerateHelp: "Переподключает локальные выходы без изменения URL.", commandClearHelp: "Удаляет указанное число сообщений из выбранного в команде канала Discord.", commandLockHelp: "Блокирует или разблокирует настроенный канал медиа.", commandChangelogHelp: "Публикует последние заметки о версии из GitHub в выбранный канал.", commandLockInactive: "Канал медиа сейчас разблокирован.", commandLockActive: "Канал медиа заблокирован. /relay lock доступна для разблокировки.", saveCommands: "Сохранить команды", commandsSaved: "Доступность команд сохранена", commandsPermission: "Блокировка требует права «Управление ролями», очистка — «Управление сообщениями». Команды доступны только администраторам Discord.",
  defaultCommands: "Стандартные команды", customCommands: "Пользовательские команды", customCommandsHelp: "Создавайте локальные подкоманды /relay на основе одного предопределённого действия Discord. Этот список используется на каждом сервере, где установлен бот.", customCommandsEmpty: "Пользовательские команды не настроены.", addCustomCommand: "Добавить команду", customCommandEditor: "Редактор команды", customCommandName: "Имя команды", customCommandAction: "Предопределённое действие", customCommandDescription: "Описание Discord", customCommandEnabled: "Зарегистрировать эту команду в Discord", customActionParameters: "Параметры действия", customAccessRestrictions: "Ограничения доступа", customAdminOnly: "Требовать администратора Discord помимо права действия", customExtraPermissions: "Дополнительные обязательные права", permissionManageGuild: "Управление сервером", permissionManageMessages: "Управление сообщениями", permissionManageRoles: "Управление ролями", permissionBanMembers: "Банить участников", permissionKickMembers: "Исключать участников", permissionModerateMembers: "Модерировать участников", customAllowedUsers: "Разрешённые ID пользователей", customAllowedRoles: "Разрешённые ID ролей", customAllowedChannels: "Разрешённые ID каналов вызова", discordIdsPlaceholder: "Один ID или упоминание Discord на строку", cancel: "Отмена", saveCommandDraft: "Сохранить команду", syncCustomCommands: "Сохранить и синхронизировать с Discord", edit: "Изменить", delete: "Удалить", active: "Активна", disabled: "Отключена",
  customActionBan: "Забанить участника или ID пользователя", customActionUnban: "Разбанить пользователя", customActionKick: "Исключить участника", customActionTimeout: "Ограничить участника", customActionRemoveTimeout: "Снять ограничение", customActionClearMessages: "Очистить сообщения", customActionAddRole: "Добавить роль", customActionRemoveRole: "Удалить роль", customActionReply: "Предопределённый ответ", customParameterMode: "Режим", customParameterValue: "Резервное или фиксированное значение", parameterRequired: "Обязательный", parameterOptional: "Необязательный", parameterFixed: "Фиксированный локально", customReason: "Причина для журнала аудита", customDeleteDays: "Удаление недавних сообщений (дни)", customDurationMinutes: "Длительность ограничения (минуты)", customChannelId: "ID канала Discord", customMessageCount: "Количество сообщений", customRoleId: "ID роли Discord", customReplyText: "Текст ответа", customReplyVisibility: "Видимость ответа", customReplyEphemeral: "Только для вызвавшего", customReplyPublic: "Публичный", customRequiredPermission: "Минимальное обязательное право: {permission}. Разрушающие действия всегда требуют однократного подтверждения.", customUnsaved: "Не сохранено", customValidating: "Проверка", customSyncing: "Синхронизация с Discord", customActive: "Активна в Discord", customMaxReached: "Relay поддерживает не более 16 пользовательских команд.", customDuplicateName: "Имена команд должны быть уникальными и не могут совпадать со стандартными командами Relay.", customInvalidIds: "Используйте один действительный ID или упоминание Discord на строку.", customDraftSaved: "Команда сохранена локально. Синхронизируйте её для активации.",
  sizeAndCrop: "Размер и обрезка", sizeAndCropHelp: "Настройте размер и обрезку локальных выходов.", mediaObsOutput: "Медиа в OBS", mediaWidgetOutput: "Медиа-виджет Windows", notificationObsOutput: "Уведомления в OBS", notificationWidgetOutput: "Виджет уведомлений Windows", contentScale: "Масштаб содержимого", cropTop: "Обрезать сверху", cropRight: "Обрезать справа", cropBottom: "Обрезать снизу", cropLeft: "Обрезать слева", outputWidth: "Ширина", outputHeight: "Высота", keepAspectRatio: "Сохранять формат 16:9", resetOutput: "Сбросить", geometrySaved: "Геометрия сохранена", geometryPreview: "Предпросмотр",
  botPresence: "Статус бота", botPresenceHelp: "Настройте отображаемый статус и активность бота Discord.", onlineStatus: "Статус в сети", statusOnline: "В сети", statusIdle: "Неактивен", statusDnd: "Не беспокоить", statusInvisible: "Невидим", activityType: "Тип активности", activityNone: "Нет", activityCustom: "Пользовательский", activityPlaying: "Играет", activityListening: "Слушает", activityWatching: "Смотрит", activityCompeting: "Соревнуется", activityText: "Текст активности", activityTextHelp: "Короткая подпись, показанная в профиле бота.", saveBotPresence: "Сохранить статус бота", botPresenceSaved: "Статус бота сохранён", nowPlaying: "Сейчас играет", previousAudio: "Предыдущее аудио", pauseAudio: "Пауза", resumeAudio: "Продолжить", skipAudio: "Пропустить аудио",
  outputReadiness: "Готовность выходов", outputReadinessHelp: "Проверьте, какие локальные выходы подключены. Тесты остаются локальными и не публикуются в Discord.", outputObs: "OBS", outputPreview: "Предпросмотр", outputWidget: "Виджет", outputDisconnected: "Не подключён", outputLastConnected: "Последнее подключение", outputNeverConnected: "Никогда не подключался", testOutput: "Тестировать выход", outputTestSent: "Тест отправлен", outputTestFailed: "Тест не удался", outputTestNeedsLiveOutput: "Подключите OBS или виджет перед тестом.",
  updatesTitle: "Обновления Relay", checkUpdates: "Проверить обновления", checkUpdatesPrompt: "Проверить на GitHub новую версию Relay.", checkingUpdates: "Проверка последней официальной версии…", updateAvailable: "Доступна Relay v{version}.", upToDate: "Relay v{version} актуальна.", downloadAndInstall: "Скачать и установить", downloadingUpdate: "Загрузка и проверка v{version}…", openReleases: "Посмотреть версии", closeUpdateMenu: "Закрыть меню обновлений", updateCheckFailed: "Не удалось проверить обновление:", updateInstallFailed: "Не удалось обновить:",
  designLabel: "Дизайн", openaiDesignCopy: "Точный, спокойный и практичный.", anthropicDesignCopy: "Тёплый, литературный и человечный.", neoDesignCopy: "Смелый, редакционный и игривый.",
  automaticFilterWords: "Автоматическая фильтрация", automaticFilterWordsHelp: "Слова фильтра сохраняются автоматически после окончания ввода. Они работают даже при выключенных ручной модерации и локальном сканировании изображений. Точные совпадения и существующие регулярные выражения блокируются сразу и не попадают в очередь ручной модерации.", manualModeration: "Ручная модерация", manualModerationHelp: "Удерживайте медиа для одобрения независимо от автоматической фильтрации.", privacyScanEnabled: "Включить локальную проверку конфиденциальности", privacyScanEnabledHelp: "Проверяет метаданные изображений и локальный OCR перед историей или OBS.", privacySuspiciousPolicy: "Политика подозрительных медиа", privacySuspiciousPolicyHelp: "Выберите, разрешать, проверять или блокировать слабые сигналы.", privacyPolicyAllow: "Разрешить", privacyPolicyReview: "Проверить", privacyPolicyBlock: "Блокировать", privacySuspiciousThreshold: "Порог проверки", privacySensitiveThreshold: "Порог чувствительности", privacySimilarityBoost: "Усиление сходства", privacyConcepts: "Слова или фразы фильтра", privacyExemptRoles: "Исключённые роли Discord", privacyExemptRolesHelp: "Введите ID ролей или упоминания ролей Discord через запятую или с новой строки. Эти роли обходят только слова фильтра; локальные сигналы конфиденциальности и ручная модерация сохраняются.", unsaved: "Несохранённые изменения", privacyConceptsHelp: "Введите слова или фразы через запятую, например: fdp, hitler. Relay обрабатывает регистр, пунктуацию и разделители, leetspeak, поддерживаемые омоглифы, повторяющиеся буквы и осторожное сходство. Существующие псевдонимы и регулярные выражения сохраняются, пока остаётся та же каноническая форма.", privacyReviewQueueEmpty: "Совпадения фильтра, требующие проверки, появляются здесь даже при выключенной ручной модерации.", privacyPendingManual: "Ручная проверка", privacyProtection: "Защита от доксинга", privacyProtectionHelp: "Сканирует локально до того, как текст или медиа Discord попадут в историю, WebSocket, виджеты Windows или OBS. Обнаруженные значения никогда не копируются в журналы.", privacyProtectionLevel: "Уровень защиты", privacyProtectionLevelHelp: "Сбалансированный режим снижает ложные срабатывания. Строгий и параноидальный режимы быстрее повышают слабые сигналы.", privacyProfileBalanced: "Сбалансированный", privacyProfileStrict: "Строгий", privacyProfileParanoid: "Параноидальный", privacyBlockThreshold: "Порог автоматической блокировки", privacyBlockThresholdHelp: "HIGH блокирует по умолчанию. CRITICAL отправляет случаи HIGH на локальную проверку.", privacyReviewIntermediate: "Проверять случаи с риском MEDIUM", privacyReviewIntermediateHelp: "Помещает медиа со средним риском в существующую локальную очередь модерации.", privacyAutoDeleteBlockedMessages: "Удалять заблокированные сообщения Discord", privacyAutoDeleteBlockedMessagesHelp: "Удаляет сообщения, заблокированные порогом конфиденциальности или словом автоматического фильтра. Требуется право «Управление сообщениями».", privacyCategories: "Включённые категории обнаружения", privacyCategoryEmail: "Электронная почта", privacyCategoryPhone: "Телефон", privacyCategoryIp: "IP-адреса", privacyCategoryGps: "GPS и координаты", privacyCategoryAddress: "Почтовые адреса", privacyCategoryFinancial: "IBAN и платёжные карты", privacyCategoryPlate: "Номерные знаки", privacyCategoryUrl: "Чувствительные URL", privacyCategoryCustom: "Защищённые приватные строки", privacyCategoryMetadata: "Метаданные EXIF", privacyCategoryOcr: "Локальный OCR", privacyCategoryDocument: "Административные документы", privacyCustomPatterns: "Приватные данные для защиты", privacyCustomPatternsHelp: "Добавьте имена, старые имена пользователей, варианты адресов, улицы, города, электронную почту, номера телефонов или другие приватные строки. Значения остаются в локальной конфигурации Relay.", privacyCustomPatternsPlaceholder: "Одно значение на строку", privacyAllowlist: "Список разрешённых", privacyAllowlistHelp: "Точные публичные значения из этого списка маскируются перед автоматическим обнаружением.", privacyAllowlistPlaceholder: "Одно публичное значение на строку",
  navigationBack: "Назад", navigationForward: "Вперёд", searchLabel: "Поиск настроек Relay", searchPlaceholder: "Поиск настройки", searchNoResults: "Нет подходящей настройки", clearSearch: "Очистить поиск", fontFamily: "Шрифт интерфейса", fontDesignDefault: "Соответствовать выбранному дизайну", fontFamilyHelp: "Применяется к Relay и его меню в области уведомлений. Типографика выходов OBS не изменяется.",
});
Object.assign(translations.zh, {
  navOverview: "概览", navMedia: "媒体", navOverlay: "叠加层", navModeration: "内容审核", navHistory: "历史记录", navHelp: "帮助", navPersonalization: "个性化", navCommands: "命令", navAbout: "关于",
  language: "语言", appearance: "外观", light: "浅色", dark: "深色", overlays: "OBS 来源", system: "系统", playback: "播放", output: "输出", safety: "安全", archive: "存档", guide: "指南", about: "关于 Relay",
  overviewKicker: "本地直播", overviewTitle: "一个频道。所有屏幕。", overviewCopy: "连接一次 Discord，选择频道，然后让 Relay 在通知区域静默运行。",
  credentialsTitle: "连接 Discord", credentialsCopy: "凭据由 Windows 加密保存，且不再显示。", clientId: "Discord 客户端 ID", botToken: "Discord 机器人令牌", connectBot: "加密并启动机器人", inviteUrl: "机器人邀请链接", openInvite: "打开", copy: "复制", copied: "已复制",
  routingTitle: "输入路由", routingCopy: "选择一个 Discord 频道接收媒体，另一个频道接收 TTS 消息。", mediaChannel: "媒体频道", ttsChannel: "TTS 消息频道", localPort: "本地端口", saveRouting: "保存路由", selectChannel: "选择可用的文字频道", ttsDisabled: "已禁用 TTS", unavailableChannel: "频道不可用", refreshChannels: "刷新频道", channelsRefreshed: "频道列表已刷新",
  mediaKicker: "播放队列", mediaTitle: "媒体按你的规则播放。", mediaCopy: "图片和 GIF 使用独立计时器。视频和音频会播放至结束。", transportLabel: "实时控制", transportReady: "准备播放下一项", skip: "跳过当前项",
  playbackTitle: "播放设置", imageDuration: "图片时长", gifDuration: "GIF 时长", imageDurationHelp: "仅适用于静态图片。", gifDurationHelp: "动画 GIF 将循环指定时间。", seconds: "秒", mediaVolume: "媒体音量", widgetSound: "小组件声音", ttsCharacterLimit: "TTS 字符限制", characters: "个字符", ttsQueueLimit: "TTS 队列大小", items: "项", ttsSpeech: "TTS 语音", obsNotifications: "在 OBS 中显示 TTS 通知", obsNotificationOutput: "OBS TTS 通知叠加层", enableObsNotifications: "启用 OBS 叠加层", windowsNotificationWidget: "Windows TTS 通知小组件", notificationSound: "通知声音", chooseNotificationSound: "选择音频文件", resetNotificationSound: "移除声音", noNotificationSound: "未选择文件。", notificationSoundObs: "OBS 中的通知声音", showAuthor: "显示作者", supportedFormats: "支持图片、GIF、MP4/WebM 和常用音频格式。", savePlayback: "保存播放设置",
  overlayKicker: "应用输出", overlayTitle: "OBS 会收到什么。", overlayCopy: "在媒体进入队列前，画布保持透明。", livePreview: "实时预览", transparentCanvas: "透明画布", browserSource: "OBS 浏览器来源", browserSourceHelp: "将每个私有 URL 添加为独立的 OBS 浏览器来源。", visualSource: "视觉媒体", ttsSource: "TTS 音频", notificationSource: "TTS 通知", audioSource: "音频、音乐和语音消息", regenerateSecret: "重新连接 OBS 来源", floatingWidget: "浮动媒体小组件", notificationWidget: "TTS 通知小组件", showNotificationWidget: "在 Windows 中显示",
  historyKicker: "最近 50 项", historyTitle: "媒体历史", historyCopy: "重播过去的项目，或清除所有已连接的叠加层。", clearOverlay: "清除叠加层", historyEmpty: "正在等待 Discord 中的第一条媒体。", replay: "重播",
  moderationKicker: "直播安全", moderationTitle: "由你决定哪些内容进入 OBS。", moderationCopy: "可在本地批准前暂时保留传入媒体。", moderationSettings: "审核设置", enableModeration: "启用手动审核", enableModerationHelp: "关闭后，媒体会直接发送到 OBS。", allowImages: "图片和 GIF", allowImagesHelp: "允许这些项目进入审批队列。", allowVideos: "视频", allowVideosHelp: "允许视频文件进入审批队列。", allowAudio: "音频", allowAudioHelp: "允许音频文件进入审批队列。", moderationLocalOnly: "决定仅保存在本地，不会通知 Discord 用户。", saveModeration: "保存审核设置", pendingMedia: "待处理媒体", clearPending: "全部拒绝", moderationEmpty: "没有待审批的媒体。", moderationDisabled: "手动审核已关闭。", approve: "批准", reject: "拒绝",
  botOffline: "机器人离线", serverOnline: "服务器在线", serverOffline: "服务器离线", notConfigured: "未配置", saving: "正在保存…", saved: "已保存", skipped: "已跳过当前项目", showWidget: "显示小组件", hideWidget: "隐藏小组件", unlockMove: "解锁以移动", lockDisplay: "锁定显示", unknownAuthor: "未知作者",
  helpKicker: "设置指南", helpTitle: "从 Discord 到 OBS。", aboutKicker: "关于此应用", aboutTitle: "Relay", aboutCopy: "本地 Discord 到 OBS 的媒体中继。", privacyCardTitle: "隐私优先", privacyCardCopy: "Relay 在此设备上处理内容。", privacyCardLink: "了解隐私控制", personalizationKicker: "工作区", personalizationTitle: "让 Relay 更符合你的习惯。", themeLabel: "主题", accentColor: "强调色", fontSize: "字体大小", previewTitle: "预览", previewButton: "示例按钮", resetDefaults: "恢复默认值", personalizationSaved: "个性化设置已保存",
  commandsKicker: "Discord 控制", commandsTitle: "命令，由你掌控。", commandsCopy: "仅在 Discord 中启用你想使用的 Relay 命令。", commandsSettings: "命令设置", saveCommands: "保存命令", commandsSaved: "命令已保存", defaultCommands: "默认命令", customCommands: "自定义命令", customCommandsHelp: "创建由预设 Discord 操作支持的本地 /relay 子命令。", customCommandsEmpty: "尚未创建自定义命令。", addCustomCommand: "添加命令", customCommandEditor: "命令编辑器", customCommandName: "命令名称", customCommandAction: "预设操作", customCommandDescription: "Discord 描述", customCommandEnabled: "在 Discord 中注册此命令", customActionParameters: "操作参数", customAccessRestrictions: "访问限制", customAdminOnly: "除操作权限外还要求 Discord 管理员权限", customExtraPermissions: "额外所需权限", customAllowedUsers: "允许的用户 ID", customAllowedRoles: "允许的角色 ID", customAllowedChannels: "允许调用的频道 ID", discordIdsPlaceholder: "每行一个 Discord ID 或提及", cancel: "取消", saveCommandDraft: "保存命令", syncCustomCommands: "保存并与 Discord 同步", edit: "编辑", delete: "删除", active: "已启用", disabled: "已禁用",
  customActionBan: "封禁成员", customActionUnban: "解除封禁成员", customActionKick: "移除成员", customActionTimeout: "禁言成员", customActionRemoveTimeout: "解除禁言", customActionClearMessages: "删除消息", customActionAddRole: "添加角色", customActionRemoveRole: "移除角色", customActionReply: "回复", parameterRequired: "必填", parameterOptional: "可选", parameterFixed: "本地固定", customReason: "审核日志原因", customDeleteDays: "删除近期消息（天）", customDurationMinutes: "禁言时长（分钟）", customChannelId: "频道", customMessageCount: "消息数量", customRoleId: "角色", customReplyText: "回复内容", customReplyVisibility: "回复可见性", customReplyEphemeral: "仅调用者可见", customReplyPublic: "公开回复",
  automaticFilterWords: "自动过滤", automaticFilterWordsHelp: "停止输入后会自动保存过滤词。即使关闭手动审核和本地扫描，它们仍然生效。", manualModeration: "手动审核", manualModerationHelp: "独立于自动过滤词进行媒体审批。", privacyScanEnabled: "启用本地隐私扫描", privacyScanEnabledHelp: "在内容进入历史记录或 OBS 前检查图像元数据和本地 OCR。", privacyProtection: "反人肉搜索保护", privacyProtectionHelp: "在 Discord 文字或媒体进入历史记录、WebSocket、Windows 小组件或 OBS 前于本地扫描。检测到的值绝不会写入日志。", privacyProtectionLevel: "保护级别", privacyProfileBalanced: "平衡", privacyProfileStrict: "严格", privacyProfileParanoid: "偏执", privacyBlockThreshold: "自动拦截阈值", privacyReviewIntermediate: "审核中等风险案例", privacyAutoDeleteBlockedMessages: "删除已拦截的 Discord 消息", privacyCategories: "已启用的检测类别", privacyCategoryEmail: "电子邮件", privacyCategoryPhone: "电话号码", privacyCategoryIp: "IP 地址", privacyCategoryGps: "GPS 和坐标", privacyCategoryAddress: "邮政地址", privacyCategoryFinancial: "IBAN 和银行卡", privacyCategoryPlate: "车牌", privacyCategoryUrl: "敏感 URL", privacyCategoryCustom: "受保护的私密字符串", privacyCategoryMetadata: "EXIF 元数据", privacyCategoryOcr: "本地 OCR", privacyCategoryDocument: "行政文件", privacyCustomPatterns: "要保护的私密数据", privacyCustomPatternsPlaceholder: "每行一个值", privacyAllowlist: "允许列表", privacyAllowlistPlaceholder: "每行一个公开值",
  navigationBack: "返回", navigationForward: "前进", searchLabel: "搜索 Relay 设置", searchPlaceholder: "搜索设置", searchNoResults: "没有匹配的设置", clearSearch: "清除搜索", fontFamily: "界面字体", fontDesignDefault: "匹配所选设计",
});

Object.assign(translations.ko, {
  navOverview: "개요", navMedia: "미디어", navOverlay: "오버레이", navModeration: "검토", navHistory: "기록", navHelp: "도움말", navPersonalization: "개인 설정", navCommands: "명령", navAbout: "정보",
  language: "언어", appearance: "모양", light: "라이트", dark: "다크", overlays: "OBS 소스", system: "시스템", playback: "재생", output: "출력", safety: "안전", archive: "보관함", guide: "안내", about: "Relay 정보",
  overviewKicker: "로컬 방송", overviewTitle: "하나의 채널. 모든 화면.", overviewCopy: "Discord를 한 번 연결하고 채널을 선택하면 Relay가 알림 영역에서 조용히 실행됩니다.",
  credentialsTitle: "Discord 연결", credentialsCopy: "자격 증명은 Windows에서 암호화되며 다시 표시되지 않습니다.", clientId: "Discord 클라이언트 ID", botToken: "Discord 봇 토큰", connectBot: "암호화하고 봇 시작", inviteUrl: "봇 초대 URL", openInvite: "열기", copy: "복사", copied: "복사됨",
  routingTitle: "입력 라우팅", routingCopy: "미디어용 Discord 채널 하나와 TTS 메시지용 채널 하나를 선택하세요.", mediaChannel: "미디어 채널", ttsChannel: "TTS 메시지 채널", localPort: "로컬 포트", saveRouting: "라우팅 저장", selectChannel: "사용 가능한 텍스트 채널 선택", ttsDisabled: "TTS 사용 안 함", unavailableChannel: "채널을 사용할 수 없음", refreshChannels: "채널 새로 고침", channelsRefreshed: "채널 목록을 새로 고쳤습니다",
  mediaKicker: "재생 대기열", mediaTitle: "내 규칙대로 재생되는 미디어.", mediaCopy: "이미지와 GIF에는 개별 타이머가 사용됩니다. 동영상과 오디오는 끝까지 재생됩니다.", transportLabel: "실시간 제어", transportReady: "다음 항목 준비 완료", skip: "현재 항목 건너뛰기",
  playbackTitle: "재생 설정", imageDuration: "이미지 표시 시간", gifDuration: "GIF 표시 시간", imageDurationHelp: "정지 이미지에만 적용됩니다.", gifDurationHelp: "애니메이션 GIF는 지정한 시간 동안 반복됩니다.", seconds: "초", mediaVolume: "미디어 볼륨", widgetSound: "위젯 사운드", ttsCharacterLimit: "TTS 글자 수 제한", characters: "자", ttsQueueLimit: "TTS 대기열 크기", items: "개", ttsSpeech: "TTS 음성", obsNotifications: "OBS에 TTS 알림 표시", obsNotificationOutput: "OBS TTS 알림 오버레이", enableObsNotifications: "OBS 오버레이 사용", windowsNotificationWidget: "Windows TTS 알림 위젯", notificationSound: "알림 소리", chooseNotificationSound: "오디오 파일 선택", resetNotificationSound: "소리 제거", noNotificationSound: "선택한 파일이 없습니다.", notificationSoundObs: "OBS 알림 소리", showAuthor: "작성자 표시", supportedFormats: "이미지, GIF, MP4/WebM 및 일반 오디오 형식을 지원합니다.", savePlayback: "재생 설정 저장",
  overlayKicker: "앱 출력", overlayTitle: "OBS에 전달되는 내용.", overlayCopy: "미디어가 대기열에 들어올 때까지 캔버스는 투명하게 유지됩니다.", livePreview: "실시간 미리 보기", transparentCanvas: "투명 캔버스", browserSource: "OBS 브라우저 소스", browserSourceHelp: "각 비공개 URL을 별도의 OBS 브라우저 소스로 추가하세요.", visualSource: "시각 미디어", ttsSource: "TTS 오디오", notificationSource: "TTS 알림", audioSource: "오디오, 음악 및 음성 메시지", regenerateSecret: "OBS 소스 다시 연결", floatingWidget: "플로팅 미디어 위젯", notificationWidget: "TTS 알림 위젯", showNotificationWidget: "Windows에 표시",
  historyKicker: "최근 50개 항목", historyTitle: "미디어 기록", historyCopy: "이전 항목을 다시 재생하거나 연결된 모든 오버레이를 지우세요.", clearOverlay: "오버레이 지우기", historyEmpty: "Discord의 첫 미디어를 기다리는 중입니다.", replay: "다시 재생",
  moderationKicker: "방송 안전", moderationTitle: "OBS에 도달하는 콘텐츠를 직접 결정하세요.", moderationCopy: "수신 미디어를 로컬 승인 전까지 선택적으로 보류합니다.", moderationSettings: "검토 설정", enableModeration: "수동 검토 사용", enableModerationHelp: "끄면 미디어가 OBS로 바로 전달됩니다.", allowImages: "이미지 및 GIF", allowImagesHelp: "이 항목을 승인 대기열에 넣습니다.", allowVideos: "동영상", allowVideosHelp: "동영상 파일을 승인 대기열에 넣습니다.", allowAudio: "오디오", allowAudioHelp: "오디오 파일을 승인 대기열에 넣습니다.", moderationLocalOnly: "결정은 로컬에만 유지되며 Discord 사용자에게 알리지 않습니다.", saveModeration: "검토 저장", pendingMedia: "대기 중인 미디어", clearPending: "모두 거부", moderationEmpty: "승인을 기다리는 미디어가 없습니다.", moderationDisabled: "수동 검토가 꺼져 있습니다.", approve: "승인", reject: "거부",
  botOffline: "봇 오프라인", serverOnline: "서버 온라인", serverOffline: "서버 오프라인", notConfigured: "구성되지 않음", saving: "저장 중…", saved: "저장됨", skipped: "현재 항목을 건너뛰었습니다", showWidget: "위젯 표시", hideWidget: "위젯 숨기기", unlockMove: "이동하려면 잠금 해제", lockDisplay: "표시 잠금", unknownAuthor: "알 수 없는 작성자",
  helpKicker: "설정 안내", helpTitle: "Discord에서 OBS까지.", aboutKicker: "앱 정보", aboutTitle: "Relay", aboutCopy: "로컬 Discord-OBS 미디어 릴레이입니다.", privacyCardTitle: "개인정보 우선", privacyCardCopy: "Relay는 이 장치에서 콘텐츠를 처리합니다.", privacyCardLink: "개인정보 제어 알아보기", personalizationKicker: "작업 공간", personalizationTitle: "Relay를 내 방식대로 사용하세요.", themeLabel: "테마", accentColor: "강조 색상", fontSize: "글꼴 크기", previewTitle: "미리 보기", previewButton: "예시 버튼", resetDefaults: "기본값 복원", personalizationSaved: "개인 설정을 저장했습니다",
  commandsKicker: "Discord 제어", commandsTitle: "내가 제어하는 명령.", commandsCopy: "Discord에서 사용할 Relay 명령만 활성화하세요.", commandsSettings: "명령 설정", saveCommands: "명령 저장", commandsSaved: "명령을 저장했습니다", defaultCommands: "기본 명령", customCommands: "사용자 지정 명령", customCommandsHelp: "미리 정의된 Discord 작업으로 동작하는 로컬 /relay 하위 명령을 만듭니다.", customCommandsEmpty: "사용자 지정 명령이 없습니다.", addCustomCommand: "명령 추가", customCommandEditor: "명령 편집기", customCommandName: "명령 이름", customCommandAction: "미리 정의된 작업", customCommandDescription: "Discord 설명", customCommandEnabled: "Discord에 이 명령 등록", customActionParameters: "작업 매개변수", customAccessRestrictions: "접근 제한", customAdminOnly: "작업 권한 외에 Discord 관리자도 요구", customExtraPermissions: "추가 필요 권한", customAllowedUsers: "허용된 사용자 ID", customAllowedRoles: "허용된 역할 ID", customAllowedChannels: "허용된 호출 채널 ID", discordIdsPlaceholder: "한 줄에 Discord ID 또는 멘션 하나", cancel: "취소", saveCommandDraft: "명령 저장", syncCustomCommands: "저장하고 Discord와 동기화", edit: "편집", delete: "삭제", active: "활성", disabled: "비활성",
  customActionBan: "멤버 차단", customActionUnban: "멤버 차단 해제", customActionKick: "멤버 내보내기", customActionTimeout: "멤버 타임아웃", customActionRemoveTimeout: "타임아웃 해제", customActionClearMessages: "메시지 삭제", customActionAddRole: "역할 추가", customActionRemoveRole: "역할 제거", customActionReply: "답장", parameterRequired: "필수", parameterOptional: "선택 사항", parameterFixed: "로컬 고정", customReason: "감사 로그 사유", customDeleteDays: "최근 메시지 삭제(일)", customDurationMinutes: "타임아웃 시간(분)", customChannelId: "채널", customMessageCount: "메시지 수", customRoleId: "역할", customReplyText: "답장 내용", customReplyVisibility: "답장 공개 범위", customReplyEphemeral: "호출자에게만 표시", customReplyPublic: "공개 답장",
  automaticFilterWords: "자동 필터링", automaticFilterWordsHelp: "입력을 멈추면 필터 단어가 자동 저장됩니다. 수동 검토와 로컬 검사가 꺼져 있어도 적용됩니다.", manualModeration: "수동 검토", manualModerationHelp: "자동 필터 단어와 별개로 미디어를 승인합니다.", privacyScanEnabled: "로컬 개인정보 검사 사용", privacyScanEnabledHelp: "기록 또는 OBS에 표시되기 전에 이미지 메타데이터와 로컬 OCR을 검사합니다.", privacyProtection: "신상 정보 노출 방지", privacyProtectionHelp: "Discord 텍스트나 미디어가 기록, WebSocket, Windows 위젯 또는 OBS에 도달하기 전에 로컬에서 검사합니다. 감지된 값은 로그에 복사되지 않습니다.", privacyProtectionLevel: "보호 수준", privacyProfileBalanced: "균형", privacyProfileStrict: "엄격", privacyProfileParanoid: "강화", privacyBlockThreshold: "자동 차단 임계값", privacyReviewIntermediate: "중간 위험 사례 검토", privacyAutoDeleteBlockedMessages: "차단된 Discord 메시지 삭제", privacyCategories: "활성화된 감지 범주", privacyCategoryEmail: "이메일", privacyCategoryPhone: "전화번호", privacyCategoryIp: "IP 주소", privacyCategoryGps: "GPS 및 좌표", privacyCategoryAddress: "우편 주소", privacyCategoryFinancial: "IBAN 및 결제 카드", privacyCategoryPlate: "번호판", privacyCategoryUrl: "민감한 URL", privacyCategoryCustom: "보호된 비공개 문자열", privacyCategoryMetadata: "EXIF 메타데이터", privacyCategoryOcr: "로컬 OCR", privacyCategoryDocument: "행정 문서", privacyCustomPatterns: "보호할 비공개 데이터", privacyCustomPatternsPlaceholder: "한 줄에 하나의 값", privacyAllowlist: "허용 목록", privacyAllowlistPlaceholder: "한 줄에 하나의 공개 값",
  navigationBack: "뒤로", navigationForward: "앞으로", searchLabel: "Relay 설정 검색", searchPlaceholder: "설정 검색", searchNoResults: "일치하는 설정이 없습니다", clearSearch: "검색 지우기", fontFamily: "인터페이스 글꼴", fontDesignDefault: "선택한 디자인과 맞추기",
});

Object.assign(translations.ja, {
  navOverview: "概要", navMedia: "メディア", navOverlay: "オーバーレイ", navModeration: "モデレーション", navHistory: "履歴", navHelp: "ヘルプ", navPersonalization: "パーソナライズ", navCommands: "コマンド", navAbout: "Relay について",
  language: "言語", appearance: "外観", light: "ライト", dark: "ダーク", overlays: "OBS ソース", system: "システム", playback: "再生", output: "出力", safety: "安全", archive: "アーカイブ", guide: "ガイド", about: "Relay について",
  overviewKicker: "ローカル配信", overviewTitle: "1 つのチャンネル。すべての画面。", overviewCopy: "Discord を一度接続し、チャンネルを選ぶだけで、Relay は通知領域で静かに動作します。",
  credentialsTitle: "Discord を接続", credentialsCopy: "資格情報は Windows により暗号化され、再表示されません。", clientId: "Discord クライアント ID", botToken: "Discord ボットトークン", connectBot: "暗号化してボットを開始", inviteUrl: "ボット招待 URL", openInvite: "開く", copy: "コピー", copied: "コピーしました",
  routingTitle: "入力ルーティング", routingCopy: "メディア用の Discord チャンネルと、TTS メッセージ用の別チャンネルを選択します。", mediaChannel: "メディアチャンネル", ttsChannel: "TTS メッセージチャンネル", localPort: "ローカルポート", saveRouting: "ルーティングを保存", selectChannel: "利用できるテキストチャンネルを選択", ttsDisabled: "TTS は無効です", unavailableChannel: "チャンネルは利用できません", refreshChannels: "チャンネルを更新", channelsRefreshed: "チャンネルリストを更新しました",
  mediaKicker: "再生キュー", mediaTitle: "あなたのルールで再生されるメディア。", mediaCopy: "画像と GIF には個別のタイマーを使用します。動画と音声は最後まで再生されます。", transportLabel: "ライブ操作", transportReady: "次の項目を再生できます", skip: "現在の項目をスキップ",
  playbackTitle: "再生設定", imageDuration: "画像の表示時間", gifDuration: "GIF の表示時間", imageDurationHelp: "静止画像にのみ適用されます。", gifDurationHelp: "アニメーション GIF は指定時間だけループします。", seconds: "秒", mediaVolume: "メディア音量", widgetSound: "ウィジェットの音声", ttsCharacterLimit: "TTS 文字数制限", characters: "文字", ttsQueueLimit: "TTS キューサイズ", items: "項目", ttsSpeech: "TTS 音声", obsNotifications: "OBS に TTS 通知を表示", obsNotificationOutput: "OBS TTS 通知オーバーレイ", enableObsNotifications: "OBS オーバーレイを有効化", windowsNotificationWidget: "Windows TTS 通知ウィジェット", notificationSound: "通知音", chooseNotificationSound: "音声ファイルを選択", resetNotificationSound: "音を削除", noNotificationSound: "ファイルが選択されていません。", notificationSoundObs: "OBS の通知音", showAuthor: "投稿者を表示", supportedFormats: "画像、GIF、MP4/WebM、および一般的な音声形式に対応しています。", savePlayback: "再生設定を保存",
  overlayKicker: "アプリ出力", overlayTitle: "OBS に届くもの。", overlayCopy: "メディアがキューに入るまで、キャンバスは透明のままです。", livePreview: "ライブプレビュー", transparentCanvas: "透明なキャンバス", browserSource: "OBS ブラウザソース", browserSourceHelp: "各プライベート URL を個別の OBS ブラウザソースとして追加します。", visualSource: "ビジュアルメディア", ttsSource: "TTS 音声", notificationSource: "TTS 通知", audioSource: "音声、音楽、ボイスメッセージ", regenerateSecret: "OBS ソースを再接続", floatingWidget: "フローティングメディアウィジェット", notificationWidget: "TTS 通知ウィジェット", showNotificationWidget: "Windows に表示",
  historyKicker: "最近の 50 件", historyTitle: "メディア履歴", historyCopy: "過去の項目を再生するか、接続中のすべてのオーバーレイを消去します。", clearOverlay: "オーバーレイを消去", historyEmpty: "Discord から最初のメディアを待機しています。", replay: "再生",
  moderationKicker: "配信の安全性", moderationTitle: "OBS に届く内容を決めるのはあなたです。", moderationCopy: "ローカルで承認するまで受信メディアを保留できます。", moderationSettings: "モデレーション設定", enableModeration: "手動モデレーションを有効化", enableModerationHelp: "無効にするとメディアは直接 OBS に流れます。", allowImages: "画像と GIF", allowImagesHelp: "これらの項目を承認キューに入れます。", allowVideos: "動画", allowVideosHelp: "動画ファイルを承認キューに入れます。", allowAudio: "音声", allowAudioHelp: "音声ファイルを承認キューに入れます。", moderationLocalOnly: "判断はローカルにのみ保存され、Discord ユーザーには通知されません。", saveModeration: "モデレーションを保存", pendingMedia: "保留中のメディア", clearPending: "すべて拒否", moderationEmpty: "承認待ちのメディアはありません。", moderationDisabled: "手動モデレーションは無効です。", approve: "承認", reject: "拒否",
  botOffline: "ボットはオフラインです", serverOnline: "サーバーはオンラインです", serverOffline: "サーバーはオフラインです", notConfigured: "未設定", saving: "保存中…", saved: "保存しました", skipped: "現在の項目をスキップしました", showWidget: "ウィジェットを表示", hideWidget: "ウィジェットを非表示", unlockMove: "移動するにはロックを解除", lockDisplay: "表示をロック", unknownAuthor: "不明な投稿者",
  helpKicker: "セットアップガイド", helpTitle: "Discord から OBS へ。", aboutKicker: "このアプリについて", aboutTitle: "Relay", aboutCopy: "ローカルの Discord から OBS へのメディアリレーです。", privacyCardTitle: "プライバシー優先", privacyCardCopy: "Relay はこのデバイスでコンテンツを処理します。", privacyCardLink: "プライバシー設定を見る", personalizationKicker: "ワークスペース", personalizationTitle: "Relay を自分好みに。", themeLabel: "テーマ", accentColor: "アクセントカラー", fontSize: "フォントサイズ", previewTitle: "プレビュー", previewButton: "サンプルボタン", resetDefaults: "初期設定に戻す", personalizationSaved: "パーソナライズ設定を保存しました",
  commandsKicker: "Discord コントロール", commandsTitle: "コマンドを、あなたの管理下に。", commandsCopy: "Discord で使いたい Relay コマンドだけを有効にします。", commandsSettings: "コマンド設定", saveCommands: "コマンドを保存", commandsSaved: "コマンドを保存しました", defaultCommands: "標準コマンド", customCommands: "カスタムコマンド", customCommandsHelp: "あらかじめ定義された Discord 操作を実行するローカルの /relay サブコマンドを作成します。", customCommandsEmpty: "カスタムコマンドはまだありません。", addCustomCommand: "コマンドを追加", customCommandEditor: "コマンドエディター", customCommandName: "コマンド名", customCommandAction: "定義済みの操作", customCommandDescription: "Discord の説明", customCommandEnabled: "このコマンドを Discord に登録", customActionParameters: "操作パラメーター", customAccessRestrictions: "アクセス制限", customAdminOnly: "操作権限に加えて Discord 管理者を要求", customExtraPermissions: "追加で必要な権限", customAllowedUsers: "許可するユーザー ID", customAllowedRoles: "許可するロール ID", customAllowedChannels: "許可する呼び出しチャンネル ID", discordIdsPlaceholder: "1 行に 1 つの Discord ID またはメンション", cancel: "キャンセル", saveCommandDraft: "コマンドを保存", syncCustomCommands: "保存して Discord と同期", edit: "編集", delete: "削除", active: "有効", disabled: "無効",
  customActionBan: "メンバーを追放", customActionUnban: "追放を解除", customActionKick: "メンバーをキック", customActionTimeout: "メンバーをタイムアウト", customActionRemoveTimeout: "タイムアウトを解除", customActionClearMessages: "メッセージを削除", customActionAddRole: "ロールを追加", customActionRemoveRole: "ロールを削除", customActionReply: "返信", parameterRequired: "必須", parameterOptional: "任意", parameterFixed: "ローカルで固定", customReason: "監査ログの理由", customDeleteDays: "最近のメッセージを削除する日数", customDurationMinutes: "タイムアウト時間（分）", customChannelId: "チャンネル", customMessageCount: "メッセージ数", customRoleId: "ロール", customReplyText: "返信内容", customReplyVisibility: "返信の公開範囲", customReplyEphemeral: "呼び出した本人だけに表示", customReplyPublic: "公開返信",
  automaticFilterWords: "自動フィルタリング", automaticFilterWordsHelp: "入力を止めるとフィルター語が自動で保存されます。手動モデレーションとローカルスキャンが無効でも適用されます。", manualModeration: "手動モデレーション", manualModerationHelp: "自動フィルター語とは別にメディアを承認します。", privacyScanEnabled: "ローカルプライバシースキャンを有効化", privacyScanEnabledHelp: "履歴または OBS に届く前に、画像メタデータとローカル OCR を検査します。", privacyProtection: "ドクシング対策", privacyProtectionHelp: "Discord のテキストやメディアが履歴、WebSocket、Windows ウィジェット、OBS に届く前にローカルで検査します。検出値がログにコピーされることはありません。", privacyProtectionLevel: "保護レベル", privacyProfileBalanced: "バランス", privacyProfileStrict: "厳格", privacyProfileParanoid: "高警戒", privacyBlockThreshold: "自動ブロックのしきい値", privacyReviewIntermediate: "中リスクのケースを確認", privacyAutoDeleteBlockedMessages: "ブロックした Discord メッセージを削除", privacyCategories: "有効な検出カテゴリ", privacyCategoryEmail: "メールアドレス", privacyCategoryPhone: "電話番号", privacyCategoryIp: "IP アドレス", privacyCategoryGps: "GPS と座標", privacyCategoryAddress: "住所", privacyCategoryFinancial: "IBAN と決済カード", privacyCategoryPlate: "ナンバープレート", privacyCategoryUrl: "機密 URL", privacyCategoryCustom: "保護対象のプライベート文字列", privacyCategoryMetadata: "EXIF メタデータ", privacyCategoryOcr: "ローカル OCR", privacyCategoryDocument: "行政書類", privacyCustomPatterns: "保護するプライベートデータ", privacyCustomPatternsPlaceholder: "1 行に 1 つの値", privacyAllowlist: "許可リスト", privacyAllowlistPlaceholder: "1 行に 1 つの公開値",
  navigationBack: "戻る", navigationForward: "進む", searchLabel: "Relay 設定を検索", searchPlaceholder: "設定を検索", searchNoResults: "一致する設定はありません", clearSearch: "検索をクリア", fontFamily: "インターフェースフォント", fontDesignDefault: "選択したデザインに合わせる",
});

Object.assign(translations.id, {
  navOverview: "Ringkasan", navMedia: "Media", navOverlay: "Overlay", navModeration: "Moderasi", navHistory: "Riwayat", navHelp: "Bantuan", navPersonalization: "Personalisasi", navCommands: "Perintah", navAbout: "Tentang",
  language: "Bahasa", appearance: "Tampilan", light: "Terang", dark: "Gelap", overlays: "Sumber OBS", system: "Sistem", playback: "Pemutaran", output: "Keluaran", safety: "Keamanan", archive: "Arsip", guide: "Panduan", about: "Tentang Relay",
  overviewKicker: "Siaran lokal", overviewTitle: "Satu channel. Setiap layar.", overviewCopy: "Hubungkan Discord sekali, pilih channel, lalu biarkan Relay berjalan diam-diam di area notifikasi.",
  credentialsTitle: "Hubungkan Discord", credentialsCopy: "Kredensial dienkripsi oleh Windows dan tidak ditampilkan lagi.", clientId: "ID klien Discord", botToken: "Token bot Discord", connectBot: "Enkripsi dan mulai bot", inviteUrl: "URL undangan bot", openInvite: "Buka", copy: "Salin", copied: "Disalin",
  routingTitle: "Perutean masukan", routingCopy: "Pilih satu channel Discord untuk media dan channel lain untuk pesan TTS.", mediaChannel: "Channel media", ttsChannel: "Channel pesan TTS", localPort: "Port lokal", saveRouting: "Simpan perutean", selectChannel: "Pilih channel teks yang tersedia", ttsDisabled: "TTS dinonaktifkan", unavailableChannel: "Channel tidak tersedia", refreshChannels: "Muat ulang channel", channelsRefreshed: "Daftar channel diperbarui",
  mediaKicker: "Antrean pemutaran", mediaTitle: "Media, sesuai aturanmu.", mediaCopy: "Gambar dan GIF menggunakan pengatur waktu terpisah. Video dan audio diputar sampai selesai.", transportLabel: "Kontrol langsung", transportReady: "Siap untuk item berikutnya", skip: "Lewati item saat ini",
  playbackTitle: "Pengaturan pemutaran", imageDuration: "Durasi gambar", gifDuration: "Durasi GIF", imageDurationHelp: "Hanya berlaku untuk gambar statis.", gifDurationHelp: "GIF animasi akan berulang selama waktu yang ditentukan.", seconds: "dtk", mediaVolume: "Volume media", widgetSound: "Suara widget", ttsCharacterLimit: "Batas karakter TTS", characters: "karakter", ttsQueueLimit: "Ukuran antrean TTS", items: "item", ttsSpeech: "Ucapan TTS", obsNotifications: "Tampilkan notifikasi TTS di OBS", obsNotificationOutput: "Overlay notifikasi TTS OBS", enableObsNotifications: "Aktifkan overlay OBS", windowsNotificationWidget: "Widget notifikasi TTS Windows", notificationSound: "Suara notifikasi", chooseNotificationSound: "Pilih file audio", resetNotificationSound: "Hapus suara", noNotificationSound: "Tidak ada file yang dipilih.", notificationSoundObs: "Suara notifikasi di OBS", showAuthor: "Tampilkan penulis", supportedFormats: "Mendukung gambar, GIF, MP4/WebM, dan format audio umum.", savePlayback: "Simpan pemutaran",
  overlayKicker: "Keluaran aplikasi", overlayTitle: "Yang diterima OBS.", overlayCopy: "Kanvas tetap transparan hingga media masuk ke antrean.", livePreview: "Pratinjau langsung", transparentCanvas: "Kanvas transparan", browserSource: "Sumber browser OBS", browserSourceHelp: "Tambahkan setiap URL privat sebagai sumber browser OBS terpisah.", visualSource: "Media visual", ttsSource: "Audio TTS", notificationSource: "Notifikasi TTS", audioSource: "Audio, musik, dan pesan suara", regenerateSecret: "Sambungkan ulang sumber OBS", floatingWidget: "Widget media mengambang", notificationWidget: "Widget notifikasi TTS", showNotificationWidget: "Tampilkan di Windows",
  historyKicker: "50 item terakhir", historyTitle: "Riwayat media", historyCopy: "Putar ulang item sebelumnya atau hapus semua overlay yang terhubung.", clearOverlay: "Hapus overlay", historyEmpty: "Menunggu media pertama dari Discord.", replay: "Putar ulang",
  moderationKicker: "Keamanan siaran", moderationTitle: "Kamu menentukan apa yang mencapai OBS.", moderationCopy: "Tahan media masuk sampai kamu menyetujuinya secara lokal.", moderationSettings: "Pengaturan moderasi", enableModeration: "Aktifkan moderasi manual", enableModerationHelp: "Saat dinonaktifkan, media langsung mengalir ke OBS.", allowImages: "Gambar dan GIF", allowImagesHelp: "Izinkan item ini masuk ke antrean persetujuan.", allowVideos: "Video", allowVideosHelp: "Izinkan file video masuk ke antrean persetujuan.", allowAudio: "Audio", allowAudioHelp: "Izinkan file audio masuk ke antrean persetujuan.", moderationLocalOnly: "Keputusan tetap lokal dan tidak pernah memberi tahu pengguna Discord.", saveModeration: "Simpan moderasi", pendingMedia: "Media tertunda", clearPending: "Tolak semua", moderationEmpty: "Tidak ada media yang menunggu persetujuan.", moderationDisabled: "Moderasi manual dinonaktifkan.", approve: "Setujui", reject: "Tolak",
  botOffline: "Bot offline", serverOnline: "Server online", serverOffline: "Server offline", notConfigured: "Belum dikonfigurasi", saving: "Menyimpan…", saved: "Tersimpan", skipped: "Item saat ini dilewati", showWidget: "Tampilkan widget", hideWidget: "Sembunyikan widget", unlockMove: "Buka kunci untuk memindahkan", lockDisplay: "Kunci tampilan", unknownAuthor: "Penulis tidak dikenal",
  helpKicker: "Panduan penyiapan", helpTitle: "Dari Discord ke OBS.", aboutKicker: "Tentang aplikasi ini", aboutTitle: "Relay", aboutCopy: "Relay media Discord-ke-OBS lokal.", privacyCardTitle: "Privasi lebih dulu", privacyCardCopy: "Relay memproses konten di perangkat ini.", privacyCardLink: "Pelajari kontrol privasi", personalizationKicker: "Ruang kerja", personalizationTitle: "Buat Relay terasa seperti milikmu.", themeLabel: "Tema", accentColor: "Warna aksen", fontSize: "Ukuran huruf", previewTitle: "Pratinjau", previewButton: "Tombol contoh", resetDefaults: "Pulihkan default", personalizationSaved: "Personalisasi disimpan",
  commandsKicker: "Kontrol Discord", commandsTitle: "Perintah, dalam kendalimu.", commandsCopy: "Aktifkan hanya perintah Relay yang ingin tersedia di Discord.", commandsSettings: "Pengaturan perintah", saveCommands: "Simpan perintah", commandsSaved: "Perintah disimpan", defaultCommands: "Perintah bawaan", customCommands: "Perintah kustom", customCommandsHelp: "Buat subperintah /relay lokal yang menjalankan satu tindakan Discord yang telah ditentukan.", customCommandsEmpty: "Belum ada perintah kustom.", addCustomCommand: "Tambah perintah", customCommandEditor: "Editor perintah", customCommandName: "Nama perintah", customCommandAction: "Tindakan yang telah ditentukan", customCommandDescription: "Deskripsi Discord", customCommandEnabled: "Daftarkan perintah ini di Discord", customActionParameters: "Parameter tindakan", customAccessRestrictions: "Pembatasan akses", customAdminOnly: "Memerlukan Administrator Discord selain izin tindakan", customExtraPermissions: "Izin tambahan yang diperlukan", customAllowedUsers: "ID pengguna yang diizinkan", customAllowedRoles: "ID peran yang diizinkan", customAllowedChannels: "ID channel pemanggilan yang diizinkan", discordIdsPlaceholder: "Satu ID Discord atau sebutan per baris", cancel: "Batal", saveCommandDraft: "Simpan perintah", syncCustomCommands: "Simpan dan sinkronkan dengan Discord", edit: "Edit", delete: "Hapus", active: "Aktif", disabled: "Nonaktif",
  customActionBan: "Cekal anggota", customActionUnban: "Batalkan cekal anggota", customActionKick: "Keluarkan anggota", customActionTimeout: "Batasi waktu anggota", customActionRemoveTimeout: "Hapus batas waktu", customActionClearMessages: "Hapus pesan", customActionAddRole: "Tambah peran", customActionRemoveRole: "Hapus peran", customActionReply: "Balas", parameterRequired: "Wajib", parameterOptional: "Opsional", parameterFixed: "Tetap secara lokal", customReason: "Alasan log audit", customDeleteDays: "Hapus pesan terbaru (hari)", customDurationMinutes: "Durasi batas waktu (menit)", customChannelId: "Channel", customMessageCount: "Jumlah pesan", customRoleId: "Peran", customReplyText: "Teks balasan", customReplyVisibility: "Visibilitas balasan", customReplyEphemeral: "Hanya terlihat oleh pemanggil", customReplyPublic: "Balasan publik",
  automaticFilterWords: "Pemfilteran otomatis", automaticFilterWordsHelp: "Kata filter disimpan otomatis setelah kamu berhenti mengetik. Kata ini tetap berlaku saat moderasi manual dan pemindaian lokal dimatikan.", manualModeration: "Moderasi manual", manualModerationHelp: "Tahan media untuk persetujuan secara terpisah dari kata filter otomatis.", privacyScanEnabled: "Aktifkan pemindaian privasi lokal", privacyScanEnabledHelp: "Periksa metadata gambar dan OCR lokal sebelum riwayat atau OBS.", privacyProtection: "Perlindungan anti-doxxing", privacyProtectionHelp: "Pindai secara lokal sebelum teks atau media Discord mencapai riwayat, WebSocket, widget Windows, atau OBS. Nilai yang terdeteksi tidak pernah disalin ke log.", privacyProtectionLevel: "Tingkat perlindungan", privacyProfileBalanced: "Seimbang", privacyProfileStrict: "Ketat", privacyProfileParanoid: "Sangat ketat", privacyBlockThreshold: "Ambang blokir otomatis", privacyReviewIntermediate: "Tinjau kasus risiko sedang", privacyAutoDeleteBlockedMessages: "Hapus pesan Discord yang diblokir", privacyCategories: "Kategori deteksi yang aktif", privacyCategoryEmail: "E-mail", privacyCategoryPhone: "Nomor telepon", privacyCategoryIp: "Alamat IP", privacyCategoryGps: "GPS dan koordinat", privacyCategoryAddress: "Alamat pos", privacyCategoryFinancial: "IBAN dan kartu pembayaran", privacyCategoryPlate: "Pelat nomor", privacyCategoryUrl: "URL sensitif", privacyCategoryCustom: "String privat yang dilindungi", privacyCategoryMetadata: "Metadata EXIF", privacyCategoryOcr: "OCR lokal", privacyCategoryDocument: "Dokumen administratif", privacyCustomPatterns: "Data privat untuk dilindungi", privacyCustomPatternsPlaceholder: "Satu nilai per baris", privacyAllowlist: "Daftar izin", privacyAllowlistPlaceholder: "Satu nilai publik per baris",
  navigationBack: "Kembali", navigationForward: "Maju", searchLabel: "Cari pengaturan Relay", searchPlaceholder: "Cari pengaturan", searchNoResults: "Tidak ada pengaturan yang cocok", clearSearch: "Hapus pencarian", fontFamily: "Font antarmuka", fontDesignDefault: "Sesuaikan dengan desain yang dipilih",
});

Object.assign(translations.ru, {
  youtubeApiKey: "API-ключ YouTube",
  youtubeApiKeyHelp: "Хранится в диспетчере учетных данных Windows и больше не отображается.",
  musicChannel: "Музыкальный канал",
  musicDisabled: "Музыка отключена",
});
Object.assign(translations.zh, {
  youtubeApiKey: "YouTube API 密钥",
  youtubeApiKeyHelp: "存储在 Windows 凭据管理器中，之后不会再次显示。",
  musicChannel: "音乐频道",
  musicDisabled: "已禁用音乐",
});
Object.assign(translations.ko, {
  youtubeApiKey: "YouTube API 키",
  youtubeApiKeyHelp: "Windows 자격 증명 관리자에 저장되며 다시 표시되지 않습니다.",
  musicChannel: "음악 채널",
  musicDisabled: "음악 사용 안 함",
});
Object.assign(translations.ja, {
  youtubeApiKey: "YouTube API キー",
  youtubeApiKeyHelp: "Windows 資格情報マネージャーに保存され、再表示されません。",
  musicChannel: "音楽チャンネル",
  musicDisabled: "音楽は無効です",
});
Object.assign(translations.id, {
  youtubeApiKey: "Kunci API YouTube",
  youtubeApiKeyHelp: "Disimpan di Windows Credential Manager dan tidak pernah ditampilkan lagi.",
  musicChannel: "Channel musik",
  musicDisabled: "Musik dinonaktifkan",
});

for (const dictionary of Object.values(translations)) {
  for (const [key, value] of Object.entries(translations.en)) {
    dictionary[key] ??= value;
  }
}

const regionalTranslations = {
  "en-US": {
    personalizationTitle: "Customize Relay your way.",
    accentColor: "Accent color",
    previewCopy: "Readable text with your chosen accent color.",
    privacyCategoryPhone: "Phone numbers",
    privacyCategoryAddress: "ZIP and postal addresses",
    customActionClearMessages: "Clear messages",
    customAllowedChannels: "Allowed command channel IDs",
  },
  "en-GB": {
    personalizationTitle: "Personalise Relay your way.",
    accentColor: "Accent colour",
    previewCopy: "Readable text with your chosen accent colour.",
    privacyCategoryPhone: "Telephone numbers",
    privacyCategoryAddress: "Postal addresses",
    customActionClearMessages: "Delete messages",
    customAllowedChannels: "Allowed command channel IDs",
  },
  "en-IN": {
    personalizationTitle: "Personalise Relay for your workspace.",
    accentColor: "Accent colour",
    previewCopy: "Readable text with your chosen accent colour.",
    privacyCategoryPhone: "Mobile and telephone numbers",
    privacyCategoryAddress: "Postal addresses and PIN codes",
    customActionClearMessages: "Delete messages",
    customAllowedChannels: "Permitted command channel IDs",
  },
};

const personalizationExtensionTranslations = {
  en: { sidebarLayout: "Sidebar layout", sidebarLayoutFixed: "Fixed · labels and icons", sidebarLayoutCompact: "Reduced · icons only", sidebarLayoutDynamic: "Dynamic · expands on hover", sidebarLayoutHelp: "Keep labels visible, use compact icons, or expand the compact sidebar on hover or keyboard focus.", gridlineDesignCopy: "Structured, clear and technical.", lumenDesignCopy: "Layered, luminous and atmospheric." },
  fr: { sidebarLayout: "Disposition de la barre latérale", sidebarLayoutFixed: "Fixe · libellés et icônes", sidebarLayoutCompact: "Réduite · icônes seules", sidebarLayoutDynamic: "Dynamique · s’agrandit au survol", sidebarLayoutHelp: "Gardez les libellés visibles, utilisez des icônes compactes ou développez la barre latérale au survol ou au focus clavier.", gridlineDesignCopy: "Structuré, clair et technique.", lumenDesignCopy: "En couches, lumineux et atmosphérique." },
  es: { sidebarLayout: "Diseño de la barra lateral", sidebarLayoutFixed: "Fijo · etiquetas e iconos", sidebarLayoutCompact: "Reducido · solo iconos", sidebarLayoutDynamic: "Dinámico · se expande al pasar el cursor", sidebarLayoutHelp: "Mantén las etiquetas visibles, usa iconos compactos o expande la barra lateral al pasar el cursor o con el foco del teclado.", gridlineDesignCopy: "Estructurado, claro y técnico.", lumenDesignCopy: "En capas, luminoso y atmosférico." },
  de: { sidebarLayout: "Layout der Seitenleiste", sidebarLayoutFixed: "Fest · Beschriftungen und Symbole", sidebarLayoutCompact: "Reduziert · nur Symbole", sidebarLayoutDynamic: "Dynamisch · erweitert sich beim Überfahren", sidebarLayoutHelp: "Lassen Sie Beschriftungen sichtbar, verwenden Sie kompakte Symbole oder erweitern Sie die Seitenleiste beim Überfahren oder Tastaturfokus.", gridlineDesignCopy: "Strukturiert, klar und technisch.", lumenDesignCopy: "Mehrschichtig, leuchtend und atmosphärisch." },
  ru: { sidebarLayout: "Макет боковой панели", sidebarLayoutFixed: "Полный · подписи и значки", sidebarLayoutCompact: "Компактный · только значки", sidebarLayoutDynamic: "Динамический · раскрывается при наведении", sidebarLayoutHelp: "Оставьте подписи видимыми, используйте компактные значки или раскрывайте панель при наведении либо фокусе с клавиатуры.", gridlineDesignCopy: "Структурный, ясный и технический.", lumenDesignCopy: "Многослойный, светящийся и атмосферный." },
  zh: { sidebarLayout: "侧边栏布局", sidebarLayoutFixed: "固定 · 标签和图标", sidebarLayoutCompact: "精简 · 仅图标", sidebarLayoutDynamic: "动态 · 悬停时展开", sidebarLayoutHelp: "保留可见标签、使用紧凑图标，或在悬停和键盘焦点时展开侧边栏。", gridlineDesignCopy: "结构化、清晰且技术化。", lumenDesignCopy: "层次分明、明亮且富有氛围。" },
  ko: { sidebarLayout: "사이드바 레이아웃", sidebarLayoutFixed: "고정 · 레이블 및 아이콘", sidebarLayoutCompact: "축소 · 아이콘만", sidebarLayoutDynamic: "동적 · 마우스를 올리면 확장", sidebarLayoutHelp: "레이블을 표시하거나 간결한 아이콘을 사용하고, 마우스를 올리거나 키보드로 초점을 맞추면 사이드바를 확장합니다.", gridlineDesignCopy: "구조적이고 명확하며 기술적입니다.", lumenDesignCopy: "겹겹이 빛나며 분위기 있습니다." },
  ja: { sidebarLayout: "サイドバーのレイアウト", sidebarLayoutFixed: "固定 · ラベルとアイコン", sidebarLayoutCompact: "縮小 · アイコンのみ", sidebarLayoutDynamic: "動的 · ホバーで展開", sidebarLayoutHelp: "ラベルを表示したままにするか、コンパクトなアイコンを使い、ホバーまたはキーボードフォーカスでサイドバーを展開します。", gridlineDesignCopy: "構造的で、明快かつ技術的。", lumenDesignCopy: "重なりがあり、明るく、雰囲気のあるデザイン。" },
  id: { sidebarLayout: "Tata letak bilah samping", sidebarLayoutFixed: "Tetap · label dan ikon", sidebarLayoutCompact: "Ringkas · ikon saja", sidebarLayoutDynamic: "Dinamis · melebar saat diarahkan", sidebarLayoutHelp: "Biarkan label terlihat, gunakan ikon ringkas, atau lebarkan bilah samping saat diarahkan atau mendapat fokus keyboard.", gridlineDesignCopy: "Terstruktur, jelas, dan teknis.", lumenDesignCopy: "Berlapis, bercahaya, dan atmosferik." },
};

for (const [languageCode, extension] of Object.entries(personalizationExtensionTranslations)) {
  Object.assign(translations[languageCode], extension);
}

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
moderationForm.addEventListener("input", () => {
  moderationSaveStateElement.textContent = t("unsaved");
});
const commandsSaveStateElement = $("#commands-save-state");
const channelLockStateElement = $("#channel-lock-state");
const commandInputs = {
  channel: $("#command-channel"), url: $("#command-url"), show: $("#command-show"),
  status: $("#command-status"), test: $("#command-test"), regenerate: $("#command-regenerate"), clear: $("#command-clear"), lock: $("#command-lock"),
  changelog: $("#command-changelog"),
};
const customCommandForm = $("#custom-command-form");
const customCommandListElement = $("#custom-command-list");
const customCommandsEmptyElement = $("#custom-commands-empty");
const customCommandCountElement = $("#custom-command-count");
const customCommandsSaveStateElement = $("#custom-commands-save-state");
const customCommandEditorStateElement = $("#custom-command-editor-state");
const customCommandPreviewElement = $("#custom-command-preview");
const customCommandNameElement = $("#custom-command-name");
const customCommandDescriptionElement = $("#custom-command-description");
const customCommandActionElement = $("#custom-command-action");
const customCommandEnabledElement = $("#custom-command-enabled");
const customActionFieldsElement = $("#custom-action-fields");
const customCommandAdminOnlyElement = $("#custom-command-admin-only");
const customCommandUsersElement = $("#custom-command-users");
const customCommandRolesElement = $("#custom-command-roles");
const customCommandChannelsElement = $("#custom-command-channels");
const customRequiredPermissionsElement = $("#custom-required-permissions");
const customPermissionInputs = $$('input[name="custom-permission"]');
const addCustomCommandButton = $("#add-custom-command");
const cancelCustomCommandButton = $("#cancel-custom-command");
const syncCustomCommandsButton = $("#sync-custom-commands");
const defaultRelayCommandNames = new Set([
  "channel", "url", "show", "status", "test", "regenerate", "clear", "lock", "changelog",
]);
let customCommands = [];
let customCommandsDirty = false;
let editingCustomCommandIndex = null;
const clientIdElement = $("#client-id");
const tokenElement = $("#discord-token");
const youtubeApiKeyElement = $("#youtube-api-key");
const credentialStateElement = $("#credential-state");
const botOnlineStatusElement = $("#bot-online-status");
const botActivityTypeElement = $("#bot-activity-type");
const botActivityTextElement = $("#bot-activity-text");
const botPresenceSaveStateElement = $("#bot-presence-save-state");
const inviteRowElement = $("#invite-row");
const inviteUrlElement = $("#invite-url");
const openInviteButton = $("#open-invite");
const channelElement = $("#channel");
const refreshChannelsButton = $("#refresh-channels");
const ttsChannelElement = $("#tts-channel");
const musicChannelElement = $("#music-channel");
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
const showMediaTextObsElement = $("#show-media-text-obs");
const showMediaTextWidgetElement = $("#show-media-text-widget");
const moderationEnabledElement = $("#moderation-enabled");
const moderationAllowImagesElement = $("#moderation-allow-images");
const moderationAllowVideosElement = $("#moderation-allow-videos");
const moderationAllowAudioElement = $("#moderation-allow-audio");
const privacyScanEnabledElement = $("#privacy-scan-enabled");
const privacyProtectionLevelElement = $("#privacy-protection-level");
const privacyBlockThresholdElement = $("#privacy-block-threshold");
const privacyReviewIntermediateElement = $("#privacy-review-intermediate");
const privacyAutoDeleteBlockedMessagesElement = $("#privacy-auto-delete-blocked-messages");
const privacyCategoryElements = $$('input[name="privacy-category"]');
const privacyCustomPatternsElement = $("#privacy-custom-patterns");
const privacyAllowlistElement = $("#privacy-allowlist");
const privacyConceptsElement = $("#privacy-concepts");
const privacyExemptRoleIdsElement = $("#privacy-exempt-role-ids");
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
const outputReadinessCards = new Map(
  $$('[data-output-card]').map((element) => [element.dataset.outputCard, element]),
);
const outputStateElements = new Map(
  $$('[data-output-state]').map((element) => [element.dataset.outputState, element]),
);
const outputLastConnectedElements = new Map(
  $$('[data-output-last-connected]').map(
    (element) => [element.dataset.outputLastConnected, element],
  ),
);
const outputTestButtons = new Map(
  $$('[data-test-output]').map((element) => [element.dataset.testOutput, element]),
);
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
const interfaceLanguageButton = $("#interface-language-button");
const interfaceLanguageOptionsElement = $("#interface-language-options");
const interfaceLanguageLabelElement = $("#interface-language-label");
const interfaceLanguageFlagElement = $("#interface-language-flag");
const sidebarLanguagePickerElement = $("#sidebar-language-picker");
const sidebarLanguageOptionsElement = $("#sidebar-language-options");
const interfaceThemeElement = $("#interface-theme");
const interfaceFontElement = $("#interface-font");
const sidebarLayoutElement = $("#sidebar-layout");
const sidebarElement = $(".sidebar");
const designPickerElement = $("#design-picker");
const designPickerSelectedElement = $("#design-picker-selected");
const designInputs = $$("input[name='interface-design']");
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
const skipShortcutKeyElement = $("#skip-shortcut-key");
const skipShortcutCaptureButton = $("#skip-shortcut-capture");
const skipShortcutValueElement = $("#skip-shortcut-value");
const languageToggleButton = $("#language-toggle");
const languageValueElement = $("#language-value");
const languageFlagElement = $("#language-flag");
const themeToggleButton = $("#theme-toggle");
const themeValueElement = $("#theme-value");
const pageTitleElement = $("#page-title");
const pageKickerElement = $("#page-kicker");
const navigationBackButton = $("#navigation-back");
const navigationForwardButton = $("#navigation-forward");
const settingsSearchControl = $("#settings-search-control");
const settingsSearchElement = $("#settings-search");
const settingsSearchClearButton = $("#settings-search-clear");
const settingsSearchResultsElement = $("#settings-search-results");
const nowPlayingElement = $("#now-playing");
const nowPlayingArtworkElement = $("#now-playing-artwork");
const nowPlayingTitleElement = $("#now-playing-title");
const nowPlayingArtistElement = $("#now-playing-artist");
const previousAudioButton = $("#previous-audio");
const toggleAudioButton = $("#toggle-audio");
const skipAudioButton = $("#skip-audio");
const pauseAudioIcon = $("#pause-audio-icon");
const playAudioIcon = $("#play-audio-icon");
const updateControlElement = $("#update-control");
const updateCheckButton = $("#update-check");
const updateAvailableDot = $("#update-available-dot");
const updateMenuElement = $("#update-menu");
const updateMenuCloseButton = $("#update-menu-close");
const updateStatusElement = $("#update-status");
const installUpdateButton = $("#install-update");

const history = [];
let bootstrap;
let socket;
let reconnectTimer;
let reconnectDelayMs = 1000;
let statusTimer;
let mediaCaptionSaveGeneration = 0;
let isUnloading = false;
let shortcutCaptureActive = false;
let currentPage = "overview";
const navigationHistory = ["overview"];
let navigationHistoryIndex = 0;
let settingsSearchIndex = [];
let settingsSearchHighlightTimer;
const languageOptions = [
  { locale: "en-US", language: "en", label: "English (US)", short: "EN-US", flag: "us" },
  { locale: "en-GB", language: "en", label: "English (UK)", short: "EN-UK", flag: "gb" },
  { locale: "en-IN", language: "en", label: "English (India)", short: "EN-IN", flag: "in" },
  { locale: "fr-FR", language: "fr", label: "Français", short: "FR", flag: "fr" },
  { locale: "de-DE", language: "de", label: "Deutsch", short: "DE", flag: "de" },
  { locale: "es-ES", language: "es", label: "Español", short: "ES", flag: "es" },
  { locale: "es-419", language: "es", label: "Español (Latinoamérica)", short: "ES-LATAM", flag: "mx" },
  { locale: "ru-RU", language: "ru", label: "Русский", short: "RU", flag: "ru" },
  { locale: "zh-CN", language: "zh", label: "简体中文", short: "ZH", flag: "cn" },
  { locale: "ko-KR", language: "ko", label: "한국어", short: "KO", flag: "kr" },
  { locale: "ja-JP", language: "ja", label: "日本語", short: "JA", flag: "jp" },
  { locale: "id-ID", language: "id", label: "Bahasa Indonesia", short: "ID", flag: "id" },
];
const languageOptionByLocale = new Map(languageOptions.map((option) => [option.locale, option]));
const defaultLocaleByLanguage = {
  en: "en-US", fr: "fr-FR", es: "es-ES", de: "de-DE", ru: "ru-RU",
  zh: "zh-CN", ko: "ko-KR", ja: "ja-JP", id: "id-ID",
};
const supportedDesigns = ["openai", "anthropic", "neo-brutalism", "gridline", "lumen"];
const supportedSidebarLayouts = ["fixed", "compact", "dynamic"];
const supportedInterfaceFonts = [
  "design", "bricolage", "dm-sans", "figtree", "inter",
  "jetbrains-mono", "manrope", "poppins", "space-grotesk",
];
const storedLanguage = localStorage.getItem("relay-language") || "en";
let locale = localStorage.getItem("relay-locale") || defaultLocaleByLanguage[storedLanguage] || "en-US";
if (!languageOptionByLocale.has(locale)) locale = "en-US";
let language = languageOptionByLocale.get(locale).language;
let design = localStorage.getItem("relay-design") || "openai";
if (!supportedDesigns.includes(design)) design = "openai";
let interfaceFont = localStorage.getItem("relay-interface-font") || "design";
if (!supportedInterfaceFonts.includes(interfaceFont)) interfaceFont = "design";
let sidebarLayout = localStorage.getItem("relay-sidebar-layout") || "fixed";
if (!supportedSidebarLayouts.includes(sidebarLayout)) sidebarLayout = "fixed";
let sidebarExpanded = false;
let theme = localStorage.getItem("relay-theme")
  || (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
let accentRgb = parseStoredAccent();
let fontScale = clamp(Number(localStorage.getItem("relay-font-scale")) || 100, 80, 140);
let personalizationTimer;
let privacyFilterSaveTimer;
let privacyFilterSaveGeneration = 0;
let privacyFilterDraft = "";
const audioPlaybackTargets = new Map();
let currentAudioPlayback;
let nowPlayingArtworkUrl;
let currentAppVersion = "1.2.7";
let latestUpdate;
let updateUiState = { kind: "idle" };

function t(key) {
  return regionalTranslations[locale]?.[key] || translations[language][key] || translations.en[key] || key;
}

function applyTranslations(root = document) {
  for (const element of $$("[data-i18n]", root)) {
    element.textContent = t(element.dataset.i18n);
  }
  for (const element of $$("[data-i18n-placeholder]", root)) {
    element.placeholder = t(element.dataset.i18nPlaceholder);
  }
}

function formatTranslation(key, values = {}) {
  return Object.entries(values).reduce(
    (message, [name, value]) => message.replaceAll(`{${name}}`, value),
    t(key),
  );
}

function setAppVersion(version) {
  const normalized = String(version).replace(/^v/, "");
  if (!/^\d+\.\d+\.\d+$/.test(normalized)) return;
  currentAppVersion = normalized;
  for (const element of $$("[data-app-version]")) element.textContent = normalized;
  updateCheckButton.setAttribute("aria-label", `${t("checkUpdates")}. Relay v${normalized}`);
}

function renderUpdateStatus() {
  const version = updateUiState.version || currentAppVersion;
  const messages = {
    idle: () => t("checkUpdatesPrompt"),
    checking: () => t("checkingUpdates"),
    available: () => formatTranslation("updateAvailable", { version }),
    current: () => formatTranslation("upToDate", { version }),
    installing: () => formatTranslation("downloadingUpdate", { version }),
    error: () => `${t(updateUiState.errorKey)} ${updateUiState.error}`,
  };
  updateStatusElement.textContent = (messages[updateUiState.kind] || messages.idle)();
  const busy = updateUiState.kind === "checking" || updateUiState.kind === "installing";
  updateCheckButton.classList.toggle("is-checking", busy);
  updateCheckButton.disabled = busy;
  installUpdateButton.disabled = busy;
  installUpdateButton.hidden = !latestUpdate?.updateAvailable;
  updateAvailableDot.hidden = !latestUpdate?.updateAvailable;
}

function setUpdateMenuOpen(open) {
  updateMenuElement.hidden = !open;
  updateCheckButton.setAttribute("aria-expanded", String(open));
  if (open) window.requestAnimationFrame(() => updateMenuCloseButton.focus());
}

function activeLanguageOption() {
  return languageOptionByLocale.get(locale) || languageOptions[0];
}

function setLanguageMenuOpen(open) {
  interfaceLanguageOptionsElement.hidden = !open;
  interfaceLanguageButton.setAttribute("aria-expanded", String(open));
  if (open) setSidebarLanguageMenuOpen(false);
}

function setSidebarLanguageMenuOpen(open) {
  sidebarLanguageOptionsElement.hidden = !open;
  languageToggleButton.setAttribute("aria-expanded", String(open));
  if (open) setLanguageMenuOpen(false);
}

function renderLanguagePicker() {
  const selected = activeLanguageOption();
  const flagUrl = `./assets/flags/${selected.flag}.svg`;
  interfaceLanguageLabelElement.textContent = selected.label;
  interfaceLanguageFlagElement.src = flagUrl;
  languageFlagElement.src = flagUrl;
  languageValueElement.textContent = selected.short;
  interfaceLanguageButton.setAttribute("aria-label", `${t("language")}: ${selected.label}`);
  languageToggleButton.setAttribute("aria-label", `${t("language")}: ${selected.label}`);
  languageToggleButton.title = selected.label;
  for (const option of $$("[data-locale]", interfaceLanguageOptionsElement)) {
    const isSelected = option.dataset.locale === selected.locale;
    option.setAttribute("aria-selected", String(isSelected));
  }
  sidebarLanguageOptionsElement.replaceChildren(...languageOptions.map((option) => {
    const button = document.createElement("button");
    const isSelected = option.locale === selected.locale;
    button.className = "language-picker__option sidebar-language-picker__option";
    button.type = "button";
    button.dataset.locale = option.locale;
    button.setAttribute("role", "option");
    button.setAttribute("aria-selected", String(isSelected));
    button.innerHTML = `<img class="flag-icon" src="./assets/flags/${option.flag}.svg" alt=""><span>${option.label}</span><i aria-hidden="true">✓</i>`;
    return button;
  }));
}

function selectInterfaceLanguage(nextLocale, focusTarget) {
  const option = languageOptionByLocale.get(nextLocale);
  if (!option) return;
  locale = option.locale;
  language = option.language;
  setLanguageMenuOpen(false);
  setSidebarLanguageMenuOpen(false);
  applyLanguage();
  applyTheme();
  applyPersonalization();
  focusTarget.focus();
}

function applyLanguage() {
  document.documentElement.lang = locale;
  localStorage.setItem("relay-language", language);
  localStorage.setItem("relay-locale", locale);
  applyTranslations();
  renderLanguagePicker();
  updateCheckButton.setAttribute("aria-label", `${t("checkUpdates")}. Relay v${currentAppVersion}`);
  updateMenuCloseButton.setAttribute("aria-label", t("closeUpdateMenu"));
  navigationBackButton.title = t("navigationBack");
  navigationBackButton.setAttribute("aria-label", t("navigationBack"));
  navigationForwardButton.title = t("navigationForward");
  navigationForwardButton.setAttribute("aria-label", t("navigationForward"));
  settingsSearchElement.setAttribute("aria-label", t("searchLabel"));
  settingsSearchClearButton.title = t("clearSearch");
  settingsSearchClearButton.setAttribute("aria-label", t("clearSearch"));
  buildSettingsSearchIndex();
  renderSettingsSearchResults();
  renderUpdateStatus();
  updatePageHeading();
  renderNowPlaying();
  renderCustomCommands();
  if (!customCommandForm.hidden) {
    let action;
    try {
      action = readCustomAction();
    } catch {
      action = defaultCustomAction(customCommandActionElement.value);
    }
    renderCustomActionFields(action);
  }
  if (bootstrap) {
    setBotStatus(bootstrap.bot);
    setServerStatus(bootstrap.server);
    setCredentials(bootstrap.credentials);
    setWidgetState(bootstrap.widget);
    setNotificationWidgetState(bootstrap.notificationWidget);
    populateChannels(channelElement, bootstrap.channels, channelElement.value, t("selectChannel"));
    populateChannels(ttsChannelElement, bootstrap.channels, ttsChannelElement.value, t("ttsDisabled"));
    populateChannels(musicChannelElement, bootstrap.channels, musicChannelElement.value, t("musicDisabled"));
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

function applyDesign() {
  document.documentElement.dataset.design = design;
  localStorage.setItem("relay-design", design);
  for (const input of designInputs) input.checked = input.value === design;
  const selectedDesign = designInputs.find((input) => input.value === design);
  designPickerSelectedElement.textContent = selectedDesign
    ?.closest(".design-choice")
    ?.querySelector(".design-choice__copy strong")
    ?.textContent || design;
  for (const element of $$('[data-relay-base-font-size]')) {
    element.style.removeProperty("font-size");
    delete element.dataset.relayBaseFontSize;
  }
}

function applySidebarLayout() {
  const appliedLayout = sidebarLayout === "dynamic"
    ? (sidebarExpanded ? "fixed" : "compact")
    : sidebarLayout;
  document.documentElement.dataset.sidebarLayout = appliedLayout;
  document.documentElement.dataset.sidebarBehavior = sidebarLayout;
  localStorage.setItem("relay-sidebar-layout", sidebarLayout);
  sidebarLayoutElement.value = sidebarLayout;
}

function setDynamicSidebarExpanded(expanded) {
  const nextExpanded = sidebarLayout === "dynamic" && expanded;
  if (sidebarExpanded === nextExpanded) return;
  sidebarExpanded = nextExpanded;
  applySidebarLayout();
}

function applyInterfaceFont() {
  document.documentElement.dataset.interfaceFont = interfaceFont;
  localStorage.setItem("relay-interface-font", interfaceFont);
  interfaceFontElement.value = interfaceFont;
}

function activeAudioPlayback() {
  const states = [...audioPlaybackTargets.values()];
  return states.find(({ status }) => status === "playing")
    || states.find(({ status }) => status === "paused");
}

function releaseNowPlayingArtwork() {
  if (nowPlayingArtworkUrl) URL.revokeObjectURL(nowPlayingArtworkUrl);
  nowPlayingArtworkUrl = undefined;
}

async function loadNowPlayingArtwork(media) {
  releaseNowPlayingArtwork();
  nowPlayingArtworkElement.onerror = () => {
    nowPlayingArtworkElement.onerror = null;
    nowPlayingArtworkElement.src = "./assets/relay-radar.png";
  };
  nowPlayingArtworkElement.src = media.author?.displayAvatarUrl || "./assets/relay-radar.png";
  if (!media.artworkId) return;
  try {
    const bytes = await invoke("get_media_artwork", { artworkId: media.artworkId });
    if (currentAudioPlayback?.media.artworkId !== media.artworkId) return;
    nowPlayingArtworkUrl = URL.createObjectURL(new Blob([bytes]));
    nowPlayingArtworkElement.src = nowPlayingArtworkUrl;
  } catch {}
}

function renderNowPlaying() {
  const playback = activeAudioPlayback();
  if (!playback) {
    currentAudioPlayback = undefined;
    nowPlayingElement.hidden = true;
    releaseNowPlayingArtwork();
    return;
  }
  const mediaChanged = currentAudioPlayback?.media.url !== playback.media.url;
  currentAudioPlayback = playback;
  nowPlayingElement.hidden = false;
  nowPlayingTitleElement.textContent = playback.media.title || playback.media.filename || "Discord audio";
  nowPlayingArtistElement.textContent = playback.media.artist || playback.media.author?.username || "Discord";
  const paused = playback.status === "paused";
  pauseAudioIcon.hidden = paused;
  playAudioIcon.hidden = !paused;
  const toggleLabel = t(paused ? "resumeAudio" : "pauseAudio");
  toggleAudioButton.title = toggleLabel;
  toggleAudioButton.setAttribute("aria-label", toggleLabel);
  previousAudioButton.title = t("previousAudio");
  previousAudioButton.setAttribute("aria-label", t("previousAudio"));
  skipAudioButton.title = t("skipAudio");
  skipAudioButton.setAttribute("aria-label", t("skipAudio"));
  const audioHistory = history.filter(({ kind }) => kind === "audio");
  const historyIndex = audioHistory.findIndex(({ url }) => url === playback.media.url);
  previousAudioButton.disabled = historyIndex < 0 || historyIndex >= audioHistory.length - 1;
  if (mediaChanged) loadNowPlayingArtwork(playback.media);
}

function updateAudioPlayback(playback) {
  const existing = audioPlaybackTargets.get(playback.target);
  if (playback.status === "idle") {
    if (!existing || existing.media.url === playback.media.url) audioPlaybackTargets.delete(playback.target);
  } else {
    audioPlaybackTargets.delete(playback.target);
    audioPlaybackTargets.set(playback.target, playback);
  }
  renderNowPlaying();
}

async function controlCurrentAudio(action) {
  if (!currentAudioPlayback) return;
  for (const button of [previousAudioButton, toggleAudioButton, skipAudioButton]) button.disabled = true;
  try {
    await invoke("control_audio", { action, currentUrl: currentAudioPlayback.media.url });
  } catch (error) {
    mediaSaveStateElement.textContent = String(error);
  } finally {
    renderNowPlaying();
  }
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
    url.searchParams.set("preview", "1");
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
  const elements = $$('body *:not(svg):not(path)');
  for (const element of elements) element.style.removeProperty("font-size");
  for (const element of elements) {
    if (!element.dataset.relayBaseFontSize) {
      const size = Number.parseFloat(window.getComputedStyle(element).fontSize);
      if (Number.isFinite(size) && size > 0) element.dataset.relayBaseFontSize = String(size);
    }
  }
  for (const element of elements) {
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
  renderLanguagePicker();
  interfaceThemeElement.value = theme;
  accentInputs.forEach((input, index) => { input.value = String(accentRgb[index]); });
  accentPickerElement.value = rgbToHex(accentRgb);
  fontScaleElement.value = String(fontScale);
  fontScaleValueElement.textContent = `${fontScale}%`;
  applyInterfaceFont();
  scaleInterfaceText();
  if (sync) syncInterfacePreferences();
}

function updatePageHeading() {
  const metadata = pageMetadata[currentPage];
  pageTitleElement.textContent = t(metadata.title);
  pageKickerElement.textContent = t(metadata.kicker);
}

function updateNavigationControls() {
  navigationBackButton.disabled = navigationHistoryIndex <= 0;
  navigationForwardButton.disabled = navigationHistoryIndex >= navigationHistory.length - 1;
}

function showPage(page, { recordHistory = true } = {}) {
  if (!pageMetadata[page]) return;
  if (recordHistory && page !== currentPage) {
    navigationHistory.splice(navigationHistoryIndex + 1);
    navigationHistory.push(page);
    navigationHistoryIndex = navigationHistory.length - 1;
  }
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
    window.requestAnimationFrame(loadHistoryVideoThumbnails);
  }
  updatePageHeading();
  updateNavigationControls();
}

function navigateHistory(offset) {
  const nextIndex = navigationHistoryIndex + offset;
  if (nextIndex < 0 || nextIndex >= navigationHistory.length) return;
  navigationHistoryIndex = nextIndex;
  showPage(navigationHistory[navigationHistoryIndex], { recordHistory: false });
}

function normalizeSettingsSearch(value) {
  return String(value)
    .normalize("NFKD")
    .replace(/\p{Mark}/gu, "")
    .toLocaleLowerCase(locale);
}

function buildSettingsSearchIndex() {
  const seen = new Set();
  settingsSearchIndex = $$('[data-page] [data-i18n]').flatMap((element) => {
    const page = element.closest("[data-page]")?.dataset.page;
    const key = element.dataset.i18n;
    const identity = `${page}:${key}`;
    if (!pageMetadata[page] || seen.has(identity)) return [];
    seen.add(identity);
    const label = t(key);
    const pageLabel = t(pageMetadata[page].title);
    const target = element.closest("label, .setting-row, details, fieldset, .panel-section, .help-step") || element;
    return [{
      label, page, pageLabel, target,
      searchable: normalizeSettingsSearch(`${label} ${pageLabel}`),
    }];
  });
}

function closeSettingsSearch() {
  settingsSearchResultsElement.hidden = true;
}

function openSettingsSearchResult(entry) {
  settingsSearchElement.value = "";
  settingsSearchClearButton.hidden = true;
  closeSettingsSearch();
  showPage(entry.page);
  for (let parent = entry.target.closest("details"); parent; parent = parent.parentElement?.closest("details")) {
    parent.open = true;
  }
  window.requestAnimationFrame(() => {
    entry.target.scrollIntoView({ behavior: "smooth", block: "center" });
    window.clearTimeout(settingsSearchHighlightTimer);
    entry.target.classList.remove("settings-search-target");
    window.requestAnimationFrame(() => entry.target.classList.add("settings-search-target"));
    settingsSearchHighlightTimer = window.setTimeout(
      () => entry.target.classList.remove("settings-search-target"),
      1400,
    );
  });
}

function renderSettingsSearchResults() {
  const query = normalizeSettingsSearch(settingsSearchElement.value.trim());
  settingsSearchClearButton.hidden = !query;
  settingsSearchResultsElement.replaceChildren();
  if (!query) {
    closeSettingsSearch();
    return;
  }
  const terms = query.split(/\s+/).filter(Boolean);
  const results = settingsSearchIndex
    .filter((entry) => terms.every((term) => entry.searchable.includes(term)))
    .sort((left, right) => {
      const leftStarts = normalizeSettingsSearch(left.label).startsWith(query);
      const rightStarts = normalizeSettingsSearch(right.label).startsWith(query);
      return Number(rightStarts) - Number(leftStarts) || left.label.localeCompare(right.label, locale);
    })
    .slice(0, 8);
  settingsSearchResultsElement.hidden = false;
  if (!results.length) {
    const empty = document.createElement("p");
    empty.className = "settings-search__empty";
    empty.textContent = t("searchNoResults");
    settingsSearchResultsElement.append(empty);
    return;
  }
  for (const entry of results) {
    const button = document.createElement("button");
    button.className = "settings-search__result";
    button.type = "button";
    button.setAttribute("role", "option");
    const label = document.createElement("strong");
    label.textContent = entry.label;
    const page = document.createElement("small");
    page.textContent = entry.pageLabel;
    button.append(label, page);
    button.addEventListener("click", () => openSettingsSearchResult(entry));
    settingsSearchResultsElement.append(button);
  }
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

function outputClientCount(value) {
  return Math.max(0, Number(value) || 0);
}

function formatOutputLastConnected(timestamp) {
  const value = Number(timestamp);
  if (!Number.isFinite(value) || value <= 0) {
    return t("outputNeverConnected");
  }
  const formattingLocale = typeof locale === "string" ? locale : language;
  return `${t("outputLastConnected")} ${new Date(value).toLocaleTimeString(formattingLocale, {
    hour: "2-digit", minute: "2-digit", second: "2-digit",
  })}`;
}

function renderOutputReadiness(status = {}) {
  const outputs = status.outputs || {};
  for (const target of ["visual", "audio", "tts", "notification", "sticker"]) {
    const output = outputs[target] || {};
    const obsClients = outputClientCount(output.obsClients);
    const previewClients = outputClientCount(output.previewClients);
    const widgetClients = outputClientCount(output.widgetClients);
    const clients = [
      [t("outputObs"), obsClients],
      [t("outputPreview"), previewClients],
      [t("outputWidget"), widgetClients],
    ].filter(([, count]) => count > 0);
    const liveOutputConnected = obsClients + widgetClients > 0;
    const stateElement = outputStateElements.get(target);
    const lastConnectedElement = outputLastConnectedElements.get(target);
    const card = outputReadinessCards.get(target);
    const testButton = outputTestButtons.get(target);

    if (stateElement) {
      stateElement.textContent = clients.length > 0
        ? clients.map(([name, count]) => `${name} ${count}`).join(" · ")
        : t("outputDisconnected");
    }
    if (lastConnectedElement) {
      lastConnectedElement.textContent = formatOutputLastConnected(output.lastConnectedAt);
    }
    if (card) {
      card.classList.toggle("is-live", liveOutputConnected);
      card.classList.toggle("is-preview-only", !liveOutputConnected && previewClients > 0);
    }
    if (testButton) {
      testButton.disabled = !liveOutputConnected;
      testButton.title = liveOutputConnected ? "" : t("outputTestNeedsLiveOutput");
    }
  }
}

function setServerStatus(status) {
  serverStatusElement.classList.toggle("is-online", status.connected);
  serverLabelElement.textContent = status.connected ? t("serverOnline") : status.error || t("serverOffline");
  clientCountElement.textContent = String(status.overlayClients || 0);
  renderOutputReadiness(status);
  if (!status.connected) {
    audioPlaybackTargets.clear();
    renderNowPlaying();
  }
}

function filterWordKey(value) {
  return String(value || "")
    .normalize("NFKC")
    .toLocaleLowerCase()
    .trim()
    .replace(/[\s._-]+/g, "");
}

function filterConceptsToLines(concepts) {
  return (Array.isArray(concepts) ? concepts : [])
    .map((concept) => typeof concept?.canonical === "string" ? concept.canonical.trim() : "")
    .filter(Boolean)
    .join(", ");
}

function filterRoleIdsToInput(roleIds) {
  return (Array.isArray(roleIds) ? roleIds : [])
    .filter((roleId) => typeof roleId === "string" && /^\d{17,20}$/.test(roleId))
    .join(", ");
}

function filterRoleIds(value) {
  const seen = new Set();
  return String(value || "")
    .split(/[\r\n,]+/)
    .map((entry) => entry.trim())
    .map((entry) => entry.match(/^<@&(\d{17,20})>$/)?.[1] || entry)
    .reduce((roleIds, entry) => {
      if (entry && !seen.has(entry)) {
        seen.add(entry);
        roleIds.push(entry);
      }
      return roleIds;
    }, []);
}

function filterWordsToConcepts(value, existingConcepts) {
  const existing = new Map(
    (Array.isArray(existingConcepts) ? existingConcepts : [])
      .filter((concept) => concept && typeof concept.canonical === "string")
      .map((concept) => [filterWordKey(concept.canonical), concept]),
  );
  const seen = new Set();
  const words = String(value || "")
    .split(/[\r\n,]+/)
    .map((word) => word.trim())
    .filter(Boolean);
  return words.reduce((concepts, word) => {
    const key = filterWordKey(word);
    if (!key || seen.has(key)) {
      return concepts;
    }
    seen.add(key);
    const previous = existing.get(key);
    concepts.push({
      canonical: word,
      aliases: Array.isArray(previous?.aliases)
        ? previous.aliases.filter((alias) => typeof alias === "string")
        : [],
      regexes: Array.isArray(previous?.regexes)
        ? previous.regexes.filter((pattern) => typeof pattern === "string")
        : [],
    });
    return concepts;
  }, []);
}

function filterWordsAreSaveable(value) {
  return String(value || "")
    .split(/[\r\n,]+/)
    .map((word) => word.trim())
    .filter(Boolean)
    .every((word) => {
      const normalized = filterWordKey(word);
      return /\p{L}/u.test(word)
        && normalized.length >= 3
        && normalized.length <= 64;
    });
}

function privacyListToInput(values) {
  return (Array.isArray(values) ? values : [])
    .filter((value) => typeof value === "string" && value.trim())
    .join("\n");
}

function privacyListFromInput(value) {
  const seen = new Set();
  return String(value || "")
    .split(/[\r\n]+/)
    .map((entry) => entry.trim())
    .filter((entry) => {
      const key = entry.normalize("NFKC").toLocaleLowerCase();
      if (!entry || seen.has(key)) {
        return false;
      }
      seen.add(key);
      return true;
    });
}

function cloneCustomCommands(commands) {
  return JSON.parse(JSON.stringify(Array.isArray(commands) ? commands : []));
}

function defaultCustomAction(type) {
  const reason = () => ({ mode: "optional", fixedValue: "" });
  const entity = () => ({ mode: "required", fixedValue: "" });
  switch (type) {
    case "ban": return { type, reason: reason(), deleteMessageDays: { mode: "fixed", fixedValue: 0 } };
    case "unban": return { type, reason: reason() };
    case "kick": return { type, reason: reason() };
    case "timeout": return { type, durationMinutes: { mode: "fixed", fixedValue: 60 }, reason: reason() };
    case "removeTimeout": return { type, reason: reason() };
    case "clearMessages": return {
      type,
      channel: { mode: "optional", fixedValue: "" },
      count: { mode: "fixed", fixedValue: 10 },
    };
    case "addRole": return { type, role: entity(), reason: reason() };
    case "removeRole": return { type, role: entity(), reason: reason() };
    case "reply": return { type, text: "Relay", ephemeral: true };
    default: return defaultCustomAction("ban");
  }
}

function customActionTranslationKey(type) {
  return {
    ban: "customActionBan", unban: "customActionUnban", kick: "customActionKick",
    timeout: "customActionTimeout", removeTimeout: "customActionRemoveTimeout",
    clearMessages: "customActionClearMessages", addRole: "customActionAddRole",
    removeRole: "customActionRemoveRole", reply: "customActionReply",
  }[type] || "customActionReply";
}

function customActionPermissionKey(type) {
  return {
    ban: "permissionBanMembers", unban: "permissionBanMembers", kick: "permissionKickMembers",
    timeout: "permissionModerateMembers", removeTimeout: "permissionModerateMembers",
    clearMessages: "permissionManageMessages", addRole: "permissionManageRoles",
    removeRole: "permissionManageRoles",
  }[type];
}

function customParameterMarkup(key, labelKey, kind, minimum = "", maximum = "") {
  const inputAttributes = kind === "integer"
    ? `type="number" min="${minimum}" max="${maximum}" step="1"`
    : `type="text" maxlength="512" autocomplete="off" spellcheck="false"`;
  return `
    <div class="custom-parameter" data-custom-parameter="${key}" data-kind="${kind}" data-min="${minimum}" data-max="${maximum}">
      <strong>${t(labelKey)}</strong>
      <label class="field">
        <span>${t("customParameterMode")}</span>
        <select data-custom-parameter-mode>
          <option value="required">${t("parameterRequired")}</option>
          <option value="optional">${t("parameterOptional")}</option>
          <option value="fixed">${t("parameterFixed")}</option>
        </select>
      </label>
      <label class="field">
        <span>${t("customParameterValue")}</span>
        <input data-custom-parameter-value ${inputAttributes}>
      </label>
    </div>`;
}

function setCustomParameterValue(key, parameter) {
  const root = customActionFieldsElement.querySelector(`[data-custom-parameter="${key}"]`);
  if (!root) return;
  root.querySelector("[data-custom-parameter-mode]").value = parameter?.mode || "optional";
  root.querySelector("[data-custom-parameter-value]").value = String(parameter?.fixedValue ?? "");
}

function updateCustomParameterAvailability(root = customActionFieldsElement) {
  const parameters = root.matches?.("[data-custom-parameter]")
    ? [root]
    : $$('[data-custom-parameter]', root);
  for (const parameter of parameters) {
    const mode = parameter.querySelector("[data-custom-parameter-mode]").value;
    const value = parameter.querySelector("[data-custom-parameter-value]");
    value.disabled = mode === "required";
    value.required = mode === "fixed" || (mode === "optional" && parameter.dataset.kind === "entity-role");
  }
}

function renderCustomRequiredPermissions() {
  const permissionKey = customActionPermissionKey(customCommandActionElement.value);
  const permission = permissionKey ? t(permissionKey) : "—";
  customRequiredPermissionsElement.textContent = formatTranslation("customRequiredPermission", { permission });
}

function renderCustomActionFields(action = defaultCustomAction(customCommandActionElement.value)) {
  const type = customCommandActionElement.value;
  if (action.type !== type) action = defaultCustomAction(type);
  switch (type) {
    case "ban":
      customActionFieldsElement.innerHTML = customParameterMarkup("reason", "customReason", "text")
        + customParameterMarkup("deleteMessageDays", "customDeleteDays", "integer", 0, 7);
      setCustomParameterValue("reason", action.reason);
      setCustomParameterValue("deleteMessageDays", action.deleteMessageDays);
      break;
    case "unban":
    case "kick":
    case "removeTimeout":
      customActionFieldsElement.innerHTML = customParameterMarkup("reason", "customReason", "text");
      setCustomParameterValue("reason", action.reason);
      break;
    case "timeout":
      customActionFieldsElement.innerHTML = customParameterMarkup("durationMinutes", "customDurationMinutes", "integer", 1, 40320)
        + customParameterMarkup("reason", "customReason", "text");
      setCustomParameterValue("durationMinutes", action.durationMinutes);
      setCustomParameterValue("reason", action.reason);
      break;
    case "clearMessages":
      customActionFieldsElement.innerHTML = customParameterMarkup("channel", "customChannelId", "entity-channel")
        + customParameterMarkup("count", "customMessageCount", "integer", 1, 1000);
      setCustomParameterValue("channel", action.channel);
      setCustomParameterValue("count", action.count);
      break;
    case "addRole":
    case "removeRole":
      customActionFieldsElement.innerHTML = customParameterMarkup("role", "customRoleId", "entity-role")
        + customParameterMarkup("reason", "customReason", "text");
      setCustomParameterValue("role", action.role);
      setCustomParameterValue("reason", action.reason);
      break;
    case "reply":
      customActionFieldsElement.innerHTML = `
        <label class="field field--full">
          <span>${t("customReplyText")}</span>
          <textarea id="custom-reply-text" minlength="1" maxlength="1900" required></textarea>
        </label>
        <label class="field field--full">
          <span>${t("customReplyVisibility")}</span>
          <select id="custom-reply-visibility">
            <option value="ephemeral">${t("customReplyEphemeral")}</option>
            <option value="public">${t("customReplyPublic")}</option>
          </select>
        </label>`;
      $("#custom-reply-text").value = action.text || "";
      $("#custom-reply-visibility").value = action.ephemeral === false ? "public" : "ephemeral";
      break;
  }
  updateCustomParameterAvailability();
  renderCustomRequiredPermissions();
}

function normalizeDiscordId(value) {
  const match = String(value || "").trim().match(/^(?:\d{17,20}|<@!?(\d{17,20})>|<@&(\d{17,20})>|<#(\d{17,20})>)$/);
  if (!match) return null;
  return match[1] || match[2] || match[3] || match[0];
}

function discordIdListFromInput(value) {
  const tokens = String(value || "").split(/[\s,]+/).filter(Boolean);
  const ids = tokens.map(normalizeDiscordId);
  if (ids.some((id) => !id) || ids.length > 100) throw new Error(t("customInvalidIds"));
  return [...new Set(ids)];
}

function readCustomParameter(key) {
  const root = customActionFieldsElement.querySelector(`[data-custom-parameter="${key}"]`);
  const mode = root.querySelector("[data-custom-parameter-mode]").value;
  const input = root.querySelector("[data-custom-parameter-value]");
  let fixedValue = input.value.trim();
  if (root.dataset.kind === "integer") {
    fixedValue = Number(fixedValue || root.dataset.min || 0);
    if (mode !== "required"
      && (!Number.isInteger(fixedValue)
        || fixedValue < Number(root.dataset.min)
        || fixedValue > Number(root.dataset.max))) {
      input.setCustomValidity(t("customParameterValue"));
      input.reportValidity();
      input.setCustomValidity("");
      throw new Error(t("customParameterValue"));
    }
  } else if (root.dataset.kind.startsWith("entity") && fixedValue) {
    const id = normalizeDiscordId(fixedValue);
    if (!id) throw new Error(t("customInvalidIds"));
    fixedValue = id;
  }
  if (root.dataset.kind === "entity-role" && mode !== "required" && !fixedValue) {
    throw new Error(t("customInvalidIds"));
  }
  if (root.dataset.kind === "entity-channel" && mode === "fixed" && !fixedValue) {
    throw new Error(t("customInvalidIds"));
  }
  return { mode, fixedValue };
}

function readCustomAction() {
  const type = customCommandActionElement.value;
  switch (type) {
    case "ban": return { type, reason: readCustomParameter("reason"), deleteMessageDays: readCustomParameter("deleteMessageDays") };
    case "unban": return { type, reason: readCustomParameter("reason") };
    case "kick": return { type, reason: readCustomParameter("reason") };
    case "timeout": return { type, durationMinutes: readCustomParameter("durationMinutes"), reason: readCustomParameter("reason") };
    case "removeTimeout": return { type, reason: readCustomParameter("reason") };
    case "clearMessages": return { type, channel: readCustomParameter("channel"), count: readCustomParameter("count") };
    case "addRole": return { type, role: readCustomParameter("role"), reason: readCustomParameter("reason") };
    case "removeRole": return { type, role: readCustomParameter("role"), reason: readCustomParameter("reason") };
    case "reply": return {
      type,
      text: $("#custom-reply-text").value,
      ephemeral: $("#custom-reply-visibility").value !== "public",
    };
    default: throw new Error(t("customCommandAction"));
  }
}

function collectCustomCommandDraft() {
  if (!customCommandForm.reportValidity()) return null;
  const name = customCommandNameElement.value.trim().toLowerCase();
  if (defaultRelayCommandNames.has(name)
    || customCommands.some((command, index) => command.name === name && index !== editingCustomCommandIndex)) {
    throw new Error(t("customDuplicateName"));
  }
  return {
    name,
    description: customCommandDescriptionElement.value.trim(),
    enabled: customCommandEnabledElement.checked,
    action: readCustomAction(),
    access: {
      administratorOnly: customCommandAdminOnlyElement.checked,
      requiredPermissions: customPermissionInputs.filter((input) => input.checked).map((input) => input.value),
      allowedUserIds: discordIdListFromInput(customCommandUsersElement.value),
      allowedRoleIds: discordIdListFromInput(customCommandRolesElement.value),
      allowedChannelIds: discordIdListFromInput(customCommandChannelsElement.value),
    },
  };
}

function renderCustomCommands() {
  customCommandListElement.replaceChildren();
  customCommandCountElement.textContent = `${customCommands.length} / 16`;
  customCommandsEmptyElement.hidden = customCommands.length !== 0;
  addCustomCommandButton.disabled = customCommands.length >= 16;
  for (const [index, command] of customCommands.entries()) {
    const item = document.createElement("li");
    item.className = "custom-command-card";
    const identity = document.createElement("div");
    identity.className = "custom-command-card__identity";
    const title = document.createElement("span");
    const code = document.createElement("strong");
    code.className = "command-code";
    code.textContent = `/relay ${command.name}`;
    const badge = document.createElement("span");
    badge.className = "custom-command-card__badge";
    badge.dataset.active = String(command.enabled !== false);
    badge.textContent = t(command.enabled !== false ? "active" : "disabled");
    title.append(code, badge);
    const details = document.createElement("small");
    details.textContent = `${t(customActionTranslationKey(command.action?.type))} · ${command.description}`;
    identity.append(title, details);
    const actions = document.createElement("div");
    actions.className = "custom-command-card__actions";
    for (const [action, key] of [["edit", "edit"], ["delete", "delete"]]) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "button button--quiet";
      button.dataset.customCommandAction = action;
      button.dataset.customCommandIndex = String(index);
      button.textContent = t(key);
      actions.append(button);
    }
    item.append(identity, actions);
    customCommandListElement.append(item);
  }
}

function closeCustomCommandEditor() {
  customCommandForm.hidden = true;
  editingCustomCommandIndex = null;
  customCommandEditorStateElement.textContent = "";
  syncCustomCommandsButton.disabled = false;
}

function openCustomCommandEditor(index = null) {
  if (index === null && customCommands.length >= 16) {
    customCommandsSaveStateElement.textContent = t("customMaxReached");
    return;
  }
  editingCustomCommandIndex = index;
  const definition = index === null ? {
    name: "",
    description: "",
    enabled: true,
    action: defaultCustomAction("ban"),
    access: { administratorOnly: true, requiredPermissions: [], allowedUserIds: [], allowedRoleIds: [], allowedChannelIds: [] },
  } : cloneCustomCommands([customCommands[index]])[0];
  customCommandNameElement.value = definition.name;
  customCommandDescriptionElement.value = definition.description;
  customCommandEnabledElement.checked = definition.enabled !== false;
  customCommandActionElement.value = definition.action.type;
  customCommandAdminOnlyElement.checked = definition.access?.administratorOnly !== false;
  const requiredPermissions = new Set(definition.access?.requiredPermissions || []);
  for (const input of customPermissionInputs) input.checked = requiredPermissions.has(input.value);
  customCommandUsersElement.value = (definition.access?.allowedUserIds || []).join("\n");
  customCommandRolesElement.value = (definition.access?.allowedRoleIds || []).join("\n");
  customCommandChannelsElement.value = (definition.access?.allowedChannelIds || []).join("\n");
  customCommandPreviewElement.textContent = `/relay ${definition.name || "command"}`;
  renderCustomActionFields(definition.action);
  customCommandForm.hidden = false;
  syncCustomCommandsButton.disabled = true;
  customCommandNameElement.focus();
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
  showMediaTextObsElement.checked = Boolean(config.showMediaTextObs);
  showMediaTextWidgetElement.checked = Boolean(config.showMediaTextWidget);
  moderationEnabledElement.checked = Boolean(config.moderationEnabled);
  moderationAllowImagesElement.checked = config.moderationAllowImages !== false;
  moderationAllowVideosElement.checked = config.moderationAllowVideos !== false;
  moderationAllowAudioElement.checked = config.moderationAllowAudio !== false;
  privacyScanEnabledElement.checked = Boolean(config.privacyScanEnabled);
  privacyProtectionLevelElement.value = config.privacyProtectionLevel || "balanced";
  privacyBlockThresholdElement.value = config.privacyBlockThreshold || "high";
  privacyReviewIntermediateElement.checked = config.privacyReviewIntermediate !== false;
  privacyAutoDeleteBlockedMessagesElement.checked = config.privacyAutoDeleteBlockedMessages !== false;
  const enabledCategories = new Set(
    config.privacyEnabledCategories || privacyCategoryElements.map((input) => input.value),
  );
  for (const input of privacyCategoryElements) {
    input.checked = enabledCategories.has(input.value);
  }
  privacyCustomPatternsElement.value = privacyListToInput(config.privacyCustomPatterns);
  privacyAllowlistElement.value = privacyListToInput(config.privacyAllowlist);
  privacyConceptsElement.value = filterConceptsToLines(config.privacyConcepts);
  privacyExemptRoleIdsElement.value = filterRoleIdsToInput(config.privacyFilterExemptRoleIds);
  channelElement.value = config.watchedChannelId;
  if (channelElement.value !== config.watchedChannelId) {
    populateChannels(channelElement, bootstrap?.channels || [], config.watchedChannelId, t("selectChannel"));
  }
  ttsChannelElement.value = config.ttsChannelId || "";
  if (ttsChannelElement.value !== (config.ttsChannelId || "")) {
    populateChannels(ttsChannelElement, bootstrap?.channels || [], config.ttsChannelId, t("ttsDisabled"));
  }
  musicChannelElement.value = config.musicChannelId || "";
  if (musicChannelElement.value !== (config.musicChannelId || "")) {
    populateChannels(musicChannelElement, bootstrap?.channels || [], config.musicChannelId, t("musicDisabled"));
  }
  commandInputs.channel.checked = config.commandChannelEnabled !== false;
  commandInputs.url.checked = config.commandUrlEnabled !== false;
  commandInputs.show.checked = config.commandShowEnabled !== false;
  commandInputs.status.checked = config.commandStatusEnabled !== false;
  commandInputs.test.checked = config.commandTestEnabled !== false;
  commandInputs.regenerate.checked = config.commandRegenerateEnabled !== false;
  commandInputs.clear.checked = config.commandClearEnabled !== false;
  commandInputs.lock.checked = config.commandLockEnabled !== false;
  commandInputs.changelog.checked = config.commandChangelogEnabled !== false;
  commandInputs.lock.disabled = Boolean(config.channelLock);
  channelLockStateElement.dataset.i18n = config.channelLock ? "commandLockActive" : "commandLockInactive";
  channelLockStateElement.textContent = t(channelLockStateElement.dataset.i18n);
  if (!customCommandsDirty) {
    customCommands = cloneCustomCommands(config.customCommands);
    renderCustomCommands();
  }
  applyOutputGeometryConfig(config);
  updateSkipShortcutDisplay(config.skipShortcut);
}

function setCredentials(status) {
  credentialStateElement.textContent = status.configured
    ? `${t("savedVia")} ${status.source}`
    : t("notConfigured");
  clientIdElement.value = status.clientId || "";
  tokenElement.value = "";
  youtubeApiKeyElement.value = "";
}

function formatShortcutLabel(shortcut) {
  return String(shortcut || "control+alt+KeyS")
    .split("+")
    .map((token) => {
      const normalized = token.trim();
      const lower = normalized.toLowerCase();
      if (lower === "control" || lower === "ctrl") return "Ctrl";
      if (lower === "alt" || lower === "option") return "Alt";
      if (lower === "shift") return "Shift";
      if (lower === "super" || lower === "command" || lower === "cmd") return "Win";
      if (/^key[a-z]$/i.test(normalized)) return normalized.slice(-1).toUpperCase();
      if (/^digit\d$/i.test(normalized)) return normalized.slice(-1);
      return normalized
        .replace(/^Arrow/, "")
        .replace(/^Numpad/, "Num ");
    })
    .join(" ");
}

function updateSkipShortcutDisplay(shortcut) {
  const label = formatShortcutLabel(shortcut);
  skipShortcutKeyElement.textContent = label;
  skipShortcutValueElement.textContent = label;
}

function shortcutTokenFromEvent(event) {
  const modifierCodes = new Set(["ControlLeft", "ControlRight", "AltLeft", "AltRight", "ShiftLeft", "ShiftRight", "MetaLeft", "MetaRight"]);
  if (modifierCodes.has(event.code)) return "";
  const supportedCode = /^(Key[A-Z]|Digit\d|F(?:[1-9]|1\d|2[0-4])|Arrow(?:Up|Down|Left|Right)|Numpad(?:\d|Add|Subtract|Multiply|Divide|Decimal|Enter|Equal)|(?:Backquote|Backslash|BracketLeft|BracketRight|Comma|Equal|Minus|Period|Quote|Semicolon|Slash|Backspace|CapsLock|Delete|End|Enter|Escape|Home|Insert|PageDown|PageUp|Pause|PrintScreen|ScrollLock|Space|Tab))$/;
  return supportedCode.test(event.code) ? event.code : "";
}

function beginShortcutCapture() {
  shortcutCaptureActive = true;
  skipShortcutCaptureButton.setAttribute("aria-pressed", "true");
  skipShortcutValueElement.textContent = t("pressShortcut");
  skipShortcutCaptureButton.focus();
}

function cancelShortcutCapture() {
  shortcutCaptureActive = false;
  skipShortcutCaptureButton.setAttribute("aria-pressed", "false");
  updateSkipShortcutDisplay(bootstrap?.config?.skipShortcut);
}

async function saveCapturedShortcut(shortcut) {
  const previousLabel = formatShortcutLabel(bootstrap?.config?.skipShortcut);
  shortcutCaptureActive = false;
  skipShortcutCaptureButton.setAttribute("aria-pressed", "false");
  skipShortcutValueElement.textContent = formatShortcutLabel(shortcut);
  mediaSaveStateElement.textContent = t("saving");
  try {
    const config = await invoke("set_skip_shortcut", { shortcut });
    bootstrap.config = config;
    updateSkipShortcutDisplay(config.skipShortcut);
    mediaSaveStateElement.textContent = t("shortcutSaved");
  } catch (error) {
    skipShortcutValueElement.textContent = previousLabel;
    mediaSaveStateElement.textContent = String(error) || t("shortcutInvalid");
  }
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
    time.textContent = new Date(mediaEvent.timestamp).toLocaleTimeString(locale, {
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
    const downloadButton = item.querySelector(".history-item__download");
    downloadButton.textContent = t("download");
    downloadButton.addEventListener("click", async () => {
      downloadButton.disabled = true;
      saveStateElement.textContent = t("downloading");
      try {
        const saved = await invoke("download_history_media", {
          messageId: mediaEvent.messageId,
          mediaUrl: mediaEvent.url,
        });
        saveStateElement.textContent = saved ? t("downloaded") : t("downloadCanceled");
      } catch (error) {
        saveStateElement.textContent = String(error);
      } finally {
        downloadButton.disabled = false;
      }
    });
    historyListElement.append(item);
  }
  loadHistoryVideoThumbnails();
}

function isVideoThumbnail(kind, contentType) {
  return kind === "video" || (kind === "gif" && contentType?.startsWith("video/"));
}

function loadHistoryVideoThumbnails() {
  for (const video of historyListElement.querySelectorAll("video[data-thumbnail-source]")) {
    if (!video.src) {
      video.src = video.dataset.thumbnailSource;
      video.load();
    }
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
  if (isVideoThumbnail(kind, mediaEvent.contentType) && source) {
    const video = document.createElement("video");
    video.className = thumbnail.className;
    video.poster = "./assets/relay-radar.png";
    video.preload = "none";
    video.muted = true;
    video.playsInline = true;
    video.dataset.thumbnailSource = source;
    video.addEventListener("loadeddata", () => {
      const time = Number.isFinite(video.duration) ? Math.min(0.1, video.duration / 2) : 0;
      if (time > 0) {
        video.addEventListener("seeked", () => video.pause(), { once: true });
        video.currentTime = time;
      } else {
        video.pause();
      }
    }, { once: true });
    video.addEventListener("error", () => {
      video.removeAttribute("src");
      video.poster = "./assets/relay-radar.png";
    }, { once: true });
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
  const config = bootstrap?.config;
  const filterWordsActive = Array.isArray(config?.privacyConcepts)
    && config.privacyConcepts.length > 0;
  const privacyReviewQueue = config?.privacyScanEnabled || filterWordsActive;
  moderationEmptyElement.textContent = t(
    config?.moderationEnabled
      ? "moderationEmpty"
      : privacyReviewQueue
        ? "privacyReviewQueueEmpty"
        : "moderationDisabled",
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
    const classification = pendingItem.privacyClassification
      || t("privacyPendingManual");
    const categories = Array.isArray(pendingItem.privacyCategories)
      ? pendingItem.privacyCategories.map((category) => String(category)
        .replace(/([A-Z])/g, " $1")
        .trim()
        .toUpperCase())
      : [];
    const detected = categories.length > 0
      ? categories.join(" + ")
      : (pendingItem.privacyReason || "manual_review").toUpperCase();
    item.querySelector(".moderation-item__privacy").textContent = `${String(classification).toUpperCase()} · ${detected}`;
    const time = item.querySelector(".history-item__time");
    time.dateTime = new Date(mediaEvent.timestamp).toISOString();
    time.textContent = new Date(mediaEvent.timestamp).toLocaleTimeString(locale, {
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
  } else if (message.type === "audioPlayback") {
    if (message.payload?.media?.kind === "audio") updateAudioPlayback(message.payload);
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
  populateChannels(musicChannelElement, bootstrap.channels, bootstrap.config.musicChannelId, t("musicDisabled"));
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
  const previewUrl = new URL(bootstrap.overlayUrl);
  previewUrl.searchParams.set("preview", "1");
  if (previewElement.src !== previewUrl.href) {
    previewElement.src = previewUrl.href;
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
    const privacyConcepts = filterWordsToConcepts(
      privacyConceptsElement.value,
      bootstrap.config.privacyConcepts,
    );
    const privacyFilterExemptRoleIds = filterRoleIds(privacyExemptRoleIdsElement.value);
    const nextBootstrap = await invoke("apply_config", {
      config: {
        watchedChannelId: channelElement.value,
        ttsChannelId: ttsChannelElement.value,
        musicChannelId: musicChannelElement.value,
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
        showMediaTextObs: showMediaTextObsElement.checked,
        showMediaTextWidget: showMediaTextWidgetElement.checked,
        widgetSoundEnabled: widgetSoundEnabledElement.checked,
        moderationEnabled: moderationEnabledElement.checked,
        moderationAllowImages: moderationAllowImagesElement.checked,
        moderationAllowVideos: moderationAllowVideosElement.checked,
        moderationAllowAudio: moderationAllowAudioElement.checked,
        privacyScanEnabled: privacyScanEnabledElement.checked,
        privacySuspiciousPolicy: bootstrap.config.privacySuspiciousPolicy || "review",
        privacySuspiciousThreshold: Number(bootstrap.config.privacySuspiciousThreshold ?? 2),
        privacySensitiveThreshold: Number(bootstrap.config.privacySensitiveThreshold ?? 4),
        privacySimilarityBoost: Number(bootstrap.config.privacySimilarityBoost ?? 4),
        privacyConcepts,
        privacyFilterExemptRoleIds,
        privacyProtectionLevel: privacyProtectionLevelElement.value,
        privacyEnabledCategories: privacyCategoryElements
          .filter((input) => input.checked)
          .map((input) => input.value),
        privacyBlockThreshold: privacyBlockThresholdElement.value,
        privacyReviewIntermediate: privacyReviewIntermediateElement.checked,
        privacyAutoDeleteBlockedMessages: privacyAutoDeleteBlockedMessagesElement.checked,
        privacyAllowlist: privacyListFromInput(privacyAllowlistElement.value),
        privacyCustomPatterns: privacyListFromInput(privacyCustomPatternsElement.value),
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

async function saveMediaCaptionVisibility() {
  const generation = ++mediaCaptionSaveGeneration;
  mediaSaveStateElement.textContent = t("saving");
  try {
    const config = await invoke("set_media_caption_visibility", {
      showMediaTextObs: showMediaTextObsElement.checked,
      showMediaTextWidget: showMediaTextWidgetElement.checked,
    });
    if (generation !== mediaCaptionSaveGeneration) return;
    bootstrap.config = config;
    mediaSaveStateElement.textContent = t("saved");
  } catch (error) {
    if (generation === mediaCaptionSaveGeneration) {
      mediaSaveStateElement.textContent = String(error);
    }
  }
}

function schedulePrivacyFilterSave() {
  privacyFilterDraft = privacyConceptsElement.value;
  const generation = ++privacyFilterSaveGeneration;
  window.clearTimeout(privacyFilterSaveTimer);
  privacyFilterSaveTimer = window.setTimeout(() => {
    privacyFilterSaveTimer = undefined;
    void savePrivacyFiltersAutomatically(generation);
  }, 750);
}

async function savePrivacyFiltersAutomatically(generation) {
  if (generation !== privacyFilterSaveGeneration
    || !bootstrap?.config
    || !filterWordsAreSaveable(privacyFilterDraft)) {
    return;
  }

  const saved = await saveConfig(moderationSaveStateElement);
  if (!saved || generation === privacyFilterSaveGeneration) {
    return;
  }

  privacyConceptsElement.value = privacyFilterDraft;
  schedulePrivacyFilterSave();
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
      || document.activeElement === ttsChannelElement
      || document.activeElement === musicChannelElement;
    if (channelsSignature !== lastChannelsSignature && !selectingChannel) {
      lastChannelsSignature = channelsSignature;
      populateChannels(channelElement, status.channels, channelElement.value, t("selectChannel"));
      populateChannels(ttsChannelElement, status.channels, ttsChannelElement.value, t("ttsDisabled"));
      populateChannels(musicChannelElement, status.channels, musicChannelElement.value, t("musicDisabled"));
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

navigationBackButton.addEventListener("click", () => navigateHistory(-1));
navigationForwardButton.addEventListener("click", () => navigateHistory(1));

settingsSearchElement.addEventListener("input", renderSettingsSearchResults);
settingsSearchElement.addEventListener("focus", renderSettingsSearchResults);
settingsSearchElement.addEventListener("keydown", (event) => {
  const results = $$(".settings-search__result", settingsSearchResultsElement);
  if (event.key === "ArrowDown" && results.length) {
    event.preventDefault();
    results[0].focus();
  }
});
settingsSearchResultsElement.addEventListener("keydown", (event) => {
  const results = $$(".settings-search__result", settingsSearchResultsElement);
  const index = results.indexOf(document.activeElement);
  if (event.key === "ArrowDown" && index < results.length - 1) {
    event.preventDefault();
    results[index + 1].focus();
  } else if (event.key === "ArrowUp") {
    event.preventDefault();
    if (index > 0) results[index - 1].focus();
    else settingsSearchElement.focus();
  }
});
settingsSearchClearButton.addEventListener("click", () => {
  settingsSearchElement.value = "";
  renderSettingsSearchResults();
  settingsSearchElement.focus();
});

for (const button of $$("[data-help-link]")) {
  button.addEventListener("click", () => invoke("open_help_link", { link: button.dataset.helpLink }));
}

updateCheckButton.addEventListener("click", async () => {
  setUpdateMenuOpen(true);
  updateUiState = { kind: "checking" };
  renderUpdateStatus();
  try {
    latestUpdate = await invoke("check_for_updates");
    setAppVersion(latestUpdate.currentVersion);
    updateUiState = {
      kind: latestUpdate.updateAvailable ? "available" : "current",
      version: latestUpdate.latestVersion,
    };
  } catch (error) {
    latestUpdate = undefined;
    updateUiState = { kind: "error", errorKey: "updateCheckFailed", error: String(error) };
  }
  renderUpdateStatus();
});

installUpdateButton.addEventListener("click", async () => {
  updateUiState = { kind: "installing", version: latestUpdate.latestVersion };
  renderUpdateStatus();
  try {
    await invoke("download_and_install_update");
  } catch (error) {
    updateUiState = { kind: "error", errorKey: "updateInstallFailed", error: String(error) };
    renderUpdateStatus();
  }
});

updateMenuCloseButton.addEventListener("click", () => {
  setUpdateMenuOpen(false);
  updateCheckButton.focus();
});

document.addEventListener("pointerdown", (event) => {
  if (!updateMenuElement.hidden && !updateControlElement.contains(event.target)) {
    setUpdateMenuOpen(false);
  }
  if (!settingsSearchResultsElement.hidden && !settingsSearchControl.contains(event.target)) {
    closeSettingsSearch();
  }
  if (!interfaceLanguageOptionsElement.hidden && !interfaceLanguageElement.contains(event.target)) {
    setLanguageMenuOpen(false);
  }
  if (!sidebarLanguageOptionsElement.hidden && !sidebarLanguagePickerElement.contains(event.target)) {
    setSidebarLanguageMenuOpen(false);
  }
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    if (!updateMenuElement.hidden) {
      setUpdateMenuOpen(false);
      updateCheckButton.focus();
    }
    if (!settingsSearchResultsElement.hidden) {
      closeSettingsSearch();
      settingsSearchElement.focus();
    }
    if (!interfaceLanguageOptionsElement.hidden) {
      setLanguageMenuOpen(false);
      interfaceLanguageButton.focus();
    }
    if (!sidebarLanguageOptionsElement.hidden) {
      setSidebarLanguageMenuOpen(false);
      languageToggleButton.focus();
    }
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLocaleLowerCase() === "k") {
    event.preventDefault();
    settingsSearchElement.focus();
    settingsSearchElement.select();
  }
  if (event.altKey && !event.ctrlKey && !event.metaKey && ["ArrowLeft", "ArrowRight"].includes(event.key)) {
    event.preventDefault();
    navigateHistory(event.key === "ArrowLeft" ? -1 : 1);
  }
});

$("#privacy-reference").addEventListener("click", () => {
  showPage("help");
  const privacyDetails = $("#privacy-details");
  privacyDetails.open = true;
  window.requestAnimationFrame(() => privacyDetails.scrollIntoView({ behavior: "smooth", block: "start" }));
});

languageToggleButton.addEventListener("click", () => {
  setSidebarLanguageMenuOpen(sidebarLanguageOptionsElement.hidden);
});

themeToggleButton.addEventListener("click", () => {
  theme = theme === "light" ? "dark" : "light";
  applyTheme();
  applyPersonalization();
});

interfaceLanguageButton.addEventListener("click", () => {
  setLanguageMenuOpen(interfaceLanguageOptionsElement.hidden);
});

for (const option of $$("[data-locale]", interfaceLanguageOptionsElement)) {
  option.addEventListener("click", () => {
    selectInterfaceLanguage(option.dataset.locale, interfaceLanguageButton);
  });
}

sidebarLanguageOptionsElement.addEventListener("click", (event) => {
  const option = event.target.closest("[data-locale]");
  if (option) selectInterfaceLanguage(option.dataset.locale, languageToggleButton);
});

interfaceThemeElement.addEventListener("change", () => {
  theme = interfaceThemeElement.value;
  applyTheme();
  applyPersonalization();
});

interfaceFontElement.addEventListener("change", () => {
  interfaceFont = interfaceFontElement.value;
  applyPersonalization();
});

sidebarLayoutElement.addEventListener("change", () => {
  sidebarLayout = sidebarLayoutElement.value;
  sidebarExpanded = false;
  applySidebarLayout();
});

sidebarElement.addEventListener("pointerenter", () => setDynamicSidebarExpanded(true));
sidebarElement.addEventListener("pointerleave", () => setDynamicSidebarExpanded(false));
sidebarElement.addEventListener("focusin", () => setDynamicSidebarExpanded(true));
sidebarElement.addEventListener("focusout", () => {
  window.requestAnimationFrame(() => {
    if (!sidebarElement.contains(document.activeElement)) setDynamicSidebarExpanded(false);
  });
});

for (const input of designInputs) {
  input.addEventListener("change", () => {
    design = input.value;
    applyDesign();
    applyPersonalization();
    designPickerElement.open = false;
  });
}

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
  locale = "en-US";
  language = "en";
  theme = "dark";
  design = "openai";
  interfaceFont = "design";
  sidebarLayout = "fixed";
  sidebarExpanded = false;
  accentRgb = [88, 185, 137];
  fontScale = 100;
  applyLanguage();
  applyTheme();
  applyDesign();
  applySidebarLayout();
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
      youtubeApiKey: youtubeApiKeyElement.value.trim(),
    }));
    credentialStateElement.textContent = t("encryptedStarting");
  } catch (error) {
    credentialStateElement.textContent = String(error);
  } finally {
    tokenElement.value = "";
    youtubeApiKeyElement.value = "";
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
    populateChannels(musicChannelElement, channels, musicChannelElement.value, t("musicDisabled"));
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

for (const input of [showMediaTextObsElement, showMediaTextWidgetElement]) {
  input.addEventListener("change", () => void saveMediaCaptionVisibility());
}

moderationForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  await saveConfig(moderationSaveStateElement);
});

privacyConceptsElement.addEventListener("input", schedulePrivacyFilterSave);

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

addCustomCommandButton.addEventListener("click", () => openCustomCommandEditor());
cancelCustomCommandButton.addEventListener("click", closeCustomCommandEditor);

customCommandNameElement.addEventListener("input", () => {
  const normalized = customCommandNameElement.value
    .toLowerCase()
    .replace(/\s+/g, "-")
    .replace(/[^a-z0-9_-]/g, "");
  if (normalized !== customCommandNameElement.value) customCommandNameElement.value = normalized;
  customCommandPreviewElement.textContent = `/relay ${normalized || "command"}`;
  customCommandEditorStateElement.textContent = "";
});

customCommandActionElement.addEventListener("change", () => {
  renderCustomActionFields(defaultCustomAction(customCommandActionElement.value));
  customCommandEditorStateElement.textContent = "";
});

customActionFieldsElement.addEventListener("change", (event) => {
  if (event.target.matches("[data-custom-parameter-mode]")) {
    updateCustomParameterAvailability(event.target.closest("[data-custom-parameter]"));
  }
  customCommandEditorStateElement.textContent = "";
});

customCommandForm.addEventListener("submit", (event) => {
  event.preventDefault();
  customCommandEditorStateElement.textContent = "";
  try {
    const definition = collectCustomCommandDraft();
    if (!definition) return;
    if (editingCustomCommandIndex === null) customCommands.push(definition);
    else customCommands[editingCustomCommandIndex] = definition;
    customCommandsDirty = true;
    renderCustomCommands();
    closeCustomCommandEditor();
    customCommandsSaveStateElement.textContent = t("customDraftSaved");
  } catch (error) {
    customCommandEditorStateElement.textContent = String(error.message || error);
  }
});

customCommandListElement.addEventListener("click", (event) => {
  const button = event.target.closest("[data-custom-command-action]");
  if (!button) return;
  const index = Number(button.dataset.customCommandIndex);
  if (!Number.isInteger(index) || !customCommands[index]) return;
  if (button.dataset.customCommandAction === "edit") {
    openCustomCommandEditor(index);
    return;
  }
  customCommands.splice(index, 1);
  customCommandsDirty = true;
  closeCustomCommandEditor();
  renderCustomCommands();
  customCommandsSaveStateElement.textContent = t("customUnsaved");
});

syncCustomCommandsButton.addEventListener("click", async () => {
  customCommandsSaveStateElement.textContent = t("customValidating");
  const names = new Set();
  const invalidName = customCommands.some((command) => {
    if (defaultRelayCommandNames.has(command.name) || names.has(command.name)) return true;
    names.add(command.name);
    return false;
  });
  if (customCommands.length > 16 || invalidName) {
    customCommandsSaveStateElement.textContent = t("customDuplicateName");
    return;
  }
  syncCustomCommandsButton.disabled = true;
  addCustomCommandButton.disabled = true;
  customCommandsSaveStateElement.textContent = t("customSyncing");
  try {
    const config = await invoke("save_custom_commands", { commands: cloneCustomCommands(customCommands) });
    customCommandsDirty = false;
    customCommands = cloneCustomCommands(config.customCommands);
    bootstrap.config = config;
    applyConfig(config);
    try {
      applyBootstrap(await invoke("get_bootstrap"));
    } catch {
      renderCustomCommands();
    }
    customCommandsSaveStateElement.textContent = t("customActive");
  } catch (error) {
    customCommandsDirty = true;
    customCommandsSaveStateElement.textContent = String(error);
  } finally {
    syncCustomCommandsButton.disabled = false;
    addCustomCommandButton.disabled = customCommands.length >= 16;
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
openInviteButton.addEventListener("click", () => invoke("open_help_link", { link: inviteUrlElement.value }));

for (const [target, button] of outputTestButtons) {
  button.addEventListener("click", async () => {
    if (button.disabled) return;
    button.disabled = true;
    try {
      await invoke("test_output", { target });
      button.textContent = t("outputTestSent");
    } catch (error) {
      button.textContent = t("outputTestFailed");
      button.title = String(error);
    }
    window.setTimeout(() => {
      button.textContent = t("testOutput");
      renderOutputReadiness(bootstrap?.server);
    }, 1600);
  });
}

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

skipShortcutCaptureButton.addEventListener("click", beginShortcutCapture);
window.addEventListener("keydown", (event) => {
  if (!shortcutCaptureActive) return;
  event.preventDefault();
  event.stopPropagation();
  if (event.key === "Escape") {
    cancelShortcutCapture();
    mediaSaveStateElement.textContent = t("shortcutCanceled");
    return;
  }
  const token = shortcutTokenFromEvent(event);
  if (!token) return;
  const modifiers = [];
  if (event.ctrlKey) modifiers.push("control");
  if (event.altKey) modifiers.push("alt");
  if (event.shiftKey) modifiers.push("shift");
  if (event.metaKey) modifiers.push("super");
  void saveCapturedShortcut([...modifiers, token].join("+"));
});

previousAudioButton.addEventListener("click", () => controlCurrentAudio("previous"));
toggleAudioButton.addEventListener("click", () => controlCurrentAudio(
  currentAudioPlayback?.status === "paused" ? "resume" : "pause",
));
skipAudioButton.addEventListener("click", () => controlCurrentAudio("skip"));

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
  releaseNowPlayingArtwork();
});

initializeOutputGeometryControls();
applyLanguage();
applyTheme();
applyDesign();
applySidebarLayout();
applyPersonalization();
showPage(currentPage);

try {
  invoke("get_app_version").then(setAppVersion).catch(() => {});
  applyBootstrap(await invoke("get_bootstrap"));
  connectPanelSocket();
  statusTimer = window.setInterval(refreshRuntimeStatus, 1500);
} catch (error) {
  saveStateElement.textContent = String(error);
  credentialStateElement.textContent = String(error);
}
