import { invoke } from '@tauri-apps/api/core'
import { ActionIcon } from '@mantine/core'
import { PlayPauseIcon, StopIcon } from '@phosphor-icons/react'
import { TRANSPORT_STOP } from '../helpers/constants'

const PlaybackControls = () => {
  const onPlayPauseClick = () => {
    console.log('play/pause')
  }
  const onStopClick = () => {
    invoke(TRANSPORT_STOP)
  }
  return (
    <div className='flex gap-0.5 p-2'>
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
