import { getVersion } from '@tauri-apps/api/app'
import { fetch } from '@tauri-apps/plugin-http'

export const CURSEFORGE_BASE_URL = 'https://api.curseforge.com/v1'

export const CURSEFORGE_API_KEY = '$2a$10$vB69qL5rZBOO5DQICbYBO.vAK9U3rN3okLrin.WgqGTg1AjUnE4CC'

export type CurseForgeMod = {
	id: number
	name: string
	summary: string
	downloadCount: number
	likeCount: number
	logo?: { url: string; thumbnailUrl?: string } | null
	authors?: { name: string; url?: string }[]
	slug?: string
	links?: {
		websiteUrl?: string
		issuesUrl?: string
		sourceUrl?: string
		wikiUrl?: string
	} | null
	dateModified?: string
	dateCreated?: string
	dateReleased?: string
	gameId?: number
	classId?: number
	categories?: { id: number; name: string; iconUrl?: string }[]
}

export type CurseForgeModFile = {
	id: number
	modId?: number
	fileName: string
	displayName: string
	fileLength: number
	fileDate: string
	gameVersions?: string[]
	modLoader?: number
	downloadUrl?: string | null
	releaseType?: number // 1=Release, 2=Beta, 3=Alpha
	dependencies?: {
		modId: number
		relationType: number // 1=EmbeddedLibrary, 2=Optional, 3=Required, 4=Tool, 5=Incompatible, 6=Include
	}[]
	hashes?: {
		value: string
		algo: number // 1=sha1, 2=md5
	}[]
}

export type CurseForgeSearchResponse = {
	data: CurseForgeMod[]
	pagination: {
		index: number
		pageSize: number
		resultCount: number
		totalCount: number
	}
}

export type CurseForgeCategory = {
	id: number
	name: string
	classId?: number
	parentCategoryId?: number
}

type CurseForgeModResponse = {
	data: CurseForgeMod
}

export type CurseForgeModFilesResponse = {
	data: CurseForgeModFile[]
	pagination: {
		index: number
		pageSize: number
		resultCount: number
		totalCount: number
	}
}

type CurseForgeDownloadUrlResponse = {
	data: string
}

async function cfFetch<T>(
	path: string,
	query?: Record<string, string | number | undefined | null>,
) {
	const version = await getVersion()
	const url = new URL(`${CURSEFORGE_BASE_URL}${path}`)
	if (query) {
		for (const [k, v] of Object.entries(query)) {
			if (v === undefined || v === null || v === '') continue
			url.searchParams.set(k, String(v))
		}
	}

	const res = await fetch(url.toString(), {
		method: 'GET',
		headers: {
			'User-Agent': `revoria/${version}`,
			Accept: 'application/json',
			'x-api-key': CURSEFORGE_API_KEY,
		},
	})

	if (!res.ok) {
		const text = await res.text().catch(() => '')
		throw new Error(`CurseForge request failed (${res.status}): ${text}`)
	}

	return (await res.json()) as T
}

export function getCurseForgeClassId(projectType: string): number | null {
	switch (projectType) {
		case 'mod':
			return 6
		case 'resourcepack':
			return 12
		case 'shader':
			return 6552
		case 'datapack':
			return 6945
		default:
			return null
	}
}

function getCurseForgeSort(
	sortType: string | undefined | null,
): { sortField: number; sortOrder: 'asc' | 'desc' } {
	// CurseForge API ModsSearchSortField:
	// 1=Featured, 2=Popularity, 3=LastUpdated, 4=Name, 5=Author,
	// 6=TotalDownloads, 7=Category, 8=GameVersion, 11=ReleasedDate
	switch (sortType) {
		case 'downloads':
			return { sortField: 6, sortOrder: 'desc' }
		case 'follows':
			return { sortField: 2, sortOrder: 'desc' }
		case 'newest':
			return { sortField: 11, sortOrder: 'desc' }
		case 'updated':
			return { sortField: 3, sortOrder: 'desc' }
		case 'name':
			return { sortField: 4, sortOrder: 'asc' }
		case 'author':
			return { sortField: 5, sortOrder: 'asc' }
		case 'relevance':
		default:
			return { sortField: 1, sortOrder: 'desc' }
	}
}

