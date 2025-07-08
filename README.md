# ChronoBolt ⚡

ChronoBolt ist ein einfaches, aber leistungsstarkes Timekeeping-Tool für das Terminal. Es hilft dir, deine Aufgaben fokussiert abzuarbeiten, indem es einen vordefinierten Zeitplan aus einer JSON-Datei liest und dich mit visuellen Countdowns durch deinen Tag führt.

> **Hinweis:** Du kannst diesen Screenshot durch ein eigenes, aktuelles Bild deines Tools ersetzen! Tools wie [termtosvg](https://github.com/nbedos/termtosvg) sind super, um animierte GIFs vom Terminal zu erstellen.

---

## Features

* **TUI-Interface:** Eine saubere, terminalbasierte Oberfläche, die ohne Maus bedient wird.
* **JSON-basierte Zeitpläne:** Definiere deine Tagesstruktur flexibel in einer einfachen JSON-Datei.
* **Duale Countdowns:** Behalte sowohl die Zeit für den aktuellen Slot als auch die gesamte verbleibende Zeit im Blick.
* **Visueller Fortschritt:** Fortschrittsbalken (`Gauge`-Widgets) zeigen dir auf einen Blick, wie weit du bist.
* **Einfache Steuerung:** Pausiere, überspringe oder wiederhole Slots mit simplen Tastenkürzeln.
* **Cross-Plattform:** Dank Rust und GitHub Actions werden Binaries für Linux, macOS und Windows automatisch erstellt.

---

## Installation

Es gibt zwei Wege, ChronoBolt zu installieren:

### 1. Von GitHub Releases (Empfohlen)

Dies ist der einfachste Weg für die meisten Benutzer.

1. Gehe zur [**Releases-Seite**](https://github.com/<DEIN_BENUTZERNAME>/<DEIN_REPO_NAME>/releases) deines Projekts.
    > Ersetze `<DEIN_BENUTZERNAME>` und `<DEIN_REPO_NAME>` mit deinen GitHub-Daten!
2. Lade die neueste Version für dein Betriebssystem herunter (z.B. `chronobolt-linux-amd64`, `chronobolt-macos-amd64` oder `chronobolt-windows-amd64.exe`).
3. **Für Linux/macOS:** Mache die Datei ausführbar.

    ```bash
    chmod +x chronobolt-linux-amd64
    ```

4. (Optional) Verschiebe die Datei in ein Verzeichnis in deinem `PATH`, um sie von überall aus aufrufen zu können.

    ```bash
    # Beispiel für Linux/macOS
    sudo mv chronobolt-linux-amd64 /usr/local/bin/chronobolt
    ```

### 2. Aus dem Quellcode (Für Entwickler)

Wenn du Rust installiert hast, kannst du ChronoBolt direkt aus dem Quellcode bauen.

1. Klone das Repository:

    ```bash
    git clone https://github.com/jnmcfly/chronobolt.git
    cd chronobolt
    ```

2. Baue das Projekt im Release-Modus:

    ```bash
    cargo build --release
    ```

3. Die fertige Binary findest du danach unter `target/release/chronobolt`.

---

## Benutzung

### Starten des Programms

ChronoBolt benötigt als Argument den Pfad zu deiner Zeitplan-Datei.

```bash
# Wenn die Binary im selben Ordner liegt:
./chronobolt timetable.json

# Wenn du es global installiert hast:
chronobolt /pfad/zu/deinem/zeitplan.json

