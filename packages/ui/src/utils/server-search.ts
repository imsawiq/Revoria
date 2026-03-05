import {
	BadgeCheckIcon,
	BlocksIcon,
	CloudIcon,
	CompassIcon,
	CrownIcon,
	GlobeIcon,
	GridIcon,
	HeartIcon,
	LockIcon,
	PickaxeIcon,
	RefreshCwIcon,
	ShieldIcon,
	SkullIcon,
	StarIcon,
	TerminalSquareIcon,
	UsersIcon,
} from '@modrinth/assets'
import { formatCategory } from '@modrinth/utils'
import { computed, type Ref, ref } from 'vue'
import { useRoute } from 'vue-router'

import type { FilterType, FilterValue, SortType, Tags } from './search'

const SERVER_REGIONS = [
	{ code: 'us_east', name: 'US East' },
	{ code: 'us_west', name: 'US West' },
	{ code: 'europe', name: 'Europe' },
	{ code: 'asia', name: 'Asia' },
	{ code: 'australia', name: 'Australia' },
	{ code: 'south_america', name: 'South America' },
	{ code: 'middle_east', name: 'Middle East' },
	{ code: 'russia', name: 'Russia' },
]

const SERVER_LANGUAGES = [
	{ code: 'en', name: 'English' },
	{ code: 'de', name: 'German' },
	{ code: 'fr', name: 'French' },
	{ code: 'es', name: 'Spanish' },
	{ code: 'pt', name: 'Portuguese' },
	{ code: 'ru', name: 'Russian' },
	{ code: 'zh', name: 'Chinese' },
	{ code: 'ja', name: 'Japanese' },
	{ code: 'ko', name: 'Korean' },
	{ code: 'nl', name: 'Dutch' },
	{ code: 'pl', name: 'Polish' },
	{ code: 'it', name: 'Italian' },
	{ code: 'tr', name: 'Turkish' },
	{ code: 'sv', name: 'Swedish' },
	{ code: 'fi', name: 'Finnish' },
]

const SERVER_SORT_TYPES: SortType[] = [
	{ display: 'Relevance', name: 'relevance' },
	{ display: 'Verified Plays', name: 'minecraft_java_server.verified_plays_2w' },
	{ display: 'Players', name: 'minecraft_java_server.ping.data.players_online' },
	{ display: 'Followers', name: 'follows' },
	{ display: 'Date Published', name: 'date_created' },
	{ display: 'Date Updated', name: 'date_modified' },
]

const FILTER_FIELD_MAP: Record<string, string> = {
	server_content_type: 'minecraft_java_server.content.kind',
	server_game_version: 'minecraft_java_server.content.supported_game_versions',
	server_status: 'minecraft_java_server.ping.data',
	server_region: 'minecraft_server.region',
	server_language: 'minecraft_server.languages',
}

const SERVER_HEADER_LABELS: Record<string, string> = {
	minecraft_server_community: 'Community',
	minecraft_server_features: 'Features',
	minecraft_server_gameplay: 'Gameplay',
	minecraft_server_meta: 'Meta',
}

const SERVER_CATEGORY_ICON_COMPONENTS: Record<string, unknown> = {
	skyblock: CloudIcon,
	prison: LockIcon,
	smp: UsersIcon,
	pokemon: GlobeIcon,
	adventure: CompassIcon,
	anarchy: SkullIcon,
	pvp: ShieldIcon,
	pve: ShieldIcon,
	bosses: CrownIcon,
	network: GlobeIcon,
	custom_content: BlocksIcon,
	custom_content_: BlocksIcon,
	minigames: GridIcon,
	parkour: StarIcon,
	questing: StarIcon,
	lifesteal: HeartIcon,
	whitelisted: BadgeCheckIcon,
	gens: PickaxeIcon,
	technical: TerminalSquareIcon,
	world_resets: RefreshCwIcon,
}

function formatServerHeader(header: string): string {
	if (SERVER_HEADER_LABELS[header]) return SERVER_HEADER_LABELS[header]
	return header
		.replaceAll('_', ' ')
		.replace(/\b\w/g, (char) => char.toUpperCase())
}

