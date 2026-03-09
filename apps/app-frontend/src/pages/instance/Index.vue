<template>
	<div>
		<template v-if="isOnInstanceRoute && instance">
			<div class="p-6 pr-2 pb-4" @contextmenu.prevent.stop="(event) => handleRightClick(event)">
				<ExportModal ref="exportModal" :instance="instance" />
				<InstanceSettingsModal ref="settingsModal" :instance="instance" :offline="offline" />
				<ContentPageHeader>
					<template #icon>
						<Avatar :src="icon" :alt="instance.name" size="96px" :tint-by="instance.path" />
					</template>
					<template #title>
						{{ instance.name }}
					</template>
					<template #summary> </template>
					<template #stats>
						<div
							class="flex items-center gap-2 font-semibold transform capitalize border-0 border-solid border-divider pr-4 md:border-r"
						>
							<GameIcon class="h-6 w-6 text-secondary" />
							{{ instance.loader }} {{ instance.game_version }}
						</div>
						<div class="flex items-center gap-2 font-semibold">
							<TimerIcon class="h-6 w-6 text-secondary" />
							<template v-if="timePlayed > 0">
								{{ timePlayedHumanized }}
							</template>
							<template v-else> {{ formatMessage(messages.neverPlayed) }} </template>
						</div>
					</template>
					<template #actions>
						<div class="flex gap-2">
							<ButtonStyled
								v-if="
									['installing', 'pack_installing', 'minecraft_installing'].includes(
										instance.install_stage,
									)
								"
								color="brand"
								size="large"
							>
								<button disabled>{{ formatMessage(messages.installing) }}</button>
							</ButtonStyled>
							<ButtonStyled
								v-else-if="instance.install_stage !== 'installed'"
								color="brand"
								size="large"
							>
								<button @click="repairInstance()">
									<DownloadIcon />
									{{ formatMessage(messages.repair) }}
								</button>
							</ButtonStyled>
							<ButtonStyled v-else-if="playing === true" color="red" size="large">
								<button @click="stopInstance('InstancePage')">
									<StopCircleIcon />
									{{ formatMessage(messages.stop) }}
								</button>
							</ButtonStyled>
							<ButtonStyled
								v-else-if="playing === false && loading === false"
								color="brand"
								size="large"
							>
								<button @click="startInstance('InstancePage')">
									<PlayIcon />
									{{ formatMessage(messages.play) }}
								</button>
							</ButtonStyled>
							<ButtonStyled
								v-else-if="loading === true && playing === false"
								color="brand"
								size="large"
							>
								<button disabled>{{ formatMessage(messages.loading) }}</button>
							</ButtonStyled>
							<ButtonStyled size="large" circular>
								<button
									v-tooltip="formatMessage(messages.instanceSettings)"
									@click="settingsModal.show()"
								>
									<SettingsIcon />
								</button>
							</ButtonStyled>
							<ButtonStyled size="large" type="transparent" circular>
								<OverflowMenu
									:options="[
										{
											id: 'open-folder',
											action: () => showProfileInFolder(instance.path),
										},
										{
											id: 'export-mrpack',
											action: () => $refs.exportModal.show(),
										},
									]"
								>
									<MoreVerticalIcon />
									<template #share-instance>
										<UserPlusIcon /> {{ formatMessage(messages.shareInstance) }}
									</template>
									<template #host-a-server>
										<ServerIcon /> {{ formatMessage(messages.createServer) }}
									</template>
									<template #open-folder>
										<FolderOpenIcon /> {{ formatMessage(messages.openFolder) }}
									</template>
									<template #export-mrpack>
										<PackageIcon /> {{ formatMessage(messages.exportModpack) }}
									</template>
								</OverflowMenu>
							</ButtonStyled>
						</div>
					</template>
				</ContentPageHeader>
			</div>
			<div v-if="tabs.length > 0" class="px-6">
				<NavTabs :links="tabs" />
			</div>
			<div class="p-6 pt-4">
				<RouterView v-slot="{ Component }" :key="instance.path">
					<template v-if="Component">
						<Suspense
							:key="instance.path"
							@pending="loadingBar.startLoading()"
							@resolve="loadingBar.stopLoading()"
						>
							<Transition name="instance-tab-swap" mode="out-in">
								<div :key="route.path" class="instance-tab-swap-shell">
									<component
										:is="Component"
										:instance="instance"
										:options="options"
										:offline="offline"
										:playing="playing"
										:versions="modrinthVersions"
										:installed="instance.install_stage === 'installed'"
										@play="updatePlayState"
										@stop="() => stopInstance('InstanceSubpage')"
									></component>
								</div>
							</Transition>
							<template #fallback>
								<LoadingIndicator />
							</template>
						</Suspense>
					</template>
				</RouterView>
			</div>
			<ContextMenu ref="options" @option-clicked="handleOptionsClick">
				<template #play> <PlayIcon /> {{ formatMessage(messages.play) }} </template>
				<template #stop> <StopCircleIcon /> {{ formatMessage(messages.stop) }} </template>
				<template #add_content> <PlusIcon /> {{ formatMessage(messages.addContent) }} </template>
				<template #edit> <EditIcon /> {{ formatMessage(messages.edit) }} </template>
				<template #copy_path>
					<ClipboardCopyIcon /> {{ formatMessage(messages.copyPath) }}
				</template>
				<template #open_folder>
					<FolderOpenIcon /> {{ formatMessage(messages.openFolder) }}
				</template>
				<template #copy_link>
					<ClipboardCopyIcon /> {{ formatMessage(messages.copyLink) }}
				</template>
				<template #open_link>
					<GlobeIcon /> {{ formatMessage(messages.openInModrinth) }} <ExternalIcon />
				</template>
				<template #copy_names><EditIcon />{{ formatMessage(messages.copyNames) }}</template>
				<template #copy_slugs><HashIcon />{{ formatMessage(messages.copySlugs) }}</template>
				<template #copy_links><GlobeIcon />{{ formatMessage(messages.copyLinks) }}</template>
				<template #toggle><EditIcon />{{ formatMessage(messages.toggleSelected) }}</template>
				<template #disable><XIcon />{{ formatMessage(messages.disableSelected) }}</template>
				<template #enable><CheckCircleIcon />{{ formatMessage(messages.enableSelected) }}</template>
				<template #hide_show><EyeIcon />{{ formatMessage(messages.showHideUnselected) }}</template>
				<template #update_all
					><UpdatedIcon />{{
						selected.length > 0
							? formatMessage(messages.updateSelected)
							: formatMessage(messages.updateAll)
					}}</template
				>
				<template #filter_update
					><UpdatedIcon />{{ formatMessage(messages.selectUpdatable) }}</template
				>
			</ContextMenu>
		</template>
		<template v-else>
			<div class="p-6">
				<LoadingIndicator />
			</div>
		</template>
	</div>
