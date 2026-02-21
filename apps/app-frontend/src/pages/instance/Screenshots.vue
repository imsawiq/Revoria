<script setup lang="ts">
import { ClipboardCopyIcon, FolderOpenIcon, ImageIcon } from '@modrinth/assets'
import { ButtonStyled, injectNotificationManager } from '@modrinth/ui'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { get_full_path } from '@/helpers/profile'
import { openPath } from '@/helpers/utils.js'

const { handleError, addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	heading: { id: 'instance.screenshots.heading', defaultMessage: 'Screenshots' },
	refresh: { id: 'action.refresh', defaultMessage: 'Refresh' },
	openFolder: { id: 'action.open-folder', defaultMessage: 'Open folder' },
	loading: { id: 'instance.screenshots.loading', defaultMessage: 'Loading screenshots...' },
	emptyTitle: { id: 'instance.screenshots.empty-title', defaultMessage: 'No screenshots yet' },
	emptyDescription: {
		id: 'instance.screenshots.empty-description',
		defaultMessage: 'Press F2 in-game to take a screenshot',
	},
	copy: { id: 'action.copy', defaultMessage: 'Copy' },
	close: { id: 'action.close', defaultMessage: 'Close' },
	copyTooltip: { id: 'instance.screenshots.copy-tooltip', defaultMessage: 'Copy screenshot' },
	errorOpenFolder: {
		id: 'instance.screenshots.error.open-folder',
		defaultMessage: 'Could not open screenshots folder.',
	},
	notificationCopiedTitle: {
		id: 'instance.screenshots.notification.copied.title',
		defaultMessage: 'Copied',
	},
	notificationCopiedText: {
		id: 'instance.screenshots.notification.copied.text',
		defaultMessage: 'Screenshot copied to clipboard',
	},
	notificationPathCopiedTitle: {
		id: 'instance.screenshots.notification.path-copied.title',
		defaultMessage: 'Path copied',
	},
	notificationPathCopiedText: {
		id: 'instance.screenshots.notification.path-copied.text',
		defaultMessage: 'Could not copy image, copied path instead',
	},
	errorCopyClipboard: {
		id: 'instance.screenshots.error.copy-clipboard',
		defaultMessage: 'Failed to copy to clipboard.',
	},
})

type DirFileEntry = {
	name: string
	path: string
	size: number
}

const props = defineProps({
	instance: {
		type: Object,
		default() {
			return {}
		},
	},
})

const screenshots = ref<DirFileEntry[]>([])
const loading = ref(true)
const screenshotsPath = ref('')
const selectedImage = ref<DirFileEntry | null>(null)

function joinPath(base: string, child: string): string {
	const sep = base.includes('\\') ? '\\' : '/'
	return base.replace(/[\\/]+$/, '') + sep + child
}

function parseScreenshotDate(name: string): string | null {
	const m = name.match(/^(\d{4})-(\d{2})-(\d{2})_(\d{2})\.(\d{2})\.(\d{2})/)
	if (!m) return null
	const [, year, month, day, hour, min, sec] = m
	const months = [
		'Jan',
		'Feb',
		'Mar',
		'Apr',
		'May',
		'Jun',
		'Jul',
		'Aug',
		'Sep',
		'Oct',
		'Nov',
		'Dec',
	]
	const monthName = months[Number(month) - 1] ?? month
	return `${Number(day)} ${monthName} ${year}, ${hour}:${min}:${sec}`
}

function getDisplayName(entry: DirFileEntry): string {
	return parseScreenshotDate(entry.name) ?? entry.name
}

async function loadScreenshots() {
	if (!props.instance?.path) return
	loading.value = true
	try {
		const fullPath = await get_full_path(props.instance.path)
		screenshotsPath.value = joinPath(fullPath, 'screenshots')
		const files: DirFileEntry[] = await invoke('plugin:utils|list_dir_files', {
			path: screenshotsPath.value,
			extensions: ['.png', '.jpg', '.jpeg', '.gif', '.bmp', '.webp'],
		})
		screenshots.value = files
	} catch (e) {
		screenshots.value = []
	} finally {
		loading.value = false
	}
}

async function openScreenshotsFolder() {
	if (!screenshotsPath.value) return
	try {
		await openPath(screenshotsPath.value)
	} catch {
		handleError(new Error(formatMessage(messages.errorOpenFolder)))
	}
}

async function copyScreenshot(entry: DirFileEntry) {
	try {
		const blob = await imageToBlob(getImageSrc(entry))
		await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })])
		addNotification({
			title: formatMessage(messages.notificationCopiedTitle),
			text: formatMessage(messages.notificationCopiedText),
			type: 'success',
		})
	} catch {
		try {
			await navigator.clipboard.writeText(entry.path)
			addNotification({
				title: formatMessage(messages.notificationPathCopiedTitle),
				text: formatMessage(messages.notificationPathCopiedText),
				type: 'success',
			})
		} catch {
			handleError(new Error(formatMessage(messages.errorCopyClipboard)))
		}
	}
}

