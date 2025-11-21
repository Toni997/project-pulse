import { ActionIcon } from '@mantine/core'
import { PlayPauseIcon, StopIcon } from '@phosphor-icons/react'
import React from 'react'

const PlaybackControls = () => {
  const onPlayPauseClick = () => {
    console.log('play/pause')
  }
  const onStopClick = () => {
    console.log('stop')
  }
  return (
    <div className='flex gap-0.5'>
      <ActionIcon variant='filled' onClick={onPlayPauseClick}>
        <PlayPauseIcon size={20} weight='light' />
      </ActionIcon>
      <ActionIcon variant='filled' onClick={onStopClick}>
        <StopIcon size={20} weight='light' />
      </ActionIcon>
    </div>
  )
}

export default PlaybackControls
