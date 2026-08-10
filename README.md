# KBO Lokaal

Een lokale Tauri-app om de KBO-database te doorzoeken. De gegevens blijven op het toestel van de gebruiker.

## Gebruiken

1. Start de app.
2. Klik op **Importeer KBO-zip…** en selecteer de officiële KBO-zip. De app bouwt de database lokaal op.
3. Vul bijvoorbeeld `Ninove` in bij **Gemeente**.
4. Klik op **Zoek bedrijflijst** of **Exporteer volledige CSV**.

De app toont het totale aantal resultaten en de eerste 100 regels.

## Ontwikkelen

```bash
npm install
npm run tauri dev
```

## Bouwen voor macOS

```bash
npm run tauri build -- --bundles app
```

De KBO-database wordt bewust niet meegeleverd in de app. Gebruikers downloaden en importeren de officiële KBO Open Data zelf. De import kan bij de volledige zip meerdere minuten duren.

## Andere platformen

De Tauri-code ondersteunt macOS, Windows en Linux. De workflow in `.github/workflows/build.yml` maakt per platform een apart installatiepakket. Bij een release moet de database als afzonderlijk bestand naast de app worden meegeleverd; de app zoekt die automatisch in dezelfde map.
