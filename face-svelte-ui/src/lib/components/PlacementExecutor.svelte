<script lang="ts">
	// Launch directives (Architecture §26.4): the Brain decides,
	// the face's own shell executes — on this machine, in the
	// user's session. Each directive is forwarded to the shell
	// bridge once per id from this surface; the shell dedupes
	// across surfaces by the same id, and logs every launch it
	// performs. Without a shell (browser dev) the directive is
	// only logged here.
	import { getTosState } from '$lib/stores/ipc.svelte';

	const tosState = $derived(getTosState());
	let actedId = $state<string | null>(null);

	$effect(() => {
		const directive = tosState.placement_directive ?? null;
		if (!directive || directive.id === actedId) return;
		actedId = directive.id;
		const wk = (
			window as unknown as {
				webkit?: { messageHandlers?: { tosFace?: { postMessage: (m: string) => void } } };
			}
		).webkit;
		if (wk?.messageHandlers?.tosFace) {
			wk.messageHandlers.tosFace.postMessage(`launch:${JSON.stringify(directive)}`);
		} else {
			console.log('[Placement] launch directive (no face shell to execute it):', directive);
		}
	});
</script>
