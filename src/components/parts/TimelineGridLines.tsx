import { useCallback, useMemo } from 'react'
import './pixiSetup'
import { Graphics } from 'pixi.js'
import { useTimelineStore } from '../../stores/timelineStore'
import { useShallow } from 'zustand/react/shallow'
import {
  BASE_PX_PER_BEAT,
  BEATS_PER_BAR,
  TRACKS_START_Y,
} from '../../helpers/constants'

interface TimelineGridLinesProps {
  // Current Pixi canvas height in screen px - grid lines always span down to
  // here regardless of vertical scroll, they never move with the tracks.
  viewportHeight: number
}

const TimelineGridLines = ({ viewportHeight }: TimelineGridLinesProps) => {
  const { pxPerBeat, barsVisible } = useTimelineStore(
    useShallow((state) => ({
      pxPerBeat: state.pxPerBeat,
      barsVisible: state.barsVisible,
    })),
  )

  const hScale = pxPerBeat / BASE_PX_PER_BEAT
  const barWidthBase = BEATS_PER_BAR * BASE_PX_PER_BEAT

  const barXs = useMemo(() => {
    const xs: number[] = []
    for (let bar = 0; bar <= barsVisible; bar++) {
      xs.push(bar * barWidthBase)
    }
    return xs
  }, [barWidthBase, barsVisible])

  const draw = useCallback(
    (graphics: Graphics) => {
      graphics.clear()
      for (const x of barXs) {
        graphics.moveTo(x, TRACKS_START_Y)
        graphics.lineTo(x, viewportHeight)
      }
      // Stroke width is drawn in base units, then scaled by the parent's
      // horizontal-only transform - dividing by hScale keeps it a crisp 1px
      // line on screen at any zoom level instead of thickening/thinning.
      graphics.stroke({ color: 0xe6e6e6, width: 1 / hScale })
    },
    [barXs, hScale, viewportHeight],
  )

  return <pixiGraphics eventMode='none' draw={draw} />
}

export default TimelineGridLines
