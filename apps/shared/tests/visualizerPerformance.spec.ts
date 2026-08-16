import { render } from '@testing-library/vue'
import { defineComponent, h, nextTick } from 'vue'
import { afterEach, describe, expect, it, vi } from 'vitest'

import AudioVisualizer from '../src/components/player/AudioVisualizer.vue'
import { useVisualizerData } from '../src/composables/useVisualizerData'

const backend = vi.hoisted(() => ({
  handler: null as null | ((event: {
    data: { frequencyData: number[]; timeDomainData: number[] }
    type: 'AudioSpectrum'
  }) => void),
}))

vi.mock('../src/audio', () => ({
  getAudioPlayer: vi.fn(),
}))

vi.mock('../src/effect', () => ({
  runAureliaEffect: vi.fn(() => Promise.resolve()),
}))

vi.mock('../src/effect/services/api', () => ({
  audioSetAnalyzerEnabledEffect: vi.fn(),
}))

vi.mock('../src/lib/backend-events', () => ({
  subscribeBackendEvents: vi.fn((handler: typeof backend.handler) => {
    backend.handler = handler
    return () => {
      backend.handler = null
    }
  }),
}))

vi.mock('../src/lib/platform', () => ({
  isElectron: () => true,
}))

afterEach(() => {
  backend.handler = null
  vi.restoreAllMocks()
  vi.unstubAllGlobals()
})

describe('visualizer performance', () => {
  it('publishes analyzer snapshots at no more than 30 FPS', async () => {
    let visualizer: ReturnType<typeof useVisualizerData> | undefined
    const Harness = defineComponent({
      setup() {
        visualizer = useVisualizerData()
        return () => h('div')
      },
    })
    const view = render(Harness)
    await visualizer!.setEnabled(true)

    const now = vi.spyOn(performance, 'now')
    const spectrum = {
      data: {
        frequencyData: new Array(128).fill(180),
        timeDomainData: new Array(256).fill(140),
      },
      type: 'AudioSpectrum' as const,
    }

    now.mockReturnValue(100)
    backend.handler!(spectrum)
    const firstFrame = visualizer!.frequencyData.value

    now.mockReturnValue(110)
    backend.handler!(spectrum)
    expect(visualizer!.frequencyData.value).toBe(firstFrame)

    now.mockReturnValue(140)
    backend.handler!(spectrum)
    expect(visualizer!.frequencyData.value).not.toBe(firstFrame)

    view.unmount()
  })

  it('draws once per update instead of running a perpetual animation loop', async () => {
    const frames: FrameRequestCallback[] = []
    vi.stubGlobal('requestAnimationFrame', vi.fn((callback: FrameRequestCallback) => {
      frames.push(callback)
      return frames.length
    }))
    vi.stubGlobal('cancelAnimationFrame', vi.fn())
    vi.stubGlobal('ResizeObserver', class {
      private readonly callback: ResizeObserverCallback

      constructor(callback: ResizeObserverCallback) {
        this.callback = callback
      }

      disconnect(): void {}

      observe(target: Element): void {
        this.callback([{
          contentRect: { height: 100, width: 200 },
          target,
        } as ResizeObserverEntry], this as unknown as ResizeObserver)
      }
    })

    const gradient = { addColorStop: vi.fn() }
    const context = {
      beginPath:           vi.fn(),
      clearRect:           vi.fn(),
      closePath:           vi.fn(),
      createLinearGradient: vi.fn(() => gradient),
      fill:                vi.fn(),
      fillRect:            vi.fn(),
      lineTo:              vi.fn(),
      moveTo:              vi.fn(),
      quadraticCurveTo:    vi.fn(),
      setTransform:        vi.fn(),
      stroke:              vi.fn(),
    }
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(context as unknown as CanvasRenderingContext2D)

    const frequencyData = new Uint8Array(128).fill(120)
    const timeDomainData = new Uint8Array(256).fill(128)
    const view = render(AudioVisualizer, {
      props: {
        frequencyData,
        isPlaying: true,
        timeDomainData,
      },
    })
    await nextTick()

    expect(requestAnimationFrame).toHaveBeenCalledTimes(1)
    frames.shift()!(0)
    expect(requestAnimationFrame).toHaveBeenCalledTimes(1)

    await view.rerender({
      frequencyData: frequencyData.slice(),
      isPlaying: true,
      timeDomainData,
    })
    await nextTick()
    expect(requestAnimationFrame).toHaveBeenCalledTimes(2)

    view.unmount()
  })
})