export async function searchCurseForgeMods(params: {
	projectType: string
	query: string
	index: number
	pageSize: number
	sortType?: string
	gameVersion?: string
	modLoaderTypes?: number[]
	categoryIds?: number[]
}) {
	const classId = getCurseForgeClassId(params.projectType)
	if (!classId) {
		return {
			data: [] as CurseForgeMod[],
			pagination: {
				index: params.index,
				pageSize: params.pageSize,
				resultCount: 0,
				totalCount: 0,
			},
		} satisfies CurseForgeSearchResponse
	}

	const sort = getCurseForgeSort(params.sortType)
	const loaderTypes = params.modLoaderTypes?.filter((v) => Number.isFinite(v)) ?? []
	const categoryIds = params.categoryIds?.filter((v) => Number.isFinite(v)) ?? []
	const res = await cfFetch<CurseForgeSearchResponse>('/mods/search', {
		gameId: 432,
		classId,
		searchFilter: params.query || undefined,
		sortField: sort.sortField,
		sortOrder: sort.sortOrder,
		gameVersion: params.gameVersion,
		modLoaderTypes:
			loaderTypes.length > 0 ? `[${loaderTypes.join(',')}]` : undefined,
		categoryIds: categoryIds.length > 0 ? `[${categoryIds.join(',')}]` : undefined,
		index: params.index,
		pageSize: params.pageSize,
	})

	return res
}

export async function getCurseForgeCategories(params: { classId: number }) {
	const res = await cfFetch<{ data: CurseForgeCategory[] }>('/categories', {
		gameId: 432,
		classId: params.classId,
	})
	return res.data ?? []
}

export async function getCurseForgeMod(modId: number) {
	const res = await cfFetch<CurseForgeModResponse>(`/mods/${modId}`)
	return res.data
}

export async function getCurseForgeModFiles(params: {
	modId: number
	index?: number
	pageSize?: number
	gameVersion?: string
}) {
	const res = await cfFetch<CurseForgeModFilesResponse>(`/mods/${params.modId}/files`, {
		index: params.index ?? 0,
		pageSize: params.pageSize ?? 50,
		gameVersion: params.gameVersion,
	})
	return res
}

export async function getCurseForgeFileDownloadUrl(modId: number, fileId: number) {
	try {
		const res = await cfFetch<CurseForgeDownloadUrlResponse>(
			`/mods/${modId}/files/${fileId}/download-url`,
		)
		if (res.data) return res.data
	} catch {
		// Fall through to CDN URL construction
	}
	return null
}

export function constructCurseForgeCdnUrl(fileId: number, fileName: string): string {
	const idStr = String(fileId)
	const first = idStr.substring(0, 4)
	const rest = idStr.substring(4)
	return `https://edge.forgecdn.net/files/${first}/${rest}/${encodeURIComponent(fileName)}`
}

export async function getCurseForgeModDescription(modId: number): Promise<string> {
	const res = await cfFetch<{ data: string }>(`/mods/${modId}/description`)
	return res.data ?? ''
}

export function getCurseForgeReleaseType(releaseType?: number): string {
	switch (releaseType) {
		case 1:
			return 'release'
		case 2:
			return 'beta'
		case 3:
			return 'alpha'
		default:
			return 'release'
	}
}

export function formatCurseForgeFileSize(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
	return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export function getCurseForgeProjectUrl(
	projectType?: string | null,
	slug?: string | null,
): string {
	switch (projectType) {
		case 'modpack':
			return `https://www.curseforge.com/minecraft/modpacks${slug ? `/${slug}` : ''}`
		case 'resourcepack':
			return `https://www.curseforge.com/minecraft/texture-packs${slug ? `/${slug}` : ''}`
		case 'shader':
			return `https://www.curseforge.com/minecraft/shaders${slug ? `/${slug}` : ''}`
		case 'datapack':
			return `https://www.curseforge.com/minecraft/data-packs${slug ? `/${slug}` : ''}`
		case 'mod':
		default:
			return `https://www.curseforge.com/minecraft/mc-mods${slug ? `/${slug}` : ''}`
	}
}