</template>
<script setup>
import {
	CheckCircleIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	EditIcon,
	ExternalIcon,
	EyeIcon,
	FolderOpenIcon,
	GameIcon,
	GlobeIcon,
	HashIcon,
	MoreVerticalIcon,
	PackageIcon,
	PlayIcon,
	PlusIcon,
	ServerIcon,
	SettingsIcon,
	StopCircleIcon,
	TimerIcon,
	UpdatedIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	ContentPageHeader,
	injectNotificationManager,
	LoadingIndicator,
	OverflowMenu,
} from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { defineMessages, useVIntl } from '@vintl/vintl'
import dayjs from 'dayjs'
import duration from 'dayjs/plugin/duration'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import ExportModal from '@/components/ui/ExportModal.vue'
import InstanceSettingsModal from '@/components/ui/modal/InstanceSettingsModal.vue'
import NavTabs from '@/components/ui/NavTabs.vue'
import { trackEvent } from '@/helpers/analytics'
import { get_project, get_version_many } from '@/helpers/cache.js'
import { process_listener, profile_listener } from '@/helpers/events'
import { get_by_profile_path } from '@/helpers/process'
import { finish_install, get, get_full_path, kill, run } from '@/helpers/profile'
import { showProfileInFolder } from '@/helpers/utils.js'
import { handleSevereError } from '@/store/error.js'
import { useBreadcrumbs, useLoading } from '@/store/state'

dayjs.extend(duration)
dayjs.extend(relativeTime)

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const route = useRoute()

