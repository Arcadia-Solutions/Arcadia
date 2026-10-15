import {
  ArtistRole,
  ContentType,
  AudioBitrateSampling,
  AudioChannels,
  AudioCodec,
  CollageCategory,
  type EditionGroupInfoLite,
  Extras,
  Features,
  Language,
  Platform,
  SiteHighlightItemType,
  Source,
  type Torrent,
  type TorrentRequest,
  VideoCodec,
  VideoResolution,
  StatsInterval,
  OrderByDirection,
} from './api-schema'

const videoContentTypes: ContentType[] = [ContentType.Movie, ContentType.TvShow, ContentType.Video, ContentType.LivePerformance, ContentType.Collection]
const audioContentTypes: ContentType[] = [...videoContentTypes, ContentType.Music, ContentType.Podcast]

export const siteHighlightLink = (itemType: SiteHighlightItemType, id: number): string => {
  switch (itemType) {
    case SiteHighlightItemType.TitleGroup:
      return `/title-group/${id}`
    case SiteHighlightItemType.Series:
      return `/series/${id}`
    case SiteHighlightItemType.Artist:
      return `/artist/${id}`
  }
}

export const timeAgo = (date: string) => {
  const diff = (Date.now() - new Date(date).getTime()) / 1000
  const absDiff = Math.abs(diff)
  const isFuture = diff < 0
  const format = (value: number, unit: string) => (isFuture ? `in ${value}${unit}` : `${value}${unit} ago`)
  return absDiff < 60
    ? format(Math.floor(absDiff), 's')
    : absDiff < 3600
      ? format(Math.floor(absDiff / 60), 'm')
      : absDiff < 86400
        ? format(Math.floor(absDiff / 3600), 'h')
        : format(Math.floor(absDiff / 86400), 'd')
}
export const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  const time = date.toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
  })
  return `${date.getDate()} ${date.toLocaleString('default', { month: 'long' })} ${date.getFullYear()}, ${time}`
}
export const bytesToReadable = (bytes: number): string => {
  const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB']
  let size = bytes
  let unitIndex = 0

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex++
  }

  return `${size.toFixed(unitIndex === 0 ? 0 : 2)} ${units[unitIndex]}`
}
export const getOrderByDirectionOptions = (t: (key: string) => string) => [
  { label: t('general.ascending'), value: OrderByDirection.Asc },
  { label: t('general.descending'), value: OrderByDirection.Desc },
]
export const getEditionGroupSlug = (editionGroup: EditionGroupInfoLite): string => {
  const attributes: (string | null)[] = []

  const formatReleaseDate = (date: string, onlyYearKnown: boolean): string => {
    if (onlyYearKnown) {
      return date.substring(0, 4)
    }
    return date
  }

  if (editionGroup.release_date) {
    let dateRange = ''
    if (editionGroup.additional_information?.date_from) {
      dateRange += formatReleaseDate(editionGroup.additional_information.date_from, editionGroup.release_date_only_year_known) + ' to '
    }
    dateRange += formatReleaseDate(editionGroup.release_date, editionGroup.release_date_only_year_known)

    let itemRange = ''
    if (editionGroup.additional_information?.first_item) {
      itemRange = ` (${editionGroup.additional_information.first_item} to ${editionGroup.additional_information.last_item})`
    }

    attributes.push(`${dateRange}${itemRange}`)
  } else {
    attributes.push(null)
  }
  if (editionGroup.name) {
    attributes.push(editionGroup.name)
  }
  if (editionGroup.additional_information?.format) {
    attributes.push(editionGroup.additional_information.format)
  }

  if (editionGroup.additional_information?.label) {
    attributes.push(editionGroup.additional_information.label)
  }
  if (editionGroup.additional_information?.catalogue_number) {
    attributes.push(editionGroup.additional_information.catalogue_number)
  }
  if (editionGroup.additional_information?.isbn_13) {
    attributes.push(editionGroup.additional_information.isbn_13)
  }
  if (editionGroup.source) {
    attributes.push(editionGroup.source)
  }
  if (editionGroup.additional_information?.location) {
    attributes.push(editionGroup.additional_information.location)
  }
  if (editionGroup.distributor) {
    attributes.push(editionGroup.distributor)
  }

  const first = attributes[0]
  const rest = attributes.slice(1).join(' / ')
  // release_date isn't always mandatory
  return `${first ? first + ' - ' : ''}${rest}`
}
export const getFeatures = (contentType: ContentType, format: string = '', source: Source | null = null): Features[] => {
  let features: Features[] = []
  if (source === Source.PhysicalBook) {
    features = features.concat([Features.Ocr])
  }
  if ((contentType == ContentType.Book && format === 'audiobook') || contentType == ContentType.Music) {
    features = features.concat([Features.Cue])
  } else if (videoContentTypes.includes(contentType)) {
    features = features.concat([Features.Hdr, Features.Hdr10, Features.Hdr102, Features.Dv, Features.Commentary, Features.Remux, Features._3D])
  }
  return features
}
export const getLanguages = (): Language[] => {
  return Object.values(Language)
}
export const getPlatforms = (): Platform[] => {
  return Object.values(Platform)
}
export const getSelectableContentTypes = (): ContentType[] => {
  return [
    ContentType.Movie,
    ContentType.Video,
    ContentType.TvShow,
    ContentType.Music,
    ContentType.Podcast,
    ContentType.Software,
    ContentType.Book,
    ContentType.LivePerformance,
    ContentType.Collection,
  ]
}
export const getCollageCategories = (): CollageCategory[] => {
  return [CollageCategory.External, CollageCategory.Personal, CollageCategory.StaffPicks, CollageCategory.Theme]
}
export const getSources = (contentType: ContentType): Source[] => {
  const sources: Source[] = [Source.Web]
  switch (contentType) {
    case ContentType.Book: {
      sources.push(Source.PhysicalBook, Source.Cd)
      break
    }
    case ContentType.Music: {
      sources.push(Source.Vinyl, Source.BluRay, Source.Cd, Source.Soundboard, Source.Sacd, Source.Dat, Source.Cassette)
      break
    }
    case ContentType.Video:
    case ContentType.Movie:
    case ContentType.TvShow:
    case ContentType.LivePerformance: {
      sources.push(Source.BluRay, Source.Dvd, Source.HdDvd, Source.Hdtv, Source.Pdtv, Source.Vhs, Source.Tv, Source.LaserDisc)
      break
    }
    case ContentType.Collection: {
      sources.push(
        Source.BluRay,
        Source.Dvd,
        Source.HdDvd,
        Source.Hdtv,
        Source.Pdtv,
        Source.Vhs,
        Source.Tv,
        Source.LaserDisc,
        Source.PhysicalBook,
        Source.Vinyl,
        Source.Cd,
        Source.Soundboard,
        Source.Sacd,
        Source.Dat,
        Source.Cassette,
      )
      break
    }
  }
  sources.push(Source.Mixed, Source.Unknown)
  return sources
}
export const getSelectableExtras = (contentType: ContentType): Extras[] => {
  const extras: Extras[] = []
  switch (contentType) {
    case ContentType.Book: {
      extras.push(Extras.Booklet)
      break
    }
    case ContentType.Music: {
      extras.push(Extras.Booklet)
      break
    }
    case ContentType.Movie: {
      extras.push(Extras.BehindTheScenes, Extras.DeletedScenes, Extras.Featurette, Extras.Trailer)
      break
    }
    case ContentType.TvShow: {
      extras.push(Extras.BehindTheScenes, Extras.DeletedScenes, Extras.Trailer)
      break
    }
    case ContentType.Video:
    case ContentType.LivePerformance: {
      extras.push(Extras.Booklet, Extras.BehindTheScenes, Extras.DeletedScenes, Extras.Featurette, Extras.Trailer)
      break
    }
  }
  extras.push(Extras.Other)
  return extras
}
export const getArtistRoles = (contentType: ContentType): ArtistRole[] => {
  const commonRoles: ArtistRole[] = [ArtistRole.Main, ArtistRole.Guest]

  switch (contentType) {
    case ContentType.Movie:
    case ContentType.TvShow:
    case ContentType.LivePerformance:
      return [...commonRoles, ArtistRole.Producer, ArtistRole.Director, ArtistRole.Cinematographer, ArtistRole.Actor, ArtistRole.Writer, ArtistRole.Composer]
    case ContentType.Video:
      return [
        ...commonRoles,
        ArtistRole.Creator,
        ArtistRole.Performer,
        ArtistRole.Presenter,
        ArtistRole.Contributor,
        ArtistRole.Producer,
        ArtistRole.Director,
        ArtistRole.Cinematographer,
        ArtistRole.Actor,
        ArtistRole.Writer,
        ArtistRole.Composer,
        ArtistRole.Remixer,
      ]
    case ContentType.Music:
      return [
        ...commonRoles,
        ArtistRole.Producer,
        ArtistRole.Composer,
        ArtistRole.Conductor,
        ArtistRole.DjCompiler,
        ArtistRole.Remixer,
        ArtistRole.Arranger,
        ArtistRole.Writer,
      ]
    case ContentType.Podcast:
      return [...commonRoles, ArtistRole.Producer, ArtistRole.Writer, ArtistRole.Host]
    case ContentType.Book:
      return [...commonRoles, ArtistRole.Author, ArtistRole.Writer, ArtistRole.Illustrator, ArtistRole.Editor]
    case ContentType.Software:
      return [...commonRoles, ArtistRole.Developer, ArtistRole.Designer, ArtistRole.Producer, ArtistRole.Writer]
    case ContentType.Collection:
      return [...commonRoles, ArtistRole.Producer, ArtistRole.Director, ArtistRole.Composer, ArtistRole.Author, ArtistRole.Writer, ArtistRole.Editor]
    default:
      return commonRoles
  }
}
export const isValidUrl = (url: string) => {
  try {
    new URL(url)
    return true
  } catch {
    return false
  }
}

