<template>
	<ModalWrapper ref="modal" @on-hide="hide(true)">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.uploadSkinTextureTitle) }}
			</span>
		</template>
		<div class="relative">
			<div v-if="mode === 'select'" class="flex flex-col gap-3">
				<button class="skin-upload-option" @click="mode = 'file'">
					<div class="skin-upload-option__icon">
						<UploadIcon class="size-8 text-secondary" />
					</div>
					<div class="skin-upload-option__text">
						<span class="skin-upload-option__title">
							{{ formatMessage(messages.fileOptionTitle) }}
						</span>
						<span class="skin-upload-option__desc">
							{{ formatMessage(messages.fileOptionDesc) }}
						</span>
					</div>
				</button>
				<button class="skin-upload-option" @click="mode = 'username'">
					<div class="skin-upload-option__icon">
						<UserIcon class="size-8 text-secondary" />
					</div>
					<div class="skin-upload-option__text">
						<span class="skin-upload-option__title">
							{{ formatMessage(messages.usernameOptionTitle) }}
						</span>
						<span class="skin-upload-option__desc">
							{{ formatMessage(messages.usernameOptionDesc) }}
						</span>
					</div>
				</button>
			</div>

			<div v-else-if="mode === 'file'" class="flex flex-col gap-3">
				<button class="skin-upload-back" type="button" @click="mode = 'select'">
					{{ formatMessage(messages.backToOptions) }}
				</button>
				<div
					class="border-2 border-dashed border-highlight-gray rounded-xl h-[173px] flex flex-col items-center justify-center p-8 cursor-pointer bg-button-bg hover:bg-button-hover transition-colors relative"
					@click="triggerFileInput"
				>
					<p class="mx-auto mb-0 text-primary font-bold text-lg text-center flex items-center gap-2">
						<UploadIcon /> {{ formatMessage(messages.selectSkinTextureFile) }}
					</p>
					<p class="mx-auto mt-0 text-secondary text-sm text-center">
						{{ formatMessage(messages.dragAndDropOrBrowse) }}
					</p>
					<input
						ref="fileInput"
						type="file"
						accept="image/png"
						class="hidden"
						@change="handleInputFileChange"
					/>
				</div>
			</div>

			<div v-else class="flex flex-col gap-3">
				<button class="skin-upload-back" type="button" @click="mode = 'select'">
					{{ formatMessage(messages.backToOptions) }}
				</button>
				<label class="skin-upload-label">
					<span>{{ formatMessage(messages.usernameLabel) }}</span>
					<input
						v-model="usernameInput"
						type="text"
						class="input skin-upload-input"
						:placeholder="formatMessage(messages.usernamePlaceholder)"
						@keydown.enter.prevent="importByUsername"
					/>
				</label>
				<div class="flex justify-end">
					<Button
						color="green"
						:disabled="!usernameInput.trim() || isImporting"
						@click="importByUsername"
					>
						<span v-if="isImporting">{{ formatMessage(messages.importing) }}</span>
						<span v-else>{{ formatMessage(messages.importAction) }}</span>
					</Button>
				</div>
			</div>
		</div>
	</ModalWrapper>
</template>

<script setup lang="ts">
import { UploadIcon, UserIcon } from '@modrinth/assets'
import { Button, injectNotificationManager } from '@modrinth/ui'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { onBeforeUnmount, ref, watch } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { get_dragged_skin_data, get_skin_by_username } from '@/helpers/skins'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	uploadSkinTextureTitle: {
		id: 'skins.upload.modal.title',
		defaultMessage: 'Upload skin texture',
	},
	selectSkinTextureFile: {
		id: 'skins.upload.select-file',
		defaultMessage: 'Select skin texture file',
	},
	dragAndDropOrBrowse: {
		id: 'skins.upload.drag-or-browse',
		defaultMessage: 'Drag and drop or click here to browse',
	},
	fileOptionTitle: {
		id: 'skins.upload.option.file.title',
		defaultMessage: 'Upload from file',
	},
	fileOptionDesc: {
		id: 'skins.upload.option.file.desc',
		defaultMessage: 'Import a PNG skin texture from your device.',
	},
	usernameOptionTitle: {
		id: 'skins.upload.option.username.title',
		defaultMessage: 'Import by username',
	},
	usernameOptionDesc: {
		id: 'skins.upload.option.username.desc',
		defaultMessage: 'Fetch a skin from a Minecraft username.',
	},
	backToOptions: {
		id: 'skins.upload.back-to-options',
		defaultMessage: 'Back to options',
	},
	usernameLabel: {
		id: 'skins.upload.username.label',
		defaultMessage: 'Minecraft username',
	},
	usernamePlaceholder: {
		id: 'skins.upload.username.placeholder',
		defaultMessage: 'Enter a username...',
	},
	importAction: {
		id: 'skins.upload.username.import',
		defaultMessage: 'Import',
	},
	importing: {
		id: 'skins.upload.username.importing',
		defaultMessage: 'Importing...',
	},
	usernameNotFound: {
		id: 'skins.upload.username.not-found',
		defaultMessage: 'Profile not found.',
	},
	usernameFetchFailed: {
		id: 'skins.upload.username.fetch-failed',
		defaultMessage: 'Failed to fetch skin for this username.',
	},
	errorProcessingFile: {
		id: 'skins.upload.error.processing-file.title',
		defaultMessage: 'Error processing file',
	},
	failedToReadDroppedFile: {
		id: 'skins.upload.error.failed-to-read-dropped-file',
		defaultMessage: 'Failed to read the dropped file.',
	},
})

