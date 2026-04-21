import { Group, Line, Rect } from 'react-konva'
import { setCursor } from '../../helpers/functions'
import { KonvaEventObject } from 'konva/lib/Node'
import { clamp } from '@mantine/hooks'
import { useTimelineStore } from '../../stores/timelineStore'
import { ECursor } from '../../helpers/enums'

const TimelineBar = () => {
  const playheadPosition = useTimelineStore((state) => state.playheadPosition)
  const setPlayheadPosition = useTimelineStore(
    (state) => state.setPlayheadPosition,
  )
  const stageWidth = useTimelineStore((state) => state.stageWidth)

  const handleMovePlayhead = (e: KonvaEventObject<DragEvent>) => {
    e.target.y(0)
    const newPlayheadPosition = clamp(e.target.x(), 0, stageWidth)
    e.target.x(newPlayheadPosition)
    setPlayheadPosition(newPlayheadPosition)
  }

  return (
    <>
      <Group x={playheadPosition}>
        <Line
          name='playhead-line'
          width={1}
          height={window.innerHeight}
          stroke='rgba(0,0,0,0.2)'
          points={[0, 0, 0, window.innerHeight]}
          listening={false}
        />
      </Group>
      <Group x={0} y={0}>
        <Rect x={0} y={0} width={stageWidth} height={20} fill='#f3f3f3' />
        <Line
          draggable
          name='playhead-mover'
          x={playheadPosition}
          points={[0, 20, -15, 0, 15, 0]}
          closed
          fill='#ccc'
          stroke='#333'
          strokeWidth={1}
          onMouseEnter={() => {
            setCursor(ECursor.Pointer)
          }}
          onMouseLeave={() => {
            setCursor(ECursor.Default)
          }}
          onDragMove={handleMovePlayhead}
        />
      </Group>
    </>
  )
}

export default TimelineBar
