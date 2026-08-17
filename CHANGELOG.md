# Changelog

All notable user-facing changes to Relay are documented in this file.
Toutes les évolutions notables de Relay sont documentées dans ce fichier.

Release 1.3.0 and later include every Relay interface language. Earlier releases remain English and French, with English as the fallback in the app.
À partir de la 1.3.0, chaque langue d’interface Relay est incluse. Les versions antérieures restent en anglais et en français, avec l’anglais comme repli dans l’application.

## Versioning policy / Politique de version

- A major Relay update increments the middle number: `1.0.0` → `1.1.0`.
- A minor update, bug fix, or simple addition increments the patch number: `1.0.0` → `1.0.1`.
- Changes remain under `Unreleased` until the matching GitHub release is published.

## [Unreleased]

## [1.3.0] - 2026-08-17

### English

#### Added

- Added YouTube music search in a configurable Discord channel, with up to 15 relevant results between one and five minutes long.
- Added 30-second previews, full-track playback, queueing, and a Now Playing card for OBS and the Windows widget.
- Added unified **Relay Visual** and **Relay Audio** OBS Browser Sources for media, stickers, TTS notifications, YouTube playback, Discord audio, and TTS voice. Legacy source URLs remain available during migration.
- Added History downloads, a configurable global media skip shortcut, and an English YouTube API setup guide in the Music panel and project documentation.
- Added the **Gridline** and **Lumen** interface designs, compact and dynamic sidebar layouts, and a collapsible design picker.
- Added CI dependency auditing for the Rust crate so known high-severity issues are checked on every verification run.
- Added an in-app **Changelog** page that shows bundled release notes in the current interface language.

#### Changed

- Media, TTS, and YouTube playback now share output scheduling so competing items do not play over one another.
- YouTube playback now respects widget sound settings and wakes the Windows audio output when needed.
- Media captions now stay compact and anchored to the active media or player card, with independent OBS and Windows widget visibility settings.
- Windows TTS notifications now use a denser toast (400×104 by default) that stays compact instead of stretching across leftover empty space. Existing generated 980×180 and 480×112 sizes migrate automatically; custom sizes are kept.
- Overlay move labels and preview copy now follow every Relay interface language, including Russian, Simplified Chinese, Korean, Japanese, and Indonesian.
- Language, theme, accent, and font-scale choices are now saved with Relay config and restored on launch, including tray-only and `--startup` sessions.

#### Fixed

- Fixed YouTube playback restoration after output wake-up and stop/start transitions.
- Fixed skip shortcut registration failures without discarding the previous shortcut.
- Fixed canceled media downloads leaving History actions in a busy state.
- Fixed GIF downloads accepting responses that were not GIF files.
- Fixed TTS and sticker Browser Sources dropping their queued items after a brief WebSocket drop.
- Fixed TTS and sticker outputs starting before the server granted the shared stage, which could overlap media or music.
- Fixed OBS TTS and sticker sources navigating blindly after a Relay port change. They now probe the new server first, with a timeout, so a failed load does not leave the Browser Source dead.
- Fixed the Windows media widget forgetting its position when hidden or locked immediately after a move.
- Fixed deferred Discord GIF updates ignoring privacy filter exemptions because role information was missing.
- Fixed YouTube track selections being discarded when the jukebox queue was already full.
- Fixed Relay reporting a Windows TTS failure after a successful visual fallback.

#### Security

- YouTube API keys are stored locally in Windows Credential Manager and are not shown again after saving.
- YouTube playback controls are restricted to the user who requested the track or a Discord administrator.
- Panel and overlay Tauri windows now use separate permission sets: overlay widgets can no longer invoke control-panel commands.
- Discord overlay URLs posted by the bot no longer include the local Relay secret. Short pages inject a page-local secret instead of a host-wide cookie.
- Unused privacy-threshold settings that no longer affected filtering were removed from the interface and config schema.

### Français

#### Ajouté

- Ajout de la recherche musicale YouTube dans un salon Discord configurable, avec jusqu’à 15 résultats pertinents d’une à cinq minutes.
- Ajout des extraits de 30 secondes, de la lecture complète, de la file d’attente et d’une carte En cours de lecture pour OBS et le widget Windows.
- Ajout des sources navigateur OBS unifiées **Relay Visual** et **Relay Audio** pour les médias, les stickers, les notifications TTS, YouTube, l’audio Discord et la voix TTS. Les anciennes URL restent disponibles pendant la migration.
- Ajout des téléchargements depuis l’Historique, d’un raccourci global de saut média configurable, et d’un guide YouTube API en anglais dans le panneau Musique et la documentation.
- Ajout des designs d’interface **Gridline** et **Lumen**, des dispositions de barre latérale compacte et dynamique, et d’un sélecteur de design repliable.
- Ajout d’un audit des dépendances Rust en CI afin de signaler les vulnérabilités connues à chaque vérification.
- Ajout d’une page **Changelog** dans l’application, qui affiche les notes de version incluses dans la langue de l’interface.

