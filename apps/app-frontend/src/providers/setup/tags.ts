import type { AbstractWebNotificationManager } from '@modrinth/ui'
import { provideTags } from '@modrinth/ui'
import { ref, watch, type Ref } from 'vue'

import { get_game_versions, get_loaders } from '@/helpers/tags'

export function setupTagsProvider(
	notificationManager: AbstractWebNotificationManager,
	stateReady?: Ref<boolean>,
) {
	const { handleError } = notificationManager

	const gameVersions = ref([])
	const loaders = ref([])

	provideTags({ gameVersions, loaders })

	let loading = false
	let loaded = false
	async function loadTags() {
		if (loading || loaded) return
		loading = true
		try {
			const [versions, loaderList] = await Promise.all([get_game_versions(), get_loaders()])
			gameVersions.value = versions
			loaders.value = loaderList
			loaded = true
		} catch (err) {
			handleError(err)
		} finally {
			loading = false
		}
	}

	if (stateReady) {
		watch(
			stateReady,
			(ready) => {
				if (ready) loadTags()
			},
			{ immediate: true },
		)
	} else {
		loadTags()
	}
}
