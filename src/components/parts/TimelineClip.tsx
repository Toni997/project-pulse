import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import './pixiSetup'
import { CanvasTextMetrics, Graphics, TextStyle } from 'pixi.js'
import { invoke } from '@tauri-apps/api/core'
import { setCursor } from '../../helpers/functions'
import { ECursor } from '../../helpers/enums'
import type { Clip } from '../../types/Clip'
import { useTimelineStore } from '../../stores/timelineStore'
import { useProjectStore } from '../../stores/projectStore'
import { useShallow } from 'zustand/react/shallow'
import {
  BASE_PX_PER_BEAT,
  MIXER_DELETE_CLIP_FROM_AUDIO_TRACK,
  MIXER_MOVE_CLIP_IN_AUDIO_TRACK,
} from '../../helpers/constants'

interface TimelineClipProps {
  clip: Clip
  trackHeightPx: number
  headerHeightPx?: number
  onAutoScrollDuringDrag?: (clientX: number) => void
  onAutoScrollDragEnd?: () => void
}

const hashToHue = (value: string) => {
  let hash = 0
  for (let i = 0; i < value.length; i++) {
    hash = (hash * 31 + value.charCodeAt(i)) | 0
  }
  return Math.abs(hash) % 360
}

const hslToRgbNumber = (hue: number, saturationPercent: number, lightPercent: number) => {
  const saturation = saturationPercent / 100
  const light = lightPercent / 100
  const chroma = (1 - Math.abs(2 * light - 1)) * saturation
  const x = chroma * (1 - Math.abs(((hue / 60) % 2) - 1))
  const m = light - chroma / 2
  let r = 0
  let g = 0
  let b = 0

  if (hue < 60) {
    r = chroma
    g = x
  } else if (hue < 120) {
    r = x
    g = chroma
  } else if (hue < 180) {
    g = chroma
    b = x
  } else if (hue < 240) {
    g = x
    b = chroma
  } else if (hue < 300) {
    r = x
    b = chroma
  } else {
    r = chroma
    b = x
  }

  const red = Math.round((r + m) * 255)
  const green = Math.round((g + m) * 255)
  const blue = Math.round((b + m) * 255)
  return (red << 16) + (green << 8) + blue
}

const truncateToWidth = (text: string, maxWidth: number, style: TextStyle) => {
  if (maxWidth <= 0) return ''
  if (CanvasTextMetrics.measureText(text, style).width <= maxWidth) return text

  const ellipsis = '…'
  let low = 0
  let high = text.length
  while (low < high) {
    const mid = Math.ceil((low + high) / 2)
    const candidateWidth = CanvasTextMetrics.measureText(
      text.slice(0, mid) + ellipsis,
      style,
    ).width
    if (candidateWidth <= maxWidth) {
      low = mid
    } else {
      high = mid - 1
    }
  }
  return low <= 0 ? '' : text.slice(0, low) + ellipsis
}

