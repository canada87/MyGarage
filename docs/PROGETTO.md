# MyGarage — documento di progetto

Web app self-hosted per gestire le scadenze delle auto private: legali (RCA, bollo, revisione),
meccaniche (tagliando, batteria, gomme) e abbonamenti. Niente flotte, niente carburante, niente costi al km.

## 1. Principi

- **Semplice da usare dal telefono**: ogni azione frequente (aggiorna km, segna fatto, chiama soccorso) in massimo 2 tap.
- **Tutto modificabile e cancellabile**: ogni registrazione si può correggere o eliminare, con "Annulla" per qualche secondo dopo l'eliminazione.
- **Una sola schermata per ogni cosa**: la scadenza si gestisce dalla scadenza, la macchina dalla macchina. Niente menu nascosti.
- **Leggera**: un solo container, un binario, SQLite.

## 2. Stack

| Livello | Scelta | Perché |
|---|---|---|
| Backend | Rust, Axum, sqlx (SQLite), tokio | ~10–20 MB RAM, singolo binario |
| Scheduler | task tokio interno (controllo giornaliero) | niente cron esterno |
| Email | `lettre` via Gmail SMTP (smtp.gmail.com:587 STARTTLS, app password) | |
| Calendario | `.ics` allegato alle email (compatibile Outlook) | |
| Frontend | SvelteKit (adapter-static) incorporato nel binario | interfaccia ricca, bundle piccolo |
| UI | CSS con design token, icone Lucide SVG, font Plus Jakarta Sans | |
| PWA | manifest + service worker | si installa da Chrome ("Aggiungi a schermata Home"), niente APK né store: una sola codebase web |
| Auth | nessuna: raggiungibile solo da LAN/Tailscale | richiesta utente |

## 3. Deploy

Un container, immagine multi-stage (build Rust + Svelte → runtime `debian:slim` o `distroless`).

```yaml
services:
  mygarage:
    image: mygarage:latest            # build locale o registry (da decidere)
    container_name: mygarage
    restart: unless-stopped
    ports:
      - "${MYGARAGE_PORT:-8080}:8080"
    environment:
      - TZ=${TZ:-Europe/Rome}
      - MYGARAGE_BASE_URL=${MYGARAGE_BASE_URL}
    volumes:
      - ${MYGARAGE_DATA_DIR}:/data            # SQLite + config — disco locale, non su share di rete
      - ${MYGARAGE_DOCS_DIR}:/documents       # PDF e foto — può stare su NAS
```

- Il database **non** va sul NAS (SMB + SQLite = problemi di locking).
- I valori reali (porta, percorsi, host) stanno in un file `.env` locale, escluso da git: vedi `.env.example`.
- La password Gmail sta nelle impostazioni dell'app, salvata in `/data`, mai nel compose né nel repository.

## 4. Modello dati

```
vehicles            id, name, make, model, plate, vin?, year?, registration_date?,
                    purchase_date?, purchased_used, initial_km,
                    color (accento UI), photo_path?, notes?, archived

odometer_readings   id, vehicle_id, date, km, source (manual | event | ha), note?

task_types          id, name, category (legale | meccanica | abbonamento | altro), icon,
                    default_interval_months?, default_interval_km?,
                    default_first_interval_months?   ← es. revisione: prima a 48 mesi, poi 24
                    builtin

schedules           id, vehicle_id, task_type_id, title?,
  (piano scadenza)  interval_months?, interval_km?, first_interval_months?,
                    next_due_date?, next_due_km?,     ← calcolati, ma sovrascrivibili a mano
                    warn_days (def. 30), warn_km (def. 2000),
                    email_enabled, active, notes?

events              id, vehicle_id, schedule_id?, task_type_id, date, km?, cost?,
  (interventi       provider_contact_id?, notes?
   e spese)

documents           id, vehicle_id?, event_id?, title, kind (libretto | fattura | polizza |
                    foto | altro), file_path, mime, size, uploaded_at

contacts            id, vehicle_id? (null = globale), name, category (assicurazione |
                    soccorso | officina | gommista | altro), phone?, email?,
                    reference? (n. polizza, n. tessera), notes?

tyre_sets           id, vehicle_id, name, season? (estive | invernali | 4 stagioni), initial_km?,
                    brand?, model?, size?, dot?, purchase_date?, cost?, notes?, retired
tyre_mounts         id, tyre_set_id, mounted_on, mounted_km?, removed_on?, removed_km?

notification_log    id, schedule_id, kind (preavviso | scadenza | sollecito), due_ref, sent_at
settings            chiave/valore (SMTP, destinatari, orario invio, preavvisi di default)
```

Le **spese** sono il campo `cost` degli eventi: una RCA pagata è un evento "RCA" con costo, il che
rinnova anche la scadenza. La vista Spese le aggrega per anno, auto e categoria.

## 5. Inserire un'auto (nuova o usata)

Nessuna auto è data per nuova. Obbligatori solo **nome** e **km attuali**; tutto il resto è facoltativo.

1. **Dati auto**: nome, marca/modello, targa, anno, data di immatricolazione, data di acquisto,
   "acquistata usata" sì/no.
2. **Km attuali**: diventano la prima lettura del contachilometri (non serve lo storico precedente).
3. **Stato delle scadenze**: per ogni modello proposto (tagliando, revisione, RCA, ...) si sceglie:
   - **"Ultima volta fatta il … / a … km"** → la prossima scadenza viene calcolata da lì;
   - **"Prossima scadenza il … / a … km"** → la si inserisce direttamente (es. data sulla polizza);
   - **"Non lo so"** → la scadenza viene creata ma segnata "da completare" in dashboard;
   - **"Non serve"** → il modello non viene attivato per questa auto.
   Per la revisione, se si conosce solo l'immatricolazione, la proposta è calcolata da quella
   (4 anni, poi ogni 2) e va confermata.
