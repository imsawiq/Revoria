<script setup lang="ts">
import { FilePageLayout, provideFileManager } from '@modrinth/ui'
import type { EditingFile, FileItem, UploadState } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'
import { computed, onMounted, ref, watch } from 'vue'

import { get_full_path } from '@/helpers/profile'
import type { GameInstance } from '@/helpers/types'
import { highlightInFolder } from '@/helpers/utils.js'

type DirEntry = {
	name: string
	path: string
	is_dir: boolean
	size?: number | null
	modified?: number | null
}

const props = defineProps<{
	instance: GameInstance
	options?: Record<string, unknown>
	offline?: boolean
	playing?: boolean
	versions?: unknown[]
	installed?: boolean
	openSettings?: (() => void) | null
}>()

defineEmits(['play', 'stop'])

const instanceRoot = ref('')
const items = ref<FileItem[]>([])
const loading = ref(true)
const error = ref<Error | null>(null)
const currentPath = ref('/')
const editingFile = ref<EditingFile | null>(null)

const uploadState = ref<UploadState>({
	isUploading: false,
	currentFileName: null,
	currentFileProgress: 0,
	uploadedBytes: 0,
	totalBytes: 0,
	completedFiles: 0,
	totalFiles: 0,
})

function normalizePath(path: string) {
	return path.replace(/\\/g, '/')
}

function ensureLeadingSlash(path: string) {
	const normalized = normalizePath(path)
	return normalized.startsWith('/') ? normalized : `/${normalized}`
}

function resolveAbsolutePath(relativePath: string) {
	const normalizedRoot = normalizePath(instanceRoot.value).replace(/\/+$/, '')
	const normalizedRelative = ensureLeadingSlash(relativePath).replace(/^\/+/, '')
	return normalizedRelative ? `${normalizedRoot}/${normalizedRelative}` : normalizedRoot
}

function toRelativePath(absolutePath: string) {
	const normalizedRoot = normalizePath(instanceRoot.value).replace(/\/+$/, '')
	const normalizedPath = normalizePath(absolutePath)
	const relative = normalizedPath.replace(normalizedRoot, '').replace(/^\/+/, '')
	return relative ? `/${relative}` : '/'
}

function joinRelativePath(base: string, child: string) {
	const normalizedBase = ensureLeadingSlash(base)
	return `${normalizedBase.replace(/\/+$/, '')}/${child}`.replace(/\/{2,}/g, '/')
}

function mapEntry(entry: DirEntry): FileItem {
	const relativePath = toRelativePath(entry.path)
	return {
		name: entry.name,
		type: entry.is_dir ? 'directory' : 'file',
		path: relativePath,
		modified: entry.modified ?? 0,
		created: entry.modified ?? 0,
		size: entry.size ?? undefined,
	}
}

async function refresh() {
	if (!instanceRoot.value) return
	loading.value = true
	error.value = null
	try {
		const data = await invoke<DirEntry[]>('plugin:utils|list_dir_entries', {
			path: resolveAbsolutePath(currentPath.value),
		})
		items.value = data.map(mapEntry)
	} catch (err) {
		error.value = err instanceof Error ? err : new Error(String(err))
		items.value = []
	} finally {
		loading.value = false
	}
}

function navigateTo(path: string) {
	currentPath.value = path || '/'
	refresh()
}

function startEditing(file: EditingFile) {
	editingFile.value = file
}

function stopEditing() {
	editingFile.value = null
}

async function createItem(name: string, type: 'file' | 'directory') {
	await invoke('plugin:utils|create_path', {
		path: resolveAbsolutePath(joinRelativePath(currentPath.value, name)),
		kind: type,
	})
	await refresh()
}

async function renameItem(path: string, newName: string) {
	await invoke('plugin:utils|rename_path', {
		path: resolveAbsolutePath(path),
		newName,
	})
	await refresh()
}

async function moveItem(source: string, destination: string) {
	const fileName = source.split('/').pop() ?? ''
	await invoke('plugin:utils|move_path', {
		source: resolveAbsolutePath(source),
		destination: resolveAbsolutePath(joinRelativePath(destination, fileName)),
	})
	await refresh()
}

async function deleteItem(path: string, _recursive: boolean) {
	await invoke('plugin:utils|delete_paths', {
		paths: [resolveAbsolutePath(path)],
	})
	await refresh()
}

