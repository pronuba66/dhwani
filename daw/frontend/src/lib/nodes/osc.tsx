import { Osc } from "@/components/nodes/Osc"
import {
  Dhwani,
  Ports,
  type Node,
  type NodeProps,
  type Track,
} from "@/lib/core"
import type { ComponentType } from "react"

export type OscMode = "Sine" | "Square" | "Saw"

export type OscNodeProps = {
  nChannels?: number
  mode: OscMode
  freq: number
  mul: number
} & NodeProps

export class OscNode implements Node {
  static readonly tag: string = "Osc"
  static readonly name: string = "Osc"
  static readonly description: string | null = "Oscillator"
  readonly id: number
  readonly track: Track
  readonly ports: Ports
  readonly View: ComponentType<{ dhwani: Dhwani; node: Node }> | null =
    Osc as ComponentType<{
      dhwani: Dhwani
      node: Node
    }>
  data: OscNodeProps

  constructor(id: number, track: Track, ports: Ports, data: OscNodeProps) {
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

  static default(): OscNodeProps {
    return {
      nChannels: 2,
      mode: "Sine",
      freq: 220,
      mul: 1,
    } satisfies OscNodeProps
  }

  static portIdEvents: number = 0
  static portIdDuty: number = 1
  static portIdPhase: number = 2
  static portIdFreq: number = 3
  static portIdMul: number = 4
  static portIdOutput: number = 5
}
