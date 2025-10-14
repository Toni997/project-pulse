import './App.css'
import Layout from './components/Layout'
import '@mantine/core/styles.css'
import { MantineProvider } from '@mantine/core'

const App = () => {
  return (
    <MantineProvider>
      <Layout />
    </MantineProvider>
  )
}

export default App
