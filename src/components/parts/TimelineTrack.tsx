import { useCallback, useMemo } from 'react'
import './pixiSetup'
import { Graphics, TextStyle } from 'pixi.js'
import { useTimelineStore } from '../../stores/timelineStore'
import { useShallow } from 'zustand/react/shallow'
import { GeneratorTrack } from '../../types/Track'
import TimelineClip from './TimelineClip'
import {
  BASE_PX_PER_BEAT,
  BASE_TRACK_HEIGHT_PX,
  BEATS_PER_BAR,
} from '../../helpers/constants'

interface TimelineTrackProps {
  track: GeneratorTrack
  trackIndex: number
  onAutoScrollDuringDrag?: (clientX: number) => void
  onAutoScrollDragEnd?: () => void
}

const TimelineTrack = ({
  track,
  trackIndex,
  onAutoScrollDuringDrag,
  onAutoScrollDragEnd,
}: TimelineTrackProps) => {
  const { pxPerBeat, trackScale, barsVisible } = useTimelineStore(
    useShallow((state) => ({
      pxPerBeat: state.pxPerBeat,
      trackScale: state.trackScale,
      barsVisible: state.barsVisible,
    })),
  )
  const trackHeightPx = BASE_TRACK_HEIGHT_PX
  const widthBase = barsVisible * BEATS_PER_BAR * BASE_PX_PER_BEAT
  const hScale = pxPerBeat / BASE_PX_PER_BEAT

  // Both drawn in base units, so they're scaled by the ancestor camera
  // containers - dividing by the perpendicular scale keeps each line a crisp
  // 1px hairline on screen regardless of horizontal/vertical zoom.
  const textStyle = useMemo(
    () =>
      new TextStyle({
        fontSize: 14,
        fill: 0x111111,
      }),
    [],
  )

  const clips = useMemo(
    () => Object.values(track.clips).sort((a, b) => a.startPpq - b.startPpq),
    [track.clips],
  )

  const drawBackground = useCallback(
    (graphics: Graphics) => {
      graphics.clear()
      graphics.rect(0, 0, widthBase, trackHeightPx)
      graphics.fill({ color: 0x000000, alpha: 0.05 })
    },
    [trackHeightPx, widthBase],
  )

  const drawSeparator = useCallback(
    (graphics: Graphics) => {
      graphics.clear()
      graphics.moveTo(0, trackHeightPx)
      graphics.lineTo(widthBase, trackHeightPx)
      graphics.stroke({ color: 0xdddddd, width: 1 / trackScale })
    },
    [trackHeightPx, trackScale, widthBase],
  )

  return (
    <pixiContainer x={0} y={trackIndex * trackHeightPx}>
      <pixiGraphics eventMode='none' draw={drawBackground} />
      <pixiText
        x={0}
        y={0}
        text={`${track.name} (${track.kind})`}
        style={textStyle}
        scale={{ x: 1 / hScale, y: 1 / trackScale }}
      />
      {clips.map((clip) => (
        <TimelineClip
          key={clip.id}
          clip={clip}
          trackHeightPx={trackHeightPx}
          headerHeightPx={20}
          onAutoScrollDuringDrag={onAutoScrollDuringDrag}
          onAutoScrollDragEnd={onAutoScrollDragEnd}
        />
      ))}
      <pixiGraphics eventMode='none' draw={drawSeparator} />
    </pixiContainer>
  )
}

export default TimelineTrack
