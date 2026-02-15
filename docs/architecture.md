# Architektur-Dokumentation: Carsharing Projekt

Diese Dokumentation beschreibt die Struktur und das Zusammenspiel der Komponenten des Carsharing-Systems.

## 1. Übersicht
Das Projekt ist als **Monorepo** strukturiert und nutzt einen **Rust Workspace**, um Backend und Frontend in einem einzigen Repository zu verwalten.

### Verzeichnisstruktur
```text
carsharing/
├── backend/          # Rust Backend (API-Logik)
├── frontend/         # Rust Frontend (Yew/WebAssembly)
├── docs/             # Dokumentation (diese Datei)
└── Cargo.toml        # Workspace-Konfiguration
```

---

## 2. Komponenten

### Backend (`/backend`)
- **Technologie:** Rust
- **Aufgabe:** Verarbeitung der Geschäftslogik, Datenbankanbindung und Bereitstellung der REST-API.
- **Port:** Läuft standardmäßig auf Port `3000`.
- **Deployment:** Wird als Systemd-Service (`carsharing.service`) auf dem Server verwaltet.

### Frontend (`/frontend`)
- **Technologie:** Rust mit dem **Yew-Framework**.
- **Kompilierung:** Wird zu WebAssembly (WASM) kompiliert.
- **Auslieferung:** Die statischen Dateien (`index.html`, `.wasm`, `.js`) liegen im Ordner `dist/` und werden direkt vom Nginx-Webserver ausgeliefert.

---

## 3. Infrastruktur & Deployment

### Webserver (Nginx)
Nginx fungiert als Reverse-Proxy und statischer Datei-Server:
1. **Statische Dateien:** Anfragen an `codeboarden.de/` werden direkt aus dem Pfad `frontend/dist/` bedient.
2. **API-Anfragen:** Anfragen an `codeboarden.de/api/` werden intern an das Backend (Port 3000) weitergeleitet.

### Server-Umgebung
- **Betriebssystem:** Ubuntu 24.04 LTS
- **Deployment-Pfad:** `/var/www/codeboarden.de/carsharing-backend`
- **Automatisierung:** Automatischer Neustart des Backends bei Absturz durch Systemd.

---

## 4. Kommunikation (Ablauf)
1. Der **Nutzer** ruft `codeboarden.de` auf.
2. **Nginx** sendet die WebAssembly-App (Frontend) an den Browser.
3. Das **Frontend** interagiert über HTTP-Requests (`/api/...`) mit dem **Backend**.
4. Das **Backend** verarbeitet die Daten und sendet Antworten im JSON-Format zurück.
