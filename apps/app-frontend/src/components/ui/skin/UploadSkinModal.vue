<template>
	<ModalWrapper ref="modal" @on-hide="hide(true)">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.uploadSkinTextureTitle) }}
			</span>
		</template>
		<div class="relative">
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
	</ModalWrapper>
</template>

<script setup lang="ts">
import { UploadIcon } from '@modrinth/assets'
import { injectNotificationManager } from '@modrinth/ui'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { onBeforeUnmount, ref, watch } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { get_dragged_skin_data } from '@/helpers/skins'

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
