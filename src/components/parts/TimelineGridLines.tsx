import { useMemo } from 'react'
import { Group, Line } from 'react-konva'
import { useTimelineStore } from '../../stores/timelineStore'
import { useShallow } from 'zustand/react/shallow'
import { BEATS_PER_BAR } from '../../helpers/constants'

const TimelineGridLines = () => {
  const { pxPerBeat, stageWidth, stageHeight } = useTimelineStore(
    useShallow((state) => ({
      pxPerBeat: state.pxPerBeat,
      stageWidth: state.stageWidth,
      stageHeight: state.stageHeight,
    })),
  )

  const headerHeight = 20
  const barWidth = pxPerBeat * BEATS_PER_BAR

  const barXs = useMemo(() => {
    if (barWidth <= 0) return []
    const barsToDraw = Math.ceil(stageWidth / barWidth)
    const xs: number[] = []
    for (let bar = 0; bar <= barsToDraw; bar++) {
      xs.push(bar * barWidth)
    }
    return xs
  }, [barWidth, stageWidth])

  return (
    <Group name='timeline-grid-lines' listening={false}>
      {barXs.map((x, i) => (
        <Line
          key={`bar-line-${i}`}
          points={[x, headerHeight, x, stageHeight]}
          stroke='#e6e6e6'
          strokeWidth={1}
        />
      ))}
    </Group>
  )
}

export default TimelineGridLines
