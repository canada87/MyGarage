<script lang="ts">
	import Icon from './Icon.svelte';
	import type { Status } from '#lib/types.ts';

	let { status, text }: { status: Status; text: string } = $props();
	const icon = $derived({ overdue: 'alert', soon: 'clock', ok: 'ok', setup: 'help' }[status]);
</script>

<span class="badge {status}"><Icon name={icon} size={16} />{text}</span>