function imageToBlob(src: string): Promise<Blob> {
	return new Promise((resolve, reject) => {
		const img = new Image()
		img.crossOrigin = 'anonymous'
		img.onload = () => {
			const canvas = document.createElement('canvas')
			canvas.width = img.naturalWidth
			canvas.height = img.naturalHeight
			const ctx = canvas.getContext('2d')!
			ctx.drawImage(img, 0, 0)
			canvas.toBlob((b) => (b ? resolve(b) : reject(new Error('toBlob failed'))), 'image/png')
		}
		img.onerror = () => reject(new Error('Failed to load image'))
		img.src = src
	})
}

function formatSize(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
	return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function getImageSrc(entry: DirFileEntry): string {
	return convertFileSrc(entry.path)
}

onMounted(() => {
	loadScreenshots()
})

onBeforeUnmount(() => {
	selectedImage.value = null
	screenshots.value = []
})

watch(
	() => props.instance?.path,
	() => {
		loadScreenshots()
	},
)
</script>

<template>
	<div class="flex flex-col gap-4">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				<ImageIcon class="w-5 h-5 text-secondary" />
				<h2 class="m-0 text-lg font-extrabold text-contrast">
					{{ formatMessage(messages.heading) }}
					<span v-if="!loading" class="text-sm font-normal text-secondary"
						>({{ screenshots.length }})</span
					>
				</h2>
			</div>
			<div class="flex gap-2">
				<ButtonStyled>
					<button @click="loadScreenshots">{{ formatMessage(messages.refresh) }}</button>
				</ButtonStyled>
				<ButtonStyled type="outlined">
					<button @click="openScreenshotsFolder">
						<FolderOpenIcon class="w-4 h-4" />
						{{ formatMessage(messages.openFolder) }}
					</button>
				</ButtonStyled>
			</div>
		</div>

		<div v-if="loading" class="flex items-center justify-center py-12 text-secondary">
			{{ formatMessage(messages.loading) }}
		</div>

		<div
			v-else-if="screenshots.length === 0"
			class="flex flex-col items-center justify-center py-12 gap-3"
		>
			<ImageIcon class="w-12 h-12 text-secondary opacity-50" />
			<p class="m-0 text-secondary text-sm">{{ formatMessage(messages.emptyTitle) }}</p>
			<p class="m-0 text-secondary text-xs">{{ formatMessage(messages.emptyDescription) }}</p>
		</div>

		<template v-else>
			<!-- Lightbox -->
			<Teleport to="body">
				<Transition name="fade">
					<div
						v-if="selectedImage"
						class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/80 backdrop-blur-sm cursor-pointer"
						@click.self="selectedImage = null"
					>
						<div class="relative max-w-[90vw] max-h-[90vh] flex flex-col items-center gap-3">
							<img
								:src="getImageSrc(selectedImage)"
								:alt="selectedImage.name"
								class="max-w-full max-h-[80vh] rounded-xl shadow-2xl object-contain"
							/>
							<div
								class="flex items-center gap-3 bg-[--color-raised-bg] rounded-xl px-4 py-2 border border-[--glass-border]"
							>
								<span class="text-sm text-contrast font-medium">{{
									getDisplayName(selectedImage)
								}}</span>
								<span class="text-xs text-secondary">{{ formatSize(selectedImage.size) }}</span>
								<ButtonStyled>
									<button class="text-xs" @click.stop="copyScreenshot(selectedImage!)">
										<ClipboardCopyIcon class="w-3.5 h-3.5" />
										{{ formatMessage(messages.copy) }}
									</button>
								</ButtonStyled>
								<ButtonStyled>
									<button class="text-xs" @click.stop="selectedImage = null">
										{{ formatMessage(messages.close) }}
									</button>
								</ButtonStyled>
							</div>
						</div>
					</div>
				</Transition>
			</Teleport>

			<!-- Grid -->
			<div class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3">
				<div
					v-for="screenshot in screenshots"
					:key="screenshot.path"
					class="group relative rounded-xl overflow-hidden border border-[--glass-border] bg-[--color-button-bg] cursor-pointer transition-all hover:border-[--color-brand] hover:shadow-lg"
					@click="selectedImage = screenshot"
				>
					<div class="aspect-video overflow-hidden">
						<img
							:src="getImageSrc(screenshot)"
							:alt="screenshot.name"
							class="w-full h-full object-cover transition-transform group-hover:scale-105"
							loading="lazy"
						/>
					</div>
					<div class="p-2 flex items-center justify-between gap-1">
						<div class="flex flex-col min-w-0">
							<span class="text-xs font-medium text-contrast truncate">{{
								getDisplayName(screenshot)
							}}</span>
							<span class="text-[10px] text-secondary">{{ formatSize(screenshot.size) }}</span>
						</div>
						<div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
							<button
								class="p-1 rounded-md bg-[--color-button-bg] hover:bg-[--color-button-bg-hover] border-none cursor-pointer text-secondary hover:text-contrast transition-colors"
								:title="formatMessage(messages.copyTooltip)"
								@click.stop="copyScreenshot(screenshot)"
							>
								<ClipboardCopyIcon class="w-3.5 h-3.5" />
							</button>
						</div>
					</div>
				</div>
			</div>
		</template>
	</div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
	transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
	opacity: 0;
}
</style>
