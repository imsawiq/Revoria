<template>
	<div v-if="props.instance">
		<template v-if="projects?.length > 0">
			<div class="flex items-center gap-2 mb-4">
				<div class="iconified-input flex-grow">
					<SearchIcon />
					<input
						v-model="searchFilter"
						type="text"
						:placeholder="searchPlaceholder"
						class="text-input search-input"
						autocomplete="off"
					/>
					<Button class="r-btn" @click="() => (searchFilter = '')">
						<XIcon />
					</Button>
				</div>
				<AddContentButton :instance="props.instance" />
			</div>
			<div class="flex items-center justify-between">
				<div v-if="filterOptions.length > 1" class="flex flex-wrap gap-1 items-center pb-4">
					<FilterIcon class="text-secondary h-5 w-5 mr-1" />
					<button
						v-for="filter in filterOptions"
						:key="`content-filter-${filter.id}`"
						:class="`px-2 py-1 rounded-full font-semibold leading-none cursor-pointer active:scale-[0.97] duration-100 transition-all ${selectedFilters.includes(filter.id) ? 'bg-[--color-selected-button-bg] text-contrast border border-[--color-brand-highlight] shadow-card' : 'bg-[--color-glass-bg-strong] text-secondary border border-[--glass-border]'}`"
						@click="toggleArray(selectedFilters, filter.id)"
					>
						{{ filter.formattedName }}
					</button>
				</div>
				<Pagination
					v-if="search.length > 0"
					:page="currentPage"
					:count="Math.ceil(search.length / 20)"
					:link-function="(page) => `?page=${page}`"
					@switch-page="(page) => (currentPage = page)"
				/>
			</div>

			<ContentListPanel
				v-model="selectedFiles"
				:locked="isPackLocked"
				:items="
					search.map((x) => {
						const item: ContentItem<any> = {
							path: x.path,
							disabled: x.disabled,
							filename: x.file_name,
							icon: x.icon ?? undefined,
							title: x.name,
							data: x,
						}

						if (x.version) {
							item.version = x.version
							item.versionId = x.version
						}

						if (x.id) {
							item.project = {
								id: x.id,
								link: {
									path: `/project/${x.id}`,
									query: { i: props.instance?.path ?? '' },
								},
								linkProps: {},
							}
						}

						if (x.author) {
							item.creator = {
								name: x.author.name,
								type: x.author.type,
								id: x.author.slug,
								link: `https://modrinth.com/${x.author.type}/${x.author.slug}`,
								linkProps: { target: '_blank' },
							}
						}

						return item
					})
				"
				:sort-column="sortColumn"
				:sort-ascending="ascending"
				:update-sort="sortProjects"
				:current-page="currentPage"
			>
				<template v-if="selectedProjects.length > 0" #headers>
					<div class="flex gap-2">
						<ButtonStyled
							v-if="!isPackLocked && selectedProjects.some((m) => m.outdated)"
							color="brand"
							color-fill="text"
							hover-color-fill="text"
						>
							<button @click="updateSelected()">
								<DownloadIcon /> {{ formatMessage(messages.update) }}
							</button>
						</ButtonStyled>
						<ButtonStyled>
							<OverflowMenu
								:options="[
									{
										id: 'share-names',
										action: () => shareNames(),
									},
									{
										id: 'share-file-names',
										action: () => shareFileNames(),
									},
									{
										id: 'share-urls',
										action: () => shareUrls(),
									},
									{
										id: 'share-markdown',
										action: () => shareMarkdown(),
									},
								]"
							>
								<ShareIcon /> {{ formatMessage(messages.share) }} <DropdownIcon />
								<template #share-names>
									<TextInputIcon /> {{ formatMessage(messages.shareProjectNames) }}
								</template>
								<template #share-file-names>
									<FileIcon /> {{ formatMessage(messages.shareFileNames) }}
								</template>
								<template #share-urls>
									<LinkIcon /> {{ formatMessage(messages.shareProjectLinks) }}
								</template>
								<template #share-markdown>
									<CodeIcon /> {{ formatMessage(messages.shareMarkdownLinks) }}
								</template>
							</OverflowMenu>
						</ButtonStyled>
						<ButtonStyled v-if="selectedProjects.some((m) => m.disabled)">
							<button @click="enableAll()">
								<CheckCircleIcon /> {{ formatMessage(messages.enable) }}
							</button>
						</ButtonStyled>
						<ButtonStyled v-if="selectedProjects.some((m) => !m.disabled)">
							<button @click="disableAll()">
								<SlashIcon /> {{ formatMessage(messages.disable) }}
							</button>
						</ButtonStyled>
						<ButtonStyled color="red">
							<button @click="deleteSelected()">
								<TrashIcon /> {{ formatMessage(messages.remove) }}
							</button>
						</ButtonStyled>
					</div>
				</template>
				<template #header-actions>
					<ButtonStyled type="transparent" color-fill="text" hover-color-fill="text">
						<button :disabled="refreshingProjects" class="w-max" @click="refreshProjects">
							<UpdatedIcon />
							{{ formatMessage(messages.refresh) }}
						</button>
					</ButtonStyled>
					<ButtonStyled
						v-if="!isPackLocked && projects.some((m) => (m as any).outdated)"
						type="transparent"
						color="brand"
						color-fill="text"
						hover-color-fill="text"
						@click="updateAll"
					>
						<button class="w-max"><DownloadIcon /> {{ formatMessage(messages.updateAll) }}</button>
					</ButtonStyled>
					<ButtonStyled
						v-if="canUpdatePack"
						type="transparent"
						color="brand"
						color-fill="text"
						hover-color-fill="text"
					>
						<button class="w-max" :disabled="installing" @click="modpackVersionModal?.show()">
							<DownloadIcon /> {{ formatMessage(messages.updatePack) }}
						</button>
					</ButtonStyled>
				</template>
				<template #actions="{ item }">
					<ButtonStyled
						v-if="!isPackLocked && (item.data as any).outdated"
						type="transparent"
						color="brand"
						circular
					>
						<button
							v-tooltip="formatMessage(messages.update)"
							:disabled="(item.data as ProjectListEntry).updating"
							@click="updateProject(item.data)"
						>
							<DownloadIcon />
						</button>
					</ButtonStyled>
					<div v-else class="w-[36px]"></div>
					<Toggle
						class="!mx-2"
						:model-value="!item.data.disabled"
						@update:model-value="toggleDisableMod(item.data)"
					/>
					<ButtonStyled type="transparent" circular>
						<button v-tooltip="formatMessage(messages.removeTooltip)" @click="removeMod(item)">
							<TrashIcon />
						</button>
					</ButtonStyled>

					<ButtonStyled type="transparent" circular>
						<OverflowMenu
							:options="[
								{
									id: 'show-file',
									action: () =>
										props.instance && highlightModInProfile(props.instance.path, item.path),
								},
								{
									id: 'copy-link',
									shown: item.data !== undefined && item.data.slug !== undefined,
									action: () => copyModLink(item),
								},
							]"
							direction="left"
						>
							<MoreVerticalIcon />
							<template #show-file>
								<ExternalIcon /> {{ formatMessage(messages.showFile) }}
							</template>
							<template #copy-link>
								<ClipboardCopyIcon /> {{ formatMessage(messages.copyLink) }}
							</template>
						</OverflowMenu>
					</ButtonStyled>
				</template>
			</ContentListPanel>
			<div class="flex justify-end mt-4">
				<Pagination
					v-if="search.length > 0"
					:page="currentPage"
					:count="Math.ceil(search.length / 20)"
					:link-function="(page) => `?page=${page}`"
					@switch-page="(page) => (currentPage = page)"
				/>
			</div>
		</template>
		<div v-else class="w-full max-w-[48rem] mx-auto flex flex-col mt-6">
			<RadialHeader class="">
				<div class="flex items-center gap-6 w-[32rem] mx-auto">
					<img src="@/assets/sad-modrinth-bot.webp" class="h-24" />
					<span class="text-contrast font-bold text-xl">{{
						formatMessage(messages.emptyTitle)
					}}</span>
				</div>
			</RadialHeader>
			<div class="flex mt-4 mx-auto">
				<AddContentButton :instance="props.instance" />
			</div>
		</div>
		<ShareModalWrapper
			ref="shareModal"
			:share-title="formatMessage(messages.shareModalTitle)"
			:share-text="formatMessage(messages.shareModalText)"
			:open-in-new-tab="false"
		/>
		<ExportModal v-if="projects.length > 0" ref="exportModal" :instance="props.instance" />
		<ModpackVersionModal
			v-if="props.instance && props.instance.linked_data"
			ref="modpackVersionModal"
			:instance="props.instance"
			:versions="props.versions"
		/>
	</div>
	<div v-else class="p-6">
		<LoadingIndicator />
	</div>
