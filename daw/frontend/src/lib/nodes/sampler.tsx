import {
  Dhwani,
  Ports,
  type Node,
  type NodeProps,
  type Track,
} from "@/lib/core"
import type { ComponentType } from "react"

export type SampleMode = "LPF" | "HPF" | "BPF" | "BSF"

export type SamplerNodeProps = {
  storageId: number
} & NodeProps

export class SamplerNode implements Node {
  static readonly tag: string = "Sampler"
  static readonly name: string = "Sampler"
  static readonly description: string | null = "Sampler"
  readonly id: number
  readonly track: Track
  readonly ports: Ports
  readonly View: ComponentType<{ dhwani: Dhwani; node: Node }> | null = null
  data: SamplerNodeProps

  constructor(id: number, track: Track, ports: Ports, data: SamplerNodeProps) {
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

  static new(storageId: number): SamplerNodeProps {
    return {
      storageId,
    } satisfies SamplerNodeProps
  }

  static portIdMul: number = 0
  static portIdOutput: number = 1
}
