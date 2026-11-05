<script lang="ts" module>
	export interface VehicleFormData {
		name: string;
		make: string;
		model: string;
		plate: string;
		vin: string;
		year: string;
		registration_date: string;
		purchase_date: string;
		purchased_used: boolean;
		color: string;
		notes: string;
		ha_entity: string;
	}

	export function emptyVehicle(color: string): VehicleFormData {
		return {
			name: '',
			make: '',
			model: '',
			plate: '',
			vin: '',
			year: '',
			registration_date: '',
			purchase_date: '',
			purchased_used: false,
			color,
			notes: '',
			ha_entity: ''
		};
	}
</script>

<script lang="ts">
	import { api } from '#lib/api.ts';
	import { fmtKm, todayIso, vehicleColors } from '#lib/format.ts';
	import type { HaSensor, Settings } from '#lib/types.ts';

	let { data = $bindable() }: { data: VehicleFormData } = $props();

	// The Home Assistant field appears only once HA is set up in the settings.
	let haReady = $state(false);
	let sensors = $state<HaSensor[]>([]);
	let sensorsError = $state('');

	$effect(() => {
		api.get<Settings>('/settings')
			.then((s) => {
				haReady = !!s.ha_url && s.ha_token_set;
				if (!haReady) return;
				return api
					.get<HaSensor[]>('/ha/sensors')
					.then((list) => (sensors = list))
					.catch((e) => (sensorsError = e.message));
			})
			.catch(() => {});
	});

	const picked = $derived(sensors.find((x) => x.entity_id === data.ha_entity.trim()));
</script>

<div class="form">
	<div class="field">
		<label for="vf-name">Nome <span class="req">*</span></label>
		<input id="vf-name" class="input" bind:value={data.name} required placeholder="Es. Golf, Auto di Marta" autocomplete="off" />
		<span class="hint">Come vuoi chiamarla nell’app.</span>
	</div>
	<div class="form-grid cols-2">
		<div class="field"><label for="vf-make">Marca</label><input id="vf-make" class="input" bind:value={data.make} placeholder="Es. Volkswagen" /></div>
		<div class="field"><label for="vf-model">Modello</label><input id="vf-model" class="input" bind:value={data.model} placeholder="Es. Golf 1.5 TSI" /></div>
		<div class="field">
			<label for="vf-plate">Targa</label>
			<input id="vf-plate" class="input" bind:value={data.plate} placeholder="Es. AB123CD" autocapitalize="characters" style="text-transform:uppercase" />
		</div>
		<div class="field">
			<label for="vf-year">Anno</label>
			<input id="vf-year" class="input num" type="text" inputmode="numeric" maxlength="4" bind:value={data.year} placeholder="Es. 2021" />
		</div>
		<div class="field">
			<label for="vf-reg">Data di immatricolazione</label>
			<input id="vf-reg" class="input" type="date" bind:value={data.registration_date} max={todayIso()} />
			<span class="hint">Serve a calcolare la revisione. La trovi sul libretto.</span>
		</div>
		<div class="field">
			<label for="vf-vin">Numero di telaio</label>
			<input id="vf-vin" class="input" bind:value={data.vin} autocapitalize="characters" />
		</div>
	</div>

	<div class="field">
		<span class="label">L’hai comprata</span>
		<div class="segmented" role="group" aria-label="Nuova o usata">
			<button type="button" aria-pressed={!data.purchased_used} onclick={() => (data.purchased_used = false)}>Nuova</button>
			<button type="button" aria-pressed={data.purchased_used} onclick={() => (data.purchased_used = true)}>Usata</button>
		</div>
	</div>
	<div class="field">
		<label for="vf-buy">Data di acquisto</label>
		<input id="vf-buy" class="input" type="date" bind:value={data.purchase_date} max={todayIso()} />
	</div>

	<div class="field">
		<span class="label">Colore nell’app</span>
		<div class="swatches" role="radiogroup" aria-label="Colore">
			{#each vehicleColors as c (c)}
				<button
					type="button"
					role="radio"
					aria-checked={data.color.toUpperCase() === c}
					aria-label="Colore {c}"
					class="swatch"
					style="background:{c}"
					onclick={() => (data.color = c)}
				></button>
			{/each}
		</div>
	</div>
	{#if haReady || data.ha_entity}
		<div class="field">
			<label for="vf-ha">Sensore km di Home Assistant</label>
			<input
				id="vf-ha"
				class="input"
				list="vf-ha-list"
				bind:value={data.ha_entity}
				placeholder="Es. sensor.auto_odometer"
				autocomplete="off"
				autocapitalize="none"
				spellcheck="false"
			/>
			<datalist id="vf-ha-list">
				{#each sensors as x (x.entity_id)}<option value={x.entity_id}>{x.name}{x.km != null ? ` · ${fmtKm(x.km)}` : ''}</option>{/each}
			</datalist>
			<span class="hint">
				{#if picked}
					{picked.name}: ora segna {fmtKm(picked.km)}. I km si aggiornano da soli ogni ora.
				{:else if sensorsError}
					Non riesco a leggere i sensori: {sensorsError}
				{:else}
					Facoltativo. Se l’auto è in Home Assistant, i km si aggiornano da soli ogni ora. Lascia vuoto per inserirli a mano.
				{/if}
			</span>
		</div>
	{/if}
	<div class="field">
		<label for="vf-notes">Note</label>
		<textarea id="vf-notes" class="input" bind:value={data.notes}></textarea>
	</div>
</div>

<style>
	.swatches {
		display: flex;
		gap: 10px;
		flex-wrap: wrap;
	}
	.swatch {
		width: 44px;
		height: 44px;
		border-radius: 12px;
		border: 3px solid transparent;
		cursor: pointer;
		box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.35);
	}
	.swatch[aria-checked='true'] {
		border-color: var(--fg);
	}
</style>