4. **Storico facoltativo**: gli interventi passati (anche precedenti all'acquisto) si possono
   aggiungere in qualsiasi momento, con le loro fatture. Non spostano la scadenza se ne esiste già
   uno più recente.
5. **Gomme montate**: si può indicare il treno montato e, se noti, i km che aveva già.
   Altrimenti si conta da zero dai km di inserimento.

Campi aggiuntivi su `vehicles`: `purchase_date?`, `purchased_used`, `initial_km`.
Su `tyre_sets`: `initial_km?` (km già percorsi prima di entrare nell'app).

## 6. Logica delle scadenze

Una scadenza può avere un limite a **tempo**, a **km** o entrambi ("quello che arriva prima").

Stato, calcolato sull'ultima lettura km disponibile:

| Stato | Condizione |
|---|---|
| **Scaduta** | data scadenza passata (il giorno stesso è ancora "in arrivo") **oppure** km attuali ≥ km scadenza |
| **In arrivo** | mancano ≤ `warn_days` giorni **oppure** ≤ `warn_km` km (default 2000) |
| **In regola** | altrimenti |

Nessuna previsione dei km: se l'ultima lettura ha più di 45 giorni la scadenza a km mostra
"km non aggiornati" e l'app chiede di aggiornarli.

**Registrare un intervento** collegato a una scadenza la rinnova:
`prossima data = data intervento + interval_months`, `prossimi km = km intervento + interval_km`.
Se l'intervento viene cancellato, la scadenza torna allo stato precedente (ricalcolo dall'ultimo
evento rimasto).

### Modelli predefiniti (modificabili)

| Attività | Categoria | Intervallo |
|---|---|---|
| Revisione | legale | prima a 4 anni dall'immatricolazione, poi ogni 2 anni |
| RCA | legale | 12 mesi |
| Bollo | legale | 12 mesi (mese di scadenza secondo la regione) |
| Tagliando | meccanica | 12 mesi o 15.000 km |
| Batteria | meccanica | 36 mesi (controllo) |
| Cambio gomme invernali | meccanica | entro 15/11 ogni anno |
| Cambio gomme estive | meccanica | entro 15/04 ogni anno |
| Freni / distribuzione / climatizzatore | meccanica | facoltativi |
| Soccorso stradale / abbonamenti | abbonamento | 12 mesi |

## 7. Gomme

Tutti i dati dei treni sono facoltativi (basta un nome, es. "Invernali").
I km percorsi da un treno = `initial_km` (se noto) + somma su ogni montaggio di `(km smontaggio − km montaggio)`.
Se un km manca, viene ricavato dalle letture del contachilometri più vicine alla data
(interpolazione lineare tra la lettura precedente e la successiva). Per il treno montato ora si
usa l'ultima lettura. Avvisi facoltativi per età (DOT > 6 anni).

## 8. Notifiche email

- Controllo giornaliero (orario configurabile, default 08:00).
- Una email quando una scadenza entra **in arrivo**, una il **giorno della scadenza**, poi un
  **sollecito settimanale** finché resta scaduta. `notification_log` evita i doppioni.
- Le scadenze con data hanno un allegato **`.ics`** (evento di un giorno intero con promemoria)
  che Outlook propone di aggiungere al calendario. Quelle solo a km non hanno allegato.
- Attivabile per singola scadenza; pulsante "Invia email di prova" nelle impostazioni.
- Gmail: serve la verifica in due passaggi e una **password per le app**.

## 9. Schermate

Navigazione: barra in basso su telefono, barra laterale da 1024 px in su.

1. **Dashboard globale** — riepilogo (scadute / in arrivo / in regola / spese dell'anno),
   prossime scadenze di tutte le auto ordinate per urgenza, card per auto, spese per categoria.
2. **Auto** — elenco → dettaglio con schede: Panoramica, Scadenze, Storico, Gomme, Documenti,
   Contatti. In cima: targa, km attuali, "Aggiorna km".
3. **Scadenze** — tutte le scadenze, filtrabili per auto, stato e categoria.
4. **Spese** — totali per anno, auto e categoria; elenco degli interventi con costo.
5. **Rubrica** — numeri globali e per auto, chiamata con un tap.
6. **Impostazioni** — email/SMTP, modelli di scadenza, preavvisi di default, tema.

Pulsante **+** sempre visibile: Aggiorna km · Registra intervento · Carica documento.

## 10. Design system

- Tema chiaro e scuro (segue il sistema, commutabile).
- Neutri slate, accento blu `#2563EB`; colori di stato **sempre affiancati da icona e testo**:
  verde = in regola, ambra = in arrivo, rosso = scaduta.
- Font Plus Jakarta Sans, cifre tabulari per km e importi.
- Spaziature a 4/8 px, target touch ≥ 44 px, focus visibili, `prefers-reduced-motion` rispettato.
- Icone Lucide (SVG), niente emoji.

## 11. Roadmap

1. **MVP**: auto, km, scadenze con modelli, interventi/spese, dashboard, documenti, rubrica.
2. Email + `.ics`, PWA.
3. Gomme con calcolo km.
4. Integrazione Home Assistant per i km (endpoint con token oppure lettura di un sensore HA).
