import './App.css'
import Layout from './components/Layout'
import { MantineProvider } from '@mantine/core'
import { Notifications } from '@mantine/notifications'
import PlaybackControls from './components/PlaybackControls'
import '@mantine/core/styles.css'
import '@mantine/notifications/styles.css'

const App = () => {
  return (
    <MantineProvider>
      <Notifications />
      <div className='fixed flex flex-col w-full h-full bg-white'>
        <PlaybackControls />
        <div className='relative w-full h-full'>
          <Layout />
        </div>
      </div>
    </MantineProvider>
  )
}

export default App