</template>
<script setup lang="ts">
import {
	CheckCircleIcon,
	ClipboardCopyIcon,
	CodeIcon,
	DownloadIcon,
	DropdownIcon,
	ExternalIcon,
	FileIcon,
	FilterIcon,
	LinkIcon,
	MoreVerticalIcon,
	SearchIcon,
	ShareIcon,
	SlashIcon,
	TrashIcon,
	UpdatedIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	ButtonStyled,
	ContentListPanel,
	injectNotificationManager,
	LoadingIndicator,
	OverflowMenu,
	Pagination,
	RadialHeader,
	Toggle,
} from '@modrinth/ui'
import type { ContentItem } from '@modrinth/ui/src/components/content/ContentListItem.vue'
import type { Organization, Project, TeamMember, Version } from '@modrinth/utils'
import { formatProjectType } from '@modrinth/utils'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useStorage } from '@vueuse/core'
import dayjs from 'dayjs'
import type { ComputedRef } from 'vue'
import { computed, onUnmounted, ref, watch } from 'vue'

import { TextInputIcon } from '@/assets/icons'
import AddContentButton from '@/components/ui/AddContentButton.vue'
import type ContextMenu from '@/components/ui/ContextMenu.vue'
import ExportModal from '@/components/ui/ExportModal.vue'
import ShareModalWrapper from '@/components/ui/modal/ShareModalWrapper.vue'
import ModpackVersionModal from '@/components/ui/ModpackVersionModal.vue'
import { trackEvent } from '@/helpers/analytics'
import {
	get_organization_many,
	get_project_many,
	get_team_many,
	get_version,
	get_version_many,
} from '@/helpers/cache.js'
import { profile_listener } from '@/helpers/events.js'
import {
	add_project_from_path,
	get,
	get_projects,
	remove_project,
	toggle_disable_project,
	update_all,
	update_project,
} from '@/helpers/profile.js'
import type { CacheBehaviour, ContentFile, GameInstance } from '@/helpers/types'
import { highlightModInProfile } from '@/helpers/utils.js'
import { installVersionDependencies } from '@/store/install'

