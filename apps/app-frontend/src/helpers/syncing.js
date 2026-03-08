import { invoke } from '@tauri-apps/api/core'

export async function getSyncState() {
	return await invoke('plugin:syncing|syncing_get_state')
}

export async function setSyncTarget(path, isFile, enabled) {
	return await invoke('plugin:syncing|syncing_set_target', {
		path,
		isFile,
		enabled,
	})
}

export async function removeSyncTarget(path, isFile) {
	return await invoke('plugin:syncing|syncing_remove_target', {
		path,
		isFile,
	})
}

export async function applyAllSyncTargets() {
	return await invoke('plugin:syncing|syncing_apply_all')
}