#### Modifié

- La lecture des médias, du TTS et de YouTube partage désormais un ordonnancement commun pour éviter les chevauchements.
- La lecture YouTube respecte désormais le son du widget et réveille la sortie audio Windows si besoin.
- Les légendes média restent compactes et ancrées à la carte active, avec des réglages de visibilité indépendants pour OBS et le widget Windows.
- Les notifications TTS Windows utilisent désormais un toast plus dense (400×104 par défaut) qui ne s’étire plus dans le vide restant. Les tailles générées 980×180 et 480×112 sont migrées automatiquement ; les tailles personnalisées sont conservées.
- Les libellés de déplacement de l’overlay et les textes d’aperçu suivent désormais toutes les langues de Relay, y compris le russe, le chinois simplifié, le coréen, le japonais et l’indonésien.
- La langue, le thème, l’accent et l’échelle de police sont désormais enregistrés avec la configuration Relay et restaurés au lancement, y compris en mode zone de notification et `--startup`.

#### Corrigé

- Correction de la restauration YouTube après un réveil de sortie et les transitions arrêt/reprise.
- Correction de l’enregistrement du raccourci de saut qui pouvait échouer en oubliant le raccourci précédent.
- Correction des téléchargements média annulés qui laissaient les actions de l’Historique bloquées.
- Correction des téléchargements GIF qui acceptaient des réponses n’étant pas des fichiers GIF.
- Correction des sources TTS et stickers qui perdaient leur file d’attente après une coupure WebSocket brève.
- Correction des sorties TTS et stickers qui démarraient avant l’accord du serveur, ce qui pouvait chevaucher un média ou la musique.
- Correction des sources OBS TTS et stickers qui changeaient de port sans vérifier le nouveau serveur. Elles sondent désormais d’abord la nouvelle instance, avec un délai, afin d’éviter une Browser Source définitivement bloquée.
- Correction du widget média Windows qui oubliait sa position s’il était masqué ou verrouillé juste après un déplacement.
- Correction des GIF Discord différés qui ignoraient les exemptions de filtrage faute de rôles.
- Correction des sélections YouTube perdues lorsque la file du jukebox était déjà pleine.
- Correction du statut Relay qui signalait un échec TTS Windows alors que le repli visuel avait réussi.

#### Sécurité

- Les clés API YouTube sont stockées localement dans le Gestionnaire d’identifiants Windows et ne sont plus réaffichées après l’enregistrement.
- Les contrôles de lecture YouTube sont réservés à l’utilisateur qui a demandé le morceau ou à un administrateur Discord.
- Les fenêtres Tauri du panneau et des overlays utilisent désormais des jeux de permissions séparés : les widgets overlay ne peuvent plus invoquer les commandes du panneau de contrôle.
- Les URL d’overlay publiées par le bot Discord n’incluent plus le secret Relay local. Les pages courtes injectent un secret limité à la page plutôt qu’un cookie pour tout l’hôte.
- Les réglages de seuils de confidentialité inutilisés, qui n’influençaient plus le filtrage, ont été retirés de l’interface et du schéma de configuration.

### Español

#### Añadido

- Añadida la búsqueda musical de YouTube en un canal de Discord configurable, con hasta 15 resultados relevantes de entre uno y cinco minutos.
- Añadidos extractos de 30 segundos, reproducción completa, cola y una tarjeta En reproducción para OBS y el widget de Windows.
- Añadidas las fuentes de navegador OBS unificadas **Relay Visual** y **Relay Audio** para medios, stickers, notificaciones TTS, reproducción de YouTube, audio de Discord y voz TTS. Las URL de fuentes antiguas siguen disponibles durante la migración.
- Añadidas las descargas del Historial, un atajo global configurable para saltar el medio, y una guía de YouTube API en inglés en el panel Música y la documentación.
- Añadidos los diseños de interfaz **Gridline** y **Lumen**, las disposiciones de barra lateral compacta y dinámica, y un selector de diseño plegable.
- Añadida una auditoría de dependencias Rust en CI para detectar problemas conocidos de alta gravedad en cada verificación.
- Añadida una página **Changelog** en la aplicación que muestra las notas de versión incluidas en el idioma de la interfaz.

#### Cambiado

- La reproducción de medios, TTS y YouTube comparte ahora una programación común para que los elementos no se solapen.
- La reproducción de YouTube respeta ahora el sonido del widget y despierta la salida de audio de Windows cuando hace falta.
- Los subtítulos de medios permanecen compactos y anclados a la tarjeta activa, con ajustes de visibilidad independientes para OBS y el widget de Windows.
- Las notificaciones TTS de Windows usan ahora un toast más denso (400×104 por defecto) que permanece compacto en lugar de estirarse por el espacio vacío. Los tamaños generados 980×180 y 480×112 se migran automáticamente; los tamaños personalizados se conservan.
- Las etiquetas de desplazamiento del overlay y los textos de vista previa siguen ahora todos los idiomas de Relay, incluidos el ruso, el chino simplificado, el coreano, el japonés y el indonesio.
- El idioma, el tema, el acento y la escala de fuente se guardan ahora con la configuración de Relay y se restauran al iniciar, incluidas las sesiones de bandeja y `--startup`.

