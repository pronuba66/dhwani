import { useState, useCallback, useEffect, useMemo } from "react"
import { X, PencilRuler } from "lucide-react"
import {
  ReactFlow,
  Background,
  applyNodeChanges,
  applyEdgeChanges,
} from "@xyflow/react"
import type {
  NodeChange,
  EdgeChange,
  Node as GraphNode,
  Edge as GraphEdge,
  Connection,
} from "@xyflow/react"
import "@xyflow/react/dist/style.css"
import {
  Dhwani,
  nodeClasses,
  type TrackInfo,
  type Node as AudioNode,
} from "@/lib/core"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { CustomNode } from "@/components/CustomNode"
import { SinkNode } from "@/lib/nodes/sink"
import { SimpleMixerNode } from "@/lib/nodes/simple-mixer"
import { trackName } from "@/lib/utils"
import { OutputNode } from "@/lib/nodes/output"
import { PianoRoll } from "./PianoRoll"
import { PianoRollNode } from "@/lib/nodes/piano-roll"

interface NodeEditorProps {
  dhwani: Dhwani
  onClose: () => void
  trackInfo?: TrackInfo
  zoomLevel: number
  tempo: number
  sigNum: number
  sigDen: number
}

export function NodeEditor({
  dhwani,
  onClose,
  trackInfo,
  zoomLevel,
  tempo,
  sigNum,
  sigDen,
}: NodeEditorProps) {
  const nodeTypes = useMemo(() => ({ custom: CustomNode }), [])
  const [nodes, setNodes] = useState<GraphNode[]>([])
  const [edges, setEdges] = useState<GraphEdge[]>([])

  const track =
    trackInfo !== undefined ? dhwani.getTrack(trackInfo.id) : undefined
  const isPianoRoll =
    track !== undefined && track.baseNodeId !== undefined
      ? dhwani.getNode(track.baseNodeId) instanceof PianoRollNode
      : false

  const onNodesChange = useCallback((changes: NodeChange[]) => {
    setNodes((nds) => applyNodeChanges(changes, nds))
  }, [])

  const onEdgesChange = useCallback((changes: EdgeChange[]) => {
    // console.log(changes)
    setEdges((eds) => applyEdgeChanges(changes, eds))
  }, [])

  const onConnect = useCallback(
    (connection: Connection) => {
      if (!trackInfo) {
        return
      }
      const track = dhwani.getTrack(trackInfo.id)
      if (!track) {
        return
      }
      if (connection.target === "output") {
        const port = dhwani
          .getNode(parseInt(connection.source))
          ?.ports.get(parseInt(connection.sourceHandle as string))
        if (!port) {
          console.error("Invalid port")
          return
        }
        dhwani.setOutputPort(port)
      } else if (connection.target === "sink") {
        const sourcePort = dhwani
          .getNode(parseInt(connection.source))
          ?.ports.get(parseInt(connection.sourceHandle as string))
        if (!sourcePort) {
          console.error("Invalid source port")
          return
        }
        const mixerNode = dhwani.getMixerNode()
        const targetPort = mixerNode?.ports.get(
          SimpleMixerNode.getInputPortId(track.order - 1)
        )
        if (!targetPort) {
          console.error("Invalid target port")
          return
        }
        dhwani.connectPorts(sourcePort, targetPort)
      } else {
        const sourcePort = dhwani
          .getNode(parseInt(connection.source))
          ?.ports.get(parseInt(connection.sourceHandle as string))
        if (!sourcePort) {
          console.error("Invalid source port")
          return
        }
        const targetPort = dhwani
          .getNode(parseInt(connection.target))
          ?.ports.get(parseInt(connection.targetHandle as string))
        if (!targetPort) {
          console.error("Invalid target port")
          return
        }
        dhwani.connectPorts(sourcePort, targetPort)
      }
    },
    [dhwani, trackInfo]
  )

  useEffect(() => {
    const graphNodes: GraphNode[] = []
    const graphEdges: GraphEdge[] = []
    if (!trackInfo) {
      return
    }
    const track = dhwani.getTrack(trackInfo.id)
    if (!track) {
      return
    }
    let left = 32
    dhwani.getNodes(track.id).forEach((node) => {
      const currentLeft = left
      left += node.width() + 32
      const lastNode = nodes.find((n) => n.id === `${node.id}`)
      if (lastNode && (lastNode.data as { node: AudioNode }).node === node) {
        graphNodes.push(lastNode)
      } else {
        let deletable = true
        if (node.id === Dhwani.mixerNodeId) {
          deletable = false
        } else if (node.id === track.baseNodeId) {
          deletable = false
        }
        graphNodes.push({
          id: `${node.id}`,
          type: "custom",
          data: { dhwani, node },
          position: lastNode ? lastNode.position : { x: currentLeft, y: 32 },
          deletable,
        } satisfies GraphNode)
      }
      node.ports.ports.forEach((port) => {
        if (port.isInput) {
          const connection = dhwani.getIncomingConnection(port.nodeId, port.id)
          if (connection) {
            const source = connection.source
            const target = connection.target
            const id = `${source.nodeId}#${source.id}-${target.nodeId}#${target.id}`
            const lastEdge = edges.find((e) => e.id === id)
            if (!lastEdge) {
              const edge = {
                id,
                source: `${source.nodeId}`,
                sourceHandle: `${source.id}`,
                target: `${target.nodeId}`,
                targetHandle: `${target.id}`,
                animated: true,
                style: { stroke: "#ff6666", strokeWidth: 2 },
              } satisfies GraphEdge
              graphEdges.push(edge)
            } else {
              graphEdges.push(lastEdge)
            }
          }
        }
      })
    })
    /** Special node */
    if (track.id === Dhwani.rootTrackId) {
      const node = new OutputNode()
      const lastNode = nodes.find((n) => n.id === "output")
      if (lastNode) {
        graphNodes.push(lastNode)
      } else {
        graphNodes.push({
          id: "output",
          type: "custom",
          data: { dhwani, node },
          position: { x: left, y: 32 },
          deletable: false,
        })
      }
      const outputPort = dhwani.getOutputPort()
      if (outputPort) {
        const id = `${outputPort.nodeId}#${outputPort.id}-output#0`
        const lastEdge = edges.find((e) => e.id === id)
        if (!lastEdge) {
          const edge = {
            id,
            source: `${outputPort.nodeId}`,
            sourceHandle: `${outputPort.id}`,
            target: "output",
            targetHandle: "0",
            animated: true,
            style: { stroke: "#ff6666", strokeWidth: 2 },
          } satisfies GraphEdge
          graphEdges.push(edge)
        } else {
          graphEdges.push(lastEdge)
        }
      }
    } else {
      const node = new SinkNode(track)
      const lastNode = nodes.find(
        (n) =>
          n.id === "sink" &&
          (n.data as { node: SinkNode }).node.track.id === trackInfo.id
      )
      if (lastNode) {
        graphNodes.push(lastNode)
      } else {
        graphNodes.push({
          id: "sink",
          type: "custom",
          data: { dhwani, node },
          position: { x: left, y: 32 },
          deletable: false,
        })
      }
      // Get mixer connections
      const connections = dhwani.getIncomingConnections(Dhwani.mixerNodeId)
      if (connections) {
        for (const connection of connections.values()) {
          if (dhwani.getNode(connection.source.nodeId)) {
            const source = connection.source
            const id = `${source.nodeId}#${source.id}-sink`
            const lastEdge = edges.find((e) => e.id === id)
            if (!lastEdge) {
              const edge = {
                id,
                source: `${source.nodeId}`,
                sourceHandle: `${source.id}`,
                target: "sink",
                targetHandle: "0",
                animated: true,
                style: { stroke: "#ff6666", strokeWidth: 2 },
              } satisfies GraphEdge
              graphEdges.push(edge)
            } else {
              graphEdges.push(lastEdge)
            }
          }
        }
      }
    }
    /** Special node ends */
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setNodes(graphNodes)
    setEdges(graphEdges)
  }, [dhwani, trackInfo])

  return (
    <div
      className="absolute top-0 z-node-editor flex h-full w-full flex-col gap-4 bg-black/20 p-4 supports-backdrop-filter:backdrop-blur-sm"
      style={{ display: trackInfo === undefined ? "none" : "" }}
    >
      {/* Header */}
      <div className="flex flex-none shrink flex-row items-center gap-4">
        <div className="font-heading text-lg font-semibold tracking-wider text-foreground uppercase">
          <PencilRuler size={14} className="inline text-primary" /> Node Editor
        </div>
        <div className="flex-1">
          {trackInfo?.id !== undefined ? trackName(trackInfo.order) : ""}
        </div>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline">Add Node</Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            {nodeClasses.map((clz) => (
              <DropdownMenuItem
                key={clz.tag}
                onClick={() => {
                  if (trackInfo) {
                    dhwani.addNode(trackInfo.id, clz, clz.default())
                  }
                }}
              >
                {clz.name}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
        <Button className="hover:text-primary" onClick={() => onClose()}>
          <X size={20} />
        </Button>
      </div>
      {/* Piano roll */}
      {track !== undefined && isPianoRoll ? (
        <div className="flex-1 overflow-auto">
          <PianoRoll
            dhwani={dhwani}
            nodeId={track.baseNodeId as number}
            zoomLevel={zoomLevel}
            tempo={tempo}
            sigNum={sigNum}
            sigDen={sigDen}
          />
        </div>
      ) : (
        <></>
      )}
      {/* Graph */}
      <div className="flex-1">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          edgesFocusable={true}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          defaultViewport={{ x: 0, y: 0, zoom: 1 }}
          fitView={false}
          snapToGrid={true}
          snapGrid={[32, 32]}
          className="dark"
        >
          <Background color="var(--ring)" gap={32} size={1} />
        </ReactFlow>
      </div>
    </div>
  )
}
