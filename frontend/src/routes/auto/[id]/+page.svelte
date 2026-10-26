<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Icon from '#lib/components/Icon.svelte';
	import Plate from '#lib/components/Plate.svelte';
	import CarSilhouette from '#lib/components/CarSilhouette.svelte';
	import ScheduleCard from '#lib/components/ScheduleCard.svelte';
	import ContactList from '#lib/components/ContactList.svelte';
	import DocumentList from '#lib/components/DocumentList.svelte';
	import { api, qs } from '#lib/api.ts';
	import {
		categoryIcon,
		categoryLabel,
		daysAgo,
		documentKinds,
		fmtDate,
		fmtEuro,
		fmtKm
	} from '#lib/format.ts';
	import { app, confirmDialog, deleteWithUndo, isPending, openDialog, refresh, toast } from '#lib/state.svelte.ts';
	import type { CarEvent, Contact, DocumentFile, OdometerReading, TyreSetView, VehicleDetail } from '#lib/types.ts';

	const id = $derived(Number(page.params.id));
	const tab = $derived(page.url.searchParams.get('tab') ?? 'panoramica');

	let v = $state<VehicleDetail | null>(null);
	let events = $state<CarEvent[]>([]);
	let readings = $state<OdometerReading[]>([]);
	let docs = $state<DocumentFile[]>([]);
	let contacts = $state<Contact[]>([]);
	let tyres = $state<TyreSetView[]>([]);
	let error = $state('');
	let docKind = $state('');
	let menuOpen = $state(false);

	$effect(() => {
		void app.version;
		const vid = id;
		Promise.all([
			api.get<VehicleDetail>(`/vehicles/${vid}`),
			api.get<CarEvent[]>(`/events${qs({ vehicle_id: vid })}`),
			api.get<OdometerReading[]>(`/vehicles/${vid}/odometer`),
			api.get<DocumentFile[]>(`/documents${qs({ vehicle_id: vid })}`),
			api.get<Contact[]>(`/contacts${qs({ vehicle_id: vid })}`),
			api.get<TyreSetView[]>(`/vehicles/${vid}/tyres`)
		])
			.then(([a, b, c, d, e, f]) => {
				if (vid !== id) return;
				v = a;
				events = b;
				readings = c;
				docs = d;
				contacts = e;
				tyres = f;
				error = '';
			})
			.catch((e) => (error = e.message));
	});

	const tabs = [
		{ key: 'panoramica', label: 'Panoramica' },
		{ key: 'scadenze', label: 'Scadenze' },
		{ key: 'storico', label: 'Storico' },
		{ key: 'gomme', label: 'Gomme' },
		{ key: 'documenti', label: 'Documenti' },
		{ key: 'contatti', label: 'Numeri utili' }
	];

	function setTab(key: string) {
		const url = new URL(page.url.href);
		url.searchParams.set('tab', key);
		url.hash = '';
		goto(url, { replace: true, reset: false });
	}

	const active = $derived(v?.schedules.filter((s) => s.active && !isPending('schedule', s.id)) ?? []);
	const inactive = $derived(v?.schedules.filter((s) => !s.active && !isPending('schedule', s.id)) ?? []);
	const overview = $derived(active.filter((s) => s.status !== 'ok').concat(active.filter((s) => s.status === 'ok')).slice(0, Math.max(4, active.filter((s) => s.status !== 'ok').length)));
	const visibleEvents = $derived(events.filter((e) => !isPending('event', e.id)));
	const eventYears = $derived([...new Set(visibleEvents.map((e) => e.date.slice(0, 4)))]);
	const visibleDocs = $derived(docs.filter((d) => !isPending('document', d.id) && (!docKind || d.kind === docKind)));
	const visibleContacts = $derived(contacts.filter((c) => !isPending('contact', c.id)));
	const mounted = $derived(tyres.find((t) => t.mounted));
	const subtitle = $derived(v ? [v.make, v.model, v.year].filter(Boolean).join(' · ') : '');

	const sourceLabel: Record<string, string> = {
		manual: 'A mano',
		event: 'Da intervento',
		initial: 'Inserimento auto',
		ha: 'Home Assistant'
	};

	async function setArchived(archived: boolean) {
		if (!v) return;
		menuOpen = false;
		const { id: _id, initial_km: _k, ...rest } = $state.snapshot(v);
		await api.put(`/vehicles/${v.id}`, { ...rest, archived });
		toast(archived ? `${v.name} archiviata` : `${v.name} ripristinata`);
		refresh();
	}

	function askDelete() {
		menuOpen = false;
		if (!v) return;
		const name = v.name;
		confirmDialog({
			title: `Eliminare ${name}?`,
			message:
				'Verranno eliminati per sempre scadenze, interventi, letture km, gomme e documenti di questa auto. Se l’hai venduta e vuoi tenerne lo storico, usa “Archivia”.',
			confirmLabel: 'Elimina definitivamente',
			danger: true,
			onConfirm: async () => {
				await api.del(`/vehicles/${id}`);
				toast(`${name} eliminata`);
				refresh();
				await goto('/auto');
			}
		});
	}
