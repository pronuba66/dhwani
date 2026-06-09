import { SimpleFilter } from "@/components/nodes/SimpleFilter"
import {
  Dhwani,
  Ports,
  type Node,
  type NodeProps,
  type Track,
} from "@/lib/core"
import type { ComponentType } from "react"

export type SimpleFilterMode = "LPF" | "HPF" | "BPF" | "BSF"

export type SimpleFilterNodeProps = {
  nChannels?: number
  mode: SimpleFilterMode
  freq: number
} & NodeProps

export class SimpleFilterNode implements Node {
  static readonly tag: string = "SimpleFilter"
  static readonly name: string = "Simple Filter"
  static readonly description: string | null = "Simple filter"
  readonly id: number
  readonly track: Track
  readonly ports: Ports
  readonly View: ComponentType<{ dhwani: Dhwani; node: Node }> | null =
    SimpleFilter as ComponentType<{
      dhwani: Dhwani
      node: Node
    }>
  data: SimpleFilterNodeProps

  constructor(
    id: number,
    track: Track,
    ports: Ports,
    data: SimpleFilterNodeProps
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
    return 64
  }

  static default(): SimpleFilterNodeProps {
    return {
      nChannels: 2,
      mode: "LPF",
      freq: 220,
    } satisfies SimpleFilterNodeProps
  }

  static portIdInput: number = 0
  static portIdOutput: number = 1
}
