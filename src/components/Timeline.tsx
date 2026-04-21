import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from 'react'
import { Stage, Layer, Group } from 'react-konva'
import { GeneratorTrack, TrackKind } from '../types/Track'
import { clamp } from '@mantine/hooks'
import { invoke } from '@tauri-apps/api/core'
import {
  BEATS_PER_BAR,
  MINIMUM_BARS_VISIBLE,
  MIXER_ADD_CLIP_TO_AUDIO_TRACK,
  MIXER_ADD_AUDIO_TRACK_WITH_CLIP,
} from '../helpers/constants'
import TimelineBar from './parts/TimelineBar'
import TimelineGridLines from './parts/TimelineGridLines'
import TimelineTrack from './parts/TimelineTrack'
import { useProjectStore } from '../stores/projectStore'
import { useTimelineStore } from '../stores/timelineStore'
import { useShallow } from 'zustand/react/shallow'
import { KonvaEventObject } from 'konva/lib/Node'
import InsertTrackMenu from './parts/InsertTrackMenu'
import type { Stage as KonvaStage } from 'konva/lib/Stage'
import { Container } from 'konva/lib/Container'
import type { Clip } from '../types/Clip'
import type { AudioTrack } from '../types/Track'
import { useGlobalStore } from '../stores/globalStore'

