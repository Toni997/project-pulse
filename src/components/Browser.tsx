import { useEffect, useState } from 'react'
import { Tree, TreeNodeData } from '@mantine/core'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'

const Browser = () => {
  const [data, setData] = useState<TreeNodeData[] | null>(null)

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      })

      if (!selected) return

      const temp: TreeNodeData[] = await invoke('scan_directory_tree', {
        path: selected,
      })
      console.log('Directory contents:', temp)
      setData(temp)
    } catch (err) {
      console.error('Error selecting folder:', err)
    }
  }

  return (
    <>
      <button onClick={handleSelectFolder}>Select folder</button>
      {data && <Tree data={data} />}
    </>
  )
}

export default Browser