function resolveFilterIcon(icon: unknown) {
	if (typeof icon === 'string') {
		const trimmed = icon.trim()
		if (
			trimmed.startsWith('<') ||
			trimmed.startsWith('http') ||
			trimmed.startsWith('data:image') ||
			trimmed.startsWith('/')
		) {
			return icon
		}
	}
	return ''
}

function resolveServerCategoryIconByName(name: string, apiIcon: unknown) {
	const normalized = name.toLowerCase().replace(/[-\s]/g, '_')
	return (
		SERVER_CATEGORY_ICON_COMPONENTS[normalized] ??
		SERVER_CATEGORY_ICON_COMPONENTS[name.toLowerCase()] ??
		resolveFilterIcon(apiIcon)
	)
}

function getFilterField(filterId: string): string | undefined {
	if (filterId.startsWith('server_category_')) return 'categories'
	return FILTER_FIELD_MAP[filterId]
}

export function useServerSearch(opts: {
	tags: Ref<Tags>
	query: Ref<string>
	maxResults: Ref<number>
	currentPage: Ref<number>
}) {
	const toPositiveInt = (value: unknown, fallback: number) => {
		const parsed = Number(value)
		if (!Number.isFinite(parsed)) return fallback
		const rounded = Math.floor(parsed)
		return rounded > 0 ? rounded : fallback
	}

	const { tags, query, maxResults, currentPage } = opts
	const route = useRoute()

	const serverCurrentSortType = ref<SortType>(SERVER_SORT_TYPES[0])
	const serverCurrentFilters = ref<FilterValue[]>([{ type: 'server_status', option: 'online' }])
	const serverToggledGroups = ref<string[]>([])

	const serverFilterTypes = computed<FilterType[]>(() => {
		const categoryFilters: Record<string, FilterType> = {}
		for (const c of (tags.value?.categories ?? []).filter(
			(c) => c.project_type === 'minecraft_java_server',
		)) {
			const filterTypeId = `server_category_${c.header}`
			if (!categoryFilters[filterTypeId]) {
				categoryFilters[filterTypeId] = {
					id: filterTypeId,
					formatted_name: formatServerHeader(c.header),
					supported_project_types: ['server'],
					display: 'all',
					query_param: 'sc',
					supports_negative_filter: true,
					searchable: false,
					options: [],
				}
			}
			categoryFilters[filterTypeId].options.push({
				id: c.name,
				formatted_name: formatCategory(c.name),
				icon: resolveServerCategoryIconByName(c.name, c.icon),
				method: 'or',
				value: c.name,
			})
		}

		const preferredSections = [
			'minecraft_server_features',
			'minecraft_server_gameplay',
			'minecraft_server_meta',
			'minecraft_server_community',
		]
		const sectionFilters = preferredSections
			.map((h) => `server_category_${h}`)
			.map((id) => categoryFilters[id])
			.filter(Boolean)

		return [
			{
				id: 'server_content_type',
				formatted_name: 'Type',
				supported_project_types: ['server'],
				display: 'all',
				query_param: 'sct',
				supports_negative_filter: false,
				searchable: false,
				options: [
					{ id: 'vanilla', formatted_name: 'Vanilla', method: 'or', value: 'vanilla' },
					{ id: 'modpack', formatted_name: 'Modded', method: 'or', value: 'modpack' },
				],
			},
			...sectionFilters,
			{
				id: 'server_game_version',
				formatted_name: 'Game Version',
				supported_project_types: ['server'],
				display: 'scrollable',
				query_param: 'sgv',
				supports_negative_filter: false,
				searchable: true,
				options: (tags.value?.gameVersions ?? []).map((gv) => ({
					id: gv.version,
					toggle_group: gv.version_type !== 'release' ? 'all_versions' : undefined,
					method: 'or',
					value: gv.version,
					query_value: gv.version,
				})),
			},
			{
				id: 'server_region',
				formatted_name: 'Region',
				supported_project_types: ['server'],
				display: 'all',
				query_param: 'sr',
				supports_negative_filter: true,
				searchable: false,
				options: SERVER_REGIONS.map((r) => ({
					id: r.code,
					formatted_name: r.name,
					method: 'or',
					value: r.code,
				})),
			},
			{
				id: 'server_language',
				formatted_name: 'Language',
				supported_project_types: ['server'],
				display: 'scrollable',
				query_param: 'sl',
				supports_negative_filter: false,
				searchable: true,
				options: SERVER_LANGUAGES.map((l) => ({
					id: l.code,
					formatted_name: l.name,
					method: 'or',
					value: l.code,
				})),
			},
			{
				id: 'server_status',
				formatted_name: 'Status',
				supported_project_types: ['server'],
				display: 'all',
				query_param: 'sst',
				supports_negative_filter: false,
				searchable: false,
				options: [
					{ id: 'online', formatted_name: 'Online', method: 'or', value: 'online' },
					{ id: 'offline', formatted_name: 'Offline', method: 'or', value: 'offline' },
				],
			},
		]
	})

	const newFilters = computed(() => {
		const parts = ['project_types = minecraft_java_server']

		for (const filterType of serverFilterTypes.value) {
			const field = getFilterField(filterType.id)
			if (!field) continue
			const matched = serverCurrentFilters.value.filter((f) => f.type === filterType.id)
			if (matched.length === 0) continue

			if (filterType.id === 'server_status') {
				const selected = matched[0]?.option
				if (selected === 'online') {
					parts.push(`${field} EXISTS`)
				} else if (selected === 'offline') {
					parts.push(`${field} NOT EXISTS`)
				}
				continue
			}

			const included = matched.filter((f) => !f.negative)
			const excluded = matched.filter((f) => f.negative)
			if (included.length > 0) {
				const values = included.map((f) => `"${f.option}"`).join(', ')
				parts.push(`${field} IN [${values}]`)
			}
			if (excluded.length > 0) {
				const values = excluded.map((f) => `"${f.option}"`).join(', ')
				parts.push(`${field} NOT IN [${values}]`)
			}
		}

		return parts.join(' AND ')
	})

	const serverRequestParams = computed(() => {
		const params = [`limit=${maxResults.value}`, `index=${serverCurrentSortType.value.name}`]
		if (query.value) params.push(`query=${encodeURIComponent(query.value)}`)
		const offset = (currentPage.value - 1) * maxResults.value
		if (offset > 0) params.push(`offset=${offset}`)
		params.push(`new_filters=${encodeURIComponent(newFilters.value)}`)
		return `?${params.join('&')}`
	})

	function readServerQueryParams() {
		const q = route.query
		if (q.q) {
			query.value = String(q.q)
		}
		if (q.m) {
			maxResults.value = toPositiveInt(q.m, 20)
		}
		if (q.page) {
			currentPage.value = toPositiveInt(q.page, 1)
		}
		if (q.ss) {
			serverCurrentSortType.value =
				SERVER_SORT_TYPES.find((s) => s.name === String(q.ss)) ?? SERVER_SORT_TYPES[0]
		}
		for (const filterType of serverFilterTypes.value) {
			const paramValue = q[filterType.query_param]
			if (!paramValue) continue
			const values =
				typeof paramValue === 'string'
					? [paramValue]
					: paramValue.filter((v): v is string => v !== null)
			for (const value of values) {
				const isNegative = value.startsWith('!')
				const cleanValue = isNegative ? value.slice(1) : value
				const option = filterType.options.find((o) => o.id === cleanValue)
				if (option) {
					serverCurrentFilters.value.push({
						type: filterType.id,
						option: option.id,
						negative: isNegative,
					})
				}
			}
		}
	}

	function createServerPageParams(): Record<string, string | string[]> {
		const items: Record<string, string[]> = {}
		if (query.value) items.q = [query.value]
		for (const filterValue of serverCurrentFilters.value) {
			const type = serverFilterTypes.value.find((t) => t.id === filterValue.type)
			if (type) {
				const value = filterValue.negative ? `!${filterValue.option}` : filterValue.option
				if (items[type.query_param]) items[type.query_param].push(value)
				else items[type.query_param] = [value]
			}
		}
		if (serverCurrentSortType.value.name !== 'relevance') {
			items.ss = [serverCurrentSortType.value.name]
		}
		if (maxResults.value !== 20) {
			items.m = [String(maxResults.value)]
		}
		if (currentPage.value > 1) {
			items.page = [String(currentPage.value)]
		}
		return items
	}

	readServerQueryParams()

	return {
		serverCurrentSortType,
		serverCurrentFilters,
		serverToggledGroups,
		serverSortTypes: SERVER_SORT_TYPES,
		serverFilterTypes,
		serverRequestParams,
		createServerPageParams,
	}
}
