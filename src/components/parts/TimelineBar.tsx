import { useCallback, useEffect, useRef } from 'react'
import './pixiSetup'
import { Graphics } from 'pixi.js'
import { setCursor } from '../../helpers/functions'
import { clamp } from '@mantine/hooks'
import { useTimelineStore } from '../../stores/timelineStore'
import { useShallow } from 'zustand/react/shallow'
import { ECursor } from '../../helpers/enums'
import {
  BASE_PX_PER_BEAT,
  BEATS_PER_BAR,
  TRACKS_START_Y,
} from '../../helpers/constants'

interface TimelineBarProps {
  // Current Pixi canvas height in screen px - the playhead line always spans
  // down to here regardless of vertical scroll, it never moves with tracks.
  viewportHeight: number
}

const TimelineBar = ({ viewportHeight }: TimelineBarProps) => {
  const {
    playheadBeats,
    setPlayheadBeats,
    pxPerBeat,
    scrollBeats,
    barsVisible,
  } = useTimelineStore(
    useShallow((state) => ({
      playheadBeats: state.playheadBeats,
      setPlayheadBeats: state.setPlayheadBeats,
      pxPerBeat: state.pxPerBeat,
      scrollBeats: state.scrollBeats,
      barsVisible: state.barsVisible,
    })),
  )
  const dragCleanupRef = useRef<(() => void) | null>(null)

  useEffect(() => {
    return () => dragCleanupRef.current?.()
  }, [])

  const hScale = pxPerBeat / BASE_PX_PER_BEAT
  const playheadX = playheadBeats * BASE_PX_PER_BEAT
  const headerWidthBase = barsVisible * BEATS_PER_BAR * BASE_PX_PER_BEAT

  const drawPlayhead = useCallback(
    (graphics: Graphics) => {
      graphics.clear()
      graphics.moveTo(0, 0)
      graphics.lineTo(0, viewportHeight)
      // Base-unit width divided by hScale keeps this a crisp 1px line on
      // screen regardless of horizontal zoom.
      graphics.stroke({ color: 0x000000, alpha: 0.2, width: 1 / hScale })
    },
    [hScale, viewportHeight],
  )

  const drawHeader = useCallback(
    (graphics: Graphics) => {
      graphics.clear()
      graphics.rect(0, 0, headerWidthBase, TRACKS_START_Y)
      graphics.fill({ color: 0xf3f3f3 })
    },
    [headerWidthBase],
  )

  const drawMover = useCallback((graphics: Graphics) => {
    graphics.clear()
    graphics.moveTo(0, TRACKS_START_Y)
    graphics.lineTo(-15, 0)
    graphics.lineTo(15, 0)
    graphics.closePath()
    graphics.fill({ color: 0xcccccc })
    graphics.stroke({ color: 0x333333, width: 1 })
  }, [])

  const beginPlayheadDrag = (e: any) => {
    e.stopPropagation?.()
    const nativeEvent = e.nativeEvent as PointerEvent | undefined
    const viewport = document.getElementById('timeline-viewport')
    if (!nativeEvent || !viewport) return

    const rect = viewport.getBoundingClientRect()
    const totalBeats = barsVisible * BEATS_PER_BAR
    const getBeats = (event: PointerEvent) =>
      (event.clientX - rect.left) / pxPerBeat + scrollBeats

    const move = (event: PointerEvent) => {
      setPlayheadBeats(clamp(getBeats(event), 0, totalBeats))
    }

    const cleanup = () => {
      document.removeEventListener('pointermove', move)
      document.removeEventListener('pointerup', end)
      document.removeEventListener('pointercancel', end)
      dragCleanupRef.current = null
    }

    const end = () => {
      cleanup()
      setCursor(ECursor.Default)
    }

    dragCleanupRef.current = cleanup

    setCursor(ECursor.Pointer)
    move(nativeEvent)
    document.addEventListener('pointermove', move)
    document.addEventListener('pointerup', end)
    document.addEventListener('pointercancel', end)
  }

  return (
    <>
      <pixiContainer x={playheadX} y={0} eventMode='none'>
        <pixiGraphics draw={drawPlayhead} />
      </pixiContainer>
      <pixiContainer x={0} y={0}>
        <pixiGraphics eventMode='none' draw={drawHeader} />
        <pixiGraphics
          x={playheadX}
          scale={{ x: 1 / hScale, y: 1 }}
          eventMode='static'
          cursor='pointer'
          draw={drawMover}
          onPointerDown={beginPlayheadDrag}
          onPointerOver={() => setCursor(ECursor.Pointer)}
          onPointerOut={() => setCursor(ECursor.Default)}
        />
      </pixiContainer>
    </>
  )
}

export default TimelineBar