const messages = defineMessages({
	neverPlayed: { id: 'instance.index.never-played', defaultMessage: 'Never played' },
	installing: { id: 'instance.index.installing', defaultMessage: 'Installing...' },
	loading: { id: 'instance.index.loading', defaultMessage: 'Loading...' },
	instanceSettings: { id: 'instance.index.instance-settings', defaultMessage: 'Instance settings' },
	repair: { id: 'instance.index.repair', defaultMessage: 'Repair' },
	stop: { id: 'instance.index.stop', defaultMessage: 'Stop' },
	play: { id: 'instance.index.play', defaultMessage: 'Play' },
	shareInstance: { id: 'instance.index.share-instance', defaultMessage: 'Share instance' },
	createServer: { id: 'instance.index.create-server', defaultMessage: 'Create a server' },
	openFolder: { id: 'action.open-folder', defaultMessage: 'Open folder' },
	exportModpack: { id: 'instance.index.export-modpack', defaultMessage: 'Export modpack' },
	addContent: { id: 'instance.index.add-content', defaultMessage: 'Add content' },
	edit: { id: 'edit', defaultMessage: 'Edit' },
	copyPath: { id: 'instance.index.copy-path', defaultMessage: 'Copy path' },
	copyLink: { id: 'copy-link', defaultMessage: 'Copy link' },
	openInModrinth: { id: 'instance.index.open-in-modrinth', defaultMessage: 'Open in Modrinth' },
	copyNames: { id: 'instance.index.copy-names', defaultMessage: 'Copy names' },
	copySlugs: { id: 'instance.index.copy-slugs', defaultMessage: 'Copy slugs' },
	copyLinks: { id: 'instance.index.copy-links', defaultMessage: 'Copy links' },
	toggleSelected: { id: 'instance.index.toggle-selected', defaultMessage: 'Toggle selected' },
	disableSelected: { id: 'instance.index.disable-selected', defaultMessage: 'Disable selected' },
	enableSelected: { id: 'instance.index.enable-selected', defaultMessage: 'Enable selected' },
	showHideUnselected: {
		id: 'instance.index.show-hide-unselected',
		defaultMessage: 'Show/Hide unselected',
	},
	updateSelected: { id: 'instance.index.update-selected', defaultMessage: 'Update selected' },
	updateAll: { id: 'instance.index.update-all', defaultMessage: 'Update all' },
	selectUpdatable: { id: 'instance.index.select-updatable', defaultMessage: 'Select Updatable' },
	tabContent: { id: 'instance.breadcrumb.content', defaultMessage: 'Content' },
	tabWorlds: { id: 'instance.index.tab.worlds', defaultMessage: 'Worlds' },
	tabScreenshots: { id: 'instance.breadcrumb.screenshots', defaultMessage: 'Screenshots' },
	tabLogs: { id: 'instance.breadcrumb.logs', defaultMessage: 'Logs' },
	breadcrumbInstance: { id: 'instance.index.breadcrumb.instance', defaultMessage: 'Instance' },
	hourSingular: { id: 'instance.index.time.hour-singular', defaultMessage: '{count} hour' },
	hourPlural: { id: 'instance.index.time.hour-plural', defaultMessage: '{count} hours' },
	minuteSingular: { id: 'instance.index.time.minute-singular', defaultMessage: '{count} minute' },
	minutePlural: { id: 'instance.index.time.minute-plural', defaultMessage: '{count} minutes' },
	secondSingular: { id: 'instance.index.time.second-singular', defaultMessage: '{count} second' },
	secondPlural: { id: 'instance.index.time.second-plural', defaultMessage: '{count} seconds' },
})

const router = useRouter()
const breadcrumbs = useBreadcrumbs()

const safeDecodeURIComponent = (value) => {
	try {
		return decodeURIComponent(value)
	} catch {
		return value
	}
}

const isOnInstanceRoute = computed(() => String(route.path ?? '').startsWith('/instance'))
const profilePathId = computed(() => safeDecodeURIComponent(String(route.params.id ?? '')))
const hasValidProfilePathId = computed(() => profilePathId.value.length > 0)

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

const instance = ref()
const modrinthVersions = ref([])
const playing = ref(false)
const loading = ref(false)

async function fetchInstance() {
	if (!hasValidProfilePathId.value) return
	instance.value = await get(profilePathId.value).catch(handleError)

	if (!offline.value && instance.value.linked_data && instance.value.linked_data.project_id) {
		get_project(instance.value.linked_data.project_id, 'must_revalidate')
			.catch(handleError)
			.then((project) => {
				if (project && project.versions) {
					get_version_many(project.versions, 'must_revalidate')
						.catch(handleError)
						.then((versions) => {
							modrinthVersions.value = versions.sort(
								(a, b) => dayjs(b.date_published) - dayjs(a.date_published),
							)
						})
				}
			})
	}
}

async function updatePlayState() {
	const runningProcesses = await get_by_profile_path(profilePathId.value).catch(handleError)

	playing.value = runningProcesses.length > 0
}

onMounted(async () => {
	if (!hasValidProfilePathId.value) {
		router.replace({ path: '/library' })
		return
	}
	await fetchInstance()
})
watch(
	() => route.params.id,
	async () => {
		if (route.params.id && isOnInstanceRoute.value) {
			await fetchInstance()
		}
	},
)

const basePath = computed(() =>
	hasValidProfilePathId.value ? `/instance/${encodeURIComponent(profilePathId.value)}` : '',
)

