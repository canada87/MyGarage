<script lang="ts">
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import { api } from '#lib/api.ts';
	import { categories, categoryIcon, categoryLabel, intervalText, parseIntOrNull } from '#lib/format.ts';
	import { confirmDialog, toast } from '#lib/state.svelte.ts';
	import { getTheme, setTheme, type Theme } from '#lib/theme.ts';
	import type { Category, Settings, TaskType } from '#lib/types.ts';

	let s = $state<Settings | null>(null);
	let password = $state('');
	let error = $state('');
	let saving = $state(false);
	let testing = $state(false);
	let theme = $state<Theme>(getTheme());
	let types = $state<TaskType[]>([]);

	function loadTypes() {
		api.get<TaskType[]>('/task-types').then((t) => (types = t));
	}

	$effect(() => {
		api.get<Settings>('/settings')
			.then((r) => (s = r))
			.catch((e) => (error = e.message));
		loadTypes();
	});

	async function save(e: SubmitEvent) {
		e.preventDefault();
		if (!s) return;
		error = '';
		if (s.email_enabled && (!s.smtp_username || (!s.password_set && !password) || !s.email_to)) {
			return (error = 'Per attivare le email servono indirizzo Gmail, password per le app e destinatari');
		}
		saving = true;
		try {
			s = await api.put<Settings>('/settings', { ...s, smtp_password: password ? password : null });
			password = '';
			toast('Impostazioni salvate');
		} catch (err) {
			error = (err as Error).message;
		} finally {
			saving = false;
		}
	}

	async function testEmail() {
		error = '';
		testing = true;
		try {
			await api.post('/settings/test-email');
			toast('Email di prova inviata: controlla la posta');
		} catch (err) {
			error = (err as Error).message;
		} finally {
			testing = false;
		}
	}

	/* ---- Task templates ---- */
	let editing = $state<{
		id: number | null;
		name: string;
		category: Category;
		months: string;
		km: string;
		first: string;
		annual: boolean;
		day: string;
		month: string;
		suggested: boolean;
	} | null>(null);
	let tError = $state('');

	function editType(t?: TaskType) {
		tError = '';
		editing = {
			id: t?.id ?? null,
			name: t?.name ?? '',
			category: t?.category ?? 'manutenzione',
			months: t?.interval_months != null ? String(t.interval_months) : '',
			km: t?.interval_km != null ? String(t.interval_km) : '',
			first: t?.first_interval_months != null ? String(t.first_interval_months) : '',
			annual: !!t?.anchor_month,
			day: t?.anchor_day != null ? String(t.anchor_day) : '',
			month: t?.anchor_month != null ? String(t.anchor_month) : '',
			suggested: t?.suggested ?? true
		};
	}

	async function saveType(e: SubmitEvent) {
		e.preventDefault();
		if (!editing) return;
		const t = editing;
		const original = types.find((x) => x.id === t.id);
		const body = {
			name: t.name,
			category: t.category,
			icon: original && original.category === t.category ? original.icon : categoryIcon[t.category],
			interval_months: t.annual ? null : parseIntOrNull(t.months),
			interval_km: t.annual ? null : parseIntOrNull(t.km),
			first_interval_months: t.annual ? null : parseIntOrNull(t.first),
			anchor_month: t.annual ? parseIntOrNull(t.month) : null,
			anchor_day: t.annual ? parseIntOrNull(t.day) : null,
			suggested: t.suggested
		};
		try {
			if (t.id) await api.put(`/task-types/${t.id}`, body);
			else await api.post('/task-types', body);
			editing = null;
			loadTypes();
			toast('Modello salvato');
		} catch (err) {
			tError = (err as Error).message;
		}
	}

	function deleteType(t: TaskType) {
		confirmDialog({
			title: `Eliminare il modello “${t.name}”?`,
			message: 'Le scadenze già create dalle auto restano come sono. Il modello non verrà più proposto.',
			confirmLabel: 'Elimina',
			danger: true,
			onConfirm: async () => {
				await api.del(`/task-types/${t.id}`);
				loadTypes();
			}
		});
	}

	const monthNames = ['gen', 'feb', 'mar', 'apr', 'mag', 'giu', 'lug', 'ago', 'set', 'ott', 'nov', 'dic'];
</script>

<svelte:head><title>Impostazioni · MyGarage</title></svelte:head>

<div class="page-head"><h1>Impostazioni</h1></div>

