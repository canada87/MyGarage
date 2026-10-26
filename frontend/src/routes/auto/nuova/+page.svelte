<script lang="ts">
	import { goto } from '$app/navigation';
	import Icon from '#lib/components/Icon.svelte';
	import VehicleFields, { emptyVehicle, type VehicleFormData } from '#lib/components/VehicleFields.svelte';
	import DueStateFields from '#lib/components/dialogs/DueStateFields.svelte';
	import { api } from '#lib/api.ts';
	import { intervalText, parseIntOrNull, proposeFirstDue, todayIso, vehicleColors } from '#lib/format.ts';
	import { app, refresh, toast } from '#lib/state.svelte.ts';
	import type { DueMode, TaskType, Vehicle } from '#lib/types.ts';

	let step = $state(1);
	let data = $state<VehicleFormData>(emptyVehicle(vehicleColors[app.vehicles.length % vehicleColors.length]));
	let km = $state('');
	let kmDate = $state(todayIso());
	let error = $state('');
	let saving = $state(false);

	interface Row {
		type: TaskType;
		enabled: boolean;
		mode: DueMode;
		date: string;
		km: string;
	}
	let rows = $state<Row[]>([]);
	let types = $state<TaskType[]>([]);

	$effect(() => {
		api.get<TaskType[]>('/task-types').then((t) => (types = t));
	});

	/** Sensible starting point for each deadline, based on what we know of the car. */
	function prepareRows() {
		const knownStart = data.purchased_used ? '' : data.purchase_date || data.registration_date;
		rows = types.map((t) => {
			const r: Row = { type: t, enabled: t.suggested, mode: 'last', date: '', km: '' };
			const proposal = proposeFirstDue(t, data.registration_date || null);
			if (t.anchor_month) {
				r.mode = 'unknown';
			} else if (proposal) {
				r.mode = 'next';
				r.date = proposal;
			} else if (['assicurazione', 'bollo', 'abbonamento'].includes(t.category)) {
				r.mode = 'next';
			} else if (knownStart) {
				r.date = knownStart;
				r.km = '0';
			}
			return r;
		});
	}

	function next1(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		if (!data.name.trim()) return (error = 'Dai un nome all’auto');
		if (data.year && !/^\d{4}$/.test(data.year)) return (error = 'L’anno deve avere 4 cifre');
		step = 2;
		scrollTo({ top: 0 });
	}

	function next2(e: SubmitEvent) {
		e.preventDefault();
		error = '';
		if (parseIntOrNull(km) == null) return (error = 'Inserisci i km attuali del contachilometri');
		prepareRows();
		step = 3;
		scrollTo({ top: 0 });
	}

	async function create() {
		error = '';
		saving = true;
		try {
			const vehicle = await api.post<Vehicle>('/vehicles', {
				...data,
				year: parseIntOrNull(data.year),
				registration_date: data.registration_date || null,
				purchase_date: data.purchase_date || null,
				current_km: parseIntOrNull(km),
				current_km_date: kmDate,
				schedules: rows
					.filter((r) => r.enabled)
					.map((r) => {
						const date = r.date || null;
						const kmv = parseIntOrNull(r.km);
						// Nothing filled in: create it as "to complete" instead of failing.
						const mode = (r.mode === 'last' || r.mode === 'next') && !date && kmv == null ? 'unknown' : r.mode;
						return { task_type_id: r.type.id, mode, date, km: kmv };
					})
			});
			toast(`${vehicle.name} aggiunta`);
			refresh();
			await goto(`/auto/${vehicle.id}`);
		} catch (err) {
			error = (err as Error).message;
		} finally {
			saving = false;
		}
	}

	const steps = ['Dati auto', 'Chilometri', 'Scadenze'];
</script>

<svelte:head><title>Nuova auto · MyGarage</title></svelte:head>

<a class="back" href="/auto"><Icon name="chevron-left" />Auto</a>
<div class="page-head">
	<div>
		<p class="eyebrow">Passo {step} di 3</p>
		<h1>Nuova auto</h1>
	</div>
</div>

