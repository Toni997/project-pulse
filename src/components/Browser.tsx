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
  PREVIEW_AUDIO_ERROR_EVENT_NAME,
  PREVIEW_PLAY,
  FS_SCAN_DIRECTORY_TREE,
} from '../helpers/constants'
import { NodeRendererProps, Tree } from 'react-arborist'
import type BrowserNode from '../types/BrowserNode'
import { Menu, Item, Separator, Submenu, useContextMenu } from 'react-contexify'
import 'react-contexify/ReactContexify.css'

const Node = ({
  node,
  style,
  showContextMenu,
}: NodeRendererProps<BrowserNode> & { showContextMenu: (e: any) => void }) => {
  const previewAudioClick = async (filePath: string) => {
    invoke(PREVIEW_PLAY, { filePath })
  }

  return (
    <>
      <Tooltip label={node.data.label} position='right' withArrow>
        <span
          style={style}
          className='flex gap-1 items-center hover:bg-gray-200 cursor-pointer'
          onContextMenu={showContextMenu}
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
  const { show } = useContextMenu({
    id: 'menu',
  })

  useLayoutEffect(() => {
    if (!fileBrowserRef.current) return

    const observer = new ResizeObserver((entries) => {
      setHeight(entries[0].contentRect.height)
    })

    observer.observe(fileBrowserRef.current)

    return () => observer.disconnect()
  }, [])

  useEffect(() => {
    const unlisten = listen(PREVIEW_AUDIO_ERROR_EVENT_NAME, (event) => {
      const errorText: string = event.payload as string
      notifications.show({
        color: 'red',
        title: 'Ooops!',
        message: errorText as string,
      })
    })

    return () => {
      unlisten.then((f) => f())
    }
  }, [])

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      })
      if (!selected) return
      setIsLoading(true)
      const temp: BrowserNode[] = await invoke(FS_SCAN_DIRECTORY_TREE, {
        path: selected,
      })
      setData(temp)
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
    return <Node {...props} showContextMenu={(e: any) => show({ event: e })} />
  }, [])

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
        <Item>Assign to new Audio Track</Item>
        <Submenu label='Assign to track'>
          <Item id='reload'>Track 1</Item>
        </Submenu>
        <Separator />
        <Item>Add to Favorites</Item>
        <Item>Add to New Group</Item>
        <Submenu label='Add to Group'>
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
