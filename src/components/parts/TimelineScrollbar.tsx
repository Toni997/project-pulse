import { useCallback, useEffect, useRef } from 'react'
import { clamp } from '@mantine/hooks'

interface TimelineScrollbarProps {
  orientation: 'horizontal' | 'vertical'
  // All sizes/positions are in the same domain unit (e.g. beats, or track rows).
  viewStart: number
  viewSize: number
  totalSize: number
  minViewSize: number
  maxViewSize: number
  onChange: (viewStart: number, viewSize: number) => void
}

const EDGE_GRAB_PX = 8
const MIN_THUMB_PX = 24

type DragMode = 'pan' | 'start' | 'end'

const TimelineScrollbar = ({
  orientation,
  viewStart,
  viewSize,
  totalSize,
  minViewSize,
  maxViewSize,
  onChange,
}: TimelineScrollbarProps) => {
  const trackRef = useRef<HTMLDivElement>(null)
  const dragCleanupRef = useRef<(() => void) | null>(null)
  const isHorizontal = orientation === 'horizontal'

  useEffect(() => {
    return () => dragCleanupRef.current?.()
  }, [])

  const beginDrag = useCallback(
    // `panStartOverride` lets a click-to-jump (which changes viewStart via
    // onChange right before this fires) hand the drag its fresh start
    // position instead of the now-stale `viewStart` prop from this render.
    (mode: DragMode, panStartOverride?: number) => (e: React.PointerEvent) => {
      e.preventDefault()
      e.stopPropagation()
      const track = trackRef.current
      if (!track) return

      const trackRect = track.getBoundingClientRect()
      const trackLengthPx = isHorizontal ? trackRect.width : trackRect.height
      if (trackLengthPx <= 0 || totalSize <= 0) return

      const getPointerPos = (event: PointerEvent) =>
        isHorizontal ? event.clientX : event.clientY

      const startPointerPos = getPointerPos(e.nativeEvent as PointerEvent)
      const startViewStart = panStartOverride ?? viewStart
      const startViewEnd = startViewStart + viewSize

      const move = (event: PointerEvent) => {
        const deltaPx = getPointerPos(event) - startPointerPos
        const deltaDomain = (deltaPx / trackLengthPx) * totalSize

        if (mode === 'pan') {
          const maxStart = Math.max(0, totalSize - viewSize)
          onChange(clamp(startViewStart + deltaDomain, 0, maxStart), viewSize)
        } else if (mode === 'start') {
          const furthestStart = startViewEnd - minViewSize
          const nextStart = clamp(startViewStart + deltaDomain, 0, furthestStart)
          const nextSize = clamp(startViewEnd - nextStart, minViewSize, maxViewSize)
          onChange(startViewEnd - nextSize, nextSize)
        } else {
          const nextEnd = Math.max(startViewStart + minViewSize, startViewEnd + deltaDomain)
          const nextSize = clamp(nextEnd - startViewStart, minViewSize, maxViewSize)
          onChange(startViewStart, nextSize)
        }
      }

      const cleanup = () => {
        document.removeEventListener('pointermove', move)
        document.removeEventListener('pointerup', end)
        document.removeEventListener('pointercancel', end)
        dragCleanupRef.current = null
      }
      const end = () => cleanup()
      dragCleanupRef.current = cleanup

      document.addEventListener('pointermove', move)
      document.addEventListener('pointerup', end)
      document.addEventListener('pointercancel', end)
    },
    [isHorizontal, maxViewSize, minViewSize, onChange, totalSize, viewSize, viewStart],
  )

  // Clicking the empty track area re-centers the thumb there and starts a pan drag.
  const handleTrackPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.target !== e.currentTarget) return
    const track = trackRef.current
    if (!track || totalSize <= 0) return

    const trackRect = track.getBoundingClientRect()
    const trackLengthPx = isHorizontal ? trackRect.width : trackRect.height
    if (trackLengthPx <= 0) return

    const pointerOffsetPx = isHorizontal
      ? e.clientX - trackRect.left
      : e.clientY - trackRect.top
    const clickedDomain = (pointerOffsetPx / trackLengthPx) * totalSize
    const maxStart = Math.max(0, totalSize - viewSize)
    const jumpedStart = clamp(clickedDomain - viewSize / 2, 0, maxStart)
    onChange(jumpedStart, viewSize)

    beginDrag('pan', jumpedStart)(e)
  }

  const thumbFraction = totalSize > 0 ? clamp(viewSize / totalSize, 0, 1) : 1
  const thumbStartFraction =
    totalSize > 0 ? clamp(viewStart / totalSize, 0, 1) : 0

  const thumbPositionStyle: React.CSSProperties = isHorizontal
    ? {
        left: `${thumbStartFraction * 100}%`,
        width: `${thumbFraction * 100}%`,
        top: 0,
        bottom: 0,
        minWidth: MIN_THUMB_PX,
      }
    : {
        top: `${thumbStartFraction * 100}%`,
        height: `${thumbFraction * 100}%`,
        left: 0,
        right: 0,
        minHeight: MIN_THUMB_PX,
      }

  const edgeStartStyle: React.CSSProperties = isHorizontal
    ? { left: 0, top: 0, bottom: 0, width: EDGE_GRAB_PX, cursor: 'ew-resize' }
    : { top: 0, left: 0, right: 0, height: EDGE_GRAB_PX, cursor: 'ns-resize' }

  const edgeEndStyle: React.CSSProperties = isHorizontal
    ? { right: 0, top: 0, bottom: 0, width: EDGE_GRAB_PX, cursor: 'ew-resize' }
    : { bottom: 0, left: 0, right: 0, height: EDGE_GRAB_PX, cursor: 'ns-resize' }

  return (
    <div
      ref={trackRef}
      className='relative w-full h-full bg-neutral-200'
      onPointerDown={handleTrackPointerDown}
    >
      <div
        className='absolute bg-neutral-400 hover:bg-neutral-500 rounded-sm'
        style={{ ...thumbPositionStyle, cursor: isHorizontal ? 'grab' : 'grab' }}
        onPointerDown={beginDrag('pan')}
      >
        <div className='absolute' style={edgeStartStyle} onPointerDown={beginDrag('start')} />
        <div className='absolute' style={edgeEndStyle} onPointerDown={beginDrag('end')} />
      </div>
    </div>
  )
}

export default TimelineScrollbar
