import { Button, Menu } from '@mantine/core'
import { invoke } from '@tauri-apps/api/core'
import {
  MIXER_ADD_AUDIO_TRACK,
  MIXER_ADD_SAMPLER_TRACK,
} from '../../helpers/constants'
import { useProjectStore } from '../../stores/projectStore'
import { useShallow } from 'zustand/react/shallow'
import { AudioTrack, SamplerTrack } from '../../types/Track'

const InsertTrackMenu = () => {
  const { addTrack } = useProjectStore(
    useShallow((state) => ({
      addTrack: state.addTrack,
    })),
  )

  const handleInsertAudioTrack = async () => {
    const newAudiotrack: AudioTrack | null = await invoke(MIXER_ADD_AUDIO_TRACK)
    if (newAudiotrack) {
      addTrack(newAudiotrack)
    }
  }

  const handleInsertSamplerTrack = async () => {
    const newSamplerTrack: SamplerTrack | null = await invoke(
      MIXER_ADD_SAMPLER_TRACK,
    )
    if (newSamplerTrack) {
      addTrack(newSamplerTrack)
    }
  }

  return (
    <Menu width={200} shadow='md' position='bottom-start'>
      <Menu.Target>
        <Button>+</Button>
      </Menu.Target>
      <Menu.Dropdown>
        <Menu.Item onClick={handleInsertAudioTrack}>Audio Track</Menu.Item>
        <Menu.Item onClick={handleInsertSamplerTrack}>Sampler Track</Menu.Item>
        <Menu.Item disabled>Instrument Track</Menu.Item>
        <Menu.Item disabled>Bus Track</Menu.Item>
      </Menu.Dropdown>
    </Menu>
  )
}

export default InsertTrackMenu
