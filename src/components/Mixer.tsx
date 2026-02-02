import { AngleSlider } from '@mantine/core'
import { Slider } from 'radix-ui'
import React from 'react'

const Mixer = () => {
  return (
    <>
      <div className='w-[80px] h-full min-h-[150px] border-r-5 border-r-blue-300 bg-blue-200 p-1 flex flex-col items-center'>
        <span className='text-xs font-bold mb-2'>Master</span>
        <AngleSlider
          className='mb-2'
          aria-label='Angle slider'
          size={50}
          thumbSize={8}
        />
        <Slider.Root
          className='SliderRoot'
          defaultValue={[50]}
          orientation='vertical'
        >
          <Slider.Track className='SliderTrack'>
            <Slider.Range className='SliderRange' />
          </Slider.Track>
          <Slider.Thumb className='SliderThumb' />
        </Slider.Root>
      </div>
    </>
  )
}

export default Mixer