const { handleError } = injectNotificationManager()

const props = withDefaults(
	defineProps<{
		instance?: GameInstance | null
		options?: InstanceType<typeof ContextMenu> | null
		offline?: boolean
		playing?: boolean
		versions?: Version[]
		installed?: boolean
	}>(),
	{
		instance: null,
		options: null,
		offline: false,
		playing: false,
		versions: () => [],
		installed: false,
	},
)

type ProjectListEntryAuthor = {
	name: string
	slug: string
	type: 'user' | 'organization'
}

type ProjectListEntry = {
	path: string
	name: string
	slug?: string
	author: ProjectListEntryAuthor | null
	version: string | null
	file_name: string
	icon: string | undefined
	disabled: boolean
	updateVersion?: string
	outdated: boolean
	updated: dayjs.Dayjs
	project_type: string
	id?: string
	updating?: boolean
	selected?: boolean
}

const isPackLocked = computed(() => {
	return !!props.instance?.linked_data?.locked
})
const canUpdatePack = computed(() => {
	if (!props.instance?.linked_data || !props.versions || !props.versions[0]) return false
	return props.instance.linked_data.version_id !== props.versions[0].id
})
const exportModal = ref(null)

const projects = ref<ProjectListEntry[]>([])
const selectedFiles = ref<string[]>([])
const selectedProjects = computed(() =>
	projects.value.filter((x) => selectedFiles.value.includes(x.file_name)),
)

const selectionMap = ref(new Map())

