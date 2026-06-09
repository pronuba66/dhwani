import {
  Dhwani,
  Ports,
  type Node,
  type NodeProps,
  type Track,
} from "@/lib/core"
import type { ComponentType } from "react"

export interface StereoNodeProps extends NodeProps {
  marker: null
}

export class StereoNode implements Node {
  static readonly tag: string = "Stereo"
  static readonly name: string = "Stereo"
  static readonly description: string | null = "Mono to Stereo"
  readonly id: number
  readonly track: Track
  readonly ports: Ports
  readonly View: ComponentType<{ dhwani: Dhwani; node: Node }> | null = null
  data: StereoNodeProps

  constructor(id: number, track: Track, ports: Ports, data: StereoNodeProps) {
    this.id = id
    this.track = track
    this.ports = ports
    this.data = data
  }

  width() {
    return 256
  }

  height() {
    return 0
  }

  static default(): StereoNodeProps {
    return {
      marker: null,
    } satisfies StereoNodeProps
  }

  static portIdInput: number = 0
  static portIdOutput: number = 1
}
