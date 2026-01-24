import './App.css'
import { MantineProvider } from '@mantine/core'
import { Notifications } from '@mantine/notifications'
import '@mantine/core/styles.css'
import '@mantine/notifications/styles.css'
import AppShell from './components/AppShell'

const App = () => {
  return (
    <MantineProvider>
      <Notifications />
      <AppShell />
    </MantineProvider>
  )
}

export default App
