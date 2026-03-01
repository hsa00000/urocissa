import { FunctionalComponent, h, Transition } from 'vue'

interface ThumbhashImageProps {
  src: string | undefined
}

const ThumbhashImage: FunctionalComponent<ThumbhashImageProps> = (props) => {
  return h(Transition, { name: 'slide-fade', appear: true }, () =>
    h('img', {
      style: {
        position: 'absolute',
        zIndex: 1
      },
      class: 'thumbhash-image w-100 h-100 bg-placeholder',
      src: props.src
    })
  )
}

ThumbhashImage.props = {
  src: {
    type: String,
    required: false
  }
}

export default ThumbhashImage
