import type { AbstractWebNotificationManager } from '@modrinth/ui'
import type { Ref } from 'vue'

import { setupCreationModal } from './setup/creation-modal'
import { setupFilePickerProvider } from './setup/file-picker'
import { setupInstanceImportProvider } from './setup/instance-import'
import { setupTagsProvider } from './setup/tags'

export function setupProviders(
	notificationManager: AbstractWebNotificationManager,
	options: { stateReady?: Ref<boolean> } = {},
) {
	setupTagsProvider(notificationManager, options.stateReady)
	setupFilePickerProvider()
	setupInstanceImportProvider(notificationManager)

	return {
		...setupCreationModal(notificationManager),
	}
}