const initProjects = async (cacheBehaviour?: CacheBehaviour) => {
	if (!props.instance?.path) return
	const newProjects: ProjectListEntry[] = []

	const profileProjects = (await get_projects(props.instance.path, cacheBehaviour)) as Record<
		string,
		ContentFile
	>
	const fetchProjects = []
	const fetchVersions = []

	for (const value of Object.values(profileProjects)) {
		if (value.metadata) {
			fetchProjects.push(value.metadata.project_id)
			fetchVersions.push(value.metadata.version_id)
		}
	}

	const [modrinthProjects, modrinthVersions] = await Promise.all([
		(await get_project_many(fetchProjects).catch(handleError)) as Project[],
		(await get_version_many(fetchVersions).catch(handleError)) as Version[],
	])

	const [modrinthTeams, modrinthOrganizations] = await Promise.all([
		(await get_team_many(modrinthProjects.map((x) => x.team)).catch(handleError)) as TeamMember[][],
		(await get_organization_many(
			modrinthProjects.map((x) => x.organization).filter((x) => !!x),
		).catch(handleError)) as Organization[],
	])

	for (const [path, file] of Object.entries(profileProjects)) {
		if (file.metadata) {
			const project = modrinthProjects.find((x) => file.metadata?.project_id === x.id)
			const version = modrinthVersions.find((x) => file.metadata?.version_id === x.id)

			if (project && version) {
				const org = project.organization
					? modrinthOrganizations.find((x) => x.id === project.organization)
					: null

				const team = modrinthTeams.find((x) => x[0].team_id === project.team)

				let author: ProjectListEntryAuthor | null = null
				if (org) {
					author = {
						name: org.name,
						slug: org.slug,
						type: 'organization',
					}
				} else if (team) {
					const teamMember = team.find((x) => x.is_owner)
					if (teamMember) {
						author = {
							name: teamMember.user.username,
							slug: teamMember.user.username,
							type: 'user',
						}
					}
				}

				newProjects.push({
					path,
					name: project.title,
					slug: project.slug,
					author,
					version: version.version_number,
					file_name: file.file_name,
					icon: project.icon_url,
					disabled: file.file_name.endsWith('.disabled'),
					updateVersion: file.update_version_id,
					updated: dayjs(version.date_published),
					outdated: !!file.update_version_id,
					project_type: project.project_type,
					id: project.id,
				})
			}

			continue
		}

		newProjects.push({
			path,
			name: file.file_name.replace('.disabled', ''),
			author: null,
			version: null,
			file_name: file.file_name,
			icon: undefined,
			disabled: file.file_name.endsWith('.disabled'),
			outdated: false,
			updated: dayjs(0),
			project_type: file.project_type === 'shaderpack' ? 'shader' : file.project_type,
		})
	}

	projects.value = newProjects ?? []

	const newSelectionMap = new Map()
	for (const project of projects.value) {
		newSelectionMap.set(
			project.path,
			selectionMap.value.get(project.path) ??
				selectionMap.value.get(project.path.slice(0, -9)) ??
				selectionMap.value.get(project.path + '.disabled') ??
				false,
		)
	}
	selectionMap.value = newSelectionMap
}

watch(
	() => props.instance?.path,
	async (path) => {
		if (!path) return
		await initProjects()
	},
	{ immediate: true },
)

const modpackVersionModal = ref<InstanceType<typeof ModpackVersionModal> | null>()
const installing = computed(() => (props.instance?.install_stage ?? 'installing') !== 'installed')

const vintl = useVIntl()
const { formatMessage } = vintl

type FilterOption = {
	id: string
	formattedName: string
}