const youtubeHostnames = ['youtube.com', 'youtu.be', 'youtube-nocookie.com']

// a youtube link has to be embedded in an iframe, any other link is played as a video file
export const isYoutubeLink = (url: string): boolean => {
  try {
    const hostname = new URL(url).hostname.toLowerCase().replace(/^(www|m)\./, '')
    return youtubeHostnames.includes(hostname)
  } catch {
    return false
  }
}

// turns a youtube link of any form (watch, share, shorts, embed) into the embed link an iframe accepts, null for the other links
export const getYoutubeEmbedLink = (url: string): string | null => {
  if (!isYoutubeLink(url)) {
    return null
  }

  // the id is the `v` parameter on a watch link, the last path segment on the other ones
  const parsedUrl = new URL(url)
  const videoId = parsedUrl.searchParams.get('v') || parsedUrl.pathname.split('/').filter(Boolean).pop()

  return videoId ? `https://www.youtube.com/embed/${videoId}` : null
}

export const getSelectableVideoCodecs = (): VideoCodec[] => {
  return Object.values(VideoCodec)
}

export const getSelectableVideoResolutions = (): VideoResolution[] => {
  return Object.values(VideoResolution)
}

export const getSelectableAudioCodecs = (): AudioCodec[] => {
  return Object.values(AudioCodec)
}