#### Corregido

- Corregida la restauración de YouTube tras un despertar de salida y las transiciones de parada/reanudación.
- Corregido el registro del atajo de salto que podía fallar y olvidar el atajo anterior.
- Corregidas las descargas de medios canceladas que dejaban las acciones del Historial bloqueadas.
- Corregidas las descargas GIF que aceptaban respuestas que no eran archivos GIF.
- Corregidas las fuentes TTS y stickers que perdían su cola tras una breve caída de WebSocket.
- Corregidas las salidas TTS y stickers que arrancaban antes de que el servidor concediera el escenario compartido, lo que podía solaparse con un medio o la música.
- Corregidas las fuentes OBS TTS y stickers que cambiaban de puerto sin comprobar el nuevo servidor. Ahora sondean primero la nueva instancia, con un tiempo de espera, para no dejar la Browser Source bloqueada.
- Corregido el widget de medios de Windows que olvidaba su posición si se ocultaba o bloqueaba justo después de un movimiento.
- Corregidos los GIF de Discord diferidos que ignoraban las exenciones del filtro de privacidad por falta de roles.
- Corregidas las selecciones de YouTube que se descartaban cuando la cola del jukebox ya estaba llena.
- Corregido el estado de Relay que informaba de un fallo TTS de Windows tras un repliegue visual correcto.

#### Seguridad

- Las claves API de YouTube se almacenan localmente en el Administrador de credenciales de Windows y no se vuelven a mostrar tras guardar.
- Los controles de reproducción de YouTube están restringidos a quien pidió la pista o a un administrador de Discord.
- Las ventanas Tauri del panel y de los overlays usan ahora conjuntos de permisos separados: los widgets overlay ya no pueden invocar comandos del panel de control.
- Las URL de overlay publicadas por el bot de Discord ya no incluyen el secreto local de Relay. Las páginas cortas inyectan un secreto limitado a la página en lugar de una cookie para todo el host.
- Los ajustes de umbral de privacidad no usados, que ya no afectaban al filtrado, se retiraron de la interfaz y del esquema de configuración.

### Deutsch

#### Hinzugefügt

- YouTube-Musiksuche in einem konfigurierbaren Discord-Kanal hinzugefügt, mit bis zu 15 relevanten Ergebnissen zwischen einer und fünf Minuten.
- 30-Sekunden-Vorschauen, vollständige Wiedergabe, Warteschlange und eine Now-Playing-Karte für OBS und das Windows-Widget hinzugefügt.
- Vereinheitlichte OBS-Browserquellen **Relay Visual** und **Relay Audio** für Medien, Sticker, TTS-Benachrichtigungen, YouTube-Wiedergabe, Discord-Audio und TTS-Stimme hinzugefügt. Alte Quellen-URLs bleiben während der Migration verfügbar.
- Downloads im Verlauf, eine konfigurierbare globale Tastenkombination zum Überspringen von Medien sowie eine englische YouTube-API-Anleitung im Musikbereich und in der Dokumentation hinzugefügt.
- Die Oberflächendesigns **Gridline** und **Lumen**, kompakte und dynamische Seitenleistenlayouts sowie einen einklappbaren Designwähler hinzugefügt.
- CI-Abhängigkeitsprüfung für die Rust-Crate hinzugefügt, damit bekannte schwerwiegende Probleme bei jeder Prüfung erkannt werden.
- Eine **Changelog**-Seite in der App hinzugefügt, die gebündelte Versionshinweise in der aktuellen Oberflächensprache anzeigt.

#### Geändert

- Medien, TTS und YouTube teilen sich jetzt eine gemeinsame Ausgabeplanung, damit sich Inhalte nicht überschneiden.
- Die YouTube-Wiedergabe berücksichtigt jetzt die Widget-Toneinstellungen und weckt bei Bedarf die Windows-Audioausgabe.
- Medienuntertitel bleiben kompakt und an der aktiven Medien- oder Playerkarte verankert, mit unabhängigen Sichtbarkeitseinstellungen für OBS und das Windows-Widget.
- Windows-TTS-Benachrichtigungen verwenden jetzt einen dichteren Toast (standardmäßig 400×104), der kompakt bleibt, statt in den restlichen Leerraum zu strecken. Die generierten Größen 980×180 und 480×112 werden automatisch migriert; benutzerdefinierte Größen bleiben erhalten.
- Overlay-Verschiebebeschriftungen und Vorschautexte folgen jetzt jeder Relay-Oberflächensprache, einschließlich Russisch, Vereinfachtes Chinesisch, Koreanisch, Japanisch und Indonesisch.
- Sprache, Thema, Akzentfarbe und Textskalierung werden jetzt mit der Relay-Konfiguration gespeichert und beim Start wiederhergestellt, einschließlich Infobereich- und `--startup`-Sitzungen.

