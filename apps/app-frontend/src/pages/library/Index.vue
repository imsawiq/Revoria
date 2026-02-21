<script setup>
import { PlusIcon } from '@modrinth/assets'
import { Button, injectNotificationManager } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { onUnmounted, ref, shallowRef } from 'vue'
import { useRoute } from 'vue-router'

import { NewInstanceImage } from '@/assets/icons'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import NavTabs from '@/components/ui/NavTabs.vue'
import { profile_listener } from '@/helpers/events.js'
import { list } from '@/helpers/profile.js'
import { useBreadcrumbs } from '@/store/breadcrumbs.js'

const { handleError } = injectNotificationManager()
const route = useRoute()
const breadcrumbs = useBreadcrumbs()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	breadcrumbLibrary: {
		id: 'app.library.breadcrumb',
		defaultMessage: 'Library',
	},
	title: {
		id: 'app.library.title',
		defaultMessage: 'Library',
	},
	subtitle: {
		id: 'app.library.subtitle',
		defaultMessage: 'Your instances, organized.',
	},
	create: {
		id: 'app.library.create',
		defaultMessage: 'Create',
	},
	tabAll: {
		id: 'app.library.tab.all',
		defaultMessage: 'All instances',
	},
	tabDownloaded: {
		id: 'app.library.tab.downloaded',
		defaultMessage: 'Downloaded',
	},
	tabCustom: {
		id: 'app.library.tab.custom',
		defaultMessage: 'Custom',
	},
	tabShared: {
		id: 'app.library.tab.shared',
		defaultMessage: 'Shared with me',
	},
	tabSaved: {
		id: 'app.library.tab.saved',
		defaultMessage: 'Saved',
	},
	emptyTitle: {
		id: 'app.library.empty.title',
		defaultMessage: 'No instances found',
	},
	createNewInstance: {
		id: 'app.library.empty.create-new',
		defaultMessage: 'Create new instance',
	},
})

breadcrumbs.setRootContext({ name: formatMessage(messages.breadcrumbLibrary), link: route.path })

const instances = shallowRef((await list().catch(handleError)) ?? [])

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

const unlistenProfile = await profile_listener(async () => {
	instances.value = (await list().catch(handleError)) ?? []
})
onUnmounted(() => {
	unlistenProfile()
})
</script>

<template>
	<div class="p-6 flex flex-col gap-4">
		<div class="flex items-start justify-between gap-4">
			<div class="min-w-0">
				<h1 class="m-0 text-2xl font-extrabold text-contrast">{{ formatMessage(messages.title) }}</h1>
				<div class="mt-1 text-sm text-secondary font-semibold leading-tight">
					{{ formatMessage(messages.subtitle) }}
				</div>
			</div>
			<Button color="primary" :disabled="offline" @click="$refs.installationModal.show()">
				<PlusIcon />
				{{ formatMessage(messages.create) }}
			</Button>
		</div>

		<div
			class="rounded-2xl bg-[--color-glass-bg-strong] border border-[--glass-border] shadow-[--glass-shadow] p-2"
		>
			<NavTabs
				:links="[
					{ label: formatMessage(messages.tabAll), href: `/library` },
					{ label: formatMessage(messages.tabDownloaded), href: `/library/downloaded` },
					{ label: formatMessage(messages.tabCustom), href: `/library/custom` },
					{ label: formatMessage(messages.tabShared), href: `/library/shared`, shown: false },
					{ label: formatMessage(messages.tabSaved), href: `/library/saved`, shown: false },
				]"
			/>
		</div>

		<template v-if="instances.length > 0">
			<RouterView :instances="instances" />
		</template>
		<div
			v-else
			class="no-instance rounded-2xl bg-[--color-glass-bg-strong] border border-[--glass-border] shadow-[--glass-shadow] p-8"
		>
			<div class="icon">
				<NewInstanceImage />
			</div>
			<h3 class="text-contrast">{{ formatMessage(messages.emptyTitle) }}</h3>
			<Button color="primary" :disabled="offline" @click="$refs.installationModal.show()">
				<PlusIcon />
				{{ formatMessage(messages.createNewInstance) }}
			</Button>
		</div>
		<InstanceCreationModal ref="installationModal" />
	</div>
</template>

<style lang="scss" scoped>
.no-instance {
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	height: 100%;
	gap: var(--gap-md);

	p,
	h3 {
		margin: 0;
	}

	.icon {
		svg {
			width: 10rem;
			height: 10rem;
		}
	}
}
</style>
