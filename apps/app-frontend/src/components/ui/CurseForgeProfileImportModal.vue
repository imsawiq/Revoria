<template>
	<ModalWrapper ref="modal" :header="formatMessage(messages.header)">
		<div class="flex flex-col gap-4 p-4">
			<div class="flex flex-col gap-1">
				<label class="text-sm font-semibold text-contrast">{{
					formatMessage(messages.profileCodeLabel)
				}}</label>
				<div class="iconified-input">
					<SearchIcon aria-hidden="true" />
					<input
						ref="codeInput"
						v-model="profileCode"
						autocomplete="off"
						spellcheck="false"
						type="text"
						:placeholder="formatMessage(messages.profileCodePlaceholder)"
						maxlength="20"
						@keyup.enter="metadata ? importProfile() : fetchMetadata()"
					/>
					<Button v-if="profileCode" class="r-btn" @click="() => (profileCode = '')">
						<XIcon />
					</Button>
				</div>
			</div>

			<div v-if="metadata && !importing" class="rounded-xl bg-[--color-bg] p-3">
				<p class="m-0 text-sm">
					<span class="font-semibold text-contrast">{{ formatMessage(messages.nameLabel) }}</span>
					{{ metadata.name }}
				</p>
			</div>

			<div v-if="error" class="rounded-xl bg-red-500/10 p-3 text-sm font-medium text-red-400">
				{{ error }}
			</div>

			<div v-if="importing" class="flex flex-col gap-2">
				<div class="flex items-center justify-between text-sm">
					<span class="text-secondary">{{ importProgress.message }}</span>
					<span class="font-semibold text-contrast">
						{{ Math.floor(importProgress.percentage) }}%
					</span>
				</div>
				<div class="h-1 w-full overflow-hidden rounded-full bg-[--color-button-bg]">
					<div
						class="h-full rounded-full bg-[--color-brand] transition-[width] duration-300"
						:style="{ width: `${importProgress.percentage}%` }"
					/>
				</div>
			</div>

			<div class="flex justify-end gap-2">
				<Button @click="hide" :disabled="importing">
					<XIcon />
					{{ formatMessage(messages.cancel) }}
				</Button>
				<Button
					v-if="!metadata"
					@click="fetchMetadata"
					:disabled="!profileCode.trim() || fetching"
					color="primary"
				>
					<SearchIcon v-if="!fetching" />
					{{ fetching ? formatMessage(messages.checking) : formatMessage(messages.checkProfile) }}
				</Button>
				<Button v-if="metadata" @click="importProfile" :disabled="importing" color="primary">
					<DownloadIcon v-if="!importing" />
					{{
						importing ? formatMessage(messages.importing) : formatMessage(messages.importProfile)
					}}
				</Button>
			</div>
		</div>
	</ModalWrapper>
</template>

<script setup>
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { loading_listener } from '@/helpers/events.js'
import { fetch_curseforge_profile_metadata, import_curseforge_profile } from '@/helpers/import.js'
import { DownloadIcon, SearchIcon, XIcon } from '@modrinth/assets'
import { Button, injectNotificationManager } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'curseforge-import.header',
		defaultMessage: 'Import from CurseForge Profile Code',
	},
	profileCodeLabel: {
		id: 'curseforge-import.profile-code.label',
		defaultMessage: 'Profile Code',
	},
	profileCodePlaceholder: {
		id: 'curseforge-import.profile-code.placeholder',
		defaultMessage: 'Enter CurseForge profile code',
	},
	nameLabel: { id: 'curseforge-import.name.label', defaultMessage: 'Name:' },
	cancel: { id: 'action.cancel', defaultMessage: 'Cancel' },
	checking: { id: 'curseforge-import.checking', defaultMessage: 'Checking...' },
	checkProfile: { id: 'curseforge-import.check-profile', defaultMessage: 'Check Profile' },
	importing: { id: 'curseforge-import.importing', defaultMessage: 'Importing...' },
	importProfile: { id: 'curseforge-import.import-profile', defaultMessage: 'Import Profile' },
	startingImport: {
		id: 'curseforge-import.progress.starting',
		defaultMessage: 'Starting import...',
	},
	importingProfile: {
		id: 'curseforge-import.progress.importing-profile',
		defaultMessage: 'Importing profile...',
	},
	importCompleted: {
		id: 'curseforge-import.progress.completed',
		defaultMessage: 'Import completed!',
	},
	errorFetchMetadata: {
		id: 'curseforge-import.error.fetch-metadata',
		defaultMessage: 'Failed to fetch profile information. Please check the code and try again.',
	},
	errorImportProfile: {
		id: 'curseforge-import.error.import-profile',
		defaultMessage: 'Failed to import profile. Please try again.',
	},
})

