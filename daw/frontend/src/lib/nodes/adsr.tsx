import { Adsr } from "@/components/nodes/Adsr"
import {
  Dhwani,
  Ports,
  type Node,
  type NodeProps,
  type Track,
} from "@/lib/core"
import type { ComponentType } from "react"

export interface AdsrNodeProps extends NodeProps {
  a: number
  d: number
  s: number
  r: number
}

export class AdsrNode implements Node {
  static readonly tag: string = "Adsr"
  static readonly name: string = "Adsr"
  static readonly description: string | null = "Adsr"
  readonly id: number
  readonly track: Track
  readonly ports: Ports
  readonly View: ComponentType<{ dhwani: Dhwani; node: Node }> | null =
    Adsr as ComponentType<{
      dhwani: Dhwani
      node: Node
    }>
  data: AdsrNodeProps

  constructor(id: number, track: Track, ports: Ports, data: AdsrNodeProps) {
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

  static default(): AdsrNodeProps {
    return {
      a: 0.01,
      d: 0.04,
      s: 0.9,
      r: 0.25,
    } satisfies AdsrNodeProps
  }

  static portIdInput: number = 0
  static portIdOutput: number = 1
}
