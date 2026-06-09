import { Dhwani, Ports, type Node, type Track } from "@/lib/core"
import type { ComponentType } from "react"

export class OutputNode implements Node {
  // static readonly tag: string = "Output" // Special node
  static readonly name: string = "Output"
  static readonly description: string | null = "Audio Output"
  readonly id: number = -1
  readonly track: Track = null as unknown as Track
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

  width() {
    return 256
  }

  height() {
    return 0
  }
}