#### Behoben

- Wiederherstellung der YouTube-Wiedergabe nach Ausgabe-Aufwecken und Stopp/Start-Übergängen behoben.
- Fehlerhafte Registrierung der Überspringen-Tastenkombination behoben, ohne die vorherige Tastenkombination zu verwerfen.
- Abgebrochene Medien-Downloads behoben, die Verlaufsaktionen im Beschäftigt-Zustand ließen.
- GIF-Downloads behoben, die Antworten akzeptierten, die keine GIF-Dateien waren.
- TTS- und Sticker-Browserquellen behoben, die nach einem kurzen WebSocket-Abbruch ihre Warteschlange verloren.
- TTS- und Sticker-Ausgaben behoben, die starteten, bevor der Server die gemeinsame Bühne gewährte, was Medien oder Musik überlappen konnte.
- OBS-TTS- und Sticker-Quellen behoben, die nach einem Relay-Portwechsel blind navigierten. Sie prüfen jetzt zuerst den neuen Server, mit Timeout, damit eine fehlgeschlagene Ladung die Browserquelle nicht tot hinterlässt.
- Windows-Medienwidget behoben, das seine Position vergaß, wenn es direkt nach einem Verschieben ausgeblendet oder gesperrt wurde.
- Verzögerte Discord-GIF-Updates behoben, die Datenschutzfilter-Ausnahmen ignorierten, weil Rolleninformationen fehlten.
- YouTube-Titelauswahl behoben, die verworfen wurde, wenn die Jukebox-Warteschlange bereits voll war.
- Relay-Status behoben, der einen Windows-TTS-Fehler meldete, obwohl der visuelle Fallback erfolgreich war.

#### Sicherheit

- YouTube-API-Schlüssel werden lokal im Windows-Anmeldeinformations-Manager gespeichert und nach dem Speichern nicht erneut angezeigt.
- YouTube-Wiedergabesteuerung ist auf die Person beschränkt, die den Titel angefordert hat, oder auf einen Discord-Administrator.
- Tauri-Fenster von Panel und Overlays verwenden jetzt getrennte Berechtigungssätze: Overlay-Widgets können keine Steuerpanel-Befehle mehr aufrufen.
- Vom Discord-Bot veröffentlichte Overlay-URLs enthalten nicht mehr das lokale Relay-Geheimnis. Kurze Seiten injizieren ein seitenlokales Geheimnis statt eines hostweiten Cookies.
- Ungenutzte Datenschutz-Schwellenwerte, die das Filtern nicht mehr beeinflussten, wurden aus Oberfläche und Konfigurationsschema entfernt.

### Русский

#### Добавлено

- Добавлен поиск музыки YouTube в настраиваемом канале Discord, до 15 релевантных результатов длительностью от одной до пяти минут.
- Добавлены 30-секундные превью, полное воспроизведение, очередь и карточка «Сейчас играет» для OBS и виджета Windows.
- Добавлены единые источники браузера OBS **Relay Visual** и **Relay Audio** для медиа, стикеров, TTS-уведомлений, YouTube, аудио Discord и голоса TTS. Старые URL источников остаются доступны во время миграции.
- Добавлены загрузки из истории, настраиваемый глобальный ярлык пропуска медиа и руководство YouTube API на английском в разделе «Музыка» и в документации.
- Добавлены дизайны интерфейса **Gridline** и **Lumen**, компактная и динамическая боковые панели, а также сворачиваемый выбор дизайна.
- Добавлена проверка зависимостей Rust в CI, чтобы известные серьёзные уязвимости выявлялись при каждой проверке.
- Добавлена страница **Changelog** в приложении, которая показывает встроенные заметки о версии на языке интерфейса.

#### Изменено

- Воспроизведение медиа, TTS и YouTube теперь использует общее планирование вывода, чтобы элементы не накладывались друг на друга.
- Воспроизведение YouTube теперь учитывает звук виджета и при необходимости будит аудиовыход Windows.
- Подписи к медиа остаются компактными и привязаны к активной карточке, с независимой видимостью для OBS и виджета Windows.
- TTS-уведомления Windows теперь используют более плотный тост (по умолчанию 400×104), который не растягивается на пустое место. Сгенерированные размеры 980×180 и 480×112 переносятся автоматически; пользовательские размеры сохраняются.
- Подписи перемещения оверлея и тексты предпросмотра теперь следуют всем языкам интерфейса Relay, включая русский, упрощённый китайский, корейский, японский и индонезийский.
- Язык, тема, акцент и масштаб шрифта теперь сохраняются в конфигурации Relay и восстанавливаются при запуске, включая режим области уведомлений и `--startup`.