const Timeline = () => {
  const timelineContainerRef = useRef<HTMLDivElement>(null)
  const stageRef = useRef<KonvaStage | null>(null)
  const [containerSize, setContainerSize] = useState({ width: 0, height: 0 })
  const lastAutoScrollTs = useRef<number | null>(null)
  const setGlobalLoading = useGlobalStore((state) => state.setGlobalLoading)
  const { ppq, tracks, generatorTracksOrder, addClip, addTrack } =
    useProjectStore(
      useShallow((state) => ({
        ppq: state.ppq,
        tracks: state.tracks,
        generatorTracksOrder: state.generatorTracksOrder,
        addClip: state.addClip,
        addTrack: state.addTrack,
      })),
    )
  const {
    stageWidth,
    stageHeight,
    setStageWidth,
    setStageHeight,
    pxPerBeat,
    setPxPerBeat,
    barsVisible,
    setBarsVisible,
  } = useTimelineStore(
    useShallow((state) => ({
      stageWidth: state.stageWidth,
      stageHeight: state.stageHeight,
      setStageWidth: state.setStageWidth,
      setStageHeight: state.setStageHeight,
      pxPerBeat: state.pxPerBeat,
      setPxPerBeat: state.setPxPerBeat,
      barsVisible: state.barsVisible,
      setBarsVisible: state.setBarsVisible,
    })),
  )

  // Make sure to always will 100% of the width
  useEffect(() => {
    if (containerSize.width <= 0) return
    const barWidth = pxPerBeat * BEATS_PER_BAR
    if (barWidth <= 0) return

    const requiredBars = Math.max(
      MINIMUM_BARS_VISIBLE,
      Math.floor(containerSize.width / barWidth),
    )

    // Only grow automatically (avoids surprising shrink when zooming in).
    if (requiredBars > barsVisible) {
      setBarsVisible(requiredBars)
    }
  }, [barsVisible, containerSize.width, pxPerBeat, setBarsVisible])

  useEffect(() => {
    const contentWidth = barsVisible * pxPerBeat * BEATS_PER_BAR
    setStageWidth(Math.max(contentWidth, containerSize.width))
  }, [barsVisible, containerSize.width, pxPerBeat, setStageWidth])

  useEffect(() => {
    if (containerSize.height > 0) {
      setStageHeight(containerSize.height)
    }
  }, [containerSize.height, setStageHeight])

  // Set width and height of the stage to 100% on mount
  useLayoutEffect(() => {
    const el = timelineContainerRef.current
    if (!el) return
    const updateSize = () => {
      setContainerSize({
        width: Math.floor(el.clientWidth),
        height: Math.floor(el.clientHeight),
      })
    }
    updateSize()
    requestAnimationFrame(updateSize)
    const observer = new ResizeObserver(() => updateSize())
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  const handleWheel = (e: KonvaEventObject<WheelEvent>) => {
    if (!e.evt.ctrlKey) return
    e.evt.preventDefault()

    const el = timelineContainerRef.current
    if (!el) return

    const rect = el.getBoundingClientRect()
    const mouseX = e.evt.clientX - rect.left
    const scrollLeft = el.scrollLeft

    const zoomFactor = 1.1
    const nextPxPerBeat =
      e.evt.deltaY < 0 ? pxPerBeat * zoomFactor : pxPerBeat / zoomFactor

    const newPxPerBeat = Math.round(clamp(nextPxPerBeat, 7, 200))
    if (newPxPerBeat === pxPerBeat) return

    const ratio = newPxPerBeat / pxPerBeat
    setPxPerBeat(newPxPerBeat)

    requestAnimationFrame(() => {
      const el2 = timelineContainerRef.current
      if (!el2) return
      const maxScrollLeft = Math.max(0, el2.scrollWidth - el2.clientWidth)
      const desiredScrollLeft = (scrollLeft + mouseX) * ratio - mouseX
      el2.scrollLeft = clamp(desiredScrollLeft, 0, maxScrollLeft)
    })
  }

  // Minimal "duck type" for Konva nodes we want to compensate during auto-scroll.
  // When the timeline container scrolls, the dragged node would appear to drift away from the cursor.
  // We counteract that by shifting the node's X by the applied scroll delta.
  interface DragXNode {
    x(): number
    x(v: number): any
  }

  const autoScrollWhileDragging = useCallback(
    (clientX: number, draggedNode?: DragXNode) => {
      const el = timelineContainerRef.current
      if (!el) return

      // FL-style edge scroll:
      // - When the pointer is within `thresholdPx` of the left/right edge of the scroll container,
      //   start auto-scrolling in that direction.
      // - The closer to the edge, the faster it scrolls (quadratic ramp).
      const rect = el.getBoundingClientRect()
      const thresholdPx = 60
      const maxSpeedPxPerSec = 1400

      // Use a real delta time so scrolling speed feels consistent across different event rates.
      const now = performance.now()
      const last = lastAutoScrollTs.current ?? now
      lastAutoScrollTs.current = now
      const dt = Math.min(0.05, Math.max(0.0, (now - last) / 1000))

      // Compute scroll intent based on cursor proximity to edges.
      let direction = 0
      let intensity = 0
      if (clientX < rect.left + thresholdPx) {
        direction = -1
        intensity = (thresholdPx - (clientX - rect.left)) / thresholdPx
      } else if (clientX > rect.right - thresholdPx) {
        direction = 1
        intensity = (thresholdPx - (rect.right - clientX)) / thresholdPx
      }

      if (direction === 0) return

      // Quadratic ramp for nicer "accelerate into edge" feel.
      const speed = maxSpeedPxPerSec * intensity * intensity
      const desiredDelta = direction * speed * dt

      const maxScrollLeft = Math.max(0, el.scrollWidth - el.clientWidth)
      const prevScrollLeft = el.scrollLeft
      const nextScrollLeft = Math.min(
        maxScrollLeft,
        Math.max(0, prevScrollLeft + desiredDelta),
      )

      // Always extend the timeline when pushing the right edge.
      // This makes "drag to the right forever" work even when you're already at max scrollLeft:
      // expanding bars increases stage width, which in turn increases scrollWidth/maxScrollLeft.
      if (direction > 0) {
        const paddingPx = 400
        // A little lookahead so you don't "stall" at the right edge between bar expansions.
        const anticipatePx = speed * dt * 2
        const visibleRightPx =
          nextScrollLeft + el.clientWidth + paddingPx + anticipatePx
        const endBeats = visibleRightPx / pxPerBeat
        const requiredBars = Math.ceil(endBeats / BEATS_PER_BAR)
        if (requiredBars > barsVisible) {
          setBarsVisible(requiredBars)
        }
      }

      const appliedDelta = nextScrollLeft - prevScrollLeft
      if (appliedDelta === 0) return

      el.scrollLeft = nextScrollLeft

      // Keep the dragged node under the cursor by compensating for scroll movement.
      // (Only applies to Konva drags; HTML5 drags from the Browser pass `draggedNode` as undefined.)
      if (draggedNode) {
        draggedNode.x(draggedNode.x() + appliedDelta)
      }
    },
    [barsVisible, pxPerBeat, setBarsVisible],
  )

  const handleAutoScrollDuringDrag = useCallback(
    (e: KonvaEventObject<DragEvent>) => {
      const clientX = (e.evt as DragEvent).clientX
      autoScrollWhileDragging(clientX, e.target as any)
    },
    [autoScrollWhileDragging],
  )

  // Ensure the timeline is wide enough to show all clips.
  useEffect(() => {
    console.log('Ensuring timeline is wide enough for all clips')
    // Find the max end position across all clips, then grow "barsVisible" so the stage is wide enough.
    // This covers cases like "drop a clip far to the right" and ensures it becomes visible.
    let maxEndPpq = 0
    for (const trackId of generatorTracksOrder) {
      const track = tracks[trackId]
      if (!track) continue
      if (track.kind === TrackKind.Bus) continue
      for (const clip of Object.values(track.clips)) {
        const end = clip.startPpq + clip.lengthPpq
        if (end > maxEndPpq) maxEndPpq = end
      }
    }

    if (maxEndPpq <= 0) return
    const endBeats = maxEndPpq / ppq
    const requiredBars = Math.ceil(endBeats / BEATS_PER_BAR)

    if (requiredBars > barsVisible) {
      setBarsVisible(requiredBars)
    }
  }, [barsVisible, generatorTracksOrder, ppq, setBarsVisible, tracks])

  const handleDrop = async (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()
    const stage = stageRef.current
    if (!stage) return
    const audioPath = e.dataTransfer.getData('audio-path')
    console.log(audioPath)
    const el = e.currentTarget
    const rect = el.getBoundingClientRect()
    const xPosition = e.clientX - rect.left + el.scrollLeft
    const beatsPosition = xPosition / pxPerBeat
    const ppqPosition = Math.max(0, Math.round(beatsPosition * ppq))
    stage.setPointersPositions(e.nativeEvent as any)
    const stagePointerPos = stage.getPointerPosition()
    if (!stagePointerPos) return
    const shape = stage.getIntersection(stagePointerPos)
    const trackNode = shape?.findAncestor(
      (node: Container) => node.hasName('track'),
      true,
    )

    // Drop on an existing audio track -> insert clip into that track.
    // Drop on empty space (no intersected track) -> create new audio track with one clip.
    const trackId = trackNode?.id() ?? null
    const trackKind = trackNode ? trackNode.getAttr('kind') : null

    if (!trackNode) {
      setGlobalLoading(true)
      const created = await invoke<AudioTrack | null>(
        MIXER_ADD_AUDIO_TRACK_WITH_CLIP,
        {
          clip: { trackId: null, startPpq: ppqPosition, sourcePath: audioPath },
        },
      )
      setGlobalLoading(false)
      if (!created) return
      addTrack(created)
      return
    }

    if (trackKind !== TrackKind.Audio) return

    setGlobalLoading(true)
    const insertedClip = await invoke<Clip | null>(
      MIXER_ADD_CLIP_TO_AUDIO_TRACK,
      {
        clip: { trackId, startPpq: ppqPosition, sourcePath: audioPath },
      },
    )
    setGlobalLoading(false)
    if (!insertedClip) return
    addClip(insertedClip)
  }

  useEffect(() => {
    // Reset dt accumulator when zoom changes so the first edge-scroll after zoom doesn't jump.
    lastAutoScrollTs.current = null
  }, [pxPerBeat])

  return (
    <div className='flex flex-col h-full'>
      <div className='p-2'>
        <InsertTrackMenu />
      </div>
      <div
        id='timeline-container'
        ref={timelineContainerRef}
        className='w-full h-full overflow-scroll'
        onDragOver={(e) => {
          e.preventDefault()
          e.stopPropagation()
          e.dataTransfer.dropEffect = 'copy'
          // HTML5 drag from the Browser (outside Konva) still needs the same edge auto-scroll + timeline expansion.
          autoScrollWhileDragging(e.clientX)
        }}
        onDrop={handleDrop}
      >
        <Stage
          ref={stageRef}
          width={stageWidth}
          height={stageHeight}
          className='h-full'
          onWheel={handleWheel}
        >
          <Layer>
            <TimelineGridLines />
            <Group name='tracks-container' y={20}>
              {generatorTracksOrder
                .map((id) => tracks[id])
                .filter(
                  (t): t is GeneratorTrack => !!t && t.kind !== TrackKind.Bus,
                )
                .map((track, index) => (
                  <TimelineTrack
                    key={track.id}
                    track={track}
                    trackIndex={index}
                    onAutoScrollDuringDrag={handleAutoScrollDuringDrag}
                  />
                ))}
            </Group>
            <TimelineBar />
          </Layer>
        </Stage>
      </div>
    </div>
  )
}

export default Timeline
