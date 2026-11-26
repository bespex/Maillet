# Maillet: Plan

Was Maillet noch bekommen soll. Stand 2026-11-26. Was gebaut ist, beschreiben README, `docs/` und die
Release-Notes.

## Brainstorm-Ideen

Die Ideen mit Kennung (A1 …) sind in `docs/Design/Funktionen.md` ausführlich beschrieben (Problem,
Nutzen, Kern, Erweiterungen); dort stehen auch die Bedienregeln aus `docs/Design/Bedienkonzept.md`.

- Suche auf dem Server (IMAP `SEARCH`, Microsoft Graph), sobald die Suche über die geladenen Mails steht.
- A1 Gruppen vervollständigen: neben Menschen und Rauschen auch Maschinen; Korrektur mit einer Taste wird
  als Regel gelernt; Zuordnung mit sichtbarer Erklärung „Warum hier?“ (DESIGN.md 5.2).
- A2 Regeln für Gruppen und Ablage, die beim Zuordnen entstehen („Immer: @bahn.example → Reisen?“).
- A3 Themen-Vorschläge auf der Karte, ein Klick übernimmt.
- A4 Bündel für Maschinen-Mails, A5 Konversationen, A6 Newsletter-Zentrale, A8 verfallende Anzeigen.
- B1 Aufgabe ausbauen: Frist erkennen, Teilaufgaben, „Warte auf“ mit Antwort-Erkennung, Nachfassen.
- B2 Wissen aus Mail (Erkannt-Chips, Text markieren → Merken), B3 Termin, B4 Kontakt aus Signatur.
- B5 Delegieren, B6 Schnellantworten, B7 Notiz an der Mail, B8 Anhänge sichern, B11 Antwortschuld.
- B10 Später (Snooze), lokal gespeichert, auch „wenn keine Antwort kommt“ (DESIGN.md 5.4).
- C Ablage als lokales Mailkonto: Ordner, Suche, Kopie oder Verschieben, Ablage-Regeln, Import,
  verschlüsselte Ablage.
- D1 Themen als Arbeitsraum, D2 Vor dem Termin.
- E1 drei Lautstärken, E2 Fokus-Modus mit Zeitplan, E3 Morgen-Briefing, E4 Tagesabschluss, E5
  Benachrichtigungen mit Aktionen (Öffnen, Erledigt, Kopieren), E6 Rückkehr nach Abwesenheit.
- F1 eine Suche für alles, F2 Personen.
- G Triage-Modus (eine Mail nach der anderen, große Entscheidungsknöpfe, Bilanz am Ende; DESIGN.md 5.3).
- H Anbindung: Kalender (CalDAV), Kommandozeile, Dateimanager, Plugins.
- I1 Board mit festen Plätzen, I2 Zustandskarten, I3 Smart-Karten, I4 Spickzettel.
- J KI-Assistenz mit lokalem Modell: J1 Prompt-Bibliothek, J2 Radar und Digest, J3 Bewertung, J4
  Kurzfassungen, J5 Erkennung von Bitten, Fragen, Fristen, J6 KI als Regelbedingung, J7 Frag deine
  Mails, J8 Schreibhilfe, J9 Kontrolle und Datenschutz.
- Mail-Stapel als Drilldown mit Wisch-Gesten (DESIGN.md 4.2).
- Offline-Lesen: Mailtexte auf Wunsch dauerhaft speichern (Vorschlag: aus, mit Einstellung).
- Antworten: nur Verweis auf den Mail-Client, oder ein einfacher eigener Editor mit Entwurf auf dem Server?
- Benachrichtigungen konfigurierbar (Absender, Ordner, Ruhezeiten).
- Tray-Menü mit ungelesenen Mails je Konto.
- Akzentfarbe aus dem Desktop-Theme übernehmen.
- Pakete: Flatpak, `.rpm`; Builds für Windows und macOS.
- OTP-Teile (Regeln, Katalog, lokale API): als Modul in Maillet weiterführen oder als eigenes Produkt
  pflegen? Eigene Katalog-Quelle statt des Repositorys von OTP Scout.
- Offene Fragen aus dem Ideen-Pool: Kopie statt Verschieben als Standard beim Ablegen für Firmenkonten;
  Aufgaben lokal oder CalDAV-kompatibel; Wissen als Datenbank oder Markdown-Dateien; ein Thema pro Mail
  genug; welches lokale KI-Modell, mit Rückfall für schwache Rechner; Zustandsmails auf dem Server
  archivieren oder nur in Maillet ausblenden.

## Geplante Umsetzungen

Die Schritte P1 und P2 setzen den gemeinsamen Rahmen der be-spex-Tray-Apps um (`../kit/PLAN.md` K9,
entschieden mit dem Nutzer am 2026-11-26, Entwürfe: https://claude.ai/artifact/HgkRkQKisYG6rGVmbtjBcD).
Gebaut ist seit 2026-11-26 das Layout: Reiter Eingang · Aufgaben · Postfächer mit `Strg+1/2/3` und
`Strg+Tab`, Kopfleiste mit Anheften und Einstellungen, Statusleiste mit Zustand und „Jetzt prüfen“,
`Strg+Z`, Pfeiltasten, Codes als betonte Karte im Raster, Rauschen als Liste darunter; dazu die Wahl
zwischen Popup und Seitenleiste für Tray-Klick und Taste. Maillets Konzept
bleibt: Es ändert sich, wo Werkzeuge stehen und wie sie bedient werden, nicht der Inhalt.

### P1 Rahmen: Rest

- **Hinweis-Banner** unter der Kopfleiste für Kontofehler (Anmeldung fehlgeschlagen, Schlüsselbund
  gesperrt) und fehlendes Tray, mit einer Aktion; heute nur im Status-Popup und rot in der Statusleiste.
- **Kartenwerkzeuge** nach K9: Erledigt bleibt der Knopf in der Ecke; die übrigen in der Reihenfolge
  Weitergeben, Umwandeln (Aufgabe), Ablegen, Rest unter `…`, sobald es „An Tasqlet“ gibt (Kit K7).
- Bausteine aus `@bespex/ui` (Kopie unter `dist/kit`), sobald das Kit `ui-v0.1.0` hat.

### P2 Fenster: Höhe und breites Lesen

Gebaut (2026-11-26): die Wahl zwischen Popup und Seitenleiste (Einstellungen → General → Open as; die
Seitenleiste liegt nur angeheftet im Vordergrund) und die Einstellungen im gemeinsamen Rahmen.
Offen:

- **Popup-Höhe nach Inhalt** in den Grenzen aus `docs/popover.md` statt fest 700 px. Ob und wie, klärt
  der Praxistest.
- **Lesen breit:** im Popup in die Bildschirmmitte, in der Seitenleiste vom Rand zur Mitte hin, mit der
  Liste links (Muster aus K9).

### P3 Suche

⌕ oder `Strg+F` öffnen eine eigene Suchseite wie in Tasqlet: Feld oben, Treffer als Karten darunter,
Esc zurück. Sucht über die geladenen Mails (Absender, Betreff, Vorschau), mit `von:`, `konto:`,
`hat:anhang`, `vor:`. Kommt nach P1 und P2.
