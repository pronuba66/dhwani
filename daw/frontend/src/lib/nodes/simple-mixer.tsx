import { SimpleMixer } from "@/components/nodes/SimpleMixer"
import {
  Dhwani,
  Ports,
  type Node,
  type NodeProps,
  type Track,
} from "@/lib/core"
import type { ComponentType } from "react"

export interface SimpleMixerNodeProps extends NodeProps {
  nChannels?: number
  muls: number[]
}

export class SimpleMixerNode implements Node {
  static readonly tag: string = "SimpleMixer"
  static readonly name: string = "Simple Mixer"
  static readonly description: string | null = "Simple mixer"
  readonly id: number
  readonly track: Track
  readonly ports: Ports
  readonly View: ComponentType<{ dhwani: Dhwani; node: Node }> | null =
    SimpleMixer as ComponentType<{ dhwani: Dhwani; node: Node }>
  data: SimpleMixerNodeProps

  constructor(
    id: number,
    track: Track,
    ports: Ports,
    data: SimpleMixerNodeProps
  ) {
    this.id = id
    this.track = track
    this.ports = ports
    this.data = data
  }

  width() {
    return 256
  }

  height() {
    return this.ports.nInputs * 32
  }

  static default(): SimpleMixerNodeProps {
    return {
      nChannels: 2,
      muls: [],
    } satisfies SimpleMixerNodeProps
  }

  static portIdOutput: number = 0

  static getInputPortId(inputNum: number): number {
    return SimpleMixerNode.portIdOutput + 1 + inputNum
  }
}
