import { Dhwani, Ports, type Node, type Track } from "@/lib/core"
import type { ComponentType } from "react"

export class SinkNode implements Node {
  // static readonly tag: string = "Sink" // Special node
  static readonly name: string = "Sink"
  static readonly description: string | null = "Routed to mixer in master track"
  readonly id: number = -1
  readonly track: Track
  readonly ports: Ports = new Ports([
    {
      id: 0,
      nodeId: -1,
      isInput: true,
      isEvent: false,
      autoConnect: true,
      name: "Input",
    },
  ])
  readonly View: ComponentType<{ dhwani: Dhwani; node: Node }> | null = null

  constructor(track: Track) {
    this.track = track
  }

  width() {
    return 256
  }

  height() {
    return 0
  }
}
