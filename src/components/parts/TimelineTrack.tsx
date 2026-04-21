import { Group, Line, Rect, Text } from 'react-konva'
import { useTimelineStore } from '../../stores/timelineStore'
import { GeneratorTrack } from '../../types/Track'
import TimelineClip from './TimelineClip'
import { useMemo } from 'react'

interface TimelineTrackProps {
  track: GeneratorTrack
  trackIndex: number
  onAutoScrollDuringDrag?: (e: any) => void
}

const TimelineTrack = ({
  track,
  trackIndex,
  onAutoScrollDuringDrag,
}: TimelineTrackProps) => {
  const stageWidth = useTimelineStore((state) => state.stageWidth)
  const trackHeightPx = 100

  const clips = useMemo(
    () => Object.values(track.clips).sort((a, b) => a.startPpq - b.startPpq),
    [track.clips],
  )

  return (
    <Group
      name='track'
      id={track.id}
      x={0}
      y={trackIndex * 100}
      width={stageWidth}
      height={trackHeightPx}
      kind={track.kind}
    >
      <Rect
        x={0}
        y={0}
        width={stageWidth}
        height={trackHeightPx}
        fill='rgba(0,0,0,0.05)'
      />
      <Text x={0} y={0} text={`${track.name} (${track.kind})`} fontSize={14} />
      {clips.map((clip) => (
        <TimelineClip
          key={clip.id}
          clip={clip}
          trackHeightPx={trackHeightPx}
          headerHeightPx={20}
          onAutoScrollDuringDrag={onAutoScrollDuringDrag}
        />
      ))}
      <Line
        name='track-separator'
        points={[0, 100, stageWidth, 100]}
        stroke='#ddd'
        strokeWidth={1}
      />
    </Group>
  )
}

export default TimelineTrack