#### Исправлено

- Исправлено восстановление YouTube после пробуждения вывода и переходов остановки/возобновления.
- Исправлена регистрация ярлыка пропуска, которая могла завершаться ошибкой и забывать предыдущий ярлык.
- Исправлены отменённые загрузки медиа, из-за которых действия истории зависали.
- Исправлены загрузки GIF, которые принимали ответы, не являющиеся файлами GIF.
- Исправлены источники TTS и стикеров, которые теряли очередь после короткого обрыва WebSocket.
- Исправлены выводы TTS и стикеров, которые запускались до разрешения сервера, из-за чего могли перекрывать медиа или музыку.
- Исправлены источники OBS TTS и стикеров, которые меняли порт, не проверяя новый сервер. Теперь они сначала опрашивают новый экземпляр с тайм-аутом, чтобы Browser Source не оставался мёртвым.
- Исправлен виджет медиа Windows, который забывал позицию, если его скрывали или блокировали сразу после перемещения.
- Исправлены отложенные GIF Discord, которые игнорировали исключения фильтра конфиденциальности из-за отсутствия ролей.
- Исправлен сброс выбора YouTube, когда очередь jukebox уже была полной.
- Исправлен статус Relay, который сообщал об ошибке TTS Windows после успешного визуального запасного варианта.

#### Безопасность

- Ключи API YouTube хранятся локально в диспетчере учётных данных Windows и больше не показываются после сохранения.
- Управление воспроизведением YouTube доступно только запросившему трек пользователю или администратору Discord.
- Окна Tauri панели и оверлеев теперь используют разные наборы прав: виджеты оверлея больше не могут вызывать команды панели управления.
- URL оверлея, публикуемые Discord-ботом, больше не содержат локальный секрет Relay. Короткие страницы внедряют секрет только для страницы, а не cookie на весь хост.
- Неиспользуемые пороги конфиденциальности, которые больше не влияли на фильтрацию, удалены из интерфейса и схемы конфигурации.

### 简体中文

#### 新增

- 新增可配置 Discord 频道中的 YouTube 音乐搜索，最多 15 条时长一到五分钟的相关结果。
- 新增 30 秒试听、完整播放、队列，以及用于 OBS 和 Windows 小组件的正在播放卡片。
- 新增统一的 OBS 浏览器源 **Relay Visual** 和 **Relay Audio**，覆盖媒体、贴纸、TTS 通知、YouTube 播放、Discord 音频和 TTS 语音。迁移期间仍可使用旧源 URL。
- 新增历史记录下载、可配置的全局媒体跳过快捷键，以及音乐面板和文档中的英文 YouTube API 设置指南。
- 新增 **Gridline** 和 **Lumen** 界面设计、紧凑与动态侧边栏布局，以及可折叠的设计选择器。
- 新增 Rust 依赖的 CI 审计，以便每次检查都能发现已知的高危问题。
- 新增应用内 **Changelog** 页面，按当前界面语言显示内置版本说明。

#### 更改

- 媒体、TTS 和 YouTube 播放现在共享输出调度，避免互相重叠。
- YouTube 播放现在遵循小组件声音设置，并在需要时唤醒 Windows 音频输出。
- 媒体说明保持紧凑并锚定到当前媒体或播放卡片，OBS 与 Windows 小组件的可见性可分别设置。
- Windows TTS 通知现在使用更紧凑的提示条（默认 400×104），不再拉伸填满空白。已生成的 980×180 和 480×112 尺寸会自动迁移；自定义尺寸会保留。
- 叠加层移动标签和预览文案现在跟随 Relay 的所有界面语言，包括俄语、简体中文、韩语、日语和印尼语。
- 语言、主题、强调色和字体缩放现在随 Relay 配置保存，并在启动时恢复，包括托盘会话和 `--startup`。

#### 修复

- 修复输出唤醒以及停止/开始切换后的 YouTube 播放恢复。
- 修复跳过快捷键注册失败时丢掉上一个快捷键的问题。
- 修复取消的媒体下载使历史记录操作一直处于忙碌状态。
- 修复 GIF 下载会接受非 GIF 文件响应的问题。
- 修复 TTS 和贴纸浏览器源在短暂 WebSocket 中断后丢失队列。
- 修复 TTS 和贴纸输出在服务器授予共享舞台之前就开始，可能与媒体或音乐重叠。
- 修复 OBS 的 TTS 和贴纸源在 Relay 端口变更后盲目跳转。现在会先探测新服务器并设超时，避免 Browser Source 彻底失效。
- 修复 Windows 媒体小组件在移动后立即隐藏或锁定时忘记位置。
- 修复延迟的 Discord GIF 更新因缺少角色信息而忽略隐私过滤豁免。
- 修复点唱机队列已满时 YouTube 曲目选择被丢弃。
- 修复视觉回退后 Relay 仍报告 Windows TTS 失败。