const messages = defineMessages({
	updatesAvailableFilter: {
		id: 'instance.filter.updates-available',
		defaultMessage: 'Updates available',
	},
	disabledFilter: {
		id: 'instance.filter.disabled',
		defaultMessage: 'Disabled projects',
	},
	searchProjects: {
		id: 'instance.mods.search-projects',
		defaultMessage: 'Search {count} project{suffix}...',
	},
	update: { id: 'instance.mods.action.update', defaultMessage: 'Update' },
	share: { id: 'instance.mods.action.share', defaultMessage: 'Share' },
	shareProjectNames: { id: 'instance.mods.share.project-names', defaultMessage: 'Project names' },
	shareFileNames: { id: 'instance.mods.share.file-names', defaultMessage: 'File names' },
	shareProjectLinks: { id: 'instance.mods.share.project-links', defaultMessage: 'Project links' },
	shareMarkdownLinks: {
		id: 'instance.mods.share.markdown-links',
		defaultMessage: 'Markdown links',
	},
	enable: { id: 'instance.mods.action.enable', defaultMessage: 'Enable' },
	disable: { id: 'instance.mods.action.disable', defaultMessage: 'Disable' },
	remove: { id: 'remove', defaultMessage: 'Remove' },
	refresh: { id: 'action.refresh', defaultMessage: 'Refresh' },
	updateAll: { id: 'instance.mods.action.update-all', defaultMessage: 'Update all' },
	updatePack: { id: 'instance.mods.action.update-pack', defaultMessage: 'Update pack' },
	removeTooltip: { id: 'instance.mods.tooltip.remove', defaultMessage: 'Remove' },
	showFile: { id: 'show-file', defaultMessage: 'Show file' },
	copyLink: { id: 'copy-link', defaultMessage: 'Copy link' },
	emptyTitle: {
		id: 'instance.mods.empty.title',
		defaultMessage: "You haven't added any content to this instance yet.",
	},
	shareModalTitle: {
		id: 'instance.mods.share-modal.title',
		defaultMessage: 'Sharing modpack content',
	},
	shareModalText: {
		id: 'instance.mods.share-modal.text',
		defaultMessage: "Check out the projects I'm using in my modpack!",
	},
})

const filterOptions: ComputedRef<FilterOption[]> = computed(() => {
	const options: FilterOption[] = []

	const frequency = projects.value.reduce((map: Record<string, number>, item) => {
		map[item.project_type] = (map[item.project_type] || 0) + 1
		return map
	}, {})

	const types = Object.keys(frequency).sort((a, b) => frequency[b] - frequency[a])

	types.forEach((type) => {
		options.push({
			id: type,
			formattedName: formatProjectType(type) + 's',
		})
	})

	if (!isPackLocked.value && projects.value.some((m) => m.outdated)) {
		options.push({
			id: 'updates',
			formattedName: formatMessage(messages.updatesAvailableFilter),
		})
	}

	if (projects.value.some((m) => m.disabled)) {
		options.push({
			id: 'disabled',
			formattedName: formatMessage(messages.disabledFilter),
		})
	}

	return options
})

const selectedFilters = useStorage<string[]>(
	`${props.instance?.name ?? 'instance'}-mod-selected-filters`,
	[],
	sessionStorage,
	{ mergeDefaults: true },
)

const filteredProjects = computed(() => {
	const updatesFilter = selectedFilters.value.includes('updates')
	const disabledFilter = selectedFilters.value.includes('disabled')

	const typeFilters = selectedFilters.value.filter(
		(filter) => filter !== 'updates' && filter !== 'disabled',
	)

	return projects.value.filter((project) => {
		return (
			(typeFilters.length === 0 || typeFilters.includes(project.project_type)) &&
			(!updatesFilter || project.outdated) &&
			(!disabledFilter || project.disabled)
		)
	})
})

watch(filterOptions, () => {
	for (let i = 0; i < selectedFilters.value.length; i++) {
		const option = selectedFilters.value[i]
		if (!filterOptions.value.some((x) => x.id === option)) {
			selectedFilters.value.splice(i, 1)
		}
	}
})

function toggleArray<T>(array: T[], value: T) {
	if (array.includes(value)) {
		array.splice(array.indexOf(value), 1)
	} else {
		array.push(value)
	}
}

const searchFilter = ref('')
const selectAll = ref(false)
const shareModal = ref<InstanceType<typeof ShareModalWrapper> | null>()
const ascending = ref(true)
const sortColumn = ref('Name')
const currentPage = ref(1)

const searchPlaceholder = computed(() =>
	formatMessage(messages.searchProjects, {
		count: filteredProjects.value.length,
		suffix: filteredProjects.value.length === 1 ? '' : 's',
	}),
)

const selected = computed(() =>
	Array.from(selectionMap.value)
		.filter((args) => {
			return args[1]
		})
		.map((args) => {
			return projects.value.find((x) => x.path === args[0])
		}),
)

const functionValues = computed(() =>
	selectedProjects.value.length > 0 ? selectedProjects.value : Array.from(projects.value.values()),
)

