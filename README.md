# KBO Lokaal

Een eenvoudige, lokale desktopapp om Belgische ondernemingen uit de officiële KBO Open Data te filteren op gemeente, postcode, activiteit/NACE-code en andere kenmerken. De gegevens worden lokaal verwerkt; de KBO-database wordt niet meegeleverd of opnieuw gepubliceerd.

## Voor gebruikers

1. Installeer de versie voor jouw besturingssysteem.
2. Download de officiële KBO Open Data-zip via de FOD Economie.
3. Open KBO Lokaal en klik op **Importeer KBO-zip…**.
4. Kies filters zoals gemeente of activiteit.
5. Bekijk de lijst of exporteer de volledige selectie naar CSV.

De eerste import van de volledige KBO-zip kan meerdere minuten duren en heeft vrije schijfruimte nodig.

## Functies

- lijsten maken per gemeente, postcode, straat en activiteit;
- zoeken op bedrijfsnaam, ondernemingsnummer, e-mail, telefoon of website;
- filteren op NACE-code, rechtsvorm, startdatum en status;
- leesbare Nederlandse NACE-omschrijvingen;
- totaal aantal resultaten tonen;
- volledige gefilterde lijsten exporteren als CSV;
- lokale SQLite-database, zonder upload naar een server.

## Ontwikkelen

Vereist: Node.js, Rust en de Tauri-prerequisites voor je besturingssysteem.

```bash
npm ci
npm run tauri dev
```

Voor een macOS-build:

```bash
npm run tauri build -- --bundles app
```

## GitHub Actions

De workflow in `.github/workflows/build.yml` bouwt automatisch pakketten voor macOS, Windows en Linux:

- bij een push naar `main`;
- handmatig via **Actions → Build KBO Lokaal → Run workflow**;
- bij een tag die begint met `v`, bijvoorbeeld `v0.1.0`.

De gebouwde installers verschijnen als Actions-artifacts. De workflow publiceert geen KBO-zip of SQLite-database.

## Databron en verantwoord gebruik

De applicatie is alleen een hulpmiddel om lokaal met KBO Open Data te werken. Gebruikers moeten zelf de officiële KBO-gebruiksvoorwaarden, bronvermelding, privacyregels en beperkingen rond direct marketing naleven.
