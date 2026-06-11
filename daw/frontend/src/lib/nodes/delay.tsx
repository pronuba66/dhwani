import { Delay } from "@/components/nodes/Delay"
import {
  Dhwani,
  Ports,
  type Node,
  type NodeProps,
  type Track,
} from "@/lib/core"
import type { ComponentType } from "react"

export type DelayNodeProps = {
  nChannels?: number
  delay: number
  mul: number
} & NodeProps

export class DelayNode implements Node {
  static readonly tag: string = "Delay"
  static readonly name: string = "Delay"
  static readonly description: string | null = "Delay"
  readonly id: number
  readonly track: Track
  readonly ports: Ports
  readonly View: ComponentType<{ dhwani: Dhwani; node: Node }> | null =
    Delay as ComponentType<{
      dhwani: Dhwani
      node: Node
    }>
  data: DelayNodeProps

  constructor(id: number, track: Track, ports: Ports, data: DelayNodeProps) {
    this.id = id
    this.track = track
    this.ports = ports
    this.data = data
  }

  width() {
    return 256
  }

  height() {
    return 64
  }

  static default(): DelayNodeProps {
    return {
      nChannels: 2,
      delay: 1,
      mul: 1,
    } satisfies DelayNodeProps
  }

  static portIdInput: number = 0
  static portIdOutput: number = 1
}
