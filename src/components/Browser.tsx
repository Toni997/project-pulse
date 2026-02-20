import {
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  useCallback,
} from 'react'
import { LoadingOverlay, Button, Box, Divider, Tooltip } from '@mantine/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { notifications } from '@mantine/notifications'
import {
  FolderIcon,
  FileAudioIcon,
  CaretRightIcon,
} from '@phosphor-icons/react'
import {
  NOTIFICATION_ERROR_EVENT,
  PREVIEW_PLAY,
  FS_SCAN_DIRECTORY_TREE,
  MIXER_ADD_AUDIO_TRACK,
  MIXER_ASSIGN_AUDIO_TO_TRACK,
} from '../helpers/constants'
import { NodeRendererProps, Tree } from 'react-arborist'
import type BrowserNode from '../types/BrowserNode'
import { Menu, Item, Separator, Submenu, useContextMenu } from 'react-contexify'
import 'react-contexify/ReactContexify.css'
import { useProjectStore } from '../stores/projectStore'
import { AudioTrack } from '../types/Track'
import { useGlobalStore } from '../stores/globalStore'

const defaultFolder =
  'C:/Users/skuez/OneDrive/Documents/#SAMPLES/Sounds of KSHMR Vol 4 Complete Edition'

const Node = ({
  node,
  style,
  showContextMenu,
}: NodeRendererProps<BrowserNode> & {
  showContextMenu: (e: any, path: string) => void
}) => {
  const previewAudioClick = async (filePath: string) => {
    invoke(PREVIEW_PLAY, { filePath })
  }

  return (
    <>
      <Tooltip label={node.data.label} position='right' withArrow>
        <span
          style={style}
          className='flex gap-1 items-center hover:bg-gray-200 cursor-pointer'
          onContextMenu={(e) => showContextMenu(e, node.data.value)}
          onClick={
            node.data.is_dir
              ? () => node.toggle()
              : () => previewAudioClick(node.data.value)
          }
        >
          {node.data.is_dir ? (
            <FolderIcon className='shrink-0' size={18} weight='light' />
          ) : (
            <FileAudioIcon className='shrink-0' size={18} weight='light' />
          )}
          <span className='text-ellipsis whitespace-nowrap overflow-hidden'>
            {node.data.label}
          </span>
        </span>
      </Tooltip>
      <Divider />
    </>
  )
}

const Browser = () => {
  const [data, setData] = useState<BrowserNode[] | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const fileBrowserRef = useRef<HTMLDivElement>(null)
  const [height, setHeight] = useState(0)
  const [contextMenuTargetPath, setContextMenuTargetPath] = useState('')
  const { show } = useContextMenu({
    id: 'menu',
  })
  const setGlobalLoading = useGlobalStore((state) => state.setGlobalLoading)
  const addAudioTrackToStore = useProjectStore((state) => state.addAudioTrack)
  const updateAudioTrackInStore = useProjectStore(
    (state) => state.updateAudioTrack,
  )
  const tracks = useProjectStore((state) => state.tracks)
  console.log(tracks)

  const handleLoadFolder = async (path: string) => {
    setIsLoading(true)
    const temp: BrowserNode[] = await invoke(FS_SCAN_DIRECTORY_TREE, { path })
    setData(temp)
    setIsLoading(false)
  }

  useLayoutEffect(() => {
    if (!fileBrowserRef.current) return
    const observer = new ResizeObserver((entries) => {
      setHeight(entries[0].contentRect.height)
    })
    observer.observe(fileBrowserRef.current)
    return () => observer.disconnect()
  }, [])

  useEffect(() => {
    const unlisten = listen(NOTIFICATION_ERROR_EVENT, (event) => {
      const errorText: string = event.payload as string
      notifications.show({
        color: 'red',
        title: 'Ooops!',
        message: errorText as string,
      })
    })
    handleLoadFolder(defaultFolder)

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      })
      if (!selected) return
      await handleLoadFolder(selected)
    } catch (err) {
      notifications.show({
        color: 'red',
        title: 'Ooops!',
        message: 'Error selecting folder',
      })
      console.error(`Error selecting folder`, err)
    } finally {
      setIsLoading(false)
    }
  }

  const nodeRenderer = useCallback((props: NodeRendererProps<BrowserNode>) => {
    const handleShowContextMenu = (e: any, path: string) => {
      setContextMenuTargetPath(path)
      show({ event: e })
    }
    return <Node {...props} showContextMenu={handleShowContextMenu} />
  }, [])

  const handleAssignToNewTrack = async () => {
    setGlobalLoading(true)
    const newAudioTrack: AudioTrack | null = await invoke(
      MIXER_ADD_AUDIO_TRACK,
      {
        sourcePath: contextMenuTargetPath,
      },
    )
    if (newAudioTrack) {
      addAudioTrackToStore(newAudioTrack)
    }
    setGlobalLoading(false)
  }

  const handleAssignToTrack = async (trackId: string) => {
    // TODO create an alert dialog if the audio track already has a source
    setGlobalLoading(true)
    const sourceId: string | null = await invoke(MIXER_ASSIGN_AUDIO_TO_TRACK, {
      trackId,
      sourcePath: contextMenuTargetPath,
    })
    if (sourceId) {
      updateAudioTrackInStore(trackId, { sourceId })
    }
    setGlobalLoading(false)
  }

  return (
    <Box pos='relative' className='flex flex-col w-full h-full'>
      <LoadingOverlay
        visible={isLoading}
        zIndex={1000}
        overlayProps={{ radius: 'sm', blur: 2 }}
      />
      <div className='shrink-0'>
        <Button onClick={handleSelectFolder} className='w-full'>
          Select folder
        </Button>
      </div>
      <Menu id='menu'>
        <Item onClick={handleAssignToNewTrack}>Assign to new Audio Track</Item>
        <Submenu label='Assign to track' disabled={!tracks.length}>
          {tracks.map((track, index) => (
            <Item
              key={track.id}
              id={`assign-track-${index}`}
              onClick={() => handleAssignToTrack(track.id)}
            >
              {track.name}
            </Item>
          ))}
        </Submenu>
        <Separator />
        <Item disabled>Add to Favorites</Item>
        <Item disabled>Add to New Group</Item>
        <Submenu label='Add to Group' disabled>
          <Item id='reload'>Group 1</Item>
        </Submenu>
      </Menu>
      <div ref={fileBrowserRef} className='w-full h-full'>
        {data && !isLoading && (
          <Tree
            initialData={data}
            idAccessor='value'
            openByDefault={false}
            width='100%'
            height={height}
            indent={15}
          >
            {nodeRenderer}
          </Tree>
        )}
      </div>
    </Box>
  )
}

export default Browser