const search = computed(() => {
	const filtered = filteredProjects.value.filter((mod) => {
		return mod.name.toLowerCase().includes(searchFilter.value.toLowerCase())
	})

	switch (sortColumn.value) {
		case 'Updated':
			return filtered.slice().sort((a, b) => {
				const updated = a.updated.isAfter(b.updated) ? 1 : -1
				return ascending.value ? -updated : updated
			})
		default:
			return filtered
				.slice()
				.sort((a, b) =>
					ascending.value ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name),
				)
	}
})

watch([sortColumn, ascending, selectedFilters.value, searchFilter], () => (currentPage.value = 1))

const sortProjects = (filter: string) => {
	if (sortColumn.value === filter) {
		ascending.value = !ascending.value
	} else {
		sortColumn.value = filter
		ascending.value = true
	}
}

const updateAll = async () => {
	if (!props.instance) return
	const instance = props.instance
	const setProjects = []
	const outdatedProjects = []

	for (const [i, project] of projects.value.entries()) {
		if (project.outdated) {
			project.updating = true
			setProjects.push(i)
			if (project.updateVersion) {
				outdatedProjects.push(project.updateVersion)
			}
		}
	}

	const paths = (await update_all(instance.path).catch(handleError)) as Record<string, string>

	for (const [oldVal, newVal] of Object.entries(paths)) {
		const index = projects.value.findIndex((x) => x.path === oldVal)
		projects.value[index].path = newVal
		projects.value[index].outdated = false

		projects.value[index].updateVersion = undefined
	}

	if (outdatedProjects.length > 0) {
		const profile = await get(instance.path).catch(handleError)

		if (profile) {
			for (const versionId of outdatedProjects) {
				const versionData = await get_version(versionId, 'must_revalidate').catch(handleError)

				if (versionData) {
					await installVersionDependencies(profile, versionData).catch(handleError)
				}
			}
		}
	}

	for (const project of setProjects) {
		projects.value[project].updating = false
	}

	trackEvent('InstanceUpdateAll', {
		loader: instance.loader,
		game_version: instance.game_version,
		count: setProjects.length,
		selected: selected.value.length > 1,
	})
}

const updateProject = async (mod: ProjectListEntry) => {
	if (!props.instance) return
	const instance = props.instance
	mod.updating = true
	await new Promise((resolve) => setTimeout(resolve, 0))
	mod.path = await update_project(instance.path, mod.path).catch(handleError)

	let versionNumber: string | undefined
	if (mod.updateVersion) {
		const versionData = await get_version(mod.updateVersion, 'must_revalidate').catch(handleError)

		if (versionData) {
			versionNumber = versionData.version_number
			const profile = await get(instance.path).catch(handleError)

			if (profile) {
				await installVersionDependencies(profile, versionData).catch(handleError)
			}
		}
	}

	mod.updating = false

	mod.outdated = false
	if (versionNumber) mod.version = versionNumber
	mod.updateVersion = undefined

	trackEvent('InstanceProjectUpdate', {
		loader: instance.loader,
		game_version: instance.game_version,
		id: mod.id,
		name: mod.name,
		project_type: mod.project_type,
	})
}

const locks: Record<string, string | null> = {}

const toggleDisableMod = async (mod: ProjectListEntry) => {
	if (!props.instance) return
	const instance = props.instance
	// Use mod's id as the key for the lock. If mod doesn't have a unique id, replace `mod.id` with some unique property.
	const lock = locks[mod.file_name]

	while (lock) {
		await new Promise((resolve) => {
			setTimeout((value: unknown) => resolve(value), 100)
		})
	}

	locks[mod.file_name] = 'lock'

	try {
		mod.path = await toggle_disable_project(instance.path, mod.path)
		mod.disabled = !mod.disabled

		trackEvent('InstanceProjectDisable', {
			loader: instance.loader,
			game_version: instance.game_version,
			id: mod.id,
			name: mod.name,
			project_type: mod.project_type,
			disabled: mod.disabled,
		})
	} catch (err) {
		handleError(err as Error)
	}

	locks[mod.file_name] = null
}

const removeMod = async (mod: ContentItem<ProjectListEntry>) => {
	if (!props.instance) return
	const instance = props.instance
	await remove_project(instance.path, mod.path).catch(handleError)
	projects.value = projects.value.filter((x) => mod.path !== x.path)

	trackEvent('InstanceProjectRemove', {
		loader: instance.loader,
		game_version: instance.game_version,
		id: mod.data.id,
		name: mod.data.name,
		project_type: mod.data.project_type,
	})
}

