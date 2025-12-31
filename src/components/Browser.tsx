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
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { FolderIcon, FileAudioIcon } from '@phosphor-icons/react'

interface TreeNodeDataExpanded extends TreeNodeData {
  is_dir: boolean
}

const Browser = () => {
  const [data, setData] = useState<TreeNodeData[] | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      })
      if (!selected) return

      setIsLoading(true)
      const temp: TreeNodeData[] = await invoke('scan_directory_tree', {
        path: selected,
      })
      setData(temp)
      setIsLoading(false)
    } catch (err) {
      console.error('Error selecting folder:', err)
    }
  }

  const preview_audio_click = async (filePath: string) => {
    try {
      await invoke('preview_audio_file', { filePath })
    } catch (err: any) {
      console.log(err)
    }
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
          renderNode={({ node, expanded, hasChildren, elementProps }) => {
            const nodeExpanded = node as TreeNodeDataExpanded
            return (
              <>
                <Tooltip label={node.label} position='right' withArrow>
                  <span
                    className={`flex gap-1 items-center hover:bg-gray-200 ${elementProps.className}`}
                    onClick={
                      nodeExpanded.is_dir
                        ? elementProps.onClick
                        : () => preview_audio_click(nodeExpanded.value)
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
