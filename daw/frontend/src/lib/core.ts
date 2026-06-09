import {
  invoke,
  isTauri,
  type InvokeArgs,
  type InvokeOptions,
} from "@tauri-apps/api/core"
import { type ComponentType } from "react"
import { SimpleMixerNode } from "./nodes/simple-mixer"
import { Faker } from "./faker"
import { OscNode } from "./nodes/osc"
import { SimpleFilterNode } from "./nodes/simple-filter"
import { StereoNode } from "./nodes/stereo"
import { PianoRollNode } from "./nodes/piano-roll"

export interface AudioFileInfo {
  readonly id: number
  readonly nChannels: number
  readonly nSamplesPerCh: number
}

export interface Port {
  readonly id: number
  readonly nodeId: number
  readonly isEvent: boolean
  readonly isInput: boolean
  readonly autoConnect: boolean
  readonly name: string
}

export class Ports {
  readonly nInputs: number
  readonly nOutput: number
  readonly ports: Port[]

  constructor(ports: Port[]) {
    let nInputs = 0
    let nOutput = 0
    ports.forEach((port) => {
      if (port.isInput) {
        nInputs += 1
      } else {
        nOutput += 1
      }
    })
    this.nInputs = nInputs
    this.nOutput = nOutput
    this.ports = ports
  }

  get(id: number) {
    return this.ports.find((port) => port.id == id)
  }
}

export interface NodeInfo {
  readonly id: number
  readonly trackId: number
}

export type NodeProps = Record<string, unknown>

export interface Node {
  readonly id: number
  readonly track: Track
  readonly ports: Ports
  readonly View: ComponentType<{ dhwani: Dhwani; node: Node }> | null
  width(): number // Width for graph node
  height(): number // Height for graph node
}

export interface NodeClass<TNode extends Node, TNodeProps extends NodeProps> {
  /* tag needs to be in camleCase to work with serde deserialization */
  readonly tag: string
  readonly name: string
  readonly description: string | null
  new (id: number, track: Track, ports: Ports, data: TNodeProps): TNode
}

export interface NodeClassWithDefault<
  TNode extends Node,
  TNodeProps extends NodeProps,
> extends NodeClass<TNode, TNodeProps> {
  default(): TNodeProps
}

export interface MidiEvent {
  id: number
  note: number
  vel: number
  start: number
  end: number
}

export interface TrackData {
  start: number // Start seconds
  end: number // End seconds
}

export interface TrackInfo {
  id: number
  order: number
}

export class Track {
  id: number
  order: number
  data: TrackData
  mul: number = 1
  outputPort?: Port
  baseNodeId?: number

  constructor(id: number, order: number, data: TrackData) {
    this.id = id
    this.order = order
    this.data = data
  }

  setDuration(start: number, end: number) {
    this.data.start = start
    this.data.end = end
  }

  setBaseNode(baseNodeId: number) {
    this.baseNodeId = baseNodeId
  }
}

export interface Connection {
  source: Port
  target: Port
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const nodeClasses: NodeClassWithDefault<any, any>[] = [
  OscNode,
  SimpleMixerNode,
  SimpleFilterNode,
  StereoNode,
  // PianoRoll, // Does not have default
  // SamplerNode, // Does not have default
]

let ongoing_request: Promise<unknown> = Promise.resolve()

function sequential_invoke<T>(
  cmd: string,
  args?: InvokeArgs,
  options?: InvokeOptions
): Promise<T> {
  if (import.meta.env.DEV) {
    console.debug(`cmd: ${cmd}, req:`, args)
    console.debug(`ongoing:`, ongoing_request)
    const req = ongoing_request.then(() => invoke<T>(cmd, args, options))
    req.then((result) => {
      console.debug(`cmd: ${cmd}, rsp:`, result)
      return result
    })
    ongoing_request = req.catch((e) => {
      console.error(`cmd: ${cmd}, failed`)
      console.error(e)
    })
    return req
  } else {
    const req = ongoing_request.then(() => invoke<T>(cmd, args, options))
    ongoing_request = req.catch(() => {})
    return req
  }
}

export interface DhwaniCb {
  onReleased(): void
  onStateChanged(state: "playing" | "paused" | "stopped"): void
  tick(time: number): void
  onNodeAdded(node: Node): void
  onNodeReplaced(node: Node): void
  onNodeRemoved(id: number): void
  onTrackRemoved(id: number): void
  onTrackDurationUpdated(track: Track, start: number, end: number): void
  onOutputPortChanged(port?: Port): void
  onPortsConnected(source: Port, target: Port): void
}

export class Dhwani {
  static rootTrackId: number = 0
  static mixerNodeId: number = 1
  #id: number
  #released: boolean = false
  #isPlaying: boolean = false
  #time: number = 0
  #timeBase: number = 0
  #timeTimer: number = 0
  #cb?: DhwaniCb
  #tracks: Map<number, Track> = new Map()
  #nodes: Map<number, Node> = new Map()
  #connections: Map<number, Map<number, Connection>> = new Map()
  #mixerNode?: SimpleMixerNode
  #faker: null | Faker = isTauri() ? null : new Faker()