#### 安全

- YouTube API 密钥存储在本地 Windows 凭据管理器中，保存后不再显示。
- YouTube 播放控制仅限请求该曲目的用户或 Discord 管理员。
- 面板和叠加层的 Tauri 窗口现在使用不同权限集：叠加层小组件无法再调用控制面板命令。
- Discord 机器人发布的叠加层 URL 不再包含本地 Relay 密钥。短页面注入仅限该页的密钥，而不是整机 cookie。
- 已不再影响过滤的无用隐私阈值设置已从界面和配置架构中移除。

### 한국어

#### 추가

- 설정 가능한 Discord 채널에서 YouTube 음악 검색을 추가했으며, 1~5분 길이의 관련 결과를 최대 15개까지 표시합니다.
- 30초 미리 듣기, 전체 재생, 대기열, OBS 및 Windows 위젯용 지금 재생 중 카드를 추가했습니다.
- 미디어, 스티커, TTS 알림, YouTube 재생, Discord 오디오, TTS 음성을 위한 통합 OBS 브라우저 소스 **Relay Visual** 및 **Relay Audio**를 추가했습니다. 마이그레이션 중에는 기존 소스 URL을 계속 사용할 수 있습니다.
- 기록 다운로드, 설정 가능한 전역 미디어 건너뛰기 단축키, 음악 패널과 문서의 영어 YouTube API 설정 가이드를 추가했습니다.
- **Gridline** 및 **Lumen** 인터페이스 디자인, 축소/동적 사이드바 레이아웃, 접을 수 있는 디자인 선택기를 추가했습니다.
- 검증마다 알려진 고위험 문제를 확인하도록 Rust 크레이트 CI 의존성 감사를 추가했습니다.
- 현재 인터페이스 언어로 포함된 릴리스 노트를 보여주는 앱 내 **Changelog** 페이지를 추가했습니다.

#### 변경

- 미디어, TTS, YouTube 재생이 서로 겹치지 않도록 출력 일정을 공유합니다.
- YouTube 재생이 위젯 소리 설정을 따르며 필요할 때 Windows 오디오 출력을 깨웁니다.
- 미디어 캡션이 활성 미디어 또는 플레이어 카드에 고정된 채 작게 유지되며, OBS와 Windows 위젯 표시 여부를 따로 설정할 수 있습니다.
- Windows TTS 알림이 더 밀도 높은 토스트(기본 400×104)를 사용해 빈 공간을 채우도록 늘어나지 않습니다. 생성된 980×180 및 480×112 크기는 자동 이전되며 사용자 지정 크기는 유지됩니다.
- 오버레이 이동 레이블과 미리 보기 문구가 러시아어, 간체 중국어, 한국어, 일본어, 인도네시아어를 포함한 모든 Relay 인터페이스 언어를 따릅니다.
- 언어, 테마, 강조색, 글꼴 배율이 Relay 구성과 함께 저장되며 트레이 전용 및 `--startup` 세션을 포함해 시작 시 복원됩니다.

#### 수정

- 출력 깨우기 및 중지/시작 전환 후 YouTube 재생 복원을 수정했습니다.
- 이전 단축키를 버리지 않고 건너뛰기 단축키 등록 실패를 수정했습니다.
- 취소된 미디어 다운로드가 기록 작업을 바쁨 상태로 남기던 문제를 수정했습니다.
- GIF가 아닌 응답을 받아들이던 GIF 다운로드를 수정했습니다.
- 짧은 WebSocket 끊김 후 TTS 및 스티커 브라우저 소스가 대기열을 잃던 문제를 수정했습니다.
- 서버가 공유 스테이지를 허용하기 전에 TTS 및 스티커 출력이 시작되어 미디어나 음악과 겹칠 수 있던 문제를 수정했습니다.
- Relay 포트 변경 후 OBS TTS 및 스티커 소스가 무작정 이동하던 문제를 수정했습니다. 이제 새 서버를 먼저 탐지하며, 시간 제한으로 Browser Source가 멈추지 않습니다.
- 이동 직후 숨기거나 잠그면 위치를 잊던 Windows 미디어 위젯을 수정했습니다.
- 역할 정보가 없어 개인정보 필터 예외를 무시하던 지연 Discord GIF 업데이트를 수정했습니다.
- 주크박스 대기열이 이미 가득 찼을 때 YouTube 트랙 선택이 버려지던 문제를 수정했습니다.
- 시각적 대체 재생이 성공한 뒤에도 Windows TTS 실패를 보고하던 Relay 상태를 수정했습니다.

#### 보안

