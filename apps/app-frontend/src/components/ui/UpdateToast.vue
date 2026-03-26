<script setup lang="ts">
import {
	CheckCircleIcon,
	DownloadIcon,
	ExternalIcon,
	RefreshCwIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { ButtonStyled, commonMessages, ProgressBar } from '@modrinth/ui'
import { formatBytes } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed } from 'vue'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	(e: 'close' | 'restart' | 'download'): void
}>()

const props = defineProps<{
	version: string
	size: number | null
	metered: boolean
	progress: number
	status: 'available' | 'downloading' | 'downloaded'
}>()

const normalizedProgress = computed(() => Math.max(0, Math.min(1, props.progress ?? 0)))
const progressPercent = computed(() => Math.round(normalizedProgress.value * 100))
const canClose = computed(() => props.status !== 'downloading')
const titleMessage = computed(() => {
	if (props.status === 'downloaded') return messages.downloadCompleteTitle
	if (props.status === 'downloading') return messages.downloadingTitle
	return messages.title
})
const bodyMessage = computed(() => {
	if (props.status === 'downloaded') return messages.downloadedBody
	if (props.status === 'downloading') return messages.downloadingBody
	return props.metered ? messages.meteredBody : messages.autoDownloadBody
})

const messages = defineMessages({
	title: {
		id: 'app.update-toast.title',
		defaultMessage: 'Update available',
	},
	autoDownloadBody: {
		id: 'app.update-toast.body.auto-download',
		defaultMessage:
			'Revoria v{version} is being prepared in the background. We will let you know when it is ready to install.',
	},
	meteredBody: {
		id: 'app.update-toast.body.metered',
		defaultMessage:
			'Revoria v{version} is available. Since you are on a metered network, the update was not downloaded automatically.',
	},
	download: {
		id: 'app.update-toast.download',
		defaultMessage: 'Download ({size})',
	},
	downloadUnknownSize: {
		id: 'app.update-toast.download.unknown-size',
		defaultMessage: 'Download',
	},
	downloadingTitle: {
		id: 'app.update-toast.title.downloading',
		defaultMessage: 'Downloading update',
	},
	downloadingBody: {
		id: 'app.update-toast.body.downloading',
		defaultMessage: 'Downloading Revoria v{version}. You can keep using the launcher while this finishes.',
	},
	downloadingProgress: {
		id: 'app.update-toast.progress',
		defaultMessage: '{percent}% downloaded',
	},
	changelog: {
		id: 'app.update-toast.changelog',
		defaultMessage: 'Changelog',
	},
	downloadCompleteTitle: {
		id: 'app.update-toast.title.download-complete',
		defaultMessage: 'Update ready',
	},
	downloadedBody: {
		id: 'app.update-toast.body.download-complete',
		defaultMessage:
			'Revoria v{version} has finished downloading. Restart now to apply the update, or install it automatically when you close the launcher.',
	},
	restart: {
		id: 'app.update-toast.restart',
		defaultMessage: 'Restart now',
	},
	backgroundDownload: {
		id: 'app.update-toast.background-download',
		defaultMessage: 'Downloading in background',
	},
})
</script>

<template>
	<div
		class="fixed top-[--top-bar-height] right-6 z-10 mt-6 w-[26rem] rounded-2xl border border-[--glass-border] bg-[--color-glass-bg-strong] p-4 shadow-[--glass-shadow]"
		:class="{
			'update-toast--downloading': status === 'downloading',
			'update-toast--downloaded': status === 'downloaded',
		}"
	>
		<div class="flex items-start gap-3">
			<div
				class="update-toast__icon mt-0.5 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
			>
				<CheckCircleIcon v-if="status === 'downloaded'" />
				<SpinnerIcon v-else-if="status === 'downloading'" class="animate-spin" />
				<DownloadIcon v-else />
			</div>
			<div class="min-w-0 grow">
				<div class="flex items-start gap-3">
					<div class="min-w-0 grow">
						<h2 class="m-0 text-base font-semibold text-contrast">
							{{ formatMessage(titleMessage) }}
						</h2>
						<p class="mb-0 mt-2 text-sm text-primary">
							{{ formatMessage(bodyMessage, { version }) }}
						</p>
					</div>
					<ButtonStyled v-if="canClose" size="small" circular>
						<button v-tooltip="formatMessage(commonMessages.closeButton)" @click="emit('close')">
							<XIcon />
						</button>
					</ButtonStyled>
				</div>

				<div v-if="status === 'downloading'" class="mt-4">
					<ProgressBar :progress="normalizedProgress" class="max-w-[unset]" />
					<div class="mt-2 flex items-center justify-between text-xs text-secondary">
						<span>{{ formatMessage(messages.backgroundDownload) }}</span>
						<span>{{ formatMessage(messages.downloadingProgress, { percent: progressPercent }) }}</span>
					</div>
				</div>

				<div class="mt-4 flex gap-2">
					<ButtonStyled color="brand">
						<button
							v-if="status === 'available'"
							@click="emit('download')"
						>
							<DownloadIcon />
							{{
								size == null
									? formatMessage(messages.downloadUnknownSize)
									: formatMessage(messages.download, { size: formatBytes(size) })
							}}
						</button>
						<button v-else-if="status === 'downloaded'" @click="emit('restart')">
							<RefreshCwIcon /> {{ formatMessage(messages.restart) }}
						</button>
						<button v-else disabled>
							<SpinnerIcon class="animate-spin" />
							{{ formatMessage(messages.downloadingTitle) }}
						</button>
					</ButtonStyled>
					<ButtonStyled>
						<a href="https://github.com/imsawiq/Revoria/releases/latest">
							{{ formatMessage(messages.changelog) }} <ExternalIcon />
						</a>
					</ButtonStyled>
				</div>
			</div>
		</div>
	</div>
</template>

<style scoped>
.update-toast__icon {
	background: color-mix(in srgb, var(--color-brand) 15%, transparent);
	color: var(--color-brand);
	border: 1px solid color-mix(in srgb, var(--color-brand) 30%, transparent);
}

.update-toast--downloading .update-toast__icon {
	background: color-mix(in srgb, var(--color-blue) 16%, transparent);
	color: var(--color-blue);
	border-color: color-mix(in srgb, var(--color-blue) 34%, transparent);
}

.update-toast--downloaded .update-toast__icon {
	background: color-mix(in srgb, var(--color-green) 16%, transparent);
	color: var(--color-green);
	border-color: color-mix(in srgb, var(--color-green) 34%, transparent);
}
</style>
