export const DEFAULT_BARS_VISIBLE = 30
export const MINIMUM_BARS_VISIBLE = 30
export const BEATS_PER_BAR = 4

// Horizontal (time) zoom. Content is drawn once in these "base" units and
// zoomed purely via a Pixi transform (scale.x), so zooming never redraws
// clip/grid geometry - only the transform and a couple of stroke widths change.
export const BASE_PX_PER_BEAT = 10
export const DEFAULT_PX_PER_BEAT = BASE_PX_PER_BEAT
export const MIN_PX_PER_BEAT = 7
export const MAX_PX_PER_BEAT = 200

// Vertical (track height) zoom, same idea as above but for track rows.
export const BASE_TRACK_HEIGHT_PX = 100
export const DEFAULT_TRACK_SCALE = 1
// Keep at least enough room for a 12px clip name to stay legible.
export const MIN_TRACK_SCALE = 0.3
export const MAX_TRACK_SCALE = 3

export const TRACKS_START_Y = 20
// Always keep one extra empty row past the last track, as a drop target for
// dragging in a new sample/instrument.
export const EXTRA_EMPTY_TRACK_SLOTS = 1

export const SCROLLBAR_THICKNESS_PX = 14

export const NOTIFICATION_ERROR_EVENT = 'notification-error'

export const FS_SCAN_DIRECTORY_TREE = 'fs_scan_directory_tree'
export const PREVIEW_PLAY = 'preview_play'
export const TRANSPORT_PLAY = 'transport_play'
export const TRANSPORT_STOP = 'transport_stop'
export const MIXER_ADD_AUDIO_TRACK = 'mixer_add_audio_track'
export const MIXER_ADD_CLIP_TO_AUDIO_TRACK = 'mixer_add_clip_to_audio_track'
export const MIXER_ADD_AUDIO_TRACK_WITH_CLIP = 'mixer_add_audio_track_with_clip'
export const MIXER_MOVE_CLIP_IN_AUDIO_TRACK = 'mixer_move_clip_in_audio_track'
export const MIXER_ADD_SAMPLER_TRACK = 'mixer_add_sampler_track'
export const MIXER_ASSIGN_SOURCE_TO_SAMPLER_TRACK =
  'mixer_assign_source_to_sampler_track'
export const MIXER_DELETE_CLIP_FROM_AUDIO_TRACK =
  'mixer_delete_clip_from_audio_track'
