import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { notifications } from '@mantine/notifications'

const NotificationListener = () => {
  useEffect(() => {
    const unlisten = listen('preview-error', (event) => {
      const data: string = event.payload as string
      console.error('Preview failed:', data)
      notifications.show({
        color: 'red',
        title: 'Ooops!',
        message: data as string,
      })
    })

    return () => {
      unlisten.then((f) => f())
    }
  }, [])

  return null
}

export default NotificationListener