{#if s}
	<form class="stack wrap" onsubmit={save}>
		<section class="card card-pad form" aria-labelledby="h-mail">
			<div class="row"><span class="tile-ico"><Icon name="mail" /></span><h2 id="h-mail">Promemoria via email</h2></div>
			<label class="check"><input type="checkbox" bind:checked={s.email_enabled} />Invia email per le scadenze in arrivo e scadute</label>
			<div class="form-grid cols-2">
				<div class="field">
					<label for="st-user">Indirizzo Gmail</label>
					<input id="st-user" class="input" type="email" autocomplete="username" bind:value={s.smtp_username} placeholder="nome@gmail.com" />
				</div>
				<div class="field">
					<label for="st-pass">Password per le app</label>
					<input
						id="st-pass"
						class="input"
						type="password"
						autocomplete="new-password"
						bind:value={password}
						placeholder={s.password_set ? '•••••••• (salvata)' : '16 caratteri'}
					/>
					<span class="hint">
						Non è la password di Gmail: creala su myaccount.google.com → Sicurezza → Password per le app (serve la verifica in due
						passaggi).
					</span>
				</div>
			</div>
			<div class="field">
				<label for="st-to">Destinatari</label>
				<input id="st-to" class="input" type="text" inputmode="email" bind:value={s.email_to} placeholder="tu@esempio.it, altro@esempio.it" />
				<span class="hint">Separati da virgola. Le email arrivano con l’evento da aggiungere al calendario di Outlook.</span>
			</div>
			<div class="form-grid cols-2">
				<div class="field">
					<label for="st-hour">Ora del controllo giornaliero</label>
					<select id="st-hour" class="input" bind:value={s.send_hour}>
						{#each Array.from({ length: 24 }, (_, i) => i) as h (h)}<option value={h}>{String(h).padStart(2, '0')}:00</option>{/each}
					</select>
				</div>
				<div class="field">
					<label for="st-from">Mittente (facoltativo)</label>
					<input id="st-from" class="input" type="email" bind:value={s.email_from} placeholder="Uguale all’indirizzo Gmail" />
				</div>
			</div>
			<details>
				<summary class="subtle">Server SMTP diverso da Gmail</summary>
				<div class="form-grid cols-2" style="margin-top:12px">
					<div class="field"><label for="st-host">Server</label><input id="st-host" class="input" bind:value={s.smtp_host} /></div>
					<div class="field">
						<label for="st-port">Porta</label>
						<input id="st-port" class="input num" type="number" bind:value={s.smtp_port} />
						<span class="hint">587 (STARTTLS) o 465 (SSL).</span>
					</div>
				</div>
			</details>
			<div class="row">
				<button class="btn" type="button" onclick={testEmail} disabled={testing || !s.password_set}>
					<Icon name="mail" size={18} />{testing ? 'Invio…' : 'Invia email di prova'}
				</button>
				{#if !s.password_set}<span class="subtle">Salva prima le credenziali.</span>{/if}
			</div>
		</section>

		<section class="card card-pad form" aria-labelledby="h-warn">
			<div class="row"><span class="tile-ico"><Icon name="clock" /></span><h2 id="h-warn">Preavviso predefinito</h2></div>
			<p class="subtle" style="margin:0">Usato per le nuove scadenze; puoi cambiarlo su ognuna.</p>
			<div class="form-grid cols-2">
				<div class="field">
					<label for="st-wd">Giorni prima</label>
					<div class="input-affix">
						<input id="st-wd" class="input num" type="number" min="0" bind:value={s.default_warn_days} /><span class="affix">gg</span>
					</div>
				</div>
				<div class="field">
					<label for="st-wk">Km prima</label>
					<div class="input-affix">
						<input id="st-wk" class="input num" type="number" min="0" step="100" bind:value={s.default_warn_km} /><span class="affix">km</span>
					</div>
				</div>
			</div>
		</section>

		{#if error}<div class="alert danger" role="alert">{error}</div>{/if}
		<div class="form-actions">
			<button class="btn btn-primary" type="submit" disabled={saving}>{saving ? 'Salvataggio…' : 'Salva impostazioni'}</button>
		</div>
	</form>
{:else if error}
	<div class="alert danger" role="alert">{error}</div>
{:else}
	<div class="skeleton" style="height:400px"></div>
{/if}

<section class="card card-pad stack wrap section" aria-labelledby="h-tpl">
	<div class="section-head" style="margin:0">
		<div class="row"><span class="tile-ico"><Icon name="calendar" /></span><h2 id="h-tpl">Modelli di scadenza</h2></div>
		<button class="btn btn-sm" type="button" onclick={() => editType()}><Icon name="plus" size={18} />Nuovo</button>
	</div>
	<p class="subtle" style="margin:0">Proposti quando aggiungi un’auto o una scadenza. Modificarli non cambia le scadenze già create.</p>
	<ul class="list" style="padding:0">
		{#each types as t (t.id)}
			<li>
				<span class="tile-ico"><Icon name={t.icon} size={18} /></span>
				<div class="grow">
					<div class="t">{t.name}{t.suggested ? '' : ' · facoltativo'}</div>
					<div class="s">{categoryLabel[t.category]} · {intervalText(t)}{t.first_interval_months ? ` · prima volta dopo ${t.first_interval_months} mesi` : ''}</div>
				</div>
				<button class="icon-btn ghost" type="button" aria-label="Modifica {t.name}" onclick={() => editType(t)}><Icon name="pencil" size={18} /></button>
				<button class="icon-btn ghost" type="button" aria-label="Elimina {t.name}" onclick={() => deleteType(t)}><Icon name="trash" size={18} /></button>
			</li>
		{/each}
	</ul>
</section>

<section class="card card-pad form wrap section" aria-labelledby="h-theme">
	<div class="row"><span class="tile-ico"><Icon name="moon" /></span><h2 id="h-theme">Aspetto</h2></div>
	<div class="segmented" role="group" aria-label="Tema">
		{#each [['system', 'Come il sistema'], ['light', 'Chiaro'], ['dark', 'Scuro']] as [k, label] (k)}
			<button
				type="button"
				aria-pressed={theme === k}
				onclick={() => {
					theme = k as Theme;
					setTheme(theme);
				}}>{label}</button
			>
		{/each}
	</div>
	<p class="subtle" style="margin:0">
		Per installare MyGarage sul telefono: apri il menu di Chrome e scegli “Aggiungi a schermata Home” o “Installa app”.
	</p>
</section>

{#if editing}
	<Modal title={editing.id ? 'Modifica modello' : 'Nuovo modello'} onclose={() => (editing = null)}>
		<form class="form" id="tpl-form" onsubmit={saveType}>
			<div class="form-grid cols-2">
				<div class="field"><label for="tp-name">Nome</label><input id="tp-name" class="input" bind:value={editing.name} required /></div>
				<div class="field">
					<label for="tp-cat">Categoria</label>
					<select id="tp-cat" class="input" bind:value={editing.category}>
						{#each categories as c (c)}<option value={c}>{categoryLabel[c]}</option>{/each}
					</select>
				</div>
			</div>
			<div class="segmented" role="group" aria-label="Ricorrenza">
				<button type="button" aria-pressed={!editing.annual} onclick={() => (editing!.annual = false)}>Intervallo</button>
				<button type="button" aria-pressed={editing.annual} onclick={() => (editing!.annual = true)}>Data fissa ogni anno</button>
			</div>
			{#if editing.annual}
				<div class="form-grid cols-2">
					<div class="field"><label for="tp-day">Giorno</label><input id="tp-day" class="input num" type="number" min="1" max="31" bind:value={editing.day} /></div>
					<div class="field">
						<label for="tp-month">Mese</label>
						<select id="tp-month" class="input" bind:value={editing.month}>
							<option value="">—</option>
							{#each monthNames as m, i (m)}<option value={String(i + 1)}>{m}</option>{/each}
						</select>
					</div>
				</div>
			{:else}
				<div class="form-grid cols-2">
					<div class="field"><label for="tp-m">Ogni (mesi)</label><input id="tp-m" class="input num" inputmode="numeric" bind:value={editing.months} /></div>
					<div class="field"><label for="tp-k">Oppure ogni (km)</label><input id="tp-k" class="input num" inputmode="numeric" bind:value={editing.km} /></div>
				</div>
				<div class="field">
					<label for="tp-f">Prima volta dopo (mesi dall’immatricolazione)</label>
					<input id="tp-f" class="input num" inputmode="numeric" bind:value={editing.first} placeholder="Es. 48 per la revisione" />
				</div>
			{/if}
			<label class="check"><input type="checkbox" bind:checked={editing.suggested} />Proponi già selezionato per le nuove auto</label>
			{#if tError}<div class="alert danger" role="alert">{tError}</div>{/if}
		</form>
		{#snippet footer()}
			<div class="form-actions">
				<button type="button" class="btn" onclick={() => (editing = null)}>Annulla</button>
				<button type="submit" form="tpl-form" class="btn btn-primary">Salva</button>
			</div>
		{/snippet}
	</Modal>
{/if}

<style>
	.wrap {
		max-width: 820px;
	}
	details summary {
		cursor: pointer;
		min-height: 40px;
		display: flex;
		align-items: center;
	}
</style>
