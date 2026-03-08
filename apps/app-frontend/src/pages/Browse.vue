<script setup lang="ts">
import {
	CheckIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	ExternalIcon,
	GlobeIcon,
	PlusIcon,
	SearchIcon,
	XIcon,
} from '@modrinth/assets'
import type { ProjectType, SortType, Tags } from '@modrinth/ui'
import {
    Avatar,
    Button,
    ButtonStyled,
    Checkbox,
    DropdownSelect,
    injectNotificationManager,
    LoadingIndicator,
    Pagination,
    SearchFilterControl,
    SearchSidebarFilter,
    useSearch,
    useServerSearch,
} from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { defineMessages, useVIntl } from '@vintl/vintl'
import type { Ref } from 'vue'
import { computed, nextTick, onMounted, onUnmounted, ref, shallowRef, watch } from 'vue'
import type { LocationQuery } from 'vue-router'
import { useRoute, useRouter } from 'vue-router'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import InstanceIndicator from '@/components/ui/InstanceIndicator.vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import NavTabs from '@/components/ui/NavTabs.vue'
import SearchCard from '@/components/ui/SearchCard.vue'
import ServerSearchCard from '@/components/ui/ServerSearchCard.vue'
import { get_search_results, get_version } from '@/helpers/cache.js'
import {
    constructCurseForgeCdnUrl,
    getCurseForgeCategories,
    getCurseForgeClassId,
    getCurseForgeFileDownloadUrl,
    getCurseForgeModFiles,
    getCurseForgeProjectUrl,
    searchCurseForgeMods,
} from '@/helpers/curseforge'
import {
	add_project_from_path,
	create as createInstanceProfile,
	edit as editInstanceProfile,
	get as getInstance,
	get_projects as getInstanceProjects,
	kill,
	list as listInstances,
} from '@/helpers/profile.js'
import { get_categories, get_game_versions, get_loaders } from '@/helpers/tags'
import { downloadUrlToTemp } from '@/helpers/utils'
import { useFetch } from '@/helpers/fetch.js'
import { process_listener } from '@/helpers/events'
import { get_by_profile_path } from '@/helpers/process.js'
import { install_to_existing_profile } from '@/helpers/pack.js'
import { add_server_to_profile, get_profile_worlds, getServerLatency, start_join_server } from '@/helpers/worlds.ts'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const { handleError, addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const router = useRouter()
const route = useRoute()

const projectType = computed(() => route.params.projectType as string)

const contentSource = computed(() => (route.query.source as string | undefined) ?? 'modrinth')
const isCurseForge = computed(() => contentSource.value === 'curseforge')
const browseTabTransitionKey = computed(
	() => `${projectType.value}|${contentSource.value}|${String(route.query.i ?? '')}`,
)

onMounted(() => {
	if (!route.query.source) {
		router.replace({
			path: route.path,
			query: { ...route.query, source: 'modrinth' },
		})
	}
})

const projectTypes = computed(() => {
	return [route.params.projectType as ProjectType]
})

const selectableSources = computed(() => {
	const sources = [{ label: formatMessage(messages.sourceModrinth), href: 'modrinth' }]
	// Hide CurseForge for modpacks — CF uses .zip, launcher uses .mrpack
	if (projectType.value !== 'modpack' && projectType.value !== 'server') {
		sources.push({ label: formatMessage(messages.sourceCurseForge), href: 'curseforge' })
	}
	return sources
})

const maxResultsOptions = computed(() =>
	isCurseForge.value ? ([5, 10, 15, 20, 50] as number[]) : ([5, 10, 15, 20, 50, 100] as number[]),
)

const modrinthCategoryList = ref<any[]>([])
const curseForgeCategoryList = ref<any[]>([])
const curseForgeCategoryIdMap = ref<Record<string, number>>({})
const categoryList = computed(() =>
	isCurseForge.value ? curseForgeCategoryList.value : modrinthCategoryList.value,
)
const loaderList = ref<any[]>([])
const gameVersionList = ref<any[]>([])

const tags: Ref<Tags> = computed(() => ({
	gameVersions: gameVersionList.value as any,
	loaders: loaderList.value as any,
	categories: categoryList.value as any,
}))

type GameInstance = {
	game_version: string
	loader: string
	path: string
	install_stage: string
	icon_path?: string
	name: string
}

type InstanceProject = {
	metadata: {
		project_id: string
	}
}

const instance: Ref<GameInstance | null> = ref(null)
const instanceProjects: Ref<Record<string, InstanceProject> | null> = ref(null)
const instanceHideInstalled = ref(false)
const newlyInstalled = ref<string[]>([])

const cfInstallModal = ref<any>(null)
const cfInstallTargetProject = ref<any>(null)
const cfInstallProfiles = ref<any[]>([])
const cfInstallSearching = ref('')
const serverInstallModal = ref<any>(null)
const serverInstallTargetProject = ref<ServerProject | null>(null)
const serverInstallProfiles = ref<any[]>([])
const serverInstallSearching = ref('')
const onHideServerInstallModal = () => {
	serverInstallTargetProject.value = null
}
let unlistenProcessEvents: (() => void) | null = null

function cfLoaderIdToString(id: number): string | null {
	switch (id) {
		case 1: return 'forge'
		case 4: return 'fabric'
		case 5: return 'quilt'
		case 6: return 'neoforge'
		default: return null
	}
}

const cfShownProfiles = computed(() => {
	const q = cfInstallSearching.value.toLowerCase()
	const proj = cfInstallTargetProject.value
	const projGameVersions: string[] = proj?.game_versions ?? []
	const projLoaderIds: number[] = proj?.cf_loader_ids ?? []
	const projLoaderStrings = projLoaderIds.map(cfLoaderIdToString).filter(Boolean) as string[]
	const isModType = (proj?.project_type ?? 'mod') === 'mod'

	return (cfInstallProfiles.value ?? [])
		.filter((p) => (p?.name ?? '').toLowerCase().includes(q))
		.filter((p) => {
			// If no game version info on project, show all instances
			if (projGameVersions.length === 0) return true
			const gvOk = projGameVersions.includes(p.game_version)
			if (!gvOk) return false
			// For mods, also check loader compatibility
			if (isModType && projLoaderStrings.length > 0) {
				return projLoaderStrings.includes(String(p.loader ?? '').toLowerCase())
			}
			return true
		})
})

const PERSISTENT_QUERY_PARAMS = ['i', 'ai', 'source']

onMounted(async () => {
	const [categories, loaders, availableGameVersions] = await Promise.all([
		get_categories()
			.catch(handleError)
			.then((v) => (v ?? []) as any[]),
		get_loaders()
			.catch(handleError)
			.then((v) => (v ?? []) as any[]),
		get_game_versions()
			.catch(handleError)
			.then((v) => (v ?? []) as any[]),
	])
	modrinthCategoryList.value = categories

	// Some launcher builds may cache old tag sets without server categories.
	// Backfill server categories from public API so server filters stay complete.
	const remoteServerCategories = await useFetch('https://api.modrinth.com/v2/tag/category', null, true)
		.then(async (response) => {
			if (!response?.ok) return []
			const data = await response.json()
			return Array.isArray(data)
				? data.filter((x: any) => x.project_type === 'minecraft_java_server')
				: []
		})
		.catch(() => [])

	if (remoteServerCategories.length > 0) {
		const existing = new Set(
			modrinthCategoryList.value.map(
				(x: any) => `${x.project_type}::${x.header ?? ''}::${x.name}`,
			),
		)
		for (const cat of remoteServerCategories) {
			const key = `${cat.project_type}::${cat.header ?? ''}::${cat.name}`
			if (!existing.has(key)) {
				modrinthCategoryList.value.push(cat)
			}
		}
	}

	loaderList.value = loaders
	gameVersionList.value = availableGameVersions

	await updateInstanceContext()
	await refreshSearch()

	unlistenProcessEvents = await process_listener((e: { event: string; profile_path_id: string }) => {
		if (e.event !== 'finished') return
		const projectId = Object.entries(runningServerProjects.value).find(
			([, path]) => path === e.profile_path_id,
		)?.[0]
		if (projectId) {
			const { [projectId]: _, ...remaining } = runningServerProjects.value
			runningServerProjects.value = remaining
		}
	})

	previousFilterState.value = JSON.stringify({
		query: query.value,
		filters: currentFilters.value,
		sort: currentSortType.value,
		maxResults: maxResults.value,
		projectTypes: projectTypes.value,
	})
})

onUnmounted(() => {
	unlistenProcessEvents?.()
	unlistenProcessEvents = null
})

watch(route, () => {
	updateInstanceContext()
})

async function updateInstanceContext() {
	if (route.query.i) {
		;[instance.value, instanceProjects.value] = await Promise.all([
			getInstance(route.query.i as string).catch(handleError),
			getInstanceProjects(route.query.i as string)
				.catch(handleError)
				.then((v) => (v ?? null) as any),
		])
		newlyInstalled.value = []
	}

	if (route.query.ai && !(projectTypes.value.length === 1 && projectTypes.value[0] === 'modpack')) {
		instanceHideInstalled.value = route.query.ai === 'true'
	}

	if (instance.value && instance.value.path !== route.query.i && route.path.startsWith('/browse')) {
		instance.value = null
		instanceHideInstalled.value = false
	}
}

const instanceFilters = computed(() => {
	const filters = []

	if (instance.value) {
		const gameVersion = instance.value.game_version
		if (gameVersion) {
			filters.push({
				type: 'game_version',
				option: gameVersion,
			})
		}

		const platform = instance.value.loader

		const supportedModLoaders = ['fabric', 'forge', 'quilt', 'neoforge']

		if (platform && projectTypes.value.includes('mod') && supportedModLoaders.includes(platform)) {
			filters.push({
				type: 'mod_loader',
				option: platform,
			})
		}

		if (instanceHideInstalled.value && instanceProjects.value) {
			const installedMods = Object.values(instanceProjects.value)
				.filter((x) => x.metadata)
				.map((x) => x.metadata.project_id)

			installedMods.push(...newlyInstalled.value)

			installedMods
				.map((x) => ({
					type: 'project_id',
					option: `project_id:${x}`,
					negative: true,
				}))
				.forEach((x) => filters.push(x))
		}
	}

	return filters
})

const curseForgeLoaderMap: Record<string, number> = {
	forge: 1,
	cauldron: 2,
	liteloader: 3,
	fabric: 4,
	quilt: 5,
	neoforge: 6,
}

const isCurseForgeFilterAllowed = (filterId: string) => {
	return (
		filterId === 'game_version' || filterId.endsWith('_loader') || filterId.startsWith('category_')
	)
}

const filtersForSource = computed(() => {
	if (!isCurseForge.value) {
		return filters.value
	}
	return filters.value.filter((filter) => isCurseForgeFilterAllowed(filter.id))
})

const {
	// Selections
	query,
	currentSortType,
	currentFilters,
	toggledGroups,
	maxResults,
	currentPage,
	overriddenProvidedFilterTypes,

	// Lists
	filters,
	sortTypes,

	// Computed
	requestParams,

	// Functions
	createPageParams,
} = useSearch(projectTypes, tags, instanceFilters)

const curseForgeSortTypes = computed(() => [
	{ display: formatMessage(messages.sortRelevancy), name: 'relevance' },
	{ display: formatMessage(messages.sortPopularity), name: 'follows' },
	{ display: formatMessage(messages.sortLatestUpdate), name: 'updated' },
	{ display: formatMessage(messages.sortCreationDate), name: 'newest' },
	{ display: formatMessage(messages.sortTotalDownloads), name: 'downloads' },
])

const sortTypesForSource = computed(() => (isCurseForge.value ? curseForgeSortTypes.value : sortTypes))
const {
	serverCurrentSortType,
	serverCurrentFilters,
	serverToggledGroups,
	serverSortTypes,
	serverFilterTypes,
	serverRequestParams,
	createServerPageParams,
} = useServerSearch({ tags, query, maxResults, currentPage })

watch(
	() => isCurseForge.value,
	(isCf) => {
		if (!isCf) return
		// Redirect away from unsupported CurseForge project types
		if (projectType.value === 'modpack' || projectType.value === 'server') {
			router.replace({
				path: '/browse/mod',
				query: { ...route.query },
			})
			return
		}
		if (!curseForgeSortTypes.value.some((s) => s.name === currentSortType.value?.name)) {
			currentSortType.value = {
				display: formatMessage(messages.sortRelevancy),
				name: 'relevance',
			}
		}
		currentFilters.value = currentFilters.value.filter((filter) =>
			isCurseForgeFilterAllowed(filter.type),
		)
		overriddenProvidedFilterTypes.value = overriddenProvidedFilterTypes.value.filter((filterType) =>
			isCurseForgeFilterAllowed(filterType),
		)
		toggledGroups.value = toggledGroups.value.filter((group) => group === 'all_versions')
	},
)

watch(
	[() => isCurseForge.value, projectType],
	async ([isCf]) => {
		if (!isCf) {
			curseForgeCategoryList.value = []
			curseForgeCategoryIdMap.value = {}
			return
		}
		const classId = getCurseForgeClassId(projectType.value as string)
		if (!classId) {
			curseForgeCategoryList.value = []
			curseForgeCategoryIdMap.value = {}
			return
		}
		const categories = await getCurseForgeCategories({ classId }).catch(handleError)
		const normalizedCategories = (categories ?? []).map((category: any) => ({
			name: category.name,
			header: 'categories',
			project_type: projectType.value as string,
			icon: null,
		}))
		curseForgeCategoryList.value = normalizedCategories
		curseForgeCategoryIdMap.value = Object.fromEntries(
			(categories ?? []).map((category: any) => [category.name, category.id]),
		)
	},
	{ immediate: true },
)

const previousFilterState = ref('')

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

const breadcrumbs = useBreadcrumbs()
breadcrumbs.setContext({
	name: formatMessage({ id: 'browse.breadcrumb.discover-content', defaultMessage: 'Discover content' }),
	link: route.path,
	query: route.query,
})

const loading = ref(true)

watch(projectType, () => {
	loading.value = true
})

type SearchResult = {
	project_id: string
	project_type?: string
	slug?: string
	installed?: boolean
	display_categories?: string[]
	title?: string
	description?: string
	icon_url?: string
	author?: string
	downloads?: number
	follows?: number
	source?: 'modrinth' | 'curseforge'
}

type SearchResults = {
	total_hits: number
	limit: number
	hits: SearchResult[]
}

type ServerProject = {
	project_id: string
	slug?: string
	name: string
	summary: string
	icon_url?: string | null
	categories?: string[]
	minecraft_server?: {
		region?: string
	}
	minecraft_java_server?: {
		address?: string
		verified_plays_2w?: number
		ping?: {
			data?: {
				players_online?: number
			}
		}
		content?: {
			kind?: string
			project_id?: string
			project_name?: string
			project_icon?: string
			version_id?: string
		}
	}
}

const results: Ref<SearchResults> = shallowRef({ total_hits: 0, limit: 1, hits: [] })
const pageCount = computed(() => {
	if (!results.value) return 1
	const total = Number(results.value.total_hits)
	const limit = Number(results.value.limit)
	if (!Number.isFinite(total) || total <= 0) return 1
	if (!Number.isFinite(limit) || limit <= 0) return 1
	return Math.max(1, Math.ceil(total / limit))
})
const serverHits: Ref<ServerProject[]> = shallowRef([])
const serverPings = ref<Record<string, number | undefined>>({})
const serverPingUpdatedAt = ref<Record<string, number>>({})
const runningServerProjects = ref<Record<string, string>>({})
const installingServerProjects = ref<string[]>([])
const addingServerProjects = ref<string[]>([])

watch(
	() => isCurseForge.value,
	() => {
		results.value = { total_hits: 0, limit: 1, hits: [] }
		currentPage.value = 1
		refreshSearch()
	},
)

watch(
	[
		query,
		currentPage,
		maxResults,
		currentSortType,
		currentFilters,
		overriddenProvidedFilterTypes,
		instanceFilters,
		() => route.query.source,
		projectType,
	],
	() => {
		if (!route.params.projectType) return
		if (!isCurseForge.value) return
		refreshSearch()
	},
	{ deep: true },
)

watch(
	() => isCurseForge.value,
	(isCf) => {
		if (!isCf) return
		if (Number(maxResults.value) > 50) {
			maxResults.value = 50
			currentPage.value = 1
		}
	},
)

const effectiveRequestParams = computed(() =>
	projectType.value === 'server' ? serverRequestParams.value : requestParams.value,
)

watch(effectiveRequestParams, () => {
	if (!route.params.projectType) return
	if (isCurseForge.value) return
	refreshSearch()
})

function getServerAddress(project: ServerProject) {
	return project.minecraft_java_server?.address ?? null
}

async function fetchServerSearchResults(queryString: string): Promise<{
	hits: ServerProject[]
	total_hits: number
	limit?: number
}> {
	const response = await useFetch(`https://api.modrinth.com/v3/search${queryString}`, null, false)
	if (!response?.ok) {
		throw new Error(`Server search failed with status ${response?.status ?? 'unknown'}`)
	}
	return await response.json()
}

async function pingServerHits(hits: ServerProject[]) {
	const now = Date.now()
	const freshForMs = 60_000
	const toPing = hits
		.filter((project) => !!getServerAddress(project))
		.filter((project) => {
			const ts = serverPingUpdatedAt.value[project.project_id] ?? 0
			return now - ts > freshForMs
		})
		.slice(0, 8)

	await Promise.all(
		toPing.map(async (project) => {
			const address = getServerAddress(project)
			if (!address) return
			const latency = await getServerLatency(address).catch(() => undefined)
			serverPings.value[project.project_id] = latency
			serverPingUpdatedAt.value[project.project_id] = Date.now()
		}),
	)
}

async function checkServerRunningStates(hits: ServerProject[]) {
	const packs = await listInstances().catch(() => [])
	const running: Record<string, string> = {}
	for (const project of hits) {
		const instance = packs.find((pack: any) => pack.linked_data?.project_id === project.project_id)
		if (!instance?.path) continue
		const processes = await get_by_profile_path(instance.path).catch(() => [])
		if (Array.isArray(processes) && processes.length > 0) {
			running[project.project_id] = instance.path
		}
	}
	runningServerProjects.value = running
}

async function ensureServerInInstance(path: string, name: string, address: string) {
	const worlds = await get_profile_worlds(path).catch(() => [])
	const exists = worlds.some(
		(world: any) =>
			world.type === 'server' &&
			(world.address === address || (address.endsWith(':25565') && world.address === address.split(':')[0])),
	)
	if (!exists) {
		await add_server_to_profile(path, name, address, 'prompt')
	}
}

const serverShownProfiles = computed(() => {
	const q = serverInstallSearching.value.toLowerCase()
	return (serverInstallProfiles.value ?? []).filter((p: any) =>
		String(p?.name ?? '').toLowerCase().includes(q),
	)
})

async function openAddServerToInstanceModal(project: ServerProject) {
	serverInstallTargetProject.value = project
	serverInstallSearching.value = ''
	const packs = await listInstances().catch(() => [])
	serverInstallProfiles.value = (packs ?? []).map((p: any) => ({
		...p,
		installing: false,
		added: false,
	}))
	serverInstallModal.value?.show?.(undefined as any)
}

async function handleAddServerToCurrentInstance(project: ServerProject, instancePath?: string) {
	const address = getServerAddress(project)
	if (!address) {
		addNotification({
			title: 'Server address unavailable',
			text: 'This server does not expose a join address.',
			type: 'warning',
		})
		return false
	}
	if (!instancePath) {
		return false
	}

	if (addingServerProjects.value.includes(project.project_id)) return
	addingServerProjects.value.push(project.project_id)
	try {
		await ensureServerInInstance(instancePath, project.name, address)
		addNotification({
			title: 'Server added',
			text: `${project.name} was added to ${instancePath}.`,
			type: 'success',
		})
		return true
	} catch (err) {
		handleError(err instanceof Error ? err : new Error(String(err)))
		return false
	} finally {
		addingServerProjects.value = addingServerProjects.value.filter((id) => id !== project.project_id)
	}
}

async function handlePlayServerProject(project: ServerProject) {
	if (installingServerProjects.value.includes(project.project_id)) return
	installingServerProjects.value.push(project.project_id)

	try {
		const address = getServerAddress(project)
		if (!address) {
			throw new Error('This server has no join address.')
		}

		const allInstances = await listInstances()
		let instance = allInstances.find(
			(pack: any) => pack.linked_data?.project_id === project.project_id,
		)

		const content = project.minecraft_java_server?.content
		if (content?.kind === 'modpack' && content.version_id) {
			const version = await get_version(content.version_id, 'must_revalidate').catch(() => null)
			const gameVersion = version?.game_versions?.[0]
			const contentProjectId = content.project_id ?? version?.project_id ?? project.project_id

			if (!instance) {
				const profilePath = await createInstanceProfile(
					project.name,
					gameVersion ?? '1.21.1',
					'vanilla',
					null,
					null,
					true,
				)
				await install_to_existing_profile(
					contentProjectId,
					content.version_id,
					project.name,
					profilePath,
				)
				await editInstanceProfile(profilePath, {
					linked_data: {
						project_id: project.project_id,
						version_id: content.version_id,
						locked: true,
					},
				})
				instance = await getInstance(profilePath)
			} else if (instance.linked_data?.version_id !== content.version_id) {
				await install_to_existing_profile(
					contentProjectId,
					content.version_id,
					project.name,
					instance.path,
				)
				await editInstanceProfile(instance.path, {
					linked_data: {
						project_id: project.project_id,
						version_id: content.version_id,
						locked: true,
					},
				})
			}
		} else if (!instance) {
			const latestRelease =
				gameVersionList.value.find((x: any) => x.version_type === 'release')?.version ?? '1.21.1'
			const profilePath = await createInstanceProfile(
				project.name,
				latestRelease,
				'vanilla',
				null,
				null,
				false,
			)
			await editInstanceProfile(profilePath, {
				linked_data: {
					project_id: project.project_id,
					version_id: '',
					locked: false,
				},
			})
			instance = await getInstance(profilePath)
		}

		if (!instance?.path) {
			throw new Error('Failed to create/find an instance for this server.')
		}

		await ensureServerInInstance(instance.path, project.name, address)
		await start_join_server(instance.path, address)
		await checkServerRunningStates(serverHits.value)
	} catch (err) {
		handleError(err instanceof Error ? err : new Error(String(err)))
	} finally {
		installingServerProjects.value = installingServerProjects.value.filter(
			(id) => id !== project.project_id,
		)
	}
}

async function handleStopServerProject(projectId: string) {
	const instancePath = runningServerProjects.value[projectId]
	if (!instancePath) return
	await kill(instancePath).catch(handleError)
	const { [projectId]: _, ...remaining } = runningServerProjects.value
	runningServerProjects.value = remaining
}

async function refreshSearch() {
	if (projectType.value === 'server' && isCurseForge.value) {
		await router.replace({
			path: route.path,
			query: { ...route.query, source: 'modrinth' },
		})
		return
	}

	if (isCurseForge.value) {
		loading.value = true
		try {
			const validProvidedFilters = instanceFilters.value.filter(
				(providedFilter: any) => !overriddenProvidedFilterTypes.value.includes(providedFilter.type),
			)
			const effectiveFilters = [...currentFilters.value, ...validProvidedFilters]
			const gameVersionFilter = effectiveFilters.find(
				(filter: any) => filter.type === 'game_version',
			)
			const modLoaderTypes = Array.from(
				new Set(
					effectiveFilters
						.filter((filter: any) => filter.type.endsWith('_loader'))
						.map((filter: any) => curseForgeLoaderMap[String(filter.option).toLowerCase()])
						.filter((value: number | undefined) => Number.isFinite(value))
						.map((value: number) => value),
				),
			)
			const categoryIds = Array.from(
				new Set(
					effectiveFilters
						.filter((filter: any) => filter.type.startsWith('category_'))
						.map((filter: any) => curseForgeCategoryIdMap.value[filter.option])
						.filter((value: number | undefined) => Number.isFinite(value))
						.map((value: number) => value),
				),
			)
			const pageSize = Math.min(Number(maxResults.value) || 10, 50)
			const index = (currentPage.value - 1) * pageSize
			const cf = await searchCurseForgeMods({
				projectType: projectType.value as string,
				query: query.value,
				sortType: currentSortType.value?.name,
				gameVersion: gameVersionFilter?.option,
				modLoaderTypes,
				categoryIds,
				index,
				pageSize,
			})

			if (!cf || !cf.pagination) {
				results.value = { total_hits: 0, limit: 1, hits: [] }
				return
			}

			results.value = {
				total_hits: cf.pagination?.totalCount ?? 0,
				limit: cf.pagination?.pageSize ?? 1,
				hits: (cf.data ?? []).map((m: any) => {
					const indexes = m.latestFilesIndexes ?? []
					const gameVersions = [...new Set(indexes.map((f: any) => f.gameVersion).filter(Boolean))]
					const cfLoaderIds = [...new Set(indexes.map((f: any) => f.modLoader).filter((v: any) => v != null && v !== 0))]
					return {
						project_id: String(m.id),
						slug: m.slug,
						project_type: projectType.value as any,
						title: m.name,
						description: m.summary,
						icon_url: m.logo?.thumbnailUrl ?? m.logo?.url ?? null,
						author: m.authors?.[0]?.name,
						downloads: m.downloadCount,
						follows: m.likeCount,
						date_modified: m.dateModified,
						display_categories: (m.categories ?? [])
							.map((c: any) => c.name)
							.filter(Boolean),
						source: 'curseforge',
						game_versions: gameVersions,
						cf_loader_ids: cfLoaderIds,
					}
				}),
			}
		} catch (e) {
			const err = e instanceof Error ? e : new Error(String(e))
			handleError(err)
			results.value = { total_hits: 0, limit: 1, hits: [] }
		} finally {
			loading.value = false
		}
	} else {
		if (projectType.value === 'server') {
			const serverResults = await fetchServerSearchResults(serverRequestParams.value).catch(
				handleError,
			)
			serverHits.value = serverResults?.hits ?? []
			results.value = {
				total_hits: serverResults?.total_hits ?? 0,
				limit: serverResults?.limit ?? maxResults.value,
				hits: [],
			}
			serverPings.value = {}
			loading.value = false
			void pingServerHits(serverHits.value)
			void checkServerRunningStates(serverHits.value)
		} else {
			let rawResults = await get_search_results(requestParams.value)
			if (!rawResults) {
				rawResults = {
					result: {
						hits: [],
						total_hits: 0,
						limit: 1,
					},
				}
			}
			if (instance.value) {
				for (const val of rawResults.result.hits as any[]) {
					val.installed =
						newlyInstalled.value.includes(val.project_id) ||
						Object.values(instanceProjects.value ?? {}).some(
							(x) => x?.metadata && x.metadata.project_id === val.project_id,
						)
				}
			}
			results.value = rawResults.result as SearchResults
			results.value.hits = (results.value.hits ?? []).map((h: any) => ({
				...h,
				source: 'modrinth',
			}))
		}
		if (projectType.value !== 'server') loading.value = false
	}

	const currentFilterState =
		projectType.value === 'server'
			? JSON.stringify({
					query: query.value,
					filters: serverCurrentFilters.value,
					sort: serverCurrentSortType.value,
					maxResults: maxResults.value,
					projectTypes: projectTypes.value,
				})
			: JSON.stringify({
					query: query.value,
					filters: currentFilters.value,
					sort: currentSortType.value,
					maxResults: maxResults.value,
					projectTypes: projectTypes.value,
				})

	if (previousFilterState.value && previousFilterState.value !== currentFilterState) {
		currentPage.value = 1
	}

	previousFilterState.value = currentFilterState

	const persistentParams: LocationQuery = {}

	for (const [key, value] of Object.entries(route.query)) {
		if (PERSISTENT_QUERY_PARAMS.includes(key)) {
			persistentParams[key] = value
		}
	}

	if (instanceHideInstalled.value) {
		persistentParams.ai = 'true'
	} else {
		delete persistentParams.ai
	}

	const params = {
		...persistentParams,
		...(projectType.value === 'server' ? createServerPageParams() : createPageParams()),
	}

	breadcrumbs.setContext({
		name: formatMessage(messages.breadcrumbDiscoverContent),
		link: `/browse/${projectType.value}`,
		query: params,
	})
	await router.replace({ path: route.path, query: params })
}

async function setPage(newPageNumber: number) {
	currentPage.value = newPageNumber

	await onSearchChangeToTop()
}

const searchWrapper: Ref<HTMLElement | null> = ref(null)

async function onSearchChangeToTop() {
	await nextTick()

	window.scrollTo({ top: 0, behavior: 'smooth' })
}

function clearSearch() {
	query.value = ''
	currentPage.value = 1
}

async function installCurseForgeProject(project: any) {
	if (!instance.value?.path) {
		await openCurseForgeInstallModal(project)
		return
	}
	return await installCurseForgeProjectToInstance(project, instance.value)
}

async function openCurseForgeInstallModal(project: any) {
	try {
		cfInstallTargetProject.value = project
		cfInstallSearching.value = ''
		const profiles = (await listInstances().catch(handleError)) ?? []
		for (const profile of profiles) {
			;(profile as any).installing = false
			;(profile as any).installed = false
		}
		cfInstallProfiles.value = profiles
		cfInstallModal.value?.show?.(undefined as any)
	} catch (e) {
		const err = e instanceof Error ? e : new Error(String(e))
		handleError(err)
	}
}

function notifyCurseForgeManualDownload(project: any, message: string) {
	addNotification({
		title: formatMessage(messages.manualDownloadRequiredTitle),
		text: message,
		type: 'warning',
	})
	openUrl(
		getCurseForgeProjectUrl(
			String(project?.project_type ?? projectType.value ?? 'mod'),
			project?.slug,
		),
	)
}

function getLoaderToken(loader: string | undefined | null) {
	switch (String(loader ?? '').toLowerCase()) {
		case 'fabric':
			return 'fabric'
		case 'forge':
			return 'forge'
		case 'quilt':
			return 'quilt'
		case 'neoforge':
			return 'neoforge'
		default:
			return null
	}
}

function fileLooksCompatible(file: any, instanceVal: GameInstance) {
	const versions: string[] = Array.isArray(file?.gameVersions) ? file.gameVersions : []
	const gvOk = instanceVal?.game_version ? versions.includes(instanceVal.game_version) : true
	const loaderToken = getLoaderToken(instanceVal?.loader)
	if (!loaderToken) return gvOk
	const loaderOk = versions.some((v) => String(v).toLowerCase() === loaderToken)
	return gvOk && loaderOk
}

async function installCurseForgeProjectToInstance(project: any, instanceVal: GameInstance) {
	const modId = Number(project.project_id ?? project.id)
	if (!Number.isFinite(modId)) {
		handleError(new Error(formatMessage(messages.invalidCurseForgeProjectId)))
		return
	}
	try {
		const files = await getCurseForgeModFiles({
			modId,
			index: 0,
			pageSize: 50,
			gameVersion: instanceVal?.game_version,
		}).catch(handleError)

		const sorted = (files?.data ?? [])
			.slice()
			.sort((a: any, b: any) => Date.parse(b.fileDate) - Date.parse(a.fileDate))
		const file =
			sorted.find((f: any) => fileLooksCompatible(f, instanceVal)) ??
			sorted.find((f: any) => !!f.downloadUrl) ??
			sorted[0]
		if (!file) {
			handleError(new Error(formatMessage(messages.noFilesAvailableForProject)))
			return
		}

		let downloadUrl = file.downloadUrl || null
		if (!downloadUrl) {
			downloadUrl = await getCurseForgeFileDownloadUrl(modId, file.id).catch(() => null)
		}
		if (!downloadUrl && file.id && file.fileName) {
			downloadUrl = constructCurseForgeCdnUrl(file.id, file.fileName)
		}
		if (!downloadUrl) {
			notifyCurseForgeManualDownload(
				project,
				formatMessage(messages.curseForgeManualDownloadFallback),
			)
			return
		}

		const suggestedName = (file.fileName || `curseforge-${modId}-${file.id}.jar`).replace(
			/[^a-zA-Z0-9._-]/g,
			'_',
		)
		const downloadedPath = await downloadUrlToTemp(downloadUrl, suggestedName, instanceVal.path)
		const backendType = (project.project_type ?? 'mod') === 'shader' ? 'shaderpack' : String(project.project_type ?? 'mod')
		await add_project_from_path(
			instanceVal.path,
			downloadedPath,
			backendType,
		)
		newlyInstalled.value.push(String(modId))
	} catch (e) {
		const err = e instanceof Error ? e : new Error(String(e))
		// Common CF cases: API returns 403 for restricted/manual-download files.
		if (String(err.message ?? '').includes('(403)') || String(err.message ?? '').includes('403')) {
			notifyCurseForgeManualDownload(
				project,
				formatMessage(messages.curseForgeBlockedAutomatedDownload),
			)
			return
		}
		handleError(err)
	}
}

watch(projectType, () => {
	currentSortType.value = { display: formatMessage(messages.sortRelevancy), name: 'relevance' }
	query.value = ''
})

const selectableProjectTypes = computed(() => {
	let dataPacks = false,
		mods = false,
		modpacks = false

	if (instance.value) {
		if (
			gameVersionList.value.findIndex((x: any) => x.version === instance.value?.game_version) <=
			gameVersionList.value.findIndex((x: any) => x.version === '1.13')
		) {
			dataPacks = true
		}

		if (instance.value.loader !== 'vanilla') {
			mods = true
		}
	} else {
		dataPacks = true
		mods = true
		// Hide modpacks for CurseForge — CF uses .zip, launcher uses .mrpack
		modpacks = !isCurseForge.value
	}

	const params: LocationQuery = {}

	if (route.query.i) {
		params.i = route.query.i
	}
	if (route.query.ai) {
		params.ai = route.query.ai
	}
	if (route.query.source) {
		params.source = route.query.source
	}

	const links = [
		{ label: formatMessage(messages.typeModpack), href: `/browse/modpack`, shown: modpacks },
		{ label: formatMessage(messages.typeMod), href: `/browse/mod`, shown: mods },
		{ label: formatMessage(messages.typeResourcePack), href: `/browse/resourcepack` },
		{ label: formatMessage(messages.typeDatapack), href: `/browse/datapack`, shown: dataPacks },
		{ label: formatMessage(messages.typeShader), href: `/browse/shader` },
		{ label: formatMessage(messages.typeServer), href: `/browse/server`, shown: !instance.value },
	]

	if (params) {
		return links.map((link) => {
			return {
				...link,
				href: {
					path: link.href,
					query: params,
				},
			}
		})
	}

	return links
})

const messages = defineMessages({
	sourceModrinth: {
		id: 'browse.source.modrinth',
		defaultMessage: 'Modrinth',
	},
	sourceCurseForge: {
		id: 'browse.source.curseforge',
		defaultMessage: 'CurseForge',
	},
	typeModpack: {
		id: 'browse.type.modpack',
		defaultMessage: 'Modpacks',
	},
	typeMod: {
		id: 'browse.type.mod',
		defaultMessage: 'Mods',
	},
	typeResourcePack: {
		id: 'browse.type.resourcepack',
		defaultMessage: 'Resource Packs',
	},
	typeDatapack: {
		id: 'browse.type.datapack',
		defaultMessage: 'Data Packs',
	},
	typeShader: {
		id: 'browse.type.shader',
		defaultMessage: 'Shaders',
	},
	typeServer: {
		id: 'browse.type.server',
		defaultMessage: 'Servers',
	},
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
	cancel: {
		id: 'action.cancel',
		defaultMessage: 'Cancel',
	},
	hideInstalled: {
		id: 'browse.filter.hide-installed',
		defaultMessage: 'Hide installed content',
	},
	installContentTitle: {
		id: 'browse.install.instance',
		defaultMessage: 'Install content to instance',
	},
	searchPlaceholder: {
		id: 'browse.search.placeholder',
		defaultMessage: 'Search {type}...',
	},
	sortByLabel: {
		id: 'browse.sort.label',
		defaultMessage: 'Sort by',
	},
	viewLabel: {
		id: 'browse.view.label',
		defaultMessage: 'View',
	},
	maxResultsLabel: {
		id: 'browse.max-results.label',
		defaultMessage: 'Max results',
	},
	offlineMessage: {
		id: 'browse.offline',
		defaultMessage: 'You are currently offline. Connect to the internet to browse Modrinth!',
	},
	openCurseForge: {
		id: 'browse.context.open-curseforge',
		defaultMessage: 'Open in CurseForge',
	},
	openModrinth: {
		id: 'browse.context.open-modrinth',
		defaultMessage: 'Open in Modrinth',
	},
	copyLink: {
		id: 'browse.context.copy-link',
		defaultMessage: 'Copy link',
	},
	gameVersionProvidedByInstance: {
		id: 'search.filter.locked.instance-game-version.title',
		defaultMessage: 'Game version is provided by the instance',
	},
	modLoaderProvidedByInstance: {
		id: 'search.filter.locked.instance-loader.title',
		defaultMessage: 'Loader is provided by the instance',
	},
	providedByInstance: {
		id: 'search.filter.locked.instance',
		defaultMessage: 'Provided by the instance',
	},
	syncFilterButton: {
		id: 'search.filter.locked.instance.sync',
		defaultMessage: 'Sync with instance',
	},
	breadcrumbDiscoverContent: {
		id: 'browse.breadcrumb.discover-content',
		defaultMessage: 'Discover content',
	},
	sortRelevancy: {
		id: 'browse.sort.relevancy',
		defaultMessage: 'Relevancy',
	},
	sortPopularity: {
		id: 'browse.sort.popularity',
		defaultMessage: 'Popularity',
	},
	sortLatestUpdate: {
		id: 'browse.sort.latest-update',
		defaultMessage: 'Latest update',
	},
	sortCreationDate: {
		id: 'browse.sort.creation-date',
		defaultMessage: 'Creation Date',
	},
	sortTotalDownloads: {
		id: 'browse.sort.total-downloads',
		defaultMessage: 'Total Downloads',
	},
	manualDownloadRequiredTitle: {
		id: 'browse.notification.manual-download-required.title',
		defaultMessage: 'Manual download required',
	},
	invalidCurseForgeProjectId: {
		id: 'browse.error.invalid-curseforge-project-id',
		defaultMessage: 'Invalid CurseForge project id.',
	},
	noFilesAvailableForProject: {
		id: 'browse.error.no-files-available',
		defaultMessage: 'No files available for this project.',
	},
	curseForgeManualDownloadFallback: {
		id: 'browse.notification.curseforge-manual-download-fallback',
		defaultMessage:
			'CurseForge could not provide a direct download. Opened the project page in your browser.',
	},
	curseForgeBlockedAutomatedDownload: {
		id: 'browse.notification.curseforge-blocked-automated-download',
		defaultMessage:
			'CurseForge blocked automated download for this file (403). Opened the project page in your browser.',
	},
})

const projectTypeLabels = computed(() => ({
	mod: formatMessage(messages.typeMod),
	modpack: formatMessage(messages.typeModpack),
	resourcepack: formatMessage(messages.typeResourcePack),
	datapack: formatMessage(messages.typeDatapack),
	shader: formatMessage(messages.typeShader),
	server: formatMessage(messages.typeServer),
}))

const options = ref<any>(null)
const handleRightClick = (event: MouseEvent, result: SearchResult) => {
	options.value?.showMenu(event, result as any, [
		{
			name: 'open_link',
		},
		{
			name: 'copy_link',
		},
	])
}
const handleOptionsClick = (args: any) => {
	switch (args.option) {
		case 'open_link':
			if (args.item?.source === 'curseforge') {
				openUrl(
					getCurseForgeProjectUrl(args.item.project_type ?? projectType.value, args.item.slug),
				)
			} else {
				openUrl(`https://modrinth.com/${args.item.project_type}/${args.item.slug}`)
			}
			break
		case 'copy_link':
			if (args.item?.source === 'curseforge') {
				navigator.clipboard.writeText(
					getCurseForgeProjectUrl(args.item.project_type ?? projectType.value, args.item.slug),
				)
			} else {
				navigator.clipboard.writeText(
					`https://modrinth.com/${args.item.project_type}/${args.item.slug}`,
				)
			}
			break
	}
}

</script>

<template>
	<ModalWrapper
		ref="serverInstallModal"
		:header="'Add server to instance'"
		:on-hide="onHideServerInstallModal"
	>
		<div class="flex flex-col gap-3 min-w-[400px] max-w-[500px]">
			<p v-if="serverInstallTargetProject" class="m-0 text-sm text-secondary">
				Select an instance to add
				<strong class="text-contrast">{{ serverInstallTargetProject.name }}</strong>
			</p>
			<div class="flex items-center gap-2 rounded-xl bg-[--color-button-bg] px-3 py-2 border border-[--glass-border]">
				<SearchIcon class="w-4 h-4 text-secondary shrink-0" />
				<input
					v-model="serverInstallSearching"
					autocomplete="off"
					type="text"
					class="w-full bg-transparent border-none outline-none text-sm text-contrast placeholder:text-secondary"
					placeholder="Search instances..."
				/>
			</div>
			<div class="flex flex-col gap-1 max-h-[20rem] overflow-y-auto pr-1">
				<div
					v-for="profile in serverShownProfiles"
					:key="profile.path"
					class="flex items-center gap-3 rounded-xl px-3 py-2.5 transition-colors hover:bg-[--color-button-bg] group"
				>
					<router-link
						class="flex items-center gap-3 flex-1 min-w-0 no-underline text-contrast"
						:to="`/instance/${encodeURIComponent(profile.path)}`"
						@click="serverInstallModal?.hide?.()"
					>
						<Avatar
							:src="profile.icon_path ? convertFileSrc(profile.icon_path) : undefined"
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
					<ButtonStyled
						:color="profile.added ? 'green' : 'brand'"
						:type="profile.added ? 'standard' : 'outlined'"
					>
						<button
							class="shrink-0 text-xs min-w-[5rem] justify-center"
							:disabled="profile.installing || profile.added || !serverInstallTargetProject"
							@click.stop="
								async () => {
									if (!serverInstallTargetProject) return
									profile.installing = true
									const added = await handleAddServerToCurrentInstance(
										serverInstallTargetProject,
										profile.path,
									)
									profile.installing = false
									if (added) {
										profile.added = true
									}
								}
							"
						>
							<PlusIcon v-if="!profile.added && !profile.installing" class="w-4 h-4" />
							<CheckIcon v-else-if="profile.added" class="w-4 h-4" />
							{{
								profile.installing
									? 'Adding...'
									: profile.added
										? 'Added'
										: 'Add'
							}}
						</button>
					</ButtonStyled>
				</div>
				<div v-if="serverShownProfiles.length === 0" class="py-6 text-center text-secondary text-sm">
					No instances found.
				</div>
			</div>
			<div class="flex justify-end pt-1">
				<ButtonStyled>
					<button @click="serverInstallModal?.hide?.()">Cancel</button>
				</ButtonStyled>
			</div>
		</div>
	</ModalWrapper>
	<ModalWrapper
		ref="cfInstallModal"
		:header="formatMessage(messages.installToInstanceTitle)"
		:on-hide="
			() => {
				cfInstallTargetProject.value = null
			}
		"
	>
		<div class="flex flex-col gap-3 min-w-[400px] max-w-[500px]">
			<p v-if="cfInstallTargetProject" class="m-0 text-sm text-secondary">
				{{ formatMessage(messages.installToInstanceDescription) }}
				<strong class="text-contrast">{{ cfInstallTargetProject.title }}</strong>
			</p>
			<div class="flex items-center gap-2 rounded-xl bg-[--color-button-bg] px-3 py-2 border border-[--glass-border]">
				<SearchIcon class="w-4 h-4 text-secondary shrink-0" />
				<input
					v-model="cfInstallSearching"
					autocomplete="off"
					type="text"
					class="w-full bg-transparent border-none outline-none text-sm text-contrast placeholder:text-secondary"
					:placeholder="formatMessage(messages.searchInstancesPlaceholder)"
				/>
			</div>
			<div class="flex flex-col gap-1 max-h-[20rem] overflow-y-auto pr-1">
				<div
					v-for="profile in cfShownProfiles"
					:key="profile.path"
					class="flex items-center gap-3 rounded-xl px-3 py-2.5 transition-colors hover:bg-[--color-button-bg] group"
				>
					<router-link
						class="flex items-center gap-3 flex-1 min-w-0 no-underline text-contrast"
						:to="`/instance/${encodeURIComponent(profile.path)}`"
						@click="cfInstallModal?.hide?.()"
					>
						<Avatar
							:src="profile.icon_path ? convertFileSrc(profile.icon_path) : undefined"
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
					<ButtonStyled :color="profile.installed ? 'green' : 'brand'" :type="profile.installed ? 'standard' : 'outlined'">
						<button
							class="shrink-0 text-xs"
							:disabled="profile.installing || profile.installed"
							@click.stop="
								async () => {
									profile.installing = true
									await installCurseForgeProjectToInstance(cfInstallTargetProject, profile)
									profile.installing = false
									profile.installed = true
								}
							"
						>
							<DownloadIcon v-if="!profile.installed && !profile.installing" class="w-4 h-4" />
							<CheckIcon v-else-if="profile.installed" class="w-4 h-4" />
							{{
								profile.installing
									? formatMessage(messages.installingAction)
									: profile.installed
										? formatMessage(messages.installedAction)
										: formatMessage(messages.installAction)
							}}
						</button>
					</ButtonStyled>
				</div>
				<div v-if="cfShownProfiles.length === 0" class="py-6 text-center text-secondary text-sm">
					{{ formatMessage(messages.noCompatibleInstances) }}
				</div>
			</div>
			<div class="flex justify-end pt-1">
				<ButtonStyled>
					<button @click="cfInstallModal?.hide?.()">{{ formatMessage(messages.cancel) }}</button>
				</ButtonStyled>
			</div>
		</div>
	</ModalWrapper>
	<Teleport v-if="filtersForSource" to="#sidebar-teleport-target">
		<div
			v-if="instance"
			class="border-0 border-b-[1px] p-4 last:border-b-0 border-[--brand-gradient-border] border-solid"
		>
			<Checkbox
				v-model="instanceHideInstalled"
				:label="formatMessage(messages.hideInstalled)"
				class="filter-checkbox"
				@update:model-value="onSearchChangeToTop()"
				@click.prevent.stop
			/>
		</div>
		<template v-if="projectType === 'server'">
			<SearchSidebarFilter
				v-for="filterType in serverFilterTypes.filter((f) => f.options.length > 0)"
				:key="`server-filter-${filterType.id}`"
				v-model:selected-filters="serverCurrentFilters"
				v-model:toggled-groups="serverToggledGroups"
				:provided-filters="[]"
				:filter-type="filterType"
				class="border-0 border-b-[1px] [&:first-child>button]:pt-4 last:border-b-0 border-[--brand-gradient-border] border-solid"
				button-class="button-animation flex flex-col gap-1 px-4 py-3 w-full bg-transparent cursor-pointer border-none hover:bg-button-bg"
				content-class="mb-3"
				inner-panel-class="ml-2 mr-3"
			>
				<template #header>
					<h3 class="text-base m-0">{{ filterType.formatted_name }}</h3>
				</template>
			</SearchSidebarFilter>
		</template>
		<template v-else>
			<SearchSidebarFilter
				v-for="filter in filtersForSource.filter((f) => f.display !== 'none')"
				:key="`filter-${filter.id}`"
				v-model:selected-filters="currentFilters"
				v-model:toggled-groups="toggledGroups"
				v-model:overridden-provided-filter-types="overriddenProvidedFilterTypes"
				:provided-filters="instanceFilters"
				:filter-type="filter"
				class="border-0 border-b-[1px] [&:first-child>button]:pt-4 last:border-b-0 border-[--brand-gradient-border] border-solid"
				button-class="button-animation flex flex-col gap-1 px-4 py-3 w-full bg-transparent cursor-pointer border-none hover:bg-button-bg"
				content-class="mb-3"
				inner-panel-class="ml-2 mr-3"
				:open-by-default="
					filter.id.startsWith('category') || filter.id === 'environment' || filter.id === 'license'
				"
			>
				<template #header>
					<h3 class="text-base m-0">{{ filter.formatted_name }}</h3>
				</template>
				<template #locked-game_version>
					{{ formatMessage(messages.gameVersionProvidedByInstance) }}
				</template>
				<template #locked-mod_loader>
					{{ formatMessage(messages.modLoaderProvidedByInstance) }}
				</template>
				<template #sync-button> {{ formatMessage(messages.syncFilterButton) }} </template>
			</SearchSidebarFilter>
		</template>
	</Teleport>
	<div ref="searchWrapper" class="flex flex-col gap-3 p-6">
		<template v-if="instance">
			<InstanceIndicator :instance="instance" />
			<h1 class="m-0 mb-1 text-xl">{{ formatMessage(messages.installContentTitle) }}</h1>
		</template>
		<NavTabs query="source" :links="selectableSources" />
		<NavTabs :links="selectableProjectTypes" />
		<div class="iconified-input">
			<SearchIcon aria-hidden="true" class="text-lg" />
			<input
				v-model="query"
				class="h-12"
				autocomplete="off"
				spellcheck="false"
				type="text"
				:placeholder="
					formatMessage(messages.searchPlaceholder, {
						type: projectTypeLabels[projectType] ?? projectType,
					})
				"
			/>
			<Button v-if="query" class="r-btn" @click="() => clearSearch()">
				<XIcon />
			</Button>
		</div>
		<div class="flex gap-2">
			<DropdownSelect
				v-slot="{ selected }"
				:model-value="projectType === 'server' ? serverCurrentSortType : currentSortType"
				class="!w-auto flex-grow md:flex-grow-0"
				:name="formatMessage(messages.sortByLabel)"
				:options="projectType === 'server' ? serverSortTypes : [...sortTypesForSource]"
				:display-name="(option?: SortType) => option?.display"
				@update:model-value="
					(v: SortType) => {
						if (projectType === 'server') {
							serverCurrentSortType = v
						} else {
							currentSortType = v
						}
						onSearchChangeToTop()
					}
				"
			>
				<span class="font-semibold text-primary">{{ formatMessage(messages.sortByLabel) }}: </span>
				<span class="font-semibold text-secondary">{{ selected }}</span>
			</DropdownSelect>
			<DropdownSelect
				v-slot="{ selected }"
				v-model="maxResults"
				:name="formatMessage(messages.maxResultsLabel)"
				:options="maxResultsOptions"
				class="max-w-[9rem]"
			>
				<span class="font-semibold text-primary">{{ formatMessage(messages.viewLabel) }}: </span>
				<span class="font-semibold text-secondary">{{ selected }}</span>
			</DropdownSelect>
			<Pagination :page="currentPage" :count="pageCount" class="ml-auto" @switch-page="setPage" />
		</div>
		<SearchFilterControl
			v-if="projectType === 'server'"
			v-model:selected-filters="serverCurrentFilters"
			:filters="serverFilterTypes"
			:provided-filters="[]"
			:overridden-provided-filter-types="[]"
		/>
		<SearchFilterControl
			v-else
			v-model:selected-filters="currentFilters"
			:filters="filtersForSource.filter((f) => f.display !== 'none')"
			:provided-filters="instanceFilters"
			:overridden-provided-filter-types="overriddenProvidedFilterTypes"
			:provided-message="messages.providedByInstance"
		/>
		<Transition name="browse-tab-swap" mode="out-in">
			<div :key="browseTabTransitionKey" class="browse-tab-swap-shell">
				<div class="search">
					<section v-if="loading" class="offline">
						<LoadingIndicator />
					</section>
					<section v-else-if="offline && !isCurseForge && results.total_hits === 0" class="offline">
						{{ formatMessage(messages.offlineMessage) }}
					</section>
					<section v-else class="project-list display-mode--list instance-results" role="list">
						<template v-if="projectType === 'server'">
							<ServerSearchCard
								v-for="project in serverHits"
								:key="`server-${project.project_id}`"
								:project="project"
								:tag-definitions="tags"
								:ping="serverPings[project.project_id]"
								:running="!!runningServerProjects[project.project_id]"
								:installing="installingServerProjects.includes(project.project_id)"
								:adding="addingServerProjects.includes(project.project_id)"
								@add="openAddServerToInstanceModal(project)"
								@play="handlePlayServerProject(project)"
								@stop="handleStopServerProject(project.project_id)"
								@contextmenu="
									(event: MouseEvent) =>
										handleRightClick(event, {
											project_type: 'server',
											slug: project.slug ?? project.project_id,
											source: 'modrinth',
										})
								"
							/>
						</template>
						<template v-else>
							<SearchCard
								v-for="result in results.hits"
								:key="result?.project_id"
								:project-type="projectType"
								:project="result"
								:tag-definitions="tags"
								:instance="instance ?? undefined"
								:installed="result.installed || newlyInstalled.includes(result.project_id)"
								@install="
									(id) => {
										if (isCurseForge) {
											installCurseForgeProject(result)
										} else {
											newlyInstalled.push(id)
										}
									}
								"
								@contextmenu.prevent.stop="(event: MouseEvent) => handleRightClick(event, result)"
							/>
						</template>
						<ContextMenu ref="options" @option-clicked="handleOptionsClick">
							<template #open_link>
								<GlobeIcon />
								{{
									isCurseForge
										? formatMessage(messages.openCurseForge)
										: formatMessage(messages.openModrinth)
								}}
								<ExternalIcon />
							</template>
							<template #copy_link>
								<ClipboardCopyIcon /> {{ formatMessage(messages.copyLink) }}
							</template>
						</ContextMenu>
					</section>
					<div class="flex justify-end">
						<Pagination
							:page="currentPage"
							:count="pageCount"
							class="pagination-after"
							@switch-page="setPage"
						/>
					</div>
				</div>
			</div>
		</Transition>
	</div>
</template>

<style scoped lang="scss">
.browse-tab-swap-shell {
	will-change: transform, opacity, filter;
}

:deep(.project-list) {
	overflow: visible !important;
	padding-top: 2px;
}

.browse-tab-swap-enter-active,
.browse-tab-swap-leave-active {
	transition:
		opacity 240ms cubic-bezier(0.2, 0.8, 0.2, 1),
		transform 300ms cubic-bezier(0.16, 1, 0.3, 1),
		filter 220ms ease;
}

.browse-tab-swap-enter-from {
	opacity: 0;
	transform: translateY(10px) scale(0.99);
	filter: blur(3px);
}

.browse-tab-swap-leave-to {
	opacity: 0;
	transform: translateY(-6px) scale(0.995);
	filter: blur(2px);
}
</style>