  constructor() {
    this.#id = Math.round(Math.random() * 65535)
  }

  id() {
    return this.#id
  }

  setCb(cb?: DhwaniCb): Dhwani {
    this.#cb = cb
    return this
  }

  getCb(): DhwaniCb | undefined {
    return this.#cb
  }

  release() {
    this.#released = true
    this.#cb?.onReleased()
  }

  released(): boolean {
    return this.#released
  }

  tracks(): Track[] {
    return [...this.#tracks.values()]
  }

  sequential_invoke<T = void>(
    cmd: string,
    args?: InvokeArgs,
    options?: InvokeOptions
  ): Promise<T> {
    if (!this.#released) {
      if (!this.#faker) {
        // Tauri
        return sequential_invoke<T>(cmd, args, options)
      } else {
        const fnName = cmd.replace(/_([0-9a-z])/g, (_, c) =>
          c.toUpperCase()
        ) as keyof Faker
        const fn = this.#faker[fnName] as (arg: unknown) => T
        if (typeof fn === "function") {
          return Promise.resolve(fn.call(this.#faker, args) as T)
        } else {
          return Promise.reject(`Invalid cmd ${cmd}`)
        }
      }
    } else {
      return Promise.reject("Already released")
    }
  }

  async defaults() {
    const track = await this.addTrack({
      start: 0,
      end: 60,
    } satisfies TrackData)
    const pianoRollNode = await this.addNode(
      track.id,
      PianoRollNode,
      PianoRollNode.new()
    )
    track.setBaseNode(pianoRollNode.id)
    const oscNode = await this.addNode(track.id, OscNode, OscNode.default())
    {
      const sourcePort = pianoRollNode.ports.get(PianoRollNode.portIdOutput)
      if (!sourcePort) {
        throw "Invalid source port"
      }
      const targetPort = oscNode.ports.get(OscNode.portIdEvents)
      if (!targetPort) {
        throw "Invalid target port"
      }
      await this.connectPorts(sourcePort, targetPort)
    }
    {
      const sourcePort = oscNode.ports.get(OscNode.portIdOutput)
      if (!sourcePort) {
        throw "Invalid source port"
      }
      const targetPort = this.#mixerNode?.ports.get(
        SimpleMixerNode.getInputPortId(track.order - 1)
      )
      if (!targetPort) {
        throw "Invalid target port"
      }
      await this.connectPorts(sourcePort, targetPort)
    }
  }

  async clear() {
    this.#tracks.clear()
    this.#nodes.clear()
    this.#connections.clear()
    this.#isPlaying = false
    await this.sequential_invoke("clear")
    // Create root track and sink node after clear
    const track = await this.addTrack({ start: 0, end: 60 })
    if (track.id !== Dhwani.rootTrackId) {
      // First operation after clear should have id = 0
      throw `Root track must have id ${Dhwani.rootTrackId}`
    }
    this.#mixerNode = (await this.addNode(track.id, SimpleMixerNode, {
      ...SimpleMixerNode.default(),
      muls: [] as number[], // Replace muls
    })) as SimpleMixerNode
    if (this.#mixerNode.id !== Dhwani.mixerNodeId) {
      throw `Mixer node in root track must have id ${Dhwani.mixerNodeId}`
    }
    const port = this.#mixerNode.ports.get(SimpleMixerNode.portIdOutput)
    if (!port) {
      return "Output port not found"
    }
    this.setOutputPort(port)
    return this.#mixerNode
  }

