import { AngleSlider } from '@mantine/core'
import { Slider } from 'radix-ui'
import React from 'react'
import { useProjectStore } from '../stores/projectStore'
import MixerTrack from './parts/MixerTrack'

const Mixer = () => {
  const projectState = useProjectStore()
  return (
    <>
      <div className='flex w-full h-full min-h-[150px]'>
        <MixerTrack track={projectState.master} />
        {projectState.tracks.map((track) => (
          <MixerTrack key={track.id} track={track} />
        ))}
      </div>
    </>
  )
}

export default Mixer
