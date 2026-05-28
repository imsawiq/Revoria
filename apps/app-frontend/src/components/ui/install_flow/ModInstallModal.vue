<script setup>
import {
	CheckIcon,
	DownloadIcon,
	PlusIcon,
	RightArrowIcon,
	SearchIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import { Avatar, Button, ButtonStyled, Card, injectNotificationManager } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import {
	add_project_from_version as installMod,
	check_installed,
	create,
	get,
	list,
} from '@/helpers/profile'
import {
	findPreferredVersion,
	installVersionDependencies,
	isVersionCompatible,
} from '@/store/install.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const router = useRouter()

const messages = defineMessages({
	installToInstanceTitle: {
		id: 'browse.install.title',
		defaultMessage: 'Install project to instance',
	},
	installToInstanceDescription: {
		id: 'browse.install.description',
		defaultMessage: 'Select an instance to install',
	},
	searchInstancesPlaceholder: {
		id: 'browse.install.search',
		defaultMessage: 'Search instances...',
	},
	installAction: {
		id: 'browse.install.action',
		defaultMessage: 'Install',
	},
	installingAction: {
		id: 'browse.install.installing',
		defaultMessage: 'Installing...',
	},
	installedAction: {
		id: 'browse.install.installed',
		defaultMessage: 'Installed',
	},
	noCompatibleInstances: {
		id: 'browse.install.none',
		defaultMessage: 'No compatible instances found.',
	},
	noCompatibleVersion: {
		id: 'install-modal.no-compatible-version',
		defaultMessage: 'No compatible version found',
	},
	lockedInstanceTooltip: {
		id: 'install-modal.locked-instance-tooltip',
		defaultMessage: 'Unpair or unlock an instance to add mods.',
	},
	selectIcon: { id: 'instance.create.select-icon', defaultMessage: 'Select icon' },
	removeIcon: { id: 'instance.create.remove-icon', defaultMessage: 'Remove icon' },
	name: { id: 'instance.create.name', defaultMessage: 'Name' },
	create: { id: 'instance.create.action.create', defaultMessage: 'Create' },
	creating: { id: 'instance.create.action.creating', defaultMessage: 'Creating...' },
	createInstance: { id: 'app.nav.create-instance', defaultMessage: 'Create new instance' },
	hideNewInstance: {
		id: 'install-modal.hide-new-instance',
		defaultMessage: 'Hide new instance',
	},
	cancel: { id: 'action.cancel', defaultMessage: 'Cancel' },
})

const versions = ref()
const project = ref()

const installModal = ref()
const searchFilter = ref('')

const showCreation = ref(false)
const icon = ref(null)
const name = ref(null)
const display_icon = ref(null)
const loader = ref(null)
const gameVersion = ref(null)
const creatingInstance = ref(false)

const profiles = ref([])

const shownProfiles = computed(() =>
	profiles.value
		.filter((profile) => {
			return profile.name.toLowerCase().includes(searchFilter.value.toLowerCase())
		})
		.filter((profile) => {
			const version = {
				game_versions: versions.value.flatMap((v) => v.game_versions),
				loaders: versions.value.flatMap((v) => v.loaders),
			}
			return isVersionCompatible(version, project.value, profile)
		}),
)

const onInstall = ref(() => {})

defineExpose({
	show: async (projectVal, versionsVal, callback) => {
		project.value = projectVal
		versions.value = versionsVal
		searchFilter.value = ''

		showCreation.value = false
		name.value = null
		icon.value = null
		display_icon.value = null
		gameVersion.value = null
		loader.value = null

		onInstall.value = callback

		const profilesVal = await list().catch(handleError)
		for (const profile of profilesVal) {
			profile.installing = false
			profile.installedMod = await check_installed(profile.path, project.value.id).catch(
				handleError,
			)
		}
		profiles.value = profilesVal

		installModal.value.show()

		trackEvent('ProjectInstallStart', { source: 'ProjectInstallModal' })
	},
})

async function install(instance) {
	instance.installing = true
	const version = findPreferredVersion(versions.value, project.value, instance)

	if (!version) {
		instance.installing = false
		handleError(formatMessage(messages.noCompatibleVersion))
		return
	}

	await installMod(instance.path, version.id).catch(handleError)
	await installVersionDependencies(instance, version).catch(handleError)

	instance.installedMod = true
	instance.installing = false

	trackEvent('ProjectInstall', {
		loader: instance.loader,
		game_version: instance.game_version,
		id: project.value.id,
		version_id: version.id,
		project_type: project.value.project_type,
		title: project.value.title,
		source: 'ProjectInstallModal',
	})

	onInstall.value(version.id)
}

const toggleCreation = () => {
	showCreation.value = !showCreation.value
	name.value = null
	icon.value = null
	display_icon.value = null
	gameVersion.value = null
	loader.value = null

	if (showCreation.value) {
		trackEvent('InstanceCreateStart', { source: 'ProjectInstallModal' })
	}
}

const upload_icon = async () => {
	const res = await open({
		multiple: false,
		filters: [
			{
				name: 'Image',
				extensions: ['png', 'jpeg'],
			},
		],
	})
	icon.value = res.path ?? res

	if (!icon.value) return
	display_icon.value = convertFileSrc(icon.value)
}

const reset_icon = () => {
	icon.value = null
	display_icon.value = null
}

const createInstance = async () => {
	creatingInstance.value = true

	const loader =
		versions.value[0].loaders[0] !== 'forge' &&
		versions.value[0].loaders[0] !== 'fabric' &&
		versions.value[0].loaders[0] !== 'quilt'
			? 'vanilla'
			: versions.value[0].loaders[0]

	const id = await create(
		name.value,
		versions.value[0].game_versions[0],
		loader,
		'latest',
		icon.value,
	).catch(handleError)

	await installMod(id, versions.value[0].id).catch(handleError)

	await router.push(`/instance/${encodeURIComponent(id)}/`)

	const instance = await get(id, true)
	await installVersionDependencies(instance, versions.value[0]).catch(handleError)

	trackEvent('InstanceCreate', {
		profile_name: name.value,
		game_version: versions.value[0].game_versions[0],
		loader: loader,
		loader_version: 'latest',
		has_icon: !!icon.value,
		source: 'ProjectInstallModal',
	})

	trackEvent('ProjectInstall', {
		loader: loader,
		game_version: versions.value[0].game_versions[0],
		id: project.value,
		version_id: versions.value[0].id,
		project_type: project.value.project_type,
		title: project.value.title,
		source: 'ProjectInstallModal',
	})

	onInstall.value(versions.value[0].id)

	if (installModal.value) installModal.value.hide()
	creatingInstance.value = false
}
</script>

<template>
	<ModalWrapper
		ref="installModal"
		:header="formatMessage(messages.installToInstanceTitle)"
		:on-hide="onInstall"
	>
		<div class="flex flex-col gap-4 w-[32rem] max-w-full">
			<p class="text-secondary m-0 text-sm">
				{{ formatMessage(messages.installToInstanceDescription) }}
			</p>
			<div
				class="flex items-center gap-2 rounded-xl bg-[--color-button-bg] px-3 py-2 border border-[--glass-border]"
			>
				<SearchIcon aria-hidden="true" class="w-4 h-4 text-secondary shrink-0" />
				<input
					v-model="searchFilter"
					autocomplete="off"
					type="text"
					class="w-full bg-transparent border-none outline-none text-sm text-contrast placeholder:text-secondary"
					:placeholder="formatMessage(messages.searchInstancesPlaceholder)"
				/>
			</div>
			<div class="flex flex-col gap-1 max-h-[20rem] overflow-y-auto pr-1">
				<div
					v-for="profile in shownProfiles"
					:key="profile.name"
					class="flex items-center gap-3 rounded-xl px-3 py-2.5 transition-colors hover:bg-[--color-button-bg] group"
				>
					<router-link
						class="flex items-center gap-3 flex-1 min-w-0 no-underline text-contrast"
						:to="`/instance/${encodeURIComponent(profile.path)}`"
						@click="installModal.hide()"
					>
						<Avatar
							:src="profile.icon_path ? convertFileSrc(profile.icon_path) : null"
							size="36px"
							class="shrink-0 rounded-lg"
						/>
						<div class="flex flex-col min-w-0">
							<span class="text-sm font-semibold text-contrast truncate">{{ profile.name }}</span>
							<span class="text-xs text-secondary">
								{{ profile.game_version }}
								<template v-if="profile.loader && profile.loader !== 'vanilla'">
									&middot; {{ profile.loader }}
								</template>
							</span>
						</div>
					</router-link>
					<div
						v-tooltip="
							profile.linked_data?.locked && !profile.installedMod
								? formatMessage(messages.lockedInstanceTooltip)
								: ''
						"
					>
						<ButtonStyled
							:color="profile.installedMod ? 'green' : 'brand'"
							:type="profile.installedMod ? 'standard' : 'outlined'"
						>
							<button
								class="shrink-0 text-xs"
								:disabled="profile.installedMod || profile.installing"
								@click.stop="install(profile)"
							>
								<DownloadIcon v-if="!profile.installedMod && !profile.installing" class="w-4 h-4" />
								<CheckIcon v-else-if="profile.installedMod" class="w-4 h-4" />
								{{
									profile.installing
										? formatMessage(messages.installingAction)
										: profile.installedMod
											? formatMessage(messages.installedAction)
											: formatMessage(messages.installAction)
								}}
							</button>
						</ButtonStyled>
					</div>
				</div>
				<div v-if="shownProfiles.length === 0" class="py-6 text-center text-secondary text-sm">
					{{ formatMessage(messages.noCompatibleInstances) }}
				</div>
			</div>
			<Card v-if="showCreation" class="creation-card">
				<div class="creation-container">
					<div class="creation-icon">
						<Avatar size="md" class="icon" :src="display_icon" />
						<div class="creation-icon__description">
							<Button @click="upload_icon()">
								<UploadIcon />
								<span class="no-wrap"> {{ formatMessage(messages.selectIcon) }} </span>
							</Button>
							<Button :disabled="!display_icon" @click="reset_icon()">
								<XIcon />
								<span class="no-wrap"> {{ formatMessage(messages.removeIcon) }} </span>
							</Button>
						</div>
					</div>
					<div class="creation-settings">
						<input
							v-model="name"
							autocomplete="off"
							type="text"
							:placeholder="formatMessage(messages.name)"
							class="creation-input"
						/>
						<Button :disabled="creatingInstance === true || !name" @click="createInstance()">
							<RightArrowIcon />
							{{
								creatingInstance ? formatMessage(messages.creating) : formatMessage(messages.create)
							}}
						</Button>
					</div>
				</div>
			</Card>
			<div class="flex justify-end gap-2 pt-1">
				<Button :color="showCreation ? '' : 'primary'" @click="toggleCreation()">
					<PlusIcon />
					{{
						showCreation
							? formatMessage(messages.hideNewInstance)
							: formatMessage(messages.createInstance)
					}}
				</Button>
				<Button @click="installModal.hide()">{{ formatMessage(messages.cancel) }}</Button>
			</div>
		</div>
	</ModalWrapper>
</template>

<style scoped lang="scss">
.creation-card {
	display: flex;
	flex-direction: column;
	gap: 1rem;
	margin: 0;
	background-color: var(--color-bg);
}

.creation-container {
	display: flex;
	flex-direction: row;
	gap: 1rem;
}

.creation-icon {
	display: flex;
	flex-direction: row;
	gap: 1rem;
	align-items: center;
	flex-grow: 1;

	.creation-icon__description {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}
}

.creation-input {
	width: 100%;
}

.no-wrap {
	white-space: nowrap;
}

.creation-dropdown {
	width: min-content !important;
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
}

.creation-settings {
	width: 100%;
	margin-left: 0.5rem;
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
	justify-content: center;
}

.modal-body {
	display: flex;
	flex-direction: column;
	gap: 1rem;
	min-width: 350px;
}

.profiles {
	max-height: 12rem;
	overflow-y: auto;

	&.hide-creation {
		max-height: 21rem;
	}
}

.option {
	width: calc(100%);
	background: var(--color-raised-bg);
	color: var(--color-base);
	box-shadow: none;
	display: flex;
	flex-direction: row;
	justify-content: space-between;
	align-items: center;
	gap: 0.5rem;

	img {
		margin-right: 0.5rem;
	}

	.name {
		display: flex;
		flex-direction: column;
		justify-content: center;
	}

	.profile-button {
		align-content: start;
		padding: 0.5rem;
		text-align: left;
	}
}

.profile-image {
	--size: 2rem !important;
}
</style>
