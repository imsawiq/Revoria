<script setup>
import { BoxIcon, FolderSearchIcon, TrashIcon } from '@modrinth/assets'
import { Button, injectNotificationManager, Slider } from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { ref, watch } from 'vue'

import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { purge_cache_types } from '@/helpers/cache.js'
import { get, set } from '@/helpers/settings.ts'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const settings = ref(await get())

const messages = defineMessages({
	selectNewAppDirectory: {
		id: 'settings.resource.select-new-app-directory',
		defaultMessage: 'Select a new app directory',
	},
	appDirectoryTitle: {
		id: 'settings.resource.app-directory.title',
		defaultMessage: 'App directory',
	},
	appDirectoryDescription: {
		id: 'settings.resource.app-directory.description',
		defaultMessage:
			'The directory where the launcher stores all of its files. Changes will be applied after restarting the launcher.',
	},
	purgeCacheConfirmTitle: {
		id: 'settings.resource.purge-cache.confirm.title',
		defaultMessage: 'Are you sure you want to purge the cache?',
	},
	purgeCacheConfirmDescription: {
		id: 'settings.resource.purge-cache.confirm.description',
		defaultMessage:
			'If you proceed, your entire cache will be purged. This may slow down the app temporarily.',
	},
	purgeCacheProceed: {
		id: 'settings.resource.purge-cache.proceed',
		defaultMessage: 'Purge cache',
	},
	appCacheTitle: { id: 'settings.resource.app-cache.title', defaultMessage: 'App cache' },
	appCacheDescription: {
		id: 'settings.resource.app-cache.description',
		defaultMessage:
			'The Modrinth app stores a cache of data to speed up loading. This can be purged to force the app to reload data. This may slow down the app temporarily.',
	},
	maxConcurrentDownloadsTitle: {
		id: 'settings.resource.max-concurrent-downloads.title',
		defaultMessage: 'Maximum concurrent downloads',
	},
	maxConcurrentDownloadsDescription: {
		id: 'settings.resource.max-concurrent-downloads.description',
		defaultMessage:
			'The maximum amount of files the launcher can download at the same time. Set this to a lower value if you have a poor internet connection. (app restart required to take effect)',
	},
	maxConcurrentWritesTitle: {
		id: 'settings.resource.max-concurrent-writes.title',
		defaultMessage: 'Maximum concurrent writes',
	},
	maxConcurrentWritesDescription: {
		id: 'settings.resource.max-concurrent-writes.description',
		defaultMessage:
			'The maximum amount of files the launcher can write to the disk at once. Set this to a lower value if you are frequently getting I/O errors. (app restart required to take effect)',
	},
})

watch(
	settings,
	async () => {
		const setSettings = JSON.parse(JSON.stringify(settings.value))

		if (!setSettings.custom_dir) {
			setSettings.custom_dir = null
		}

		await set(setSettings)
	},
	{ deep: true },
)

async function purgeCache() {
	await purge_cache_types([
		'project',
		'version',
		'user',
		'team',
		'organization',
		'loader_manifest',
		'minecraft_manifest',
		'categories',
		'report_types',
		'loaders',
		'game_versions',
		'donation_platforms',
		'file_update',
		'search_results',
	]).catch(handleError)
}

async function findLauncherDir() {
	const newDir = await open({
		multiple: false,
		directory: true,
		title: formatMessage(messages.selectNewAppDirectory),
	})

	if (newDir) {
		settings.value.custom_dir = newDir
	}
}
</script>

<template>
	<h2 class="m-0 text-lg font-extrabold text-contrast">
		{{ formatMessage(messages.appDirectoryTitle) }}
	</h2>
	<p class="m-0 mt-1 mb-2 leading-tight text-secondary">
		{{ formatMessage(messages.appDirectoryDescription) }}
	</p>

	<div class="m-1 my-2">
		<div class="iconified-input w-full">
			<BoxIcon />
			<input id="appDir" v-model="settings.custom_dir" type="text" class="text-input" />
			<Button class="r-btn" @click="findLauncherDir">
				<FolderSearchIcon />
			</Button>
		</div>
	</div>

	<div>
		<ConfirmModalWrapper
			ref="purgeCacheConfirmModal"
			:title="formatMessage(messages.purgeCacheConfirmTitle)"
			:description="formatMessage(messages.purgeCacheConfirmDescription)"
			:has-to-type="false"
			:proceed-label="formatMessage(messages.purgeCacheProceed)"
			:show-ad-on-close="false"
			@proceed="purgeCache"
		/>

		<h2 class="m-0 text-lg font-extrabold text-contrast">
			{{ formatMessage(messages.appCacheTitle) }}
		</h2>
		<p class="m-0 mt-1 mb-2 leading-tight text-secondary">
			{{ formatMessage(messages.appCacheDescription) }}
		</p>
	</div>
	<button id="purge-cache" class="btn min-w-max" @click="$refs.purgeCacheConfirmModal.show()">
		<TrashIcon />
		{{ formatMessage(messages.purgeCacheProceed) }}
	</button>

	<h2 class="m-0 text-lg font-extrabold text-contrast mt-4">
		{{ formatMessage(messages.maxConcurrentDownloadsTitle) }}
	</h2>
	<p class="m-0 mt-1 mb-2 leading-tight text-secondary">
		{{ formatMessage(messages.maxConcurrentDownloadsDescription) }}
	</p>
	<Slider
		id="max-downloads"
		v-model="settings.max_concurrent_downloads"
		:min="1"
		:max="10"
		:step="1"
	/>

	<h2 class="mt-4 m-0 text-lg font-extrabold text-contrast">
		{{ formatMessage(messages.maxConcurrentWritesTitle) }}
	</h2>
	<p class="m-0 mt-1 mb-2 leading-tight text-secondary">
		{{ formatMessage(messages.maxConcurrentWritesDescription) }}
	</p>
	<Slider id="max-writes" v-model="settings.max_concurrent_writes" :min="1" :max="50" :step="1" />
</template>
