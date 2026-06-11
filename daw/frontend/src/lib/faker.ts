/**
 * Faker is a fake data store for emulating frontend on a browser
 */

import type { NodeInfo, Port } from "./core"
import { DelayNode, type DelayNodeProps } from "./nodes/delay"
import { OscNode, type OscNodeProps } from "./nodes/osc"
import { PianoRollNode, type PianoRollNodeProps } from "./nodes/piano-roll"
import {
  SimpleMixerNode,
  type SimpleMixerNodeProps,
} from "./nodes/simple-mixer"

interface FakerPort {
  port: Port
}

interface FakerNode {
  node: NodeInfo
  ports: FakerPort[]
}

interface FakerTrack {
  id: number
  nodes: FakerNode[]
}

export class Faker {
  #fakeIdCounter: number = 0
  tracks: FakerTrack[] = []
  playing: boolean = false

  constructor() {
    this.clear()
  }

  #fakeId() {
    const fakeId = this.#fakeIdCounter
    this.#fakeIdCounter += 1
    return fakeId
  }

  #getTrack(trackId: number) {
    return this.tracks.find((track) => track.id == trackId)
  }

  #getNode(nodeId: number) {
    let node = undefined
    for (const track of this.tracks) {
      node = track.nodes.find((node) => node.node.id === nodeId)
      if (node) {
        break
      }
    }
    return node
  }

  // #getPort(nodeId: number, portId: number) {
  //   const node = this.#getNode(nodeId)
  //   if (!node) {
  //     return undefined
  //   }
  //   return node.ports.find((port) => port.port.id == portId)
  // }

  clear() {
    this.#fakeIdCounter = 0
    this.tracks = []
  }

  play(args: { enable: boolean }) {
    this.playing = args.enable
  }

  seek(args: { mode: "start" | "end" | "current"; time: number }): number {
    const mode = args.mode
    const time = args.time
    switch (mode) {
      case "start": {
        return time
      }
      case "end": {
        return Math.round(Math.random() * 60)
      }
      case "current": {
        return Math.round(Math.random() * 60)
      }
    }
  }

  getTracks(): number[] {
    return this.tracks.map((track) => track.id)
  }

  getNodes(args: { trackId: number }): NodeInfo[] {
    const track = this.#getTrack(args.trackId)
    if (!track) {
      throw "Track not found"
    }
    return track.nodes.map((node) => node.node)
  }

  getPorts(args: { nodeId: number }): Port[] {
    const node = this.#getNode(args.nodeId)
    if (!node) {
      throw "Node not found"
    }
    return node.ports.map((port) => port.port)
  }

  addTrack(/* args: { start: number; end: number } */) {
    const id = this.#fakeId()
    this.tracks.push({
      id,
      nodes: [],
    })
    return id
  }

  removeTrack(args: { id: number }) {
    const id = args.id
    const index = this.tracks.findIndex((track) => track.id === id)
    if (index < 0) {
      throw "Track not found"
    }
    this.tracks.splice(index, 1)
  }

  setTrackTimeRange(args: { id: number; start: number; end: number }) {
    const id = args.id
    // const start = args.start;
    // const end = args.end;
    const track = this.#getTrack(id)
    if (!track) {
      throw "Track not found"
    }
  }

  addNode(args: { trackId: number; props: { tag: string } }) {
    const trackId = args.trackId
    const props = args.props
    const track = this.#getTrack(trackId)
    if (!track) {
      throw "Track not found"
    }
    const id = this.#fakeId()
    const node2 = {
      id: id,
      trackId,
    }
    const node: FakerNode = {
      node: node2,
      ports: [],
    }
    track.nodes.push(node)
    let ports: Port[]
    switch (props.tag) {
      case SimpleMixerNode.tag: {
        ports = SimpleMixerNodeFaker.ports(
          id,
          props as unknown as SimpleMixerNodeProps
        )
        break
      }
      case PianoRollNode.tag: {
        ports = PianoRollNodeFaker.ports(
          id,
          props as unknown as PianoRollNodeProps
        )
        break
      }
      case OscNode.tag: {
        ports = OscNodeFaker.ports(id, props as unknown as OscNodeProps)
        break
      }
      case DelayNode.tag: {
        ports = DelayNodeFaker.ports(id, props as unknown as DelayNodeProps)
        break
      }
      default: {
        throw "Unknown node type"
      }
    }
    node.ports = ports.map(
      (port) =>
        ({
          port: port satisfies Port,
        }) as FakerPort
    )
    return id
  }

  replaceNode(args: { id: number; props: { tag: string } }) {
    const id = args.id
    const props = args.props
    const node = this.#getNode(id)
    if (!node) {
      throw "Node not found"
    }
    let ports: Port[]
    switch (props.tag) {
      case SimpleMixerNode.tag: {
        ports = SimpleMixerNodeFaker.ports(
          id,
          props as unknown as SimpleMixerNodeProps
        )
        break
      }
      case PianoRollNode.tag: {
        ports = PianoRollNodeFaker.ports(
          id,
          props as unknown as PianoRollNodeProps
        )
        break
      }
      case OscNode.tag: {
        ports = OscNodeFaker.ports(id, props as unknown as OscNodeProps)
        break
      }
      case DelayNode.tag: {
        ports = DelayNodeFaker.ports(id, props as unknown as DelayNodeProps)
        break
      }
      default: {
        throw "Unknown node type"
      }
    }
    node.ports = ports.map(
      (port) =>
        ({
          port: port satisfies Port,
        }) as FakerPort
    )
    return false
  }

  removeNode(args: { id: number }) {
    const id = args.id
    const node = this.#getNode(id)
    if (!node) {
      throw "Node not found"
    }
    const track = this.#getTrack(node.node.trackId)
    if (!track) {
      throw "Something went wrong"
    }
    const index = track.nodes.indexOf(node)
    if (index < 0) {
      throw "Something went wrong"
    }
    track.nodes.splice(index, 1)
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  setOutputPort(args: { port?: [number, number] }) {
    // this.#outputPort = args.port
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  connectPorts(args: { source: [number, number]; target: [number, number] }) {
    // this.#outputPort = args.port
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  unlinkPort(args: { target: [number, number] }) {
    // this.#outputPort = args.port
  }
}

class SimpleMixerNodeFaker {
  static ports(nodeId: number, props: SimpleMixerNodeProps): Port[] {
    const ports: Port[] = [
      {
        id: SimpleMixerNode.portIdOutput,
        nodeId,
        isEvent: false,
        isInput: false,
        autoConnect: true,
        name: "Output",
      },
    ]
    Array.from({ length: props.muls.length }, (_, i) => {
      ports.push({
        id: SimpleMixerNode.portIdOutput + 1 + i,
        nodeId,
        isEvent: false,
        isInput: true,
        autoConnect: true,
        name: "Input",
      })
    })
    return ports
  }
}

class PianoRollNodeFaker {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  static ports(nodeId: number, props: PianoRollNodeProps): Port[] {
    const ports: Port[] = [
      {
        id: PianoRollNode.portIdOutput,
        nodeId,
        isEvent: true,
        isInput: false,
        autoConnect: true,
        name: "Output",
      },
    ]
    return ports
  }
}

class OscNodeFaker {
  static ports(nodeId: number, props: OscNodeProps): Port[] {
    const ports: Port[] = [
      {
        id: OscNode.portIdEvents,
        nodeId,
        isEvent: true,
        isInput: true,
        autoConnect: true,
        name: "Events",
      },
      {
        id: OscNode.portIdPhase,
        nodeId,
        isEvent: false,
        isInput: true,
        autoConnect: false,
        name: "Phase",
      },
      {
        id: OscNode.portIdFreq,
        nodeId,
        isEvent: false,
        isInput: true,
        autoConnect: false,
        name: "Frequency",
      },
      {
        id: OscNode.portIdMul,
        nodeId,
        isEvent: false,
        isInput: true,
        autoConnect: false,
        name: "Multiplier",
      },
      {
        id: OscNode.portIdOutput,
        nodeId,
        isEvent: false,
        isInput: false,
        autoConnect: true,
        name: "Output",
      },
    ]
    if (props.mode !== "Sine") {
      ports.push({
        id: OscNode.portIdDuty,
        nodeId,
        isEvent: false,
        isInput: true,
        autoConnect: false,
        name: "Duty Cycle",
      })
    }
    return ports
  }
}

class DelayNodeFaker {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  static ports(nodeId: number, props: DelayNodeProps): Port[] {
    const ports: Port[] = [
      {
        id: DelayNode.portIdInput,
        nodeId,
        isEvent: false,
        isInput: true,
        autoConnect: true,
        name: "Input",
      },
      {
        id: DelayNode.portIdOutput,
        nodeId,
        isEvent: false,
        isInput: false,
        autoConnect: true,
        name: "Output",
      },
    ]
    return ports
  }
}