<ol class="stepper" aria-label="Avanzamento">
	{#each steps as s, i (s)}
		<li class:done={step > i + 1} class:current={step === i + 1} aria-current={step === i + 1 ? 'step' : undefined}>
			<span class="n">{#if step > i + 1}<Icon name="check" size={16} />{:else}{i + 1}{/if}</span>{s}
		</li>
	{/each}
</ol>

<div class="card wizard">
	{#if step === 1}
		<form onsubmit={next1} class="form">
			<VehicleFields bind:data />
			{#if error}<div class="alert danger" role="alert">{error}</div>{/if}
			<div class="form-actions">
				<a class="btn" href="/auto">Annulla</a>
				<button class="btn btn-primary" type="submit">Avanti<Icon name="chevron-right" /></button>
			</div>
		</form>
	{:else if step === 2}
		<form onsubmit={next2} class="form">
			<p class="muted" style="margin:0">
				Quanti km segna adesso il contachilometri? Da qui in poi l’app terrà il conto: non serve lo storico precedente.
			</p>
			<div class="form-grid cols-2">
				<div class="field">
					<label for="nv-km">Km attuali <span class="req">*</span></label>
					<div class="input-affix">
						<input id="nv-km" class="input num" type="text" inputmode="numeric" bind:value={km} required autocomplete="off" />
						<span class="affix">km</span>
					</div>
				</div>
				<div class="field">
					<label for="nv-kmdate">Letti il</label>
					<input id="nv-kmdate" class="input" type="date" bind:value={kmDate} max={todayIso()} />
				</div>
			</div>
			{#if error}<div class="alert danger" role="alert">{error}</div>{/if}
			<div class="form-actions">
				<button class="btn" type="button" onclick={() => (step = 1)}><Icon name="chevron-left" />Indietro</button>
				<button class="btn btn-primary" type="submit">Avanti<Icon name="chevron-right" /></button>
			</div>
		</form>
	{:else}
		<div class="form">
			<p class="muted" style="margin:0">
				Scegli le scadenze da seguire. Per ognuna dimmi quando è stata fatta l’ultima volta o quando scade; se non lo sai, scegli
				“Non lo so” e la completi dopo.
			</p>
			<ul class="tasks">
				{#each rows as r (r.type.id)}
					<li class:off={!r.enabled}>
						<label class="check">
							<input type="checkbox" bind:checked={r.enabled} />
							<span class="tile-ico lg"><Icon name={r.type.icon} /></span>
							<span class="tt"><span class="t">{r.type.name}</span><span class="subtle">{intervalText(r.type)}</span></span>
						</label>
						{#if r.enabled}
							<div class="state">
								<DueStateFields
									bind:mode={r.mode}
									bind:date={r.date}
									bind:km={r.km}
									compact
									usesDate={!!r.type.interval_months || !!r.type.anchor_month}
									usesKm={!!r.type.interval_km}
									annual={!!r.type.anchor_month}
								/>
							</div>
						{/if}
					</li>
				{/each}
			</ul>
			<p class="subtle" style="margin:0">Potrai aggiungere altre scadenze personalizzate dalla pagina dell’auto.</p>
			{#if error}<div class="alert danger" role="alert">{error}</div>{/if}
			<div class="form-actions">
				<button class="btn" type="button" onclick={() => (step = 2)}><Icon name="chevron-left" />Indietro</button>
				<button class="btn btn-primary" type="button" onclick={create} disabled={saving}>
					<Icon name="check" />{saving ? 'Salvataggio…' : 'Crea auto'}
				</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.back {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--fg-muted);
		font-weight: 600;
		min-height: 44px;
	}
	.stepper {
		list-style: none;
		padding: 0;
		margin: 0 0 16px;
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}
	.stepper li {
		display: flex;
		align-items: center;
		gap: 8px;
		font-weight: 600;
		font-size: 14px;
		color: var(--fg-subtle);
	}
	.stepper li + li::before {
		content: '';
		width: 20px;
		height: 2px;
		background: var(--border);
		border-radius: 2px;
	}
	.n {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		display: grid;
		place-items: center;
		border: 2px solid var(--border-strong);
		font-size: 13px;
	}
	.current {
		color: var(--fg) !important;
	}
	.current .n {
		border-color: var(--primary);
		background: var(--primary);
		color: var(--on-primary);
	}
	.done .n {
		border-color: var(--ok);
		background: var(--ok);
		color: #fff;
	}
	.wizard {
		padding: 20px;
		max-width: 760px;
	}
	.tasks {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: 10px;
	}
	.tasks li {
		border: 1px solid var(--border);
		border-radius: 14px;
		padding: 8px 14px 12px;
		display: grid;
		gap: 10px;
	}
	.tasks li.off {
		padding-bottom: 8px;
		opacity: 0.75;
	}
	.tt {
		display: grid;
		line-height: 1.3;
	}
	.t {
		font-weight: 700;
	}
	.state {
		display: grid;
		gap: 12px;
	}
</style>