const modal = ref()
const fileInput = ref<HTMLInputElement>()
const unlisten = ref<() => void>()
const modalVisible = ref(false)
const mode = ref<'select' | 'file' | 'username'>('select')
const usernameInput = ref('')
const isImporting = ref(false)

const emit = defineEmits<{
	(e: 'uploaded', data: ArrayBuffer): void
	(e: 'canceled'): void
}>()

function show() {
	modal.value?.show()
	modalVisible.value = true
	setupDragDropListener()
}

function hide(emitCanceled = false) {
	modal.value?.hide()
	modalVisible.value = false
	cleanupDragDropListener()
	resetState()
	if (emitCanceled) {
		emit('canceled')
	}
}

function resetState() {
	if (fileInput.value) fileInput.value.value = ''
	mode.value = 'select'
	usernameInput.value = ''
	isImporting.value = false
}

function triggerFileInput() {
	fileInput.value?.click()
}

async function handleInputFileChange(e: Event) {
	const files = (e.target as HTMLInputElement).files
	if (!files || files.length === 0) {
		return
	}
	const file = files[0]
	const buffer = await file.arrayBuffer()
	await processData(buffer)
}

async function setupDragDropListener() {
	try {
		if (modalVisible.value) {
			await cleanupDragDropListener()
			unlisten.value = await getCurrentWebview().onDragDropEvent(async (event) => {
				if (event.payload.type !== 'drop') {
					return
				}

				if (!event.payload.paths || event.payload.paths.length === 0) {
					return
				}

				const filePath = event.payload.paths[0]

				try {
					const data = await get_dragged_skin_data(filePath)
					await processData(data.buffer)
				} catch (error) {
					addNotification({
						title: formatMessage(messages.errorProcessingFile),
						text:
							error instanceof Error
								? error.message
								: formatMessage(messages.failedToReadDroppedFile),
						type: 'error',
					})
				}
			})
		}
	} catch (error) {
		console.error('Failed to set up drag and drop listener:', error)
	}
}

async function cleanupDragDropListener() {
	if (unlisten.value) {
		unlisten.value()
		unlisten.value = undefined
	}
}

async function processData(buffer: ArrayBuffer) {
	emit('uploaded', buffer)
	hide()
}

async function importByUsername() {
	const username = usernameInput.value.trim()
	if (!username || isImporting.value) return

	isImporting.value = true
	try {
		const data = await get_skin_by_username(username)
		const buffer = data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength)
		await processData(buffer)
	} catch (error) {
		console.error('Failed to import skin by username:', error)
		const message =
			error instanceof Error && error.message.includes('Profile not found')
				? formatMessage(messages.usernameNotFound)
				: formatMessage(messages.usernameFetchFailed)
		addNotification({
			title: formatMessage(messages.errorProcessingFile),
			text: message,
			type: 'error',
		})
	} finally {
		isImporting.value = false
	}
}

watch(modalVisible, (isVisible) => {
	if (isVisible) {
		setupDragDropListener()
	} else {
		cleanupDragDropListener()
	}
})

onBeforeUnmount(() => {
	cleanupDragDropListener()
})

defineExpose({ show, hide })
</script>

<style scoped lang="scss">
.skin-upload-option {
	display: flex;
	width: 100%;
	align-items: center;
	gap: 0.75rem;
	border-radius: 20px;
	padding: 0.75rem;
	border: none;
	text-align: left;
	background: var(--surface-4);
	transition: filter 120ms ease, transform 120ms ease;
}

.skin-upload-option:hover {
	filter: brightness(1.1);
}

.skin-upload-option:active {
	transform: scale(0.985);
}

.skin-upload-option__icon {
	display: flex;
	height: 3.5rem;
	width: 3.5rem;
	align-items: center;
	justify-content: center;
	border-radius: 1rem;
	border: 1px solid var(--surface-5);
}

.skin-upload-option__text {
	display: flex;
	flex: 1;
	flex-direction: column;
	gap: 0.25rem;
}

.skin-upload-option__title {
	font-size: 1rem;
	font-weight: 600;
	color: var(--color-contrast);
}

.skin-upload-option__desc {
	font-size: 0.875rem;
	font-weight: 500;
	color: var(--color-secondary);
}

.skin-upload-back {
	border: none;
	background: transparent;
	color: var(--color-secondary);
	font-weight: 600;
	text-align: left;
	padding: 0;
}

.skin-upload-back:hover {
	color: var(--color-contrast);
}

.skin-upload-label {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
	color: var(--color-secondary);
	font-weight: 600;
}

.skin-upload-input {
	width: 100%;
}
</style>