const tabs = computed(() => {
	if (!isOnInstanceRoute.value || !hasValidProfilePathId.value) return []
	return [
		{ label: formatMessage(messages.tabContent), href: `${basePath.value}` },
		{ label: formatMessage(messages.tabWorlds), href: `${basePath.value}/worlds` },
		{ label: formatMessage(messages.tabScreenshots), href: `${basePath.value}/screenshots` },
		{ label: formatMessage(messages.tabLogs), href: `${basePath.value}/logs` },
	]
})

if (instance.value?.name) {
	breadcrumbs.setName(
		formatMessage(messages.breadcrumbInstance),
		instance.value.name.length > 40
			? instance.value.name.substring(0, 40) + '...'
			: instance.value.name,
	)
}

if (instance.value?.name) {
	breadcrumbs.setContext({
		name: instance.value.name,
		link: route.path,
		query: route.query,
	})
}

const loadingBar = useLoading()

const options = ref(null)

const startInstance = async (context) => {
	loading.value = true
	try {
		await run(profilePathId.value)
		playing.value = true
	} catch (err) {
		handleSevereError(err, { profilePath: profilePathId.value })
	}
	loading.value = false

	trackEvent('InstanceStart', {
		loader: instance.value.loader,
		game_version: instance.value.game_version,
		source: context,
	})
}

const stopInstance = async (context) => {
	playing.value = false
	await kill(profilePathId.value).catch(handleError)

	trackEvent('InstanceStop', {
		loader: instance.value.loader,
		game_version: instance.value.game_version,
		source: context,
	})
}

const repairInstance = async () => {
	await finish_install(instance.value).catch(handleError)
}

const handleRightClick = (event) => {
	const baseOptions = [
		{ name: 'add_content' },
		{ type: 'divider' },
		{ name: 'edit' },
		{ name: 'open_folder' },
		{ name: 'copy_path' },
	]

	options.value.showMenu(
		event,
		instance.value,
		playing.value
			? [
					{
						name: 'stop',
						color: 'danger',
					},
					...baseOptions,
				]
			: [
					{
						name: 'play',
						color: 'primary',
					},
					...baseOptions,
				],
	)
}

const handleOptionsClick = async (args) => {
	switch (args.option) {
		case 'play':
			await startInstance('InstancePageContextMenu')
			break
		case 'stop':
			await stopInstance('InstancePageContextMenu')
			break
		case 'add_content':
			await router.push({
				path: `/browse/${instance.value.loader === 'vanilla' ? 'datapack' : 'mod'}`,
				query: { i: profilePathId.value },
			})
			break
		case 'edit':
			await router.push({
				path: `/instance/${encodeURIComponent(profilePathId.value)}/options`,
			})
			break
		case 'open_folder':
			await showProfileInFolder(instance.value.path)
			break
		case 'copy_path': {
			const fullPath = await get_full_path(instance.value.path)
			await navigator.clipboard.writeText(fullPath)
			break
		}
	}
}

let unlistenProfiles = null
let unlistenProcesses = null

onMounted(async () => {
	unlistenProfiles = await profile_listener(async (event) => {
		if (event.profile_path_id === profilePathId.value) {
			if (event.event === 'removed') {
				await router.push({
					path: '/',
				})
				return
			}
			instance.value = await get(profilePathId.value).catch(handleError)
		}
	}).catch(() => null)

	unlistenProcesses = await process_listener((e) => {
		if (e.event === 'finished' && e.profile_path_id === profilePathId.value) {
			playing.value = false
		}
	}).catch(() => null)
})

const icon = computed(() =>
	instance.value?.icon_path ? convertFileSrc(instance.value.icon_path) : null,
)

const settingsModal = ref()

const timePlayed = computed(() => {
	if (!instance.value) return 0
	return instance.value.recent_time_played + instance.value.submitted_time_played
})

const timePlayedHumanized = computed(() => {
	const duration = dayjs.duration(timePlayed.value, 'seconds')
	const hours = Math.floor(duration.asHours())
	if (hours >= 1) {
		return formatMessage(hours > 1 ? messages.hourPlural : messages.hourSingular, { count: hours })
	}

	const minutes = Math.floor(duration.asMinutes())
	if (minutes >= 1) {
		return formatMessage(minutes > 1 ? messages.minutePlural : messages.minuteSingular, {
			count: minutes,
		})
	}

	const seconds = Math.floor(duration.asSeconds())
	return formatMessage(seconds > 1 ? messages.secondPlural : messages.secondSingular, {
		count: seconds,
	})
})

onBeforeUnmount(() => {
	unlistenProcesses?.()
	unlistenProfiles?.()
})
</script>

