import {
  Dhwani,
  Ports,
  type MidiEvent,
  type Node,
  type NodeProps,
  type Track,
} from "@/lib/core"
import type { ComponentType } from "react"

export interface PianoRollNodeProps extends NodeProps {
  events: MidiEvent[]
}

export class PianoRollNode implements Node {
  static readonly tag: string = "PianoRoll"
  static readonly name: string = "Piano Roll"
  static readonly description: string | null = "Piano roll"
  readonly id: number
  readonly track: Track
  readonly ports: Ports
  readonly View: ComponentType<{ dhwani: Dhwani; node: Node }> | null = null
  data: PianoRollNodeProps

  constructor(
    id: number,
    track: Track,
    ports: Ports,
    data: PianoRollNodeProps
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
    return 0
  }

  static new(): PianoRollNodeProps {
    return {
      events: [],
    } satisfies PianoRollNodeProps
  }

  static portIdOutput: number = 0
}
