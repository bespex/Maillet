# Maillet-Arbeitsweise in Outlook im Web

Eine Anleitung, die den Arbeitsablauf von Maillet in Outlook im Web (Microsoft 365) möglichst nah nachbaut,
ohne IT-Freigabe und ohne Zusatzprogramme.

## Das Prinzip

| Maillet          | Outlook im Web                                   |
|------------------|--------------------------------------------------|
| Neue Mails       | **Posteingang** – alles darin wartet auf eine Entscheidung |
| Löschen          | Löschen (`Entf`)                                 |
| In Ordner (gelesen) | Quick Step **„Ablegen: …“** bzw. Archivieren  |
| Erledigt (ohne Task) | Quick Step **„Erledigt“** → Ordner `Archiv`  |
| Task anlegen     | Quick Step **„Task“** → Ordner `Tasks`           |
| In Arbeit nehmen | Quick Step **„In Arbeit“** → Ordner `Tasks/In Progress` |
| Task beendet     | Quick Step **„Task fertig“** → Ordner `Tasks/Done` |

Die Regel: **Keine Mail bleibt nach dem Lesen im Posteingang.** Jede Mail verlässt ihn über genau eine der
Aktionen oben. Outlook erzwingt das nicht – die Einstellungen in Schritt 1 machen es aber deutlich leichter.

## Schritt 1: Nichts automatisch als gelesen markieren

Sonst verschwindet der Unterschied zwischen „gesehen“ und „entschieden“.

1. Zahnrad oben rechts → **E-Mail** → **Nachrichtenbehandlung**.
2. Unter **Als gelesen markieren**: **„Elemente nicht automatisch als gelesen markieren“**.
3. **Speichern**.

Ab jetzt wird eine Mail nur gelesen, wenn eine deiner Aktionen sie dazu macht.

## Schritt 2: Ordner anlegen

1. Im Ordnerbereich links: Rechtsklick auf dein Postfach → **Neuen Ordner erstellen** → `Tasks`.
   Dieser Ordner selbst enthält die **offenen** Aufgaben.
2. Rechtsklick auf `Tasks` → **Neuen Unterordner erstellen** → `In Progress`.
3. Genauso `Done` unter `Tasks`.
4. Falls noch nicht vorhanden: Ordner `Archiv` (Outlook legt ihn meist schon an).
5. Optional Ablageordner für Mails, die nur aufbewahrt werden (z. B. `Ablage/Projekt X`).

Die englischen Namen sind Absicht: Maillet verwendet genau diese Ordner. Wer nie wechseln will, kann sie
beliebig benennen.

**Als Favoriten anheften:** Rechtsklick auf `Tasks` und `Tasks/In Progress` → **Zu Favoriten hinzufügen**.
Dann stehen Posteingang, offene Tasks und In Arbeit oben untereinander – deine drei Spalten.

## Schritt 3: Quick Steps anlegen

Zahnrad → **E-Mail** → **Quick Steps** → **Neuer Quick Step**
(oder an einer Mail: **…** → **Quick Steps** → **Quick Steps verwalten**).

Für jeden Quick Step: Namen eingeben, Aktionen in der angegebenen Reihenfolge hinzufügen, unter
**Optionale Tastenkombination** eine Taste wählen, **Speichern**.

### „Task“ – Mail wird zur Aufgabe

| Aktion | Wert |
|---|---|
| Als gelesen markieren | |
| In Ordner verschieben | `Tasks` |

Tastenkombination: `Strg+Umschalt+1`

### „In Arbeit“ – Aufgabe in Arbeit nehmen

| Aktion | Wert |
|---|---|
| In Ordner verschieben | `Tasks/In Progress` |

Tastenkombination: `Strg+Umschalt+2`

### „Task fertig“ – Aufgabe abschließen

| Aktion | Wert |
|---|---|
| In Ordner verschieben | `Tasks/Done` |

Tastenkombination: `Strg+Umschalt+3`

### „Erledigt“ – neue Mail ohne Aufgabe ablegen

| Aktion | Wert |
|---|---|
| Als gelesen markieren | |
| In Ordner verschieben | `Archiv` |

Tastenkombination: `Strg+Umschalt+4`

### Optional: „Ablegen: Projekt X“

Pro Ablageordner, den du häufig brauchst, ein eigener Quick Step:

| Aktion | Wert |
|---|---|
| Als gelesen markieren | |
| In Ordner verschieben | `Ablage/Projekt X` |

Tastenkombination: `Strg+Umschalt+5` usw. (bis 9).

**Löschen** braucht keinen Quick Step: `Entf` verschiebt nach *Gelöschte Elemente*.

## Der tägliche Ablauf

1. **Posteingang** durchgehen, von oben nach unten. Für jede Mail genau eine Entscheidung:
   - unwichtig → `Entf`
   - nur aufbewahren → **Erledigt** oder **Ablegen: …**
   - muss ich etwas tun → **Task**
   - in unter zwei Minuten erledigt → sofort erledigen, dann **Erledigt**

   Ziel: Posteingang leer.
2. **Tasks** ansehen und das Nächste mit **In Arbeit** in Arbeit nehmen.
3. **Tasks/In Progress** abarbeiten; fertige Aufgaben mit **Task fertig** nach `Tasks/Done`.

Tipp: `Tasks/In Progress` klein halten (z. B. höchstens drei Mails) – sonst ist es nur ein zweiter Posteingang.

## Was gegenüber Maillet fehlt

- Outlook **zwingt** dich nicht zur Entscheidung; gelesene Mails können im Posteingang liegen bleiben.
  Schritt 1 und das Ziel „Posteingang leer“ ersetzen den Zwang.
- Keine gemeinsame Ansicht von neuen Mails und Tasks – es sind drei Ordner nebeneinander.
- Keine Anzeige von Einmal-Codes (OTP) im Tray.

## Später zu Maillet wechseln

Die Ordner sind dieselben, die Maillet verwendet (`Tasks`, `Tasks/In Progress`, `Tasks/Done`, erledigte
neue Mails im Archiv). Gibt die IT Maillet später frei (siehe `docs/outlook.md`), übernimmt Maillet
die vorhandenen Aufgaben ohne Umbau.
