import { useMemo } from 'react'
import { Group, Rect, Text } from 'react-konva'
import { invoke } from '@tauri-apps/api/core'
import { setCursor } from '../../helpers/functions'
import { ECursor } from '../../helpers/enums'
import type { Clip } from '../../types/Clip'
import { useTimelineStore } from '../../stores/timelineStore'
import { useProjectStore } from '../../stores/projectStore'
import { MIXER_MOVE_CLIP_IN_AUDIO_TRACK } from '../../helpers/constants'
import type { KonvaEventObject } from 'konva/lib/Node'

interface TimelineClipProps {
  clip: Clip
  trackHeightPx: number
  headerHeightPx?: number
  onAutoScrollDuringDrag?: (e: KonvaEventObject<DragEvent>) => void
}

const hashToHue = (value: string) => {
  let hash = 0
  for (let i = 0; i < value.length; i++) {
    hash = (hash * 31 + value.charCodeAt(i)) | 0
  }
  return Math.abs(hash) % 360
}

const TimelineClip = ({
  clip,
  trackHeightPx,
  headerHeightPx = 20,
  onAutoScrollDuringDrag,
}: TimelineClipProps) => {
  const ppq = useProjectStore((state) => state.ppq)
  const pxPerBeat = useTimelineStore((state) => state.pxPerBeat)
  const updateClip = useProjectStore((state) => state.updateClip)

  const { x, y, width, height, fill } = useMemo(() => {
    const padding = 4

    const x = (clip.startPpq / ppq) * pxPerBeat
    const width = Math.max(6, (clip.lengthPpq / ppq) * pxPerBeat)

    const y = headerHeightPx + padding
    const height = Math.max(12, trackHeightPx - headerHeightPx - padding * 2)

    const hue = hashToHue(clip.sourceId)
    const fill = `hsla(${hue}, 70%, 55%, 0.75)`

    return { x, y, width, height, fill }
  }, [
    clip.lengthPpq,
    clip.name,
    clip.sourceId,
    clip.startPpq,
    headerHeightPx,
    ppq,
    pxPerBeat,
    trackHeightPx,
  ])

  const handleDragStart = (e: KonvaEventObject<DragEvent>) => {
    e.target.y(y)
  }

  const handleDragMove = (e: KonvaEventObject<DragEvent>) => {
    e.target.y(y)
    const newX = Math.max(0, e.target.x())
    e.target.x(newX)
    onAutoScrollDuringDrag?.(e)
  }

  const handleDragEnd = async (e: KonvaEventObject<DragEvent>) => {
    const newX = e.target.x()
    const safePpq = ppq > 0 ? ppq : 960
    const beats = newX / pxPerBeat
    const startPpq = Math.max(0, Math.round(beats * safePpq))

    const updated = await invoke<Clip | null>(MIXER_MOVE_CLIP_IN_AUDIO_TRACK, {
      trackId: clip.trackId,
      clipId: clip.id,
      startPpq,
    })
    if (!updated) return
    updateClip(updated)
  }

  return (
    <Group
      name='timeline-clip'
      x={x}
      y={y}
      id={clip.id}
      draggable
      onDragStart={handleDragStart}
      onDragMove={handleDragMove}
      onDragEnd={handleDragEnd}
    >
      <Rect
        width={width}
        height={height}
        fill={fill}
        stroke='rgba(0,0,0,0.75)'
        strokeWidth={1}
        onMouseEnter={() => setCursor(ECursor.Move)}
        onMouseLeave={() => setCursor(ECursor.Default)}
      />
      {width >= 40 && (
        <Text
          x={6}
          y={4}
          width={Math.max(0, width - 12)}
          text={clip.name}
          fontSize={12}
          fill='rgba(0,0,0,0.75)'
          ellipsis
          wrap='none'
          height={14}
          listening={false}
        />
      )}
    </Group>
  )
}

export default TimelineClip