const copyModLink = async (mod: ContentItem<ProjectListEntry>) => {
	await navigator.clipboard.writeText(
		`https://modrinth.com/${mod.data.project_type}/${mod.data.slug}`,
	)
}

const deleteSelected = async () => {
	if (!props.instance) return
	const instance = props.instance
	for (const project of functionValues.value) {
		await remove_project(instance.path, project.path).catch(handleError)
	}

	projects.value = projects.value.filter((x) => !x.selected)
}

const shareNames = async () => {
	await shareModal.value?.show(functionValues.value.map((x) => x.name).join('\n'))
}

const shareFileNames = async () => {
	await shareModal.value?.show(functionValues.value.map((x) => x.file_name).join('\n'))
}

const shareUrls = async () => {
	await shareModal.value?.show(
		functionValues.value
			.filter((x) => x.slug)
			.map((x) => `https://modrinth.com/${x.project_type}/${x.slug}`)
			.join('\n'),
	)
}

const shareMarkdown = async () => {
	await shareModal.value?.show(
		functionValues.value
			.map((x) => {
				if (x.slug) {
					return `[${x.name}](https://modrinth.com/${x.project_type}/${x.slug})`
				}
				return x.name
			})
			.join('\n'),
	)
}

const updateSelected = async () => {
	const promises = []
	for (const project of functionValues.value) {
		if (project.outdated) promises.push(updateProject(project))
	}
	await Promise.all(promises).catch(handleError)
}

const enableAll = async () => {
	const promises = []
	for (const project of functionValues.value) {
		if (project.disabled) {
			promises.push(toggleDisableMod(project))
		}
	}
	await Promise.all(promises).catch(handleError)
}

const disableAll = async () => {
	const promises = []
	for (const project of functionValues.value) {
		if (!project.disabled) {
			promises.push(toggleDisableMod(project))
		}
	}
	await Promise.all(promises).catch(handleError)
}

watch(selectAll, () => {
	for (const [key, value] of Array.from(selectionMap.value)) {
		if (value !== selectAll.value) {
			selectionMap.value.set(key, selectAll.value)
		}
	}
})

const refreshingProjects = ref(false)
async function refreshProjects() {
	refreshingProjects.value = true
	await initProjects('bypass')
	refreshingProjects.value = false
}

const unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
	if (event.payload.type !== 'drop') return
	if (!props.instance) return

	for (const file of event.payload.paths) {
		if (file.endsWith('.mrpack')) continue
		await add_project_from_path(props.instance.path, file).catch(handleError)
	}
	await initProjects()
})

const unlistenProfiles = await profile_listener(
	async (event: { event: string; profile_path_id: string }) => {
		if (!props.instance) return
		if (
			event.profile_path_id === props.instance.path &&
			event.event === 'synced' &&
			props.instance.install_stage !== 'pack_installing'
		) {
			await initProjects()
		}
	},
)

onUnmounted(() => {
	unlisten()
	unlistenProfiles()
})
</script>

<style scoped lang="scss">
.text-input {
	width: 100%;
}

.manage {
	display: flex;
	gap: 0.5rem;
}

.table {
	margin-block-start: 0;
	border-radius: var(--radius-lg);
	border: 2px solid var(--color-bg);
}

.table-row {
	grid-template-columns: min-content 2fr 1fr 13.25rem;

	&.show-options {
		grid-template-columns: min-content auto;

		.options {
			display: flex;
			flex-direction: row;
			align-items: center;
			gap: var(--gap-md);
		}
	}
}

.static {
	.table-row {
		grid-template-areas: 'manage name version';
		grid-template-columns: 4.25rem 1fr 1fr;
	}

	.name-cell {
		grid-area: name;
	}

	.version {
		grid-area: version;
	}

	.manage {
		justify-content: center;
		grid-area: manage;
	}
}

.table-cell {
	align-items: center;
}

.card-row {
	display: flex;
	align-items: center;
	gap: var(--gap-md);
	justify-content: space-between;
	background-color: var(--color-glass-bg-strong);
	border: 1px solid var(--glass-border);
	box-shadow: var(--shadow-card);
}

