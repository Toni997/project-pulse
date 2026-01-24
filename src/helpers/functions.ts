import { ECursor } from './enums'

const setCursor = (cursor: ECursor) => {
  document.body.style.cursor = cursor
}

export { setCursor }