- YouTube API 키는 로컬 Windows 자격 증명 관리자에 저장되며 저장 후 다시 표시되지 않습니다.
- YouTube 재생 제어는 해당 트랙을 요청한 사용자 또는 Discord 관리자로 제한됩니다.
- 패널과 오버레이 Tauri 창이 서로 다른 권한 집합을 사용합니다. 오버레이 위젯은 더 이상 제어판 명령을 호출할 수 없습니다.
- Discord 봇이 게시하는 오버레이 URL에 더 이상 로컬 Relay 비밀이 포함되지 않습니다. 짧은 페이지는 호스트 전체 쿠키 대신 페이지 전용 비밀을 삽입합니다.
- 더 이상 필터링에 영향을 주지 않던 사용하지 않는 개인정보 임계값 설정을 인터페이스와 구성 스키마에서 제거했습니다.

### 日本語

#### 追加

- 設定可能な Discord チャンネルで YouTube 音楽検索を追加し、1〜5 分の関連結果を最大 15 件表示します。
- 30 秒プレビュー、全曲再生、キュー、OBS と Windows ウィジェット向けの再生中カードを追加しました。
- メディア、ステッカー、TTS 通知、YouTube 再生、Discord オーディオ、TTS 音声向けの統合 OBS ブラウザソース **Relay Visual** と **Relay Audio** を追加しました。移行中は従来のソース URL も利用できます。
- 履歴からのダウンロード、設定可能なグローバルメディアスキップショートカット、ミュージックパネルとドキュメント内の英語 YouTube API セットアップガイドを追加しました。
- インターフェースデザイン **Gridline** と **Lumen**、コンパクト／ダイナミックなサイドバー、折りたたみ可能なデザイン選択を追加しました。
- 検証のたびに既知の重大な問題を確認できるよう、Rust クレートの CI 依存関係監査を追加しました。
- 現在のインターフェース言語で同梱のリリースノートを表示するアプリ内 **Changelog** ページを追加しました。

#### 変更

- メディア、TTS、YouTube 再生が出力スケジュールを共有し、同時再生を避けます。
- YouTube 再生がウィジェットのサウンド設定に従い、必要に応じて Windows オーディオ出力を起こします。
- メディアキャプションはコンパクトなままアクティブなカードに固定され、OBS と Windows ウィジェットの表示を個別に設定できます。
- Windows TTS 通知はより密度の高いトースト（既定 400×104）になり、余白いっぱいに伸びません。生成済みの 980×180 と 480×112 は自動移行し、カスタムサイズは保持されます。
- オーバーレイ移動ラベルとプレビュー文言が、ロシア語、簡体字中国語、韓国語、日本語、インドネシア語を含むすべての Relay インターフェース言語に従います。
- 言語、テーマ、アクセント、フォント倍率は Relay 設定と一緒に保存され、トレイ専用や `--startup` セッションを含む起動時に復元されます。

#### 修正

- 出力ウェイクアップおよび停止／開始の切り替え後の YouTube 再生復元を修正しました。
- 以前のショートカットを捨てずに、スキップショートカット登録の失敗を修正しました。
- キャンセルしたメディアダウンロードが履歴操作を処理中のままにする問題を修正しました。
- GIF 以外の応答を受け入れていた GIF ダウンロードを修正しました。
- 短い WebSocket 切断後に TTS とステッカーのブラウザソースがキューを失う問題を修正しました。
- サーバーが共有ステージを許可する前に TTS とステッカー出力が始まり、メディアや音楽と重なる問題を修正しました。
- Relay のポート変更後に OBS の TTS／ステッカーソースが無確認で遷移する問題を修正しました。新しいサーバーを先に確認し、タイムアウトで Browser Source が停止しないようにしました。
- 移動直後に非表示またはロックすると位置を忘れる Windows メディアウィジェットを修正しました。
- ロール情報がないためにプライバシーフィルターの除外を無視していた遅延 Discord GIF 更新を修正しました。
- ジュークボックスのキューが満杯のときに YouTube 曲の選択が破棄される問題を修正しました。
- 視覚フォールバック成功後も Windows TTS 失敗と報告していた Relay ステータスを修正しました。

#### セキュリティ

- YouTube API キーはローカルの Windows 資格情報マネージャーに保存され、保存後は再表示されません。
- YouTube 再生操作は曲をリクエストしたユーザーまたは Discord 管理者に制限されます。
- パネルとオーバーレイの Tauri ウィンドウは別の権限セットを使います。オーバーレイウィジェットはコントロールパネルコマンドを呼び出せません。
- Discord ボットが投稿するオーバーレイ URL にローカル Relay シークレットは含まれません。短いページはホスト全体の Cookie ではなく、ページ限定のシークレットを注入します。
- フィルタリングに影響しなくなった未使用のプライバシーしきい値設定を、インターフェースと設定スキーマから削除しました。

### Bahasa Indonesia

#### Ditambahkan

