import {
  Card,
  CardDescription,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Dhwani, type Node, type NodeClass } from "@/lib/core"
import {
  Handle,
  Position,
  type HandleType,
  type NodeProps,
} from "@xyflow/react"

export function CustomNode(props: NodeProps) {
  const data = props.data as unknown as { dhwani: Dhwani; node: Node }
  const name = (
    Object.getPrototypeOf(data.node).constructor as NodeClass<Node, NodeProps>
  ).name
  const description = (
    Object.getPrototypeOf(data.node).constructor as NodeClass<Node, NodeProps>
  ).description
  let iInput = 0
  let iOutput = 0
  const ports: {
    id: number
    top: number
    type: HandleType
    position: Position
    name: string
  }[] = []
  for (const item of data.node.ports.ports) {
    // Do not show ports of root mixer node
    if (item.isInput) {
      if (data.node.id === Dhwani.mixerNodeId) {
        continue
      }
      const top = ((iInput + 1) / (data.node.ports.nInputs + 1)) * 100
      ports.push({
        id: item.id,
        top,
        type: "target" as HandleType,
        position: Position.Left,
        name: item.name,
      })
      iInput += 1
    } else {
      const top = ((iOutput + 1) / (data.node.ports.nOutput + 1)) * 100
      ports.push({
        id: item.id,
        top,
        type: "source" as HandleType,
        position: Position.Right,
        name: item.name,
      })
      iOutput += 1
    }
  }
  return (
    <Card className="p-8" style={{ width: data.node.width(), gap: 0 }}>
      <CardHeader style={{ height: `${64}px` }}>
        <CardTitle>
          {data.node.id >= 0 ? (
            <div className="absolute top-0 right-0 p-2 text-xs">
              #{data.node.id}
            </div>
          ) : (
            ""
          )}
          {name}
        </CardTitle>
        {description ? <CardDescription>{description}</CardDescription> : <></>}
      </CardHeader>
      <CardContent
        className="nodrag cursor-auto"
        style={{ minHeight: data.node.height() }}
      >
        {data.node.View ? (
          <data.node.View dhwani={data.dhwani} node={data.node} />
        ) : (
          <></>
        )}
      </CardContent>
      {ports.map((item) => (
        <Handle
          key={item.id}
          style={{ top: `${item.top}%` }}
          id={`${item.id}`}
          type={item.type}
          position={item.position}
          title={item.name}
          isConnectable={true}
        >
          <div
            className="absolute -top-2 h-4 w-8 overflow-hidden text-ellipsis"
            style={{
              left:
                item.position == Position.Left
                  ? "calc(var(--spacing) * 2)"
                  : "",
              right:
                item.position == Position.Right
                  ? "calc(var(--spacing) * 2)"
                  : "",
            }}
          >
            {item.name}
          </div>
        </Handle>
      ))}
    </Card>
  )
}
