import { ref } from 'vue'

export const allowState = ref(false)
export const installState = ref(false)
export const updateState = ref(false)

export async function getRemote(isDownloadState) {
	updateState.value = false
	allowState.value = false
	installState.value = false
	return false
}
