import MenuPanel from './MenuPanel'
import PlaybackControls from './PlaybackControls'
import Layout from './Layout'

const AppShell = () => {
  return (
    <div className='fixed flex flex-col w-full h-full bg-white'>
      <MenuPanel />
      <PlaybackControls />
      <div className='relative w-full h-full'>
        <Layout />
      </div>
    </div>
  )
}

export default AppShell
