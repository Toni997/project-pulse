import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from 'react'
import { Application, type ApplicationRef } from '@pixi/react'
import './parts/pixiSetup'
import { GeneratorTrack, TrackKind } from '../types/Track'
import { clamp } from '@mantine/hooks'
import { invoke } from '@tauri-apps/api/core'
import {
  BASE_PX_PER_BEAT,
  BASE_TRACK_HEIGHT_PX,
  BEATS_PER_BAR,
  EXTRA_EMPTY_TRACK_SLOTS,
  MAX_PX_PER_BEAT,
  MAX_TRACK_SCALE,
  MINIMUM_BARS_VISIBLE,
  MIN_PX_PER_BEAT,
  MIN_TRACK_SCALE,
  MIXER_ADD_CLIP_TO_AUDIO_TRACK,
  MIXER_ADD_AUDIO_TRACK_WITH_CLIP,
  SCROLLBAR_THICKNESS_PX,
  TRACKS_START_Y,
} from '../helpers/constants'
import TimelineBar from './parts/TimelineBar'
import TimelineGridLines from './parts/TimelineGridLines'
import TimelineTrack from './parts/TimelineTrack'
import TimelineScrollbar from './parts/TimelineScrollbar'
import { useProjectStore } from '../stores/projectStore'
import { useTimelineStore } from '../stores/timelineStore'
import { useShallow } from 'zustand/react/shallow'
import InsertTrackMenu from './parts/InsertTrackMenu'
import type { Clip } from '../types/Clip'
import type { AudioTrack } from '../types/Track'
import { useGlobalStore } from '../stores/globalStore'