export const getSelectableAudioBitrateSamplings = (): AudioBitrateSampling[] => {
  return Object.values(AudioBitrateSampling)
}

export const getSelectableAudioChannels = (): AudioChannels[] => {
  return Object.values(AudioChannels)
}

export const getSelectableContainers = () => {
  return [
    'mkv',
    'mp4',
    'avi',
    'mov',
    'wmv',
    'flv',
    'webm',
    'm4v',
    '3gp',
    'ogv',
    'ts',
    'mts',
    'm2ts',
    'vob',
    'iso',
    'img',
    'bin',
    'cue',
    'flac',
    'mp3',
    'wav',
    'aac',
    'ogg',
    'm4a',
    'wma',
    'opus',
    'pdf',
    'epub',
    'mobi',
    'azw3',
    'cbz',
    'cbr',
    'zip',
    'rar',
    '7z',
    'tar',
    'gz',
    'bz2',
    'xz',
  ]
}

export const isAttributeUsed = (attribute: keyof Torrent | keyof TorrentRequest, contentType: ContentType): boolean => {
  switch (attribute) {
    case 'mediainfo':
    case 'video_codec':
    case 'video_resolution':
    case 'video_resolution_other_x':
    case 'video_resolution_other_y':
    case 'audio_channels':
    case 'subtitle_languages':
      return videoContentTypes.includes(contentType)
    case 'audio_bitrate_sampling':
    case 'audio_codec':
      return audioContentTypes.includes(contentType)
    default:
      return true
  }
}
export const scrollToHash = () => {
  ;(function h() {
    const e = document.querySelector(location.hash)
    if (e) {
      e.scrollIntoView({ behavior: 'smooth' })
    } else {
      setTimeout(h, 100)
    }
  })()
}
export const getHostname = () => {
  return window.location.hostname
}
export const isRouteProtected = (path: string) => {
  return ['/login', '/register', '/apply', '/reset-password', '/home/index.html'].indexOf(path) < 0
}
// the login page, sending the user back to the current page once logged in. On the login page
// itself, its own return target is kept.
export const loginLocation = () => {
  const { pathname, search } = window.location
  return pathname === '/login' ? `/login${search}` : `/login?redirect=${encodeURIComponent(pathname + search)}`
}
export const isReleaseDateRequired = (contentType: ContentType): boolean => {
  const contentTypesRequiringReleaseDate: ContentType[] = [
    ContentType.Movie,
    ContentType.TvShow,
    ContentType.Music,
    ContentType.Podcast,
    ContentType.Software,
    ContentType.LivePerformance,
  ]
  return contentTypesRequiringReleaseDate.includes(contentType)
}