<style scoped lang="scss">
.instance-card {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}

Button {
	width: 100%;
}

.button-group {
	display: flex;
	flex-direction: row;
	gap: 0.5rem;
}

.side-cards {
	position: fixed;
	width: 300px;
	display: flex;
	flex-direction: column;

	min-height: calc(100vh - 3.25rem);
	max-height: calc(100vh - 3.25rem);
	overflow-y: auto;
	-ms-overflow-style: none;
	scrollbar-width: none;

	&::-webkit-scrollbar {
		width: 0;
		background: transparent;
	}

	.card {
		min-height: unset;
		margin-bottom: 0;
	}
}

.instance-nav {
	display: flex;
	flex-direction: column;
	align-items: flex-start;
	justify-content: center;
	padding: 1rem;
	gap: 0.5rem;
	background: linear-gradient(
		180deg,
		color-mix(in oklch, var(--color-glass-bg-strong) 88%, transparent),
		color-mix(in oklch, var(--color-glass-bg) 92%, transparent)
	);
	border: 1px solid var(--glass-border);
	border-radius: var(--radius-xl);
	box-shadow: var(--shadow-card);
	height: 100%;
}

.name {
	font-size: 1.25rem;
	color: var(--color-contrast);
	overflow: hidden;
	text-overflow: ellipsis;
}

.metadata {
	text-transform: capitalize;
}

.instance-container {
	display: flex;
	flex-direction: row;
	overflow: auto;
	gap: 1rem;
	min-height: 100%;
	padding: 1rem;
}

.instance-info {
	display: flex;
	flex-direction: column;
	width: 100%;
}

.instance-tab-swap-shell {
	will-change: transform, opacity, filter;
}

.instance-tab-swap-enter-active,
.instance-tab-swap-leave-active {
	transition:
		opacity 260ms cubic-bezier(0.2, 0.8, 0.2, 1),
		transform 320ms cubic-bezier(0.16, 1, 0.3, 1),
		filter 240ms ease;
}

.instance-tab-swap-enter-from {
	opacity: 0;
	transform: translateY(10px) scale(0.988);
	filter: blur(3px);
}

.instance-tab-swap-leave-to {
	opacity: 0;
	transform: translateY(-6px) scale(0.994);
	filter: blur(2px);
}

.badge {
	display: flex;
	align-items: center;
	font-weight: bold;
	width: fit-content;
	color: var(--color-orange);
}

.pages-list {
	display: flex;
	flex-direction: column;
	gap: var(--gap-xs);

	.btn {
		font-size: 100%;
		font-weight: 400;
		background: inherit;
		transition: all ease-in-out 0.1s;
		width: 100%;
		color: var(--color-primary);
		box-shadow: none;

		&.router-link-exact-active {
			box-shadow: 0 0 0 1px var(--color-brand-highlight);
			background: var(--color-selected-button-bg);
			color: var(--color-contrast);
		}

		&:hover {
			background-color: var(--color-button-bg-hover);
			color: var(--color-contrast);
			box-shadow: var(--shadow-card);
			text-decoration: none;
		}

		svg {
			width: 1.3rem;
			height: 1.3rem;
		}
	}
}

.instance-nav {
	display: flex;
	flex-direction: row;
	align-items: flex-start;
	justify-content: left;
	padding: 1rem;
	gap: 0.5rem;
	height: min-content;
	width: 100%;
}

.instance-button {
	width: fit-content;
}

.actions {
	display: flex;
	flex-direction: column;
	justify-content: flex-start;
	gap: 0.5rem;
}

.content {
	margin: 0 1rem 0.5rem 20rem;
	width: calc(100% - 20rem);
	display: flex;
	flex-direction: column;
	overflow: auto;
}

.stats {
	grid-area: stats;
	display: flex;
	flex-direction: column;
	flex-wrap: wrap;
	gap: var(--gap-md);

	.stat {
		display: flex;
		flex-direction: row;
		align-items: center;
		width: fit-content;
		gap: var(--gap-xs);
		--stat-strong-size: 1.25rem;

		strong {
			font-size: var(--stat-strong-size);
		}

		p {
			margin: 0;
		}

		svg {
			height: var(--stat-strong-size);
			width: var(--stat-strong-size);
		}
	}

	.date {
		margin-top: auto;
	}

	@media screen and (max-width: 750px) {
		flex-direction: row;
		column-gap: var(--gap-md);
		margin-top: var(--gap-xs);
	}

	@media screen and (max-width: 600px) {
		margin-top: 0;

		.stat-label {
			display: none;
		}
	}
}
</style>
