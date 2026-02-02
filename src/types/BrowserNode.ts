interface BrowserNode {
  value: string
  label: string
  is_dir: boolean
  children: BrowserNode[] | null
}

export default BrowserNode