export const formatDateToLocalString = (date: Date): string => {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

export const formatNumber = (value: number): string => {
  return value.toLocaleString()
}

export const formatBp = (rawValue: number, decimalPlaces: number, showDecimals = false, displayDecimalPlaces?: number): string => {
  const shifted = rawValue / Math.pow(10, decimalPlaces)
  const display = displayDecimalPlaces ?? decimalPlaces
  const truncated = showDecimals ? shifted : Math.trunc(shifted)
  return truncated.toLocaleString(undefined, {
    minimumFractionDigits: showDecimals ? display : 0,
    maximumFractionDigits: showDecimals ? display : 0,
  })
}

export const secondsToReadable = (seconds: number): string => {
  if (seconds < 60) return `${seconds}s`
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (days > 0) return hours > 0 ? `${days}d ${hours}h` : `${days}d`
  if (hours > 0) return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`
  return `${minutes}m`
}

export const rawToDisplayBp = (rawValue: number, decimalPlaces: number): number => {
  return rawValue / Math.pow(10, decimalPlaces)
}

export const displayToRawBp = (displayValue: number, decimalPlaces: number): number => {
  return Math.round(displayValue * Math.pow(10, decimalPlaces))
}

export const parseDateStringToLocal = (dateStr: string): Date | null => {
  const match = dateStr.match(/^(\d{4})-(\d{2})-(\d{2})/)
  if (!match) return null
  const [, year, month, day] = match.map(Number)
  return new Date(year, month - 1, day)
}

export const formatDateTimeLabel = (period: string, interval: StatsInterval): string => {
  const periodFormatOptions: Record<string, Intl.DateTimeFormatOptions> = {
    [StatsInterval.Year]: { year: 'numeric', timeZone: 'UTC' },
    [StatsInterval.Month]: { month: 'long', year: 'numeric', timeZone: 'UTC' },
    [StatsInterval.Day]: { day: 'numeric', month: 'short', year: 'numeric', timeZone: 'UTC' },
    [StatsInterval.Hour]: { day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit', timeZone: 'UTC' },
  }
  const date = new Date(period)
  if (interval === StatsInterval.Week) {
    const tmp = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate()))
    tmp.setUTCDate(tmp.getUTCDate() + 4 - (tmp.getUTCDay() || 7))
    const weekNumber = Math.ceil(((tmp.getTime() - new Date(Date.UTC(tmp.getUTCFullYear(), 0, 1)).getTime()) / 86400000 + 1) / 7)
    return `W${weekNumber} ${date.getUTCFullYear()}`
  }
  return date.toLocaleString('default', periodFormatOptions[interval])
}

/**
 * Handle keydown events on a form and prevent implicit submit when the return key is pressed
 * Only use via @keydown.enter for now
 * @param {KeyboardEvent} event The keyboard event emmited by the form
 *
 */
export const validatePasswordStrength = (password: string, t: (key: string) => string): { isValid: boolean; message: string } => {
  if (password.length < 12) {
    return { isValid: false, message: t('auth_validation.password_too_short') }
  }
  if (!/[A-Z]/.test(password)) {
    return { isValid: false, message: t('auth_validation.password_no_uppercase') }
  }
  if (!/[a-z]/.test(password)) {
    return { isValid: false, message: t('auth_validation.password_no_lowercase') }
  }
  if (!/\d/.test(password)) {
    return { isValid: false, message: t('auth_validation.password_no_number') }
  }
  return { isValid: true, message: '' }
}

export const onFormKeydown = (event: KeyboardEvent) => {
  const target = event.target

  if (target instanceof HTMLInputElement && !['button', 'submit', 'reset', 'checkbox', 'radio', 'file'].includes(target.type)) {
    event.preventDefault()
  }
}