const props = defineProps({
	closeParent: {
		type: Function,
		default: null,
	},
})

const router = useRouter()
const modal = ref(null)
const codeInput = ref(null)
const profileCode = ref('')
const metadata = ref(null)
const fetching = ref(false)
const importing = ref(false)
const error = ref('')
const importProgress = ref({
	visible: false,
	percentage: 0,
	message: formatMessage(messages.startingImport),
	totalMods: 0,
	downloadedMods: 0,
})

let unlistenLoading = null
let activeLoadingBarId = null
let progressFallbackTimer = null

defineExpose({
	show: () => {
		profileCode.value = ''
		metadata.value = null
		fetching.value = false
		importing.value = false
		error.value = ''
		importProgress.value = {
			visible: false,
			percentage: 0,
			message: formatMessage(messages.startingImport),
			totalMods: 0,
			downloadedMods: 0,
		}
		modal.value?.show()

		nextTick(() => {
			setTimeout(() => {
				codeInput.value?.focus()
			}, 100)
		})

		trackEvent('CurseForgeProfileImportStart', { source: 'ImportModal' })
	},
})

const hide = () => {
	modal.value?.hide()
}

const fetchMetadata = async () => {
	if (!profileCode.value.trim()) return

	fetching.value = true
	error.value = ''

	try {
		const result = await fetch_curseforge_profile_metadata(profileCode.value.trim())
		metadata.value = result
		trackEvent('CurseForgeProfileMetadataFetched', {
			profileCode: profileCode.value.trim(),
		})
	} catch (err) {
		console.error('Failed to fetch CurseForge profile metadata:', err)
		error.value = formatMessage(messages.errorFetchMetadata)
		handleError(err)
	} finally {
		fetching.value = false
	}
}

const importProfile = async () => {
	if (!profileCode.value.trim()) return

	importing.value = true
	error.value = ''
	activeLoadingBarId = null // Reset for new import session
	importProgress.value = {
		visible: true,
		percentage: 0,
		message: formatMessage(messages.startingImport),
		totalMods: 0,
		downloadedMods: 0,
	}

	// Fallback progress timer in case loading events don't work
	progressFallbackTimer = setInterval(() => {
		if (importing.value && importProgress.value.percentage < 90) {
			// Slowly increment progress as a fallback
			importProgress.value.percentage = Math.min(90, importProgress.value.percentage + 1)
		}
	}, 1000)

	try {
		const { result, profilePath } = await import_curseforge_profile(profileCode.value.trim())

		trackEvent('CurseForgeProfileImported', {
			profileCode: profileCode.value.trim(),
		})

		hide()

		// Close the parent modal if provided
		if (props.closeParent) {
			props.closeParent()
		}

		// Navigate to the imported profile
		await router.push(`/instance/${encodeURIComponent(profilePath)}`)
	} catch (err) {
		console.error('Failed to import CurseForge profile:', err)
		error.value = formatMessage(messages.errorImportProfile)
		handleError(err)
	} finally {
		importing.value = false
		importProgress.value.visible = false
		if (progressFallbackTimer) {
			clearInterval(progressFallbackTimer)
			progressFallbackTimer = null
		}
		activeLoadingBarId = null
	}
}

onMounted(async () => {
	// Listen for loading events to update progress
	unlistenLoading = await loading_listener((event) => {
		if (!importing.value) return

		// Track the loading bar for this import session
		if (!activeLoadingBarId) {
			activeLoadingBarId = event.loader_uuid
		}

		if (event.loader_uuid !== activeLoadingBarId) return

		if (event.fraction !== null && event.fraction !== undefined) {
			const percentage = Math.min(100, Math.max(0, (event.fraction || 0) * 100))
			const message = event.message || formatMessage(messages.importingProfile)

			// Only move forward, never backward
			importProgress.value.percentage = Math.max(importProgress.value.percentage, percentage)
			importProgress.value.message = message
		} else {
			importProgress.value.percentage = 100
			importProgress.value.message = formatMessage(messages.importCompleted)
			activeLoadingBarId = null
		}
	})
})

onUnmounted(() => {
	if (unlistenLoading) {
		unlistenLoading()
	}
	if (progressFallbackTimer) {
		clearInterval(progressFallbackTimer)
	}
})
</script>
