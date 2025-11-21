import './App.css'
import Layout from './components/Layout'
import '@mantine/core/styles.css'
import { MantineProvider } from '@mantine/core'
import PlaybackControls from './components/PlaybackControls'

const App = () => {
  return (
    <MantineProvider>
      <div className='fixed w-full h-full bg-white'>
        <PlaybackControls />
        <div className='relative w-full h-full'>
          <Layout />
        </div>
      </div>
    </MantineProvider>
  )
}

export default App
