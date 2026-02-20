import React, { useState } from 'react'
import { Stage, Layer, Rect, Circle, Text, Group, Line } from 'react-konva'
import { KonvaEventObject } from 'konva/lib/Node'
import { AudioTrack } from '../types/Track'
import { clamp } from '@mantine/hooks'
import { setCursor } from '../helpers/functions'
import { ECursor } from '../helpers/enums'
import { Button } from '@mantine/core'
import { invoke } from '@tauri-apps/api/core'
import { MIXER_ADD_AUDIO_TRACK } from '../helpers/constants'
import { useProjectStore } from '../stores/projectStore'

const Timeline = () => {
  // should start with 30 bars, which is 115,200 ticks at 960 PPQ (pulses per quarter note) resolution
  const bars = 30
  const pixelsPerBar = 50
  const [playheadPosition, setPlayheadPosition] = useState<number>(0)
  const [stageWidth, setStageWidth] = useState<number>(bars * pixelsPerBar)
  const [stageHeight, setStageHeight] = useState<number>(window.innerHeight)
  const projectStore = useProjectStore()

  const handleMovePlayhead = (e: KonvaEventObject<DragEvent>) => {
    e.target.y(0)
    e.target.x(clamp(e.target.x(), 0, stageWidth))
    setPlayheadPosition(clamp(e.target.x(), 0, stageWidth))
  }

  const handleTimelineEventResizeLeft = (e: KonvaEventObject<DragEvent>) => {
    // TODO Resize left
    setCursor(ECursor.WResize)
  }

  const handleTimelineEventResizeRight = (e: KonvaEventObject<DragEvent>) => {
    // TODO Resize right
    setCursor(ECursor.EResize)
  }

  const handleTimelineEventResizeEnd = (e: KonvaEventObject<DragEvent>) => {
    e.target.x(0)
    e.target.y(0)
    setCursor(ECursor.Default)
  }

  const handleInsertAudioTrack = async () => {
    const newAudioTrack: AudioTrack | null = await invoke(MIXER_ADD_AUDIO_TRACK)
    if (newAudioTrack) {
      projectStore.addAudioTrack(newAudioTrack)
    }
  }

  return (
    <div className='flex flex-col h-full'>
      <div className='p-2'>
        <Button variant='filled' onClick={handleInsertAudioTrack}>
          Insert Audio Track
        </Button>
      </div>
      <Stage
        width={stageWidth}
        height={window.innerHeight}
        className='h-full overflow-auto'
      >
        <Layer>
          <Group name='tracks-container' y={20}>
            <Group name='track-1' width={stageWidth}>
              <Line
                name='track-separator'
                points={[0, 100, stageWidth, 100]}
                stroke='#f3f3f3'
                strokeWidth={1}
              />
              <Group
                name='timeline-event'
                x={20}
                width={100}
                height={100}
                draggable
                onDragMove={(e) => e.target.y(0)}
              >
                <Rect
                  width={100}
                  height={100}
                  fill='#074'
                  onMouseEnter={() => {
                    setCursor(ECursor.Move)
                  }}
                  onMouseLeave={() => {
                    setCursor(ECursor.Default)
                  }}
                />
                <Line
                  name='timeline-event-resize-left'
                  points={[0, 0, 0, 100]}
                  stroke='transparent'
                  strokeWidth={5}
                  draggable
                  onMouseEnter={() => {
                    setCursor(ECursor.WResize)
                  }}
                  onMouseLeave={() => {
                    setCursor(ECursor.Default)
                  }}
                  onDragMove={handleTimelineEventResizeLeft}
                  onDragEnd={handleTimelineEventResizeEnd}
                />
                <Line
                  name='timeline-event-resize-right'
                  points={[100, 0, 100, 100]}
                  stroke='transparent'
                  strokeWidth={5}
                  draggable
                  onMouseEnter={() => {
                    setCursor(ECursor.EResize)
                  }}
                  onMouseLeave={() => {
                    setCursor(ECursor.Default)
                  }}
                  onDragMove={handleTimelineEventResizeRight}
                  onDragEnd={handleTimelineEventResizeEnd}
                />
              </Group>
            </Group>
          </Group>
          <Group x={playheadPosition}>
            <Line
              name='playhead-line'
              width={1}
              height={window.innerHeight}
              stroke='#ddd'
              points={[0, 0, 0, window.innerHeight]}
            />
          </Group>
          <Group x={0} y={0}>
            <Rect x={0} y={0} width={stageWidth} height={20} fill='#f3f3f3' />
            <Line
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
              draggable
              onDragMove={handleMovePlayhead}
            />
          </Group>
        </Layer>
      </Stage>
    </div>
  )
}

export default Timeline