const TimelineClip = ({
  clip,
  trackHeightPx,
  headerHeightPx = 20,
  onAutoScrollDuringDrag,
  onAutoScrollDragEnd,
}: TimelineClipProps) => {
  const ppq = useProjectStore((state) => state.ppq)
  const { pxPerBeat, scrollBeats, trackScale } = useTimelineStore(
    useShallow((state) => ({
      pxPerBeat: state.pxPerBeat,
      scrollBeats: state.scrollBeats,
      trackScale: state.trackScale,
    })),
  )
  const updateClip = useProjectStore((state) => state.updateClip)
  const removeClip = useProjectStore((state) => state.removeClip)
  // The raw last-known pointer X (screen space), not a computed beat
  // position - it only changes on an actual pointermove. The beat position
  // is derived from it reactively below, together with the live scrollBeats,
  // so that auto-scrolling (which moves scrollBeats every frame while the
  // pointer sits still at an edge) keeps the clip following the cursor
  // instead of freezing at its last-computed spot.
  const [dragPointerX, setDragPointerX] = useState<number | null>(null)
  const dragViewportLeftRef = useRef(0)
  const dragOffsetBeatsRef = useRef(0)
  const dragCleanupRef = useRef<(() => void) | null>(null)

  useEffect(() => {
    return () => dragCleanupRef.current?.()
  }, [])

  const textStyle = useMemo(
    () => new TextStyle({ fontSize: 12, fill: 0x222222 }),
    [],
  )

  const hScale = pxPerBeat / BASE_PX_PER_BEAT

  const startBeats = clip.startPpq / ppq
  const visualBeats = useMemo(() => {
    if (dragPointerX === null) return startBeats
    const beats =
      (dragPointerX - dragViewportLeftRef.current) / pxPerBeat + scrollBeats
    return Math.max(0, beats - dragOffsetBeatsRef.current)
  }, [dragPointerX, pxPerBeat, scrollBeats, startBeats])

  // Geometry is drawn once in base units and zoomed purely via the ancestor
  // camera containers' transforms, so zooming never needs to redraw a clip.
  // The min-size clamps still target a screen-px minimum, so they're divided
  // by the relevant scale to compensate.
  const { y, width, height, fill } = useMemo(() => {
    const padding = 4
    const lengthBeats = clip.lengthPpq / ppq
    const width = Math.max(6 / hScale, lengthBeats * BASE_PX_PER_BEAT)

    const y = headerHeightPx + padding
    const height = Math.max(
      12 / trackScale,
      trackHeightPx - headerHeightPx - padding * 2,
    )

    const hue = hashToHue(clip.sourceId)
    const fill = hslToRgbNumber(hue, 70, 55)

    return { y, width, height, fill }
  }, [
    clip.lengthPpq,
    clip.sourceId,
    hScale,
    headerHeightPx,
    ppq,
    trackHeightPx,
    trackScale,
  ])

  const visualX = visualBeats * BASE_PX_PER_BEAT
  const screenWidth = width * hScale

  const displayName = useMemo(
    () => truncateToWidth(clip.name, Math.max(0, screenWidth - 12), textStyle),
    [clip.name, screenWidth, textStyle],
  )

  const drawClip = useCallback(
    (graphics: Graphics) => {
      graphics.clear()
      graphics.rect(0, 0, width, height)
      graphics.fill({ color: fill, alpha: 0.75 })
      // A single stroke width can't stay crisp on both axes when horizontal
      // and vertical zoom differ, so compensate for whichever is more
      // extreme to avoid the border ballooning in thickness.
      graphics.stroke({
        color: 0x000000,
        alpha: 0.75,
        width: 1 / Math.max(hScale, trackScale),
      })
    },
    [fill, hScale, height, trackScale, width],
  )

  const handleDragEnd = async (newBeats: number) => {
    const safePpq = ppq > 0 ? ppq : 960
    const startPpq = Math.max(0, Math.round(newBeats * safePpq))

    try {
      const updated = await invoke<Clip | null>(MIXER_MOVE_CLIP_IN_AUDIO_TRACK, {
        trackId: clip.trackId,
        clipId: clip.id,
        startPpq,
      })
      if (updated) updateClip(updated)
    } finally {
      // Only fall back to the prop-derived position once the store reflects
      // the outcome, so the clip doesn't flash back to its pre-drag position
      // while the move is still being persisted.
      setDragPointerX(null)
    }
  }

  const handlePointerDown = (e: any) => {
    if (e.nativeEvent?.button === 2) return
    e.stopPropagation?.()

    const nativeEvent = e.nativeEvent as PointerEvent | undefined
    const viewport = document.getElementById('timeline-viewport')
    if (!nativeEvent || !viewport) return

    // Read scrollBeats/pxPerBeat fresh (not the render's closed-over values)
    // since this listener stays registered for the whole drag - by the time
    // it ends, auto-scroll may have moved scrollBeats well past what this
    // render saw when the drag started.
    const getBeatsAt = (clientX: number, viewportLeft: number) =>
      (clientX - viewportLeft) / useTimelineStore.getState().pxPerBeat +
      useTimelineStore.getState().scrollBeats

    const viewportLeft = viewport.getBoundingClientRect().left
    dragViewportLeftRef.current = viewportLeft
    dragOffsetBeatsRef.current =
      getBeatsAt(nativeEvent.clientX, viewportLeft) - visualBeats

    const move = (event: PointerEvent) => {
      setDragPointerX(event.clientX)
      onAutoScrollDuringDrag?.(event.clientX)
    }

    const cleanup = () => {
      document.removeEventListener('pointermove', move)
      document.removeEventListener('pointerup', end)
      document.removeEventListener('pointercancel', end)
      dragCleanupRef.current = null
      onAutoScrollDragEnd?.()
    }

    const end = (event: PointerEvent) => {
      const finalBeats = Math.max(
        0,
        getBeatsAt(event.clientX, dragViewportLeftRef.current) -
          dragOffsetBeatsRef.current,
      )
      cleanup()
      // Keep holding the dropped-at pointer position (don't clear it yet) so
      // the clip doesn't flash back to its pre-drag spot while the move is
      // still being persisted - handleDragEnd's `finally` clears it once the
      // store reflects the outcome.
      setDragPointerX(event.clientX)
      setCursor(ECursor.Default)
      handleDragEnd(finalBeats)
    }

    dragCleanupRef.current = cleanup

    setCursor(ECursor.Move)
    setDragPointerX(nativeEvent.clientX)
    document.addEventListener('pointermove', move)
    document.addEventListener('pointerup', end)
    document.addEventListener('pointercancel', end)
  }

  const handleDelete = () => {
    removeClip(clip.trackId, clip.id)
    setCursor(ECursor.Default)
    invoke(MIXER_DELETE_CLIP_FROM_AUDIO_TRACK, {
      trackId: clip.trackId,
      clipId: clip.id,
    })
  }

  return (
    <pixiContainer x={visualX} y={y}>
      <pixiGraphics
        eventMode='static'
        cursor='move'
        draw={drawClip}
        onPointerDown={handlePointerDown}
        onPointerOver={() => setCursor(ECursor.Move)}
        onPointerOut={() => setCursor(ECursor.Default)}
        onRightClick={(e: any) => {
          e.stopPropagation?.()
          handleDelete()
        }}
      />
      {screenWidth >= 40 && (
        <pixiText
          x={6 / hScale}
          y={4 / trackScale}
          text={displayName}
          eventMode='none'
          style={textStyle}
          scale={{ x: 1 / hScale, y: 1 / trackScale }}
        />
      )}
    </pixiContainer>
  )
}

export default TimelineClip
