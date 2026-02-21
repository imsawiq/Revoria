<script setup>
import { DownloadIcon, XIcon } from '@modrinth/assets'
import { Button, injectNotificationManager } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { ref } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { create_profile_and_install as pack_install } from '@/helpers/pack'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: { id: 'install-confirm.header', defaultMessage: 'Are you sure?' },
	description: {
		id: 'install-confirm.description',
		defaultMessage:
			'You already have this modpack installed. Are you sure you want to install it again?',
	},
	cancel: { id: 'action.cancel', defaultMessage: 'Cancel' },
	installing: { id: 'install-confirm.installing', defaultMessage: 'Installing' },
	install: { id: 'install-confirm.install', defaultMessage: 'Install' },
})

const versionId = ref()
const project = ref()
const confirmModal = ref(null)
const installing = ref(false)

const onInstall = ref(() => {})
const onCreateInstance = ref(() => {})

defineExpose({
	show: (projectVal, versionIdVal, callback, createInstanceCallback) => {
		project.value = projectVal
		versionId.value = versionIdVal
		installing.value = false
		confirmModal.value.show()

		onInstall.value = callback
		onCreateInstance.value = createInstanceCallback

		trackEvent('PackInstallStart')
	},
})

async function install() {
	installing.value = true
	confirmModal.value.hide()

	await pack_install(
		project.value.id,
		versionId.value,
		project.value.title,
		project.value.icon_url,
		onCreateInstance.value,
	).catch(handleError)
	trackEvent('PackInstall', {
		id: project.value.id,
		version_id: versionId.value,
		title: project.value.title,
		source: 'ConfirmModal',
	})

	onInstall.value(versionId.value)
	installing.value = false
}
</script>

<template>
	<ModalWrapper ref="confirmModal" :header="formatMessage(messages.header)" :on-hide="onInstall">
		<div class="modal-body">
			<p>{{ formatMessage(messages.description) }}</p>
			<div class="input-group push-right">
				<Button @click="() => $refs.confirmModal.hide()"
					><XIcon />{{ formatMessage(messages.cancel) }}</Button
				>
				<Button color="primary" :disabled="installing" @click="install()"
					><DownloadIcon />
					{{
						installing ? formatMessage(messages.installing) : formatMessage(messages.install)
					}}</Button
				>
			</div>
		</div>
	</ModalWrapper>
</template>

<style lang="scss" scoped>
.modal-body {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}
</style>