.mod-card {
	display: flex;
	flex-direction: row;
	flex-wrap: wrap;
	gap: var(--gap-sm);
	justify-content: flex-start;
	margin-bottom: 0.5rem;
	white-space: nowrap;
	align-items: center;

	:deep(.dropdown-row) {
		.btn {
			height: 2.5rem !important;
		}
	}

	:deep(.btn) {
		height: 2.5rem;
	}

	.dropdown-input {
		flex-grow: 1;

		.animated-dropdown {
			width: unset;

			:deep(.selected) {
				border-radius: var(--radius-md) 0 0 var(--radius-md);
			}
		}

		.iconified-input {
			width: 100%;

			input {
				flex-basis: unset;
			}
		}

		:deep(.animated-dropdown) {
			.render-down {
				border-radius: var(--radius-md) 0 0 var(--radius-md) !important;
			}

			.options-wrapper {
				margin-top: 0.25rem;
				width: unset;
				border-radius: var(--radius-md);
			}

			.options {
				border-radius: var(--radius-md);
				border: 1px solid var(--color);
			}
		}
	}
}

.list-card {
	margin-top: 0.5rem;
}

.text-combo {
	display: flex;
	align-items: center;
	gap: 0.5rem;
}

.name-cell {
	padding-left: 0;

	.btn {
		margin-left: var(--gap-sm);
		min-width: unset;
	}
}

.dropdown {
	width: 7rem !important;
}

.sort {
	padding-left: 0.5rem;
}

.second-row {
	display: flex;
	align-items: flex-start;
	flex-wrap: wrap;
	gap: var(--gap-sm);

	.chips {
		flex-grow: 1;
	}
}

.modal-body {
	display: flex;
	flex-direction: column;
	gap: 1rem;
	padding: var(--gap-lg);

	.button-group {
		display: flex;
		justify-content: flex-end;
		gap: 0.5rem;
	}

	strong {
		color: var(--color-contrast);
	}
}

.mod-content {
	display: flex;
	align-items: center;
	gap: 1rem;

	.mod-text {
		display: flex;
		flex-direction: column;
	}

	.title {
		color: var(--color-contrast);
		font-weight: bolder;
	}
}

.actions-cell {
	display: flex;
	align-items: center;
	gap: 0.5rem;

	.btn {
		height: unset;
		width: unset;
		padding: 0;

		&.trash {
			color: var(--color-red);
		}

		&.update {
			color: var(--color-green);
		}

		&.share {
			color: var(--color-blue);
		}
	}
}

.more-box {
	display: flex;
	background: linear-gradient(
		180deg,
		color-mix(in oklch, var(--color-glass-bg-strong) 90%, transparent),
		color-mix(in oklch, var(--color-glass-bg) 94%, transparent)
	);
	border: 1px solid var(--glass-border);
	border-radius: var(--radius-xl);
	box-shadow: var(--shadow-card);
	padding: var(--gap-lg);

	.options {
		display: flex;
		flex-wrap: wrap;
		flex-direction: row;
		gap: var(--gap-md);
		flex-grow: 1;
	}
}

.btn {
	&.transparent {
		height: unset;
		width: unset;
		padding: 0;
		color: var(--color-base);
		gap: var(--gap-xs);
		white-space: nowrap;

		svg {
			margin-right: 0 !important;
			transition: transform 0.2s ease-in-out;

			&.open {
				transform: rotate(90deg);
			}

			&.down {
				transform: rotate(180deg);
			}
		}
	}
}
.empty-prompt {
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	gap: var(--gap-md);
	height: 100%;
	width: 100%;
	margin: auto;

	.empty-icon {
		svg {
			width: 10rem;
			height: 10rem;
			color: var(--color-contrast);
		}
	}

	p,
	h3 {
		margin: 0;
	}
}
</style>

<style lang="scss">
.select-checkbox {
	button.checkbox {
		border: none;
		margin: 0;
	}
}

.search-input {
	min-height: 2.25rem;
	background-color: transparent;
}

.top-box {
	background-image: radial-gradient(
		50% 100% at 50% 100%,
		color-mix(in oklch, var(--color-brand-highlight) 50%, transparent) 10%,
		#ffffff00 100%
	);
}

.top-box-divider {
	background-image: linear-gradient(90deg, #ffffff00 0%, var(--color-brand) 50%, #ffffff00 100%);
	width: 100%;
	height: 1px;
	opacity: 0.8;
}
</style>
