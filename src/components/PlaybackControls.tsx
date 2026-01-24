import { invoke } from '@tauri-apps/api/core'
import { ActionIcon } from '@mantine/core'
import { PlayPauseIcon, StopIcon } from '@phosphor-icons/react'
import { STOP_AUDIO_FUNC_NAME } from '../helpers/constants'

const PlaybackControls = () => {
  const onPlayPauseClick = () => {
    console.log('play/pause')
  }
  const onStopClick = () => {
    console.log('stop')
    invoke(STOP_AUDIO_FUNC_NAME)
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