- Pencarian musik YouTube di channel Discord yang dapat dikonfigurasi, dengan hingga 15 hasil relevan berdurasi satu hingga lima menit.
- Pratinjau 30 detik, pemutaran penuh, antrean, dan kartu Sedang diputar untuk OBS serta widget Windows.
- Sumber browser OBS terpadu **Relay Visual** dan **Relay Audio** untuk media, stiker, notifikasi TTS, pemutaran YouTube, audio Discord, dan suara TTS. URL sumber lama tetap tersedia selama migrasi.
- Unduhan Riwayat, pintasan loncat media global yang dapat dikonfigurasi, dan panduan YouTube API berbahasa Inggris di panel Musik serta dokumentasi.
- Desain antarmuka **Gridline** dan **Lumen**, tata letak bilah sisi ringkas dan dinamis, serta pemilih desain yang dapat dilipat.
- Audit dependensi Rust di CI agar masalah tingkat tinggi yang diketahui diperiksa pada setiap verifikasi.
- Halaman **Changelog** dalam aplikasi yang menampilkan catatan rilis terbundel dalam bahasa antarmuka saat ini.

#### Diubah

- Pemutaran media, TTS, dan YouTube kini berbagi penjadwalan keluaran agar item tidak saling tumpang tindih.
- Pemutaran YouTube kini mengikuti pengaturan suara widget dan membangunkan keluaran audio Windows jika diperlukan.
- Keterangan media tetap ringkas dan tertambat pada kartu aktif, dengan visibilitas terpisah untuk OBS dan widget Windows.
- Notifikasi TTS Windows kini memakai toast yang lebih padat (default 400×104) dan tidak meregang ke ruang kosong. Ukuran generated 980×180 dan 480×112 dimigrasikan otomatis; ukuran kustom dipertahankan.
- Label geser overlay dan teks pratinjau kini mengikuti semua bahasa antarmuka Relay, termasuk Rusia, Tionghoa Sederhana, Korea, Jepang, dan Indonesia.
- Bahasa, tema, aksen, dan skala font kini disimpan bersama konfigurasi Relay dan dipulihkan saat peluncuran, termasuk sesi baki dan `--startup`.

#### Diperbaiki

- Pemulihan pemutaran YouTube setelah keluaran dibangunkan serta transisi berhenti/mulai.
- Kegagalan pendaftaran pintasan loncat tanpa membuang pintasan sebelumnya.
- Unduhan media yang dibatalkan membuat aksi Riwayat tetap sibuk.
- Unduhan GIF yang menerima respons yang bukan berkas GIF.
- Sumber browser TTS dan stiker yang kehilangan antrean setelah WebSocket terputus sebentar.
- Keluaran TTS dan stiker yang mulai sebelum server memberi izin panggung bersama, sehingga bisa tumpang tindih dengan media atau musik.
- Sumber OBS TTS dan stiker yang pindah port tanpa memeriksa server baru. Sekarang mereka men-probe instance baru dulu, dengan batas waktu, agar Browser Source tidak macet.
- Widget media Windows yang lupa posisi jika disembunyikan atau dikunci tepat setelah dipindah.
- Pembaruan GIF Discord tertunda yang mengabaikan pengecualian filter privasi karena informasi peran hilang.
- Pilihan trek YouTube yang dibuang saat antrean jukebox sudah penuh.
- Status Relay yang melaporkan kegagalan TTS Windows setelah fallback visual berhasil.

#### Keamanan

- Kunci API YouTube disimpan secara lokal di Windows Credential Manager dan tidak ditampilkan lagi setelah disimpan.
- Kontrol pemutaran YouTube dibatasi untuk pengguna yang meminta trek atau administrator Discord.
- Jendela Tauri panel dan overlay kini memakai kumpulan izin terpisah: widget overlay tidak dapat lagi memanggil perintah panel kontrol.
- URL overlay yang diposting bot Discord tidak lagi menyertakan rahasia Relay lokal. Halaman singkat menyuntikkan rahasia khusus halaman, bukan cookie untuk seluruh host.
- Pengaturan ambang privasi yang tidak terpakai dan tidak lagi memengaruhi penyaringan dihapus dari antarmuka dan skema konfigurasi.

## [1.2.7] - 2026-08-14

### English

#### Added

- Added a local settings search bar with `Ctrl+K`, keyboard-accessible results, and page-aware Back and Forward controls.
- Added regional language choices with bundled SVG flags for English (US, UK, and India), French, German, Spanish, and Latin American Spanish while preserving the complete English, French, Spanish, and German dictionaries.
- Added Russian, Simplified Chinese, Korean, Japanese, and Indonesian choices with bundled SVG flags, translated core Relay controls, moderation, privacy protection, custom commands, and the system tray.
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
- Ajout du russe, du chinois simplifié, du coréen, du japonais et de l’indonésien avec leurs drapeaux SVG embarqués, ainsi que des traductions des contrôles principaux, de la modération, de la protection de la vie privée, des commandes personnalisées et du menu de zone.
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

[Unreleased]: https://github.com/stealthsrc/relay/compare/v1.3.0...HEAD
[1.3.0]: https://github.com/stealthsrc/relay/compare/v1.2.7...v1.3.0
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