async function readFile(path: string) {
	return await invoke<string>('plugin:utils|read_text_file', {
		path: resolveAbsolutePath(path),
	})
}

async function readFileAsBlob(path: string) {
	const bytes = await invoke<number[]>('plugin:utils|read_binary_file', {
		path: resolveAbsolutePath(path),
	})
	return new Blob([new Uint8Array(bytes)])
}

async function writeFile(path: string, content: string) {
	await invoke('plugin:utils|write_text_file', {
		path: resolveAbsolutePath(path),
		content,
	})
	await refresh()
}

async function downloadFile(path: string, fileName: string) {
	const bytes = await invoke<number[]>('plugin:utils|read_binary_file', {
		path: resolveAbsolutePath(path),
	})
	const blob = new Blob([new Uint8Array(bytes)], { type: 'application/octet-stream' })
	const url = URL.createObjectURL(blob)
	const link = document.createElement('a')
	link.href = url
	link.download = fileName
	link.click()
	URL.revokeObjectURL(url)
}

async function uploadFiles(files: File[]) {
	if (files.length === 0) return

	uploadState.value = {
		isUploading: true,
		currentFileName: '',
		currentFileProgress: 0,
		uploadedBytes: 0,
		totalBytes: files.reduce((sum, file) => sum + file.size, 0),
		completedFiles: 0,
		totalFiles: files.length,
	}

	try {
		for (const file of files) {
			uploadState.value.currentFileName = file.name
			const buffer = await file.arrayBuffer()
			await invoke('plugin:utils|write_binary_file', {
				path: resolveAbsolutePath(joinRelativePath(currentPath.value, file.name)),
				content: Array.from(new Uint8Array(buffer)),
			})
			uploadState.value.completedFiles += 1
			uploadState.value.uploadedBytes += file.size
			uploadState.value.currentFileProgress = 1
		}
	} finally {
		uploadState.value = {
			isUploading: false,
			currentFileName: null,
			currentFileProgress: 0,
			uploadedBytes: 0,
			totalBytes: 0,
			completedFiles: 0,
			totalFiles: 0,
		}
		await refresh()
	}
}

const basePath = computed(() => instanceRoot.value)

watch(
	() => props.instance.path,
	async () => {
		instanceRoot.value = await get_full_path(props.instance.path)
		currentPath.value = '/'
		editingFile.value = null
		await refresh()
	},
)

onMounted(async () => {
	instanceRoot.value = await get_full_path(props.instance.path)
	await refresh()
})

provideFileManager({
	items,
	loading,
	error,
	currentPath,
	navigateTo,
	editingFile,
	startEditing,
	stopEditing,
	createItem,
	renameItem,
	moveItem,
	deleteItem,
	readFile,
	readFileAsBlob,
	writeFile,
	downloadFile,
	uploadFiles,
	uploadState,
	refresh,
	basePath,
	openInFolder: (path: string) => highlightInFolder(resolveAbsolutePath(path)),
})
</script>

<template>
	<div class="instance-files-page">
		<FilePageLayout :show-refresh-button="true" />
	</div>
</template>

<style scoped lang="scss">
.instance-files-page {
	display: flex;
	flex: 1 1 auto;
	min-height: 0;
	--surface-1: color-mix(in srgb, var(--color-glass-bg) 94%, transparent);
	--surface-1_5: color-mix(in srgb, var(--color-glass-bg) 88%, var(--color-glass-bg-strong) 12%);
	--surface-2: color-mix(in srgb, var(--color-glass-bg-strong) 90%, transparent);
	--surface-2_5: color-mix(in srgb, var(--color-glass-bg-strong) 82%, var(--color-brand-highlight) 18%);
	--surface-3: color-mix(in srgb, var(--color-button-bg) 84%, transparent);
	--surface-4: color-mix(in srgb, var(--glass-border) 82%, transparent);
	--surface-5: color-mix(in srgb, var(--glass-border) 96%, var(--color-brand-highlight) 4%);
	--color-button-bg: color-mix(in srgb, var(--color-button-bg) 80%, var(--color-glass-bg-strong) 20%);
	--color-button-bg-hover: color-mix(
		in srgb,
		var(--color-button-bg-hover) 78%,
		var(--color-brand-highlight) 22%
	);
	--color-button-bg-active: color-mix(in srgb, var(--color-button-bg-hover) 68%, var(--color-brand) 32%);
}
</style>
