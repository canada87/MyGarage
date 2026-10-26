# MyGarage

Web app self-hosted per tenere sotto controllo le scadenze delle auto private: revisione, RCA,
bollo, tagliando, batteria, cambio gomme e qualsiasi altra cosa ricorrente. Pensata per un
homelab: un solo container leggero (~15 MB di RAM), nessun account, nessun servizio esterno.

## Funzioni

- **Più auto**, nuove o usate: per inserirne una bastano nome e km attuali.
- **Scadenze a tempo, a km o entrambe** ("ogni 12 mesi o 15.000 km, quello che arriva prima"),
  oppure a data fissa ogni anno (gomme invernali entro il 15/11). Modelli pronti e modificabili.
- **Stato a colpo d'occhio**: scadute, in arrivo (default 30 giorni / 2.000 km prima), in regola,
  da completare.
- **Interventi e spese**: segnare fatta una scadenza registra l'intervento, il costo e i km, e
  ricalcola la prossima. Tutto si modifica o si elimina, con "Annulla".
- **Documenti**: libretto, polizze, fatture (anche allegate all'intervento), PDF o foto.
- **Gomme**: treni estivi/invernali, montaggi e km percorsi, stimati dalle letture del
  contachilometri quando non li sai.
- **Numeri utili**: assicurazione, soccorso stradale, officina, con chiamata in un tap.
- **Promemoria via email** (Gmail o altro SMTP) con l'evento `.ics` da aggiungere al calendario.
- **Dashboard** globale e per auto, tema chiaro/scuro, interfaccia pensata per il telefono e
  installabile come app (PWA).

## Avvio rapido con Docker

```bash
mkdir mygarage && cd mygarage
# scarica docker-compose.yml e .env.example dal repository, poi:
cp .env.example .env      # imposta porta, URL e percorsi
docker compose pull && docker compose up -d
```

L'immagine è pubblicata su GitHub Container Registry (`ghcr.io/canada87/mygarage`) a ogni
release. Per aggiornare: `docker compose pull && docker compose up -d`.

Poi apri `http://<host>:<porta>`.

| Variabile | Default | Descrizione |
|---|---|---|
| `MYGARAGE_IMAGE` | `ghcr.io/canada87/mygarage:latest` | Immagine da usare; fissa una versione (es. `:0.1.0`) per aggiornare solo quando vuoi |
| `MYGARAGE_PORT` | `8080` | Porta esposta sull'host |
| `MYGARAGE_BASE_URL` | `http://localhost:8080` | URL usato nei link delle email |
| `MYGARAGE_DATA_DIR` | `./data` | Database SQLite e impostazioni. **Tienilo su disco locale**, non su una share di rete (SQLite su SMB/NFS rischia lock e corruzione) |
| `MYGARAGE_DOCS_DIR` | `./documents` | Documenti caricati. Può stare su un NAS |
| `TZ` | `Europe/Rome` | Fuso orario per il controllo giornaliero delle scadenze |

Il compose funziona anche incollato come stack in Portainer.

### Email con Gmail

1. Attiva la verifica in due passaggi sull'account Google.
2. Crea una **password per le app** (Account Google → Sicurezza → Password per le app).
3. In MyGarage → Impostazioni inserisci indirizzo Gmail, password per le app e destinatari,
   salva e premi "Invia email di prova".

Il controllo parte una volta al giorno all'ora scelta. Per ogni scadenza arriva un'email quando
entra "in arrivo", una il giorno della scadenza e un sollecito a settimana finché resta scaduta.

### Installarla sul telefono

Chrome permette di installare una web app solo se è servita in **HTTPS** (o da `localhost`).
Su `http://indirizzo-lan:porta` l'app funziona ma non si installa. Il modo più semplice, se usi
Tailscale, è pubblicarla con un certificato valido del tuo tailnet:

```bash
sudo tailscale serve --bg --https=443 http://localhost:8080
```

e aprire `https://<nome-macchina>.<tailnet>.ts.net` dal telefono → menu di Chrome →
"Installa app". In alternativa va bene un reverse proxy (nginx, Caddy…) con un certificato.

### Backup

Tutto lo stato è in `MYGARAGE_DATA_DIR/mygarage.db` (più i file in `MYGARAGE_DOCS_DIR`).
Per una copia coerente a caldo: `sqlite3 mygarage.db ".backup mygarage-backup.db"`.

## Sicurezza

MyGarage **non ha login**: è pensata per essere raggiungibile solo dalla rete di casa o da una
VPN (es. Tailscale). Non esporla direttamente su Internet.

## Release

Il workflow [`.github/workflows/release.yml`](.github/workflows/release.yml) parte quando si
pubblica un tag di versione: esegue i test, costruisce l'immagine e la carica su GHCR con i tag
`X.Y.Z`, `X.Y` e `latest`.

```bash
git tag v0.1.0
git push origin v0.1.0
```

Si può anche creare la release dalla pagina GitHub ("Draft a new release" con un nuovo tag
`vX.Y.Z`): il push del tag avvia lo stesso workflow.

Per costruire l'immagine in locale: `docker build -t mygarage .` e `MYGARAGE_IMAGE=mygarage`
nel file `.env`.

## Sviluppo

Requisiti: Rust stabile, Node 22+.

```bash
cd frontend && npm install && npm run build   # il backend incorpora frontend/build
cd ../backend && cargo run                    # http://localhost:8080
```

Per lavorare sull'interfaccia con hot reload: `cargo run` in `backend/` e `npm run dev` in
`frontend/` (Vite inoltra `/api` al backend sulla porta 8080).

```bash
cd backend && cargo test          # logica scadenze, gomme, calendario
cd frontend && npm run check      # controllo tipi Svelte/TypeScript
```

Struttura:

```
backend/    Rust (Axum + SQLx/SQLite): API REST, scheduler email, file statici incorporati
frontend/   SvelteKit (SPA statica), Svelte 5, icone Lucide
docs/       documento di progetto
mockup/     mockup HTML iniziale
```