const Timeline = () => {
  const timelineViewportRef = useRef<HTMLDivElement>(null)
  const applicationRef = useRef<ApplicationRef>(null)
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
    pxPerBeat,
    setPxPerBeat,
    trackScale,
    setTrackScale,
    barsVisible,
    setBarsVisible,
    scrollBeats,
    setScrollBeats,
    scrollTracks,
    setScrollTracks,
  } = useTimelineStore(
    useShallow((state) => ({
      pxPerBeat: state.pxPerBeat,
      setPxPerBeat: state.setPxPerBeat,
      trackScale: state.trackScale,
      setTrackScale: state.setTrackScale,
      barsVisible: state.barsVisible,
      setBarsVisible: state.setBarsVisible,
      scrollBeats: state.scrollBeats,
      setScrollBeats: state.setScrollBeats,
      scrollTracks: state.scrollTracks,
      setScrollTracks: state.setScrollTracks,
    })),
  )

  const visibleTracks = useMemo(
    () =>
      generatorTracksOrder
        .map((id) => tracks[id])
        .filter((t): t is GeneratorTrack => !!t && t.kind !== TrackKind.Bus),
    [generatorTracksOrder, tracks],
  )

  const hScale = pxPerBeat / BASE_PX_PER_BEAT
  const totalBeats = barsVisible * BEATS_PER_BAR
  const totalTrackRows = visibleTracks.length + EXTRA_EMPTY_TRACK_SLOTS
  const trackAreaHeight = Math.max(0, containerSize.height - TRACKS_START_Y)

  // How much of the content is currently visible, in domain units - drives
  // both the pixi camera transforms and the custom scrollbars' thumb sizes.
  const viewBeats = pxPerBeat > 0 ? containerSize.width / pxPerBeat : 0
  const viewTrackRows =
    trackScale > 0 ? trackAreaHeight / (BASE_TRACK_HEIGHT_PX * trackScale) : 0

  const minViewBeats = containerSize.width / MAX_PX_PER_BEAT
  const maxViewBeats = containerSize.width / MIN_PX_PER_BEAT
  const minViewTrackRows = trackAreaHeight / (BASE_TRACK_HEIGHT_PX * MAX_TRACK_SCALE)
  const maxViewTrackRows = trackAreaHeight / (BASE_TRACK_HEIGHT_PX * MIN_TRACK_SCALE)

  // Measure the viewport (the area the Pixi canvas fills, below/left of the
  // scrollbars) so we know how much content fits on screen.
  useLayoutEffect(() => {
    const el = timelineViewportRef.current
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

  // Pixi's `resizeTo` only re-measures on the browser window's own 'resize'
  // event - it has no ResizeObserver of its own, so it never notices when
  // this container resizes for any other reason (flex layout settling,
  // sidebar toggles, etc). Force a resync whenever our own ResizeObserver
  // above sees a change, so the canvas can't get stuck at a stale size.
  useLayoutEffect(() => {
    applicationRef.current?.getApplication()?.resize()
  }, [containerSize.width, containerSize.height])

  // Application.init() is async (it spins up a WebGL/WebGPU context), so on
  // a slow first load it can finish AFTER the two effects above already ran
  // with applicationRef still null - and since neither of them re-fires on
  // its own, nothing would ever tell the freshly-ready app to resize, so it
  // renders at whatever default size it started with (grid lines end up
  // drawn for the wrong height) until some unrelated event forces a
  // re-render. Re-measure and resize explicitly the moment init completes.
  const handleApplicationInit = useCallback(() => {
    const el = timelineViewportRef.current
    if (!el) return
    setContainerSize({
      width: Math.floor(el.clientWidth),
      height: Math.floor(el.clientHeight),
    })
    applicationRef.current?.getApplication()?.resize()
  }, [])

  // Ensure the timeline is wide enough to show all clips.
  useEffect(() => {
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
      setBarsVisible(Math.max(MINIMUM_BARS_VISIBLE, requiredBars))
    }
  }, [barsVisible, generatorTracksOrder, ppq, setBarsVisible, tracks])

  // Grid lines/header/track backgrounds are all drawn out to `barsVisible`,
  // so if zooming out (or a short project on a wide window) ever makes more
  // beats visible than that covers, grow it - otherwise they'd stop short of
  // the canvas edge instead of tiling all the way across.
  useEffect(() => {
    const requiredBars = Math.ceil((scrollBeats + viewBeats) / BEATS_PER_BAR)
    if (requiredBars > barsVisible) {
      setBarsVisible(requiredBars)
    }
  }, [barsVisible, scrollBeats, setBarsVisible, viewBeats])

  // If the content shrinks (or the viewport grows) such that the current
  // scroll position would show past the end of the content, pull it back in.
  useEffect(() => {
    const maxScrollBeats = Math.max(0, totalBeats - viewBeats)
    if (scrollBeats > maxScrollBeats) {
      setScrollBeats(maxScrollBeats)
    }
  }, [scrollBeats, setScrollBeats, totalBeats, viewBeats])

  useEffect(() => {
    const maxScrollTracks = Math.max(0, totalTrackRows - viewTrackRows)
    if (scrollTracks > maxScrollTracks) {
      setScrollTracks(maxScrollTracks)
    }
  }, [scrollTracks, setScrollTracks, totalTrackRows, viewTrackRows])

  useEffect(() => {
    // Reset dt accumulator when zoom changes so the first edge-scroll after zoom doesn't jump.
    lastAutoScrollTs.current = null
  }, [pxPerBeat])

  const handleWheel = (e: React.WheelEvent<HTMLDivElement>) => {
    const el = timelineViewportRef.current
    if (!el) return

    if (e.ctrlKey) {
      e.preventDefault()
      const rect = el.getBoundingClientRect()
      const cursorBeats = (e.clientX - rect.left) / pxPerBeat + scrollBeats

      const zoomFactor = 1.1
      const nextPxPerBeat =
        e.deltaY < 0 ? pxPerBeat * zoomFactor : pxPerBeat / zoomFactor
      const newPxPerBeat = Math.round(
        clamp(nextPxPerBeat, MIN_PX_PER_BEAT, MAX_PX_PER_BEAT),
      )
      if (newPxPerBeat === pxPerBeat) return

      // Keep the beat under the cursor fixed on screen while zooming.
      const newScrollBeats = cursorBeats - (e.clientX - rect.left) / newPxPerBeat
      const newViewBeats = containerSize.width / newPxPerBeat
      const maxScrollBeats = Math.max(0, totalBeats - newViewBeats)
      setPxPerBeat(newPxPerBeat)
      setScrollBeats(clamp(newScrollBeats, 0, maxScrollBeats))
      return
    }

    if (e.shiftKey) {
      e.preventDefault()
      const rect = el.getBoundingClientRect()
      const cursorTrackRows =
        (e.clientY - rect.top - TRACKS_START_Y) /
          (BASE_TRACK_HEIGHT_PX * trackScale) +
        scrollTracks

      const zoomFactor = 1.1
      const nextTrackScale =
        e.deltaY < 0 ? trackScale * zoomFactor : trackScale / zoomFactor
      const newTrackScale = clamp(nextTrackScale, MIN_TRACK_SCALE, MAX_TRACK_SCALE)
      if (newTrackScale === trackScale) return

      // Keep the track row under the cursor fixed on screen while zooming.
      const newScrollTracks =
        cursorTrackRows -
        (e.clientY - rect.top - TRACKS_START_Y) /
          (BASE_TRACK_HEIGHT_PX * newTrackScale)
      const newViewTrackRows = trackAreaHeight / (BASE_TRACK_HEIGHT_PX * newTrackScale)
      const maxScrollTracks = Math.max(0, totalTrackRows - newViewTrackRows)
      setTrackScale(newTrackScale)
      setScrollTracks(clamp(newScrollTracks, 0, maxScrollTracks))
      return
    }

    e.preventDefault()
    const deltaTracks = e.deltaY / (BASE_TRACK_HEIGHT_PX * trackScale)
    const maxScrollTracks = Math.max(0, totalTrackRows - viewTrackRows)
    setScrollTracks(clamp(scrollTracks + deltaTracks, 0, maxScrollTracks))
  }

  // Edge auto-scroll needs to keep advancing every frame for as long as the
  // pointer sits near an edge, not just when a new pointermove/dragover
  // event happens to fire - otherwise it stalls the instant the cursor stops
  // moving, and native HTML5 `dragover` events are throttled/irregular in
  // the first place. A rAF loop reads the latest pointer X from a ref and
  // reads/writes the timeline store directly via getState()/setState-style
  // setters (bypassing the React hook) so the loop's own function identity
  // can stay stable across renders without ever going stale.
  const autoScrollDragRef = useRef<{ clientX: number | null; rafId: number | null }>(
    { clientX: null, rafId: null },
  )

  const stepAutoScroll = useCallback(() => {
    const dragState = autoScrollDragRef.current
    const el = timelineViewportRef.current
    if (dragState.clientX === null || !el) {
      dragState.rafId = null
      return
    }

    const clientX = dragState.clientX
    const rect = el.getBoundingClientRect()
    const thresholdPx = 60
    const maxSpeedPxPerSec = 1400

    const now = performance.now()
    const last = lastAutoScrollTs.current ?? now
    lastAutoScrollTs.current = now
    const dt = Math.min(0.05, Math.max(0.0, (now - last) / 1000))

    let direction = 0
    let intensity = 0
    if (clientX < rect.left + thresholdPx) {
      direction = -1
      intensity = (thresholdPx - (clientX - rect.left)) / thresholdPx
    } else if (clientX > rect.right - thresholdPx) {
      direction = 1
      intensity = (thresholdPx - (rect.right - clientX)) / thresholdPx
    }

    if (direction !== 0) {
      const store = useTimelineStore.getState()
      const speed = maxSpeedPxPerSec * intensity * intensity
      const desiredDeltaBeats = (direction * speed * dt) / store.pxPerBeat
      const viewportBeats = el.clientWidth / store.pxPerBeat

      let barsVisibleNow = store.barsVisible
      if (direction > 0) {
        const paddingBeats = 400 / store.pxPerBeat
        const anticipateBeats = (speed * dt * 2) / store.pxPerBeat
        const visibleRightBeats =
          store.scrollBeats + viewportBeats + paddingBeats + anticipateBeats
        const requiredBars = Math.ceil(visibleRightBeats / BEATS_PER_BAR)
        if (requiredBars > barsVisibleNow) {
          store.setBarsVisible(requiredBars)
          barsVisibleNow = requiredBars
        }
      }

      const maxScrollBeats = Math.max(0, barsVisibleNow * BEATS_PER_BAR - viewportBeats)
      const nextScrollBeats = clamp(
        store.scrollBeats + desiredDeltaBeats,
        0,
        maxScrollBeats,
      )
      if (nextScrollBeats !== store.scrollBeats) {
        store.setScrollBeats(nextScrollBeats)
      }
    }

    dragState.rafId = requestAnimationFrame(stepAutoScroll)
  }, [])

  const autoScrollWhileDragging = useCallback(
    (clientX: number) => {
      const dragState = autoScrollDragRef.current
      dragState.clientX = clientX
      if (dragState.rafId === null) {
        lastAutoScrollTs.current = null
        dragState.rafId = requestAnimationFrame(stepAutoScroll)
      }
    },
    [stepAutoScroll],
  )

  const stopAutoScrollDragging = useCallback(() => {
    const dragState = autoScrollDragRef.current
    dragState.clientX = null
    if (dragState.rafId !== null) {
      cancelAnimationFrame(dragState.rafId)
      dragState.rafId = null
    }
    lastAutoScrollTs.current = null
  }, [])

  useEffect(() => stopAutoScrollDragging, [stopAutoScrollDragging])

  const handleHorizontalScrollChange = useCallback(
    (newViewStart: number, newViewSize: number) => {
      const totalBeatsNow = barsVisible * BEATS_PER_BAR
      const newViewEnd = newViewStart + newViewSize
      if (newViewEnd > totalBeatsNow) {
        setBarsVisible(
          Math.max(barsVisible, Math.ceil(newViewEnd / BEATS_PER_BAR)),
        )
      }
      const newPxPerBeat = clamp(
        containerSize.width / newViewSize,
        MIN_PX_PER_BEAT,
        MAX_PX_PER_BEAT,
      )
      setPxPerBeat(newPxPerBeat)
      setScrollBeats(Math.max(0, newViewStart))
    },
    [barsVisible, containerSize.width, setBarsVisible, setPxPerBeat, setScrollBeats],
  )

  const handleVerticalScrollChange = useCallback(
    (newViewStart: number, newViewSize: number) => {
      const newTrackScale = clamp(
        trackAreaHeight / (newViewSize * BASE_TRACK_HEIGHT_PX),
        MIN_TRACK_SCALE,
        MAX_TRACK_SCALE,
      )
      setTrackScale(newTrackScale)
      const newViewTrackRows = trackAreaHeight / (BASE_TRACK_HEIGHT_PX * newTrackScale)
      const maxScrollTracks = Math.max(0, totalTrackRows - newViewTrackRows)
      setScrollTracks(clamp(newViewStart, 0, maxScrollTracks))
    },
    [setScrollTracks, setTrackScale, totalTrackRows, trackAreaHeight],
  )

  const handleDrop = async (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault()
    e.stopPropagation()
    stopAutoScrollDragging()
    const audioPath = e.dataTransfer.getData('audio-path')
    if (!audioPath) return

    const el = e.currentTarget
    const rect = el.getBoundingClientRect()
    const xPosition = e.clientX - rect.left
    const yPosition = e.clientY - rect.top
    const beatsPosition = xPosition / pxPerBeat + scrollBeats
    const ppqPosition = Math.max(0, Math.round(beatsPosition * ppq))
    const trackRowUnits =
      (yPosition - TRACKS_START_Y) / (BASE_TRACK_HEIGHT_PX * trackScale) +
      scrollTracks
    const trackIndex = Math.floor(trackRowUnits)
    const targetTrack = trackIndex >= 0 ? visibleTracks[trackIndex] : null

    // Drop on an existing audio track -> insert clip into that track.
    // Drop on empty space (no track) -> create new audio track with one clip.
    if (!targetTrack) {
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

    if (targetTrack.kind !== TrackKind.Audio) return

    setGlobalLoading(true)
    const insertedClip = await invoke<Clip | null>(
      MIXER_ADD_CLIP_TO_AUDIO_TRACK,
      {
        clip: {
          trackId: targetTrack.id,
          startPpq: ppqPosition,
          sourcePath: audioPath,
        },
      },
    )
    setGlobalLoading(false)
    if (!insertedClip) return
    addClip(insertedClip)
  }

  return (
    <div className='flex flex-col h-full'>
      <div className='p-2'>
        <InsertTrackMenu />
      </div>
      <div className='relative flex-1 min-h-0'>
        <div
          className='absolute top-0 left-0'
          style={{ right: SCROLLBAR_THICKNESS_PX, height: SCROLLBAR_THICKNESS_PX }}
        >
          <TimelineScrollbar
            orientation='horizontal'
            viewStart={scrollBeats}
            viewSize={viewBeats}
            totalSize={totalBeats}
            minViewSize={minViewBeats}
            maxViewSize={maxViewBeats}
            onChange={handleHorizontalScrollChange}
          />
        </div>
        <div
          className='absolute top-0 right-0 bottom-0'
          style={{ width: SCROLLBAR_THICKNESS_PX }}
        >
          <TimelineScrollbar
            orientation='vertical'
            viewStart={scrollTracks}
            viewSize={viewTrackRows}
            totalSize={totalTrackRows}
            minViewSize={minViewTrackRows}
            maxViewSize={maxViewTrackRows}
            onChange={handleVerticalScrollChange}
          />
        </div>
        <div
          id='timeline-viewport'
          ref={timelineViewportRef}
          className='absolute left-0 overflow-hidden'
          style={{
            top: SCROLLBAR_THICKNESS_PX,
            right: SCROLLBAR_THICKNESS_PX,
            bottom: 0,
          }}
          onWheel={handleWheel}
          onContextMenu={(e) => e.preventDefault()}
          onDragOver={(e) => {
            e.preventDefault()
            e.stopPropagation()
            e.dataTransfer.dropEffect = 'copy'
            autoScrollWhileDragging(e.clientX)
          }}
          onDragLeave={stopAutoScrollDragging}
          onDrop={handleDrop}
        >
          <Application
            ref={applicationRef}
            resizeTo={timelineViewportRef}
            onInit={handleApplicationInit}
            className='block'
            backgroundAlpha={0}
            antialias={false}
            autoDensity
          >
            <pixiContainer
              x={-scrollBeats * pxPerBeat}
              scale={{ x: hScale, y: 1 }}
            >
              <TimelineGridLines viewportHeight={containerSize.height} />
              <pixiContainer
                y={TRACKS_START_Y - scrollTracks * BASE_TRACK_HEIGHT_PX * trackScale}
                scale={{ x: 1, y: trackScale }}
              >
                {visibleTracks.map((track, index) => (
                  <TimelineTrack
                    key={track.id}
                    track={track}
                    trackIndex={index}
                    onAutoScrollDuringDrag={autoScrollWhileDragging}
                    onAutoScrollDragEnd={stopAutoScrollDragging}
                  />
                ))}
              </pixiContainer>
              {/* Drawn last so the ruler/playhead-handle stay on top of any
                  track rows that scroll up underneath them. */}
              <TimelineBar viewportHeight={containerSize.height} />
            </pixiContainer>
          </Application>
        </div>
      </div>
    </div>
  )
}

export default Timeline