  #timeTimerHandle(): void {
    this.#time = Date.now() - this.#timeBase
    this.#cb?.tick(this.#time)
    this.#timeTimer = requestAnimationFrame(() => {
      this.#timeTimerHandle()
    })
  }

  #getMuls(): number[] {
    // Get track multiplier
    return [...this.#tracks.keys()]
      .filter((key) => key !== Dhwani.rootTrackId)
      .map((key) => this.#tracks.get(key) as Track)
      .sort((a, b) => a.order - b.order)
      .map((track) => track.mul)
  }

  time(): number {
    return this.#time
  }

  isPlaying(): boolean {
    return this.#isPlaying
  }

  async play(enable: boolean): Promise<void> {
    await this.sequential_invoke("play", { enable: enable })
    this.#isPlaying = enable
    if (!enable) {
      cancelAnimationFrame(this.#timeTimer)
    }
    this.#cb?.onStateChanged(enable ? "playing" : "paused")
    if (enable) {
      this.#timeBase = Date.now() - this.#time
      this.#timeTimerHandle()
    }
  }

  async seek(mode: "start" | "end" | "current", time: number) {
    this.#time =
      (await this.sequential_invoke<number>("seek", { mode, time })) * 1000
    this.#timeBase = Date.now() - this.#time
    this.#cb?.tick(this.#time)
  }

  async stop(): Promise<void> {
    if (this.#isPlaying) {
      await this.play(false)
    }
    await this.seek("start", 0)
  }

  getMixerNode(): SimpleMixerNode | undefined {
    return this.#mixerNode
  }

  getOutputPort(): Port | undefined {
    return this.#tracks.get(Dhwani.rootTrackId)?.outputPort
  }

  getTrack(id: number): Track | undefined {
    return this.#tracks.get(id)
  }

  getNode(id: number): Node | undefined {
    return this.#nodes.get(id)
  }

  getNodes(trackId?: number) {
    const nodes = [...this.#nodes.values()]
    if (typeof trackId === "number") {
      return nodes.filter((node) => node.track.id === trackId)
    }
    return nodes
  }

  getAllConnections(nodeId: number): Connection[] {
    const connections: Connection[] = []
    for (const map of this.#connections.values()) {
      for (const connection of map.values()) {
        if (
          connection.source.nodeId == nodeId ||
          connection.target.nodeId === nodeId
        ) {
          connections.push(connection)
        }
      }
    }
    return connections
  }

  getIncomingConnections(nodeId: number): Map<number, Connection> | undefined {
    return this.#connections.get(nodeId)
  }

  getIncomingConnection(
    nodeId: number,
    portId: number
  ): Connection | undefined {
    return this.getIncomingConnections(nodeId)?.get(portId)
  }

  async setTrackDuration(id: number, start: number, end: number) {
    const track = this.#tracks.get(id)
    if (!track) {
      throw "Track not found"
    }
    await this.sequential_invoke<number>("set_track_time_range", {
      id,
      start,
      end,
    })
    track.setDuration(start, end)
    this.#cb?.onTrackDurationUpdated(track, start, end)
  }

  async addTrack(data: TrackData): Promise<Track> {
    const id = await this.sequential_invoke<number>("add_track", {
      start: data.start,
      end: data.end,
    })
    const track = new Track(id, this.#tracks.size, data)
    this.#tracks.set(id, track)
    if (track.id !== 0) {
      // Not a root track
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { node, connections_invalidated } = await this.replaceNode(
        (this.#mixerNode as Node).id,
        SimpleMixerNode,
        {
          ...SimpleMixerNode.default(),
          muls: this.#getMuls(), // Replace muls
        }
      )
      this.#mixerNode = node as SimpleMixerNode
    }
    return track
  }

  async removeTrack(id: number) {
    const track = this.#tracks.get(id)
    if (!track) {
      throw "Track not found"
    }
    for (const key of this.#nodes.keys()) {
      const node = this.#nodes.get(key) as Node
      if (node.track.id === id) {
        await this.removeNode(node.id)
      }
    }
    await this.sequential_invoke<number>("remove_track", {
      id,
    })
    this.#tracks.delete(id)
    ;[...this.#tracks.values()]
      .sort((a, b) => a.order - b.order)
      .forEach((track, index) => {
        track.order = index
      })
    if (id !== 0) {
      // Not a root track
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { node, connections_invalidated } = await this.replaceNode(
        (this.#mixerNode as Node).id,
        SimpleMixerNode,
        {
          ...SimpleMixerNode.default(),
          muls: this.#getMuls(), // Replace muls
        }
      )
      this.#mixerNode = node as SimpleMixerNode
      // do not care for result.connections_invalidated
      // since already connection will be removed due to removeNode()
      for (const track of this.#tracks.values()) {
        if (track.id === Dhwani.rootTrackId) {
          continue
        }
        if (track.outputPort) {
          const target = this.#mixerNode.ports.get(
            SimpleMixerNode.getInputPortId(track.order - 1)
          )
          if (!target) {
            throw "Something went wrong"
          }
          await this.connectPorts(track.outputPort, target)
        }
      }
    }
    this.#cb?.onTrackRemoved(id)
  }

  async addNode<TNode extends Node, TNodeProps extends NodeProps>(
    trackId: number,
    cls: NodeClass<TNode, TNodeProps>,
    data: TNodeProps
  ): Promise<Node> {
    const track = this.#tracks.get(trackId)
    if (!track) {
      throw "Track not found"
    }
    const id = await this.sequential_invoke<number>("add_node", {
      trackId,
      props: { tag: cls.tag, ...data },
    })
    const ports: Port[] = await this.sequential_invoke<Port[]>("get_ports", {
      nodeId: id,
    })
    const node = new cls(id, track, new Ports(ports), data)
    this.#nodes.set(id, node)
    this.#cb?.onNodeAdded(node)
    return node
  }

  async replaceNode<TNode extends Node, TNodeProps extends NodeProps>(
    id: number,
    cls: NodeClass<TNode, TNodeProps>,
    data: TNodeProps,
    cb: boolean = true
  ): Promise<{ node: Node; connections_invalidated: boolean }> {
    const node = this.#nodes.get(id)
    if (!node) {
      throw "Node not found"
    }
    const track = node.track
    const connections_invalidated = await this.sequential_invoke<boolean>(
      "replace_node",
      {
        id: node.id,
        props: { tag: cls.tag, ...data },
      }
    )
    const ports: Port[] = await this.sequential_invoke<Port[]>("get_ports", {
      nodeId: id,
    })
    const newNode = new cls(id, track, new Ports(ports), data)
    this.#nodes.set(id, newNode)
    if (connections_invalidated) {
      // Remove connections linked to the node
      this.#removeConnections(newNode)
    }
    if (cb) {
      this.#cb?.onNodeReplaced(node)
    }
    return { node: newNode, connections_invalidated }
  }

  async removeNode(id: number) {
    const node = this.#nodes.get(id)
    if (!node) {
      throw "Node not found"
    }
    await this.sequential_invoke("remove_node", { id: node.id })
    // Remove connections linked to the node
    this.#removeConnections(node)
    this.#nodes.delete(id)
    this.#cb?.onNodeRemoved(node.id)
  }

  async setOutputPort(port?: Port) {
    await this.sequential_invoke<number>("set_output_port", {
      port: port ? [port.nodeId, port.id] : undefined,
    })
    const rootTrack = this.#tracks.get(Dhwani.rootTrackId)
    if (rootTrack) {
      rootTrack.outputPort = port
    }
    this.#cb?.onOutputPortChanged(port)
  }

  async connectPorts(source: Port, target: Port) {
    await this.sequential_invoke<number>("connect_ports", {
      source: [source.nodeId, source.id],
      target: [target.nodeId, target.id],
    })
    if (!this.#connections.has(target.nodeId)) {
      this.#connections.set(target.nodeId, new Map())
    }
    const map = this.#connections.get(target.nodeId)
    if (!map) {
      throw "Something went wrong"
    }
    map.set(target.id, { source, target })
    if (target.nodeId === Dhwani.mixerNodeId) {
      const node = this.#nodes.get(source.nodeId)
      if (!node) {
        throw "Something went wrong"
      }
      node.track.outputPort = node.ports.get(source.id)
    }
    this.#cb?.onPortsConnected(source, target)
  }

  async unlinkPort(target: Port) {
    await this.sequential_invoke<number>("unlink_port", {
      target: [target.nodeId, target.id],
    })
    const map = this.#connections.get(target.nodeId)
    if (!map) {
      throw "Something went wrong"
    }
    const connection = map.get(target.id)
    if (!connection || !map.delete(target.id)) {
      throw "Something went wrong"
    }
    if (target.nodeId === Dhwani.mixerNodeId) {
      const node = this.#nodes.get(connection.source.nodeId)
      if (!node) {
        throw "Something went wrong"
      }
      node.track.outputPort = undefined
    }
  }

  async storeFile(path: string) {
    const data = await this.sequential_invoke<AudioFileInfo>("store_file", {
      path,
    })
    return data
  }

  #removeConnections(node: Node) {
    // Remove output port
    for (const track of this.#tracks.values()) {
      if (track.outputPort && track.outputPort.nodeId === node.id) {
        track.outputPort = undefined
      }
    }
    for (const key of [...this.#connections.keys()]) {
      const map = this.#connections.get(key) as Map<number, Connection>
      for (const connection of [...map.values()]) {
        if (
          connection.source.nodeId === node.id ||
          connection.target.nodeId === node.id
        ) {
          map.delete(connection.target.id)
        }
      }
      if (map.size == 0) {
        this.#connections.delete(key)
      }
    }
  }
}