</script>

<svelte:head><title>{v?.name ?? 'Auto'} · MyGarage</title></svelte:head>

<a class="back" href="/auto"><Icon name="chevron-left" />Auto</a>

{#if error}
	<div class="alert danger" role="alert"><Icon name="circle-alert" />{error}</div>
{:else if !v}
	<div class="skeleton" style="height:200px"></div>
{:else}
	<section class="card hero" style="--car:{v.color}">
		<span class="sil"><CarSilhouette /></span>
		<div class="hero-top">
			<div>
				<h1>{v.name}</h1>
				{#if subtitle}<div class="model">{subtitle}</div>{/if}
				<div class="row">
					<Plate plate={v.plate} />
					{#if v.archived}<span class="badge neutral"><Icon name="archive" size={16} />Archiviata</span>{/if}
					{#if v.purchased_used}<span class="badge neutral">Usata</span>{/if}
				</div>
			</div>
			<div class="menu-wrap">
				<button class="icon-btn" type="button" aria-label="Altre azioni" aria-expanded={menuOpen} onclick={() => (menuOpen = !menuOpen)}>
					<Icon name="more" />
				</button>
				{#if menuOpen}
					<div class="menu card" role="menu">
						<a role="menuitem" href="/auto/{v.id}/modifica"><Icon name="pencil" size={18} />Modifica dati</a>
						<button role="menuitem" type="button" onclick={() => setArchived(!v!.archived)}>
							<Icon name={v.archived ? 'unarchive' : 'archive'} size={18} />{v.archived ? 'Ripristina' : 'Archivia (venduta)'}
						</button>
						<button role="menuitem" type="button" class="danger" onclick={askDelete}><Icon name="trash" size={18} />Elimina</button>
					</div>
				{/if}
			</div>
		</div>
		<div class="hero-km">
			<div>
				<div class="stat-lbl">Contachilometri</div>
				<div class="km"><span class="km-val num">{fmtKm(v.current_km)}</span><span class="km-sub">{daysAgo(v.current_km_date)}</span></div>
			</div>
			<button class="btn btn-primary" type="button" onclick={() => openDialog({ kind: 'km', vehicleId: v!.id })}>
				<Icon name="gauge" />Aggiorna km
			</button>
		</div>
	</section>

	<div class="tabs" role="tablist" aria-label="Sezioni auto">
		{#each tabs as t (t.key)}
			<button class="chip" role="tab" type="button" aria-selected={tab === t.key} onclick={() => setTab(t.key)}>{t.label}</button>
		{/each}
	</div>

	<div class="section" role="tabpanel">
		{#if tab === 'panoramica'}
			{#if v.counts.setup}
				<button class="alert info setup-alert" type="button" onclick={() => setTab('scadenze')}>
					<Icon name="help" />
					<span>{v.counts.setup} scadenz{v.counts.setup === 1 ? 'a' : 'e'} da completare</span>
				</button>
			{/if}
			<div class="grid-2">
				<div>
					<div class="section-head">
						<h2>Scadenze</h2>
						<button class="link" type="button" onclick={() => setTab('scadenze')}>Tutte <Icon name="chevron-right" size={16} /></button>
					</div>
					{#if overview.length}
						<div class="sched-grid">{#each overview as s (s.id)}<ScheduleCard {s} />{/each}</div>
					{:else}
						<div class="card empty">
							<p>Nessuna scadenza per questa auto.</p>
							<button class="btn btn-primary" type="button" onclick={() => openDialog({ kind: 'schedule', vehicleId: v!.id })}>
								<Icon name="plus" />Nuova scadenza
							</button>
						</div>
					{/if}
				</div>
				<div class="stack">
					<div>
						<div class="section-head">
							<h2>Numeri utili</h2>
							<button class="link" type="button" onclick={() => setTab('contatti')}>Tutti <Icon name="chevron-right" size={16} /></button>
						</div>
						<ContactList contacts={visibleContacts.slice(0, 4)} vehicleId={v.id} compact />
					</div>
					<div>
						<div class="section-head">
							<h2>Gomme montate</h2>
							<button class="link" type="button" onclick={() => setTab('gomme')}>Gomme <Icon name="chevron-right" size={16} /></button>
						</div>
						<div class="card card-pad tyre-now">
							{#if mounted}
								<div class="row">
									<span class="tile-ico"><Icon name={mounted.season === 'invernali' ? 'snow' : 'tyre'} /></span>
									<div>
										<div class="t">{mounted.name}{mounted.brand ? ` · ${mounted.brand}` : ''} {mounted.model ?? ''}</div>
										<div class="subtle">{mounted.size ?? ''}{mounted.size ? ' · ' : ''}montate il {fmtDate(mounted.mounts.find((m) => !m.removed_on)?.mounted_on)}</div>
									</div>
								</div>
								<div class="km"><span class="km-val num">{fmtKm(mounted.total_km)}</span><span class="km-sub">totali{mounted.estimated ? ' (stimati)' : ''}</span></div>
							{:else}
								<p class="subtle" style="margin:0">Nessun treno di gomme registrato come montato.</p>
							{/if}
						</div>
					</div>
					<div>
						<div class="section-head">
							<h2>Documenti</h2>
							<button class="link" type="button" onclick={() => openDialog({ kind: 'document', vehicleId: v!.id })}><Icon name="plus" size={16} />Carica</button>
						</div>
						<DocumentList docs={visibleDocs.slice(0, 4)} />
					</div>
				</div>
			</div>
		{:else if tab === 'scadenze'}
			<div class="section-head">
				<h2>Scadenze attive</h2>
				<button class="btn btn-primary btn-sm" type="button" onclick={() => openDialog({ kind: 'schedule', vehicleId: v!.id })}>
					<Icon name="plus" size={18} />Nuova
				</button>
			</div>
			{#if active.length}
				<div class="sched-grid">{#each active as s (s.id)}<ScheduleCard {s} />{/each}</div>
			{:else}
				<div class="card empty"><p>Nessuna scadenza attiva.</p></div>
			{/if}
			{#if inactive.length}
				<h2 class="section" style="margin-bottom:12px">Disattivate</h2>
				<div class="sched-grid">{#each inactive as s (s.id)}<ScheduleCard {s} />{/each}</div>
			{/if}
		{:else if tab === 'storico'}
			<div class="section-head">
				<h2>Interventi e spese</h2>
				<button class="btn btn-primary btn-sm" type="button" onclick={() => openDialog({ kind: 'event', vehicleId: v!.id })}>
					<Icon name="plus" size={18} />Registra
				</button>
			</div>
			{#if visibleEvents.length}
				{#each eventYears as y (y)}
					<h3 class="year">{y}</h3>
					<ul class="card list">
						{#each visibleEvents.filter((e) => e.date.startsWith(y)) as e (e.id)}
							<li>
								<button class="ev" type="button" onclick={() => openDialog({ kind: 'event', vehicleId: v!.id, event: e })}>
									<span class="tile-ico"><Icon name={categoryIcon[e.category] ?? 'wrench'} size={18} /></span>
									<span class="grow">
										<span class="t">{e.title}</span>
										<span class="s num">{fmtDate(e.date)}{e.km != null ? ` · ${fmtKm(e.km)}` : ''}{e.provider ? ` · ${e.provider}` : ''}</span>
									</span>
									{#if e.cost_cents != null}<span class="cost num">{fmtEuro(e.cost_cents)}</span>{/if}
									<Icon name="pencil" size={16} />
								</button>
							</li>
						{/each}
					</ul>
				{/each}
			{:else}
				<div class="card empty">
					<span class="tile-ico"><Icon name="history" size={26} /></span>
					<p>Nessun intervento. Registra tagliandi, riparazioni e pagamenti (RCA, bollo…) con il loro costo.</p>
				</div>
			{/if}

			<div class="section-head section">
				<h2>Letture km</h2>
				<button class="btn btn-sm" type="button" onclick={() => openDialog({ kind: 'km', vehicleId: v!.id })}><Icon name="plus" size={18} />Aggiungi</button>
			</div>
			<ul class="card list">
				{#each readings.filter((r) => !isPending('reading', r.id)) as r (r.id)}
					<li>
						<span class="tile-ico"><Icon name="gauge" size={18} /></span>
						<div class="grow">
							<div class="t num">{fmtKm(r.km)}</div>
							<div class="s">{fmtDate(r.date)} · {sourceLabel[r.source] ?? r.source}{r.note && r.source === 'manual' ? ` · ${r.note}` : ''}</div>
						</div>
						{#if r.source === 'manual' || r.source === 'ha'}
							<button
								class="icon-btn ghost"
								type="button"
								aria-label="Elimina lettura del {fmtDate(r.date)}"
								onclick={() => deleteWithUndo('reading', r.id, 'Lettura', `/odometer/${r.id}`)}><Icon name="trash" size={18} /></button
							>
						{/if}
					</li>
				{/each}
			</ul>
		{:else if tab === 'gomme'}
			<div class="section-head">
				<h2>Treni di gomme</h2>
				<button class="btn btn-primary btn-sm" type="button" onclick={() => openDialog({ kind: 'tyreSet', vehicleId: v!.id })}>
					<Icon name="plus" size={18} />Aggiungi
				</button>
			</div>
			{#if tyres.filter((t) => !isPending('tyreSet', t.id)).length}
				<div class="sched-grid">
					{#each tyres.filter((t) => !isPending('tyreSet', t.id)) as t (t.id)}
						<article class="card card-pad tyre" class:retired={t.retired}>
							<div class="row">
								<span class="tile-ico lg"><Icon name={t.season === 'invernali' ? 'snow' : t.season === 'estive' ? 'sun' : 'tyre'} /></span>
								<div class="grow">
									<h3>{t.name}</h3>
									<div class="subtle">{[t.brand, t.model, t.size, t.dot ? `DOT ${t.dot}` : null].filter(Boolean).join(' · ') || 'Nessun dettaglio'}</div>
								</div>
								{#if t.mounted}<span class="badge ok"><Icon name="check" size={16} />Montate</span>{:else if t.retired}<span class="badge neutral">Dismesse</span>{/if}
							</div>
							<div class="km"><span class="km-val num">{fmtKm(t.total_km)}</span><span class="km-sub">percorsi{t.estimated ? ' · in parte stimati' : ''}</span></div>
							{#if t.mounts.length}
								<details>
									<summary class="subtle">Storico montaggi ({t.mounts.length})</summary>
									<ul class="mounts">
										{#each t.mounts as m (m.id)}
											<li class="num">
												{fmtDate(m.mounted_on)} → {m.removed_on ? fmtDate(m.removed_on) : 'oggi'}
												<strong>{m.distance != null ? fmtKm(m.distance) : '—'}</strong>{m.estimated ? ' (stima)' : ''}
											</li>
										{/each}
									</ul>
								</details>
							{/if}
							<div class="row">
								{#if t.mounted}
									<button class="btn btn-sm" type="button" onclick={() => openDialog({ kind: 'mount', set: t, action: 'unmount' })}>Smonta</button>
								{:else if !t.retired}
									<button class="btn btn-primary btn-sm" type="button" onclick={() => openDialog({ kind: 'mount', set: t, action: 'mount' })}>Monta</button>
								{/if}
								<button class="btn btn-sm" type="button" onclick={() => openDialog({ kind: 'tyreSet', vehicleId: v!.id, set: t })}>Modifica</button>
							</div>
						</article>
					{/each}
				</div>
			{:else}
				<div class="card empty">
					<span class="tile-ico"><Icon name="tyre" size={26} /></span>
					<p>Aggiungi le gomme per sapere quanti km hanno fatto. Basta un nome, il resto è facoltativo.</p>
				</div>
			{/if}
		{:else if tab === 'documenti'}
			<div class="section-head">
				<h2>Documenti</h2>
				<button class="btn btn-primary btn-sm" type="button" onclick={() => openDialog({ kind: 'document', vehicleId: v!.id })}>
					<Icon name="upload" size={18} />Carica
				</button>
			</div>
			<div class="filters" role="group" aria-label="Filtra per tipo">
				<button class="chip" type="button" aria-pressed={docKind === ''} onclick={() => (docKind = '')}>Tutti</button>
				{#each Object.entries(documentKinds) as [k, label] (k)}
					<button class="chip" type="button" aria-pressed={docKind === k} onclick={() => (docKind = k)}>{label}</button>
				{/each}
			</div>
			<DocumentList docs={visibleDocs} />
		{:else if tab === 'contatti'}
			<div class="section-head">
				<h2>Numeri utili</h2>
				<button class="btn btn-primary btn-sm" type="button" onclick={() => openDialog({ kind: 'contact', vehicleId: v!.id })}>
					<Icon name="plus" size={18} />Aggiungi
				</button>
			</div>
			<ContactList contacts={visibleContacts} vehicleId={v.id} />
		{/if}
	</div>
{/if}

<svelte:window onclick={(e) => menuOpen && !(e.target as Element).closest('.menu-wrap') && (menuOpen = false)} />

<style>
	.back {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--fg-muted);
		font-weight: 600;
		min-height: 44px;
	}
	.hero {
		padding: 20px;
		position: relative;
		display: grid;
		gap: 16px;
		background: linear-gradient(135deg, color-mix(in srgb, var(--car) 14%, var(--surface)) 0%, var(--surface) 60%);
	}
	.sil {
		position: absolute;
		right: 12px;
		top: 30px;
		width: 240px;
		color: var(--car);
		opacity: 0.13;
		pointer-events: none;
		overflow: hidden;
	}
	.hero-top {
		display: flex;
		justify-content: space-between;
		gap: 16px;
		align-items: flex-start;
		position: relative;
	}
	.model {
		color: var(--fg-muted);
		font-size: 14px;
		margin: 2px 0 10px;
	}
	.hero-km {
		display: flex;
		align-items: flex-end;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
		position: relative;
	}
	.stat-lbl {
		font-size: 14px;
		color: var(--fg-muted);
		font-weight: 500;
	}
	.km {
		display: flex;
		align-items: baseline;
		gap: 8px;
		flex-wrap: wrap;
	}
	.km-val {
		font-size: 26px;
		font-weight: 800;
		letter-spacing: -0.02em;
	}
	.hero .km-val {
		font-size: 34px;
	}
	.km-sub {
		font-size: 13px;
		color: var(--fg-subtle);
	}
	.menu-wrap {
		position: relative;
	}
	.menu {
		position: absolute;
		right: 0;
		top: 50px;
		z-index: 40;
		min-width: 220px;
		padding: 6px;
		display: grid;
		box-shadow: var(--shadow-lg);
	}
	.menu a,
	.menu button {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 44px;
		padding: 0 12px;
		border-radius: 10px;
		border: 0;
		background: transparent;
		font-weight: 600;
		cursor: pointer;
		text-align: left;
	}
	.menu a:hover,
	.menu button:hover {
		background: var(--surface-2);
	}
	.menu .danger {
		color: var(--danger);
	}
	.tabs {
		display: flex;
		gap: 6px;
		overflow-x: auto;
		scrollbar-width: none;
		margin: 16px -16px 0;
		padding: 0 16px;
	}
	.tabs::-webkit-scrollbar {
		display: none;
	}
	.tabs .chip {
		flex: none;
	}
	@media (min-width: 1024px) {
		.tabs {
			margin: 16px 0 0;
			padding: 0;
			flex-wrap: wrap;
		}
	}
	.setup-alert {
		width: 100%;
		border: 0;
		cursor: pointer;
		margin-bottom: 16px;
		text-align: left;
	}
	.sched-grid {
		display: grid;
		gap: 12px;
		grid-template-columns: minmax(0, 1fr);
	}
	@media (min-width: 768px) {
		.sched-grid {
			grid-template-columns: repeat(2, minmax(0, 1fr));
		}
	}
	@media (min-width: 1100px) {
		.grid-2 .sched-grid {
			grid-template-columns: minmax(0, 1fr);
		}
	}
	.tyre-now {
		display: grid;
		gap: 12px;
	}
	.t {
		font-weight: 700;
	}
	.year {
		margin: 16px 0 8px;
		color: var(--fg-subtle);
		font-size: 14px;
	}
	.list li:has(.ev) {
		padding: 0;
	}
	.ev {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px;
		border: 0;
		background: transparent;
		border-radius: 12px;
		cursor: pointer;
		text-align: left;
		color: var(--fg-subtle);
	}
	.ev:hover {
		background: var(--surface-2);
	}
	.ev .grow {
		display: grid;
		color: var(--fg);
	}
	.cost {
		font-weight: 700;
		color: var(--fg);
	}
	.tyre {
		display: grid;
		gap: 12px;
	}
	.tyre.retired {
		opacity: 0.7;
	}
	.mounts {
		margin: 8px 0 0;
		padding-left: 18px;
		font-size: 14px;
		color: var(--fg-muted);
		display: grid;
		gap: 4px;
	}
	details summary {
		cursor: pointer;
		min-height: 32px;
		display: flex;
		align-items: center;
	}
	.filters {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
		margin-bottom: 12px;
	}
</style>
