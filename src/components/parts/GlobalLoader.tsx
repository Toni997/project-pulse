import { LoadingOverlay } from '@mantine/core'
import { useGlobalStore } from '../../stores/globalStore'

const GlobalLoader = () => {
  const isGlobalLoading = useGlobalStore((state) => state.isGlobalLoading)
  return (
    <LoadingOverlay
      visible={isGlobalLoading}
      zIndex={999999}
      overlayProps={{ radius: 'sm', blur: 0 }}
    />
  )
}

export default GlobalLoader
