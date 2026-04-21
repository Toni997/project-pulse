import { useProjectStore } from '../stores/projectStore'
import MixerTrack from './parts/MixerTrack'
import { useShallow } from 'zustand/react/shallow'
import type { GeneratorOrBusTrack } from '../types/Track'

const Mixer = () => {
  const { master, tracks, generatorTracksOrder, busesOrder } = useProjectStore(
    useShallow((state) => ({
      master: state.master,
      tracks: state.tracks,
      generatorTracksOrder: state.generatorTracksOrder,
      busesOrder: state.busesOrder,
    })),
  )

  return (
    <>
      <div className='flex w-full h-full min-h-[150px]'>
        <MixerTrack track={master} />
        {generatorTracksOrder
          .map((id) => tracks[id])
          .filter((t): t is GeneratorOrBusTrack => !!t)
          .map((track) => (
            <MixerTrack key={track.id} track={track} />
          ))}
        {busesOrder
          .map((id) => tracks[id])
          .filter((t): t is GeneratorOrBusTrack => !!t)
          .map((track) => (
            <MixerTrack key={track.id} track={track} />
          ))}
      </div>
    </>
  )
}

export default Mixer
