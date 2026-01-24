import { useEffect, useState } from 'react'
import {
  Tree,
  LoadingOverlay,
  Button,
  Box,
  TreeNodeData,
  Divider,
  Tooltip,
} from '@mantine/core'
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
  PREVIEW_AUDIO_FUNC_NAME,
  SCAN_DIRECTORY_TREE_FUNC_NAME,
} from '../helpers/constants'
import { ContextMenu } from 'radix-ui'

interface TreeNodeDataExpanded extends TreeNodeData {
  is_dir: boolean
}

const Browser = () => {
  const [data, setData] = useState<TreeNodeData[] | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  console.log('UPDATED')

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
      const temp: TreeNodeData[] = await invoke(SCAN_DIRECTORY_TREE_FUNC_NAME, {
        path: selected,
      })
      setData(temp)
      setIsLoading(false)
    } catch (err) {
      console.error('Error selecting folder:', err)
    }
  }

  const previewAudioClick = async (filePath: string) => {
    invoke(PREVIEW_AUDIO_FUNC_NAME, { filePath })
  }

  return (
    <Box pos='relative' className='p-2'>
      <LoadingOverlay
        visible={isLoading}
        zIndex={1000}
        overlayProps={{ radius: 'sm', blur: 2 }}
      />
      <Button onClick={handleSelectFolder} className='mb-2'>
        Select folder
      </Button>
      {data && !isLoading && (
        <Tree
          data={data}
          levelOffset={18}
          renderNode={({ node, elementProps }) => {
            const nodeExpanded = node as TreeNodeDataExpanded
            return (
              <>
                <ContextMenu.Root>
                  <ContextMenu.Trigger className='ContextMenuTrigger'>
                    <Tooltip label={node.label} position='right' withArrow>
                      <span
                        className={`flex gap-1 items-center hover:bg-gray-200 ${elementProps.className}`}
                        onClick={
                          nodeExpanded.is_dir
                            ? elementProps.onClick
                            : () => previewAudioClick(nodeExpanded.value)
                        }
                      >
                        {nodeExpanded.is_dir ? (
                          <FolderIcon
                            className='shrink-0'
                            size={18}
                            weight='light'
                          />
                        ) : (
                          <FileAudioIcon
                            className='shrink-0'
                            size={18}
                            weight='light'
                          />
                        )}
                        <span className='text-ellipsis whitespace-nowrap overflow-hidden'>
                          {node.label}
                        </span>
                      </span>
                    </Tooltip>
                  </ContextMenu.Trigger>
                  {!nodeExpanded.is_dir && (
                    <ContextMenu.Content className='ContextMenuContent'>
                      <ContextMenu.Item className='ContextMenuItem'>
                        Assign to new Audio Track
                      </ContextMenu.Item>
                      <ContextMenu.Sub>
                        <ContextMenu.SubTrigger className='ContextMenuSubTrigger'>
                          Assign to track
                          <div className='RightSlot'>
                            <CaretRightIcon />
                          </div>
                        </ContextMenu.SubTrigger>
                        <ContextMenu.SubContent className='ContextMenuSubContent'>
                          <ContextMenu.Item className='ContextMenuItem'>
                            Track 1
                          </ContextMenu.Item>
                        </ContextMenu.SubContent>
                      </ContextMenu.Sub>
                      <ContextMenu.Separator className='ContextMenuSeparator' />
                      <ContextMenu.Item className='ContextMenuItem'>
                        Add to Favorites
                      </ContextMenu.Item>
                      <ContextMenu.Item className='ContextMenuItem'>
                        Add to New Group
                      </ContextMenu.Item>
                      <ContextMenu.Sub>
                        <ContextMenu.SubTrigger className='ContextMenuSubTrigger'>
                          Add to Group
                          <div className='RightSlot'>
                            <CaretRightIcon />
                          </div>
                        </ContextMenu.SubTrigger>
                        <ContextMenu.SubContent className='ContextMenuSubContent'>
                          <ContextMenu.Item className='ContextMenuItem'>
                            Group 1
                          </ContextMenu.Item>
                        </ContextMenu.SubContent>
                      </ContextMenu.Sub>
                    </ContextMenu.Content>
                  )}
                </ContextMenu.Root>
                <Divider />
              </>
            )
          }}
        />
      )}
    </Box>
  )
}

export default Browser
