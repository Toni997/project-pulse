import { Model, Layout as ModelLayout, TabNode } from 'flexlayout-react'
import 'flexlayout-react/style/light.css'
import Browser from './Browser'
import defaultLayoutModel from '../helpers/defaultLayoutModel'
import Timeline from './Timeline'
import Mixer from './Mixer'

const Layout = () => {
  const model = Model.fromJson(defaultLayoutModel)

  const factory = (node: TabNode) => {
    switch (node.getComponent()) {
      case Browser.name:
        return <Browser />
      case Timeline.name:
        return <Timeline />
      case Mixer.name:
        return <Mixer />
      default:
        return node.getName()
    }
  }

  return <ModelLayout model={model} factory={factory} />
}

export default Layout
