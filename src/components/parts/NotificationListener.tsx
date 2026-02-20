import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { notifications } from '@mantine/notifications'

const NotificationListener = () => {
  useEffect(() => {
    const unlisten = listen('notification-error', (event) => {
      const data = event.payload as string
      notifications.show({
        color: 'red',
        title: 'Ooops!',
        message: data,
      })
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  return null
}

export default NotificationListener
