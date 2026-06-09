import { Dhwani } from "@/lib/core"
import { SimpleMixerNode } from "@/lib/nodes/simple-mixer"
import { useCallback } from "react"
import { Slider } from "../ui/slider"
import { Button } from "../ui/button"
import { MinusIcon, PlusIcon } from "lucide-react"

export function SimpleMixer({
  dhwani,
  node,
}: {
  dhwani: Dhwani
  node: SimpleMixerNode
}) {
  const ports = node.ports.ports.filter((port) => port.isInput == true)
  const onVolumeChange = useCallback(
    (index: number, value: number) => {
      const mul = value <= -60 ? 0 : Math.pow(10, value / 20)
      const muls = [...node.data.muls]
      muls[index] = mul
      console.log(mul)
      // Disable callback to stop constant re-renders
      const cb = dhwani.getCb()
      dhwani.setCb(undefined)
      dhwani
        .replaceNode(node.id, SimpleMixerNode, { ...node.data, muls })
        .then(() => {
          dhwani.setCb(cb)
        })
    },
    [dhwani, node]
  )
  const setNChannels = useCallback(
    (count: number) => {
      console.log(count)
      const muls = [...node.data.muls]
      if (count < muls.length) {
        muls.splice(count)
      } else {
        muls.push(1)
      }
      dhwani.replaceNode(node.id, SimpleMixerNode, { ...node.data, muls })
    },
    [dhwani, node]
  )
  const mulToDb = (mul: number) => (mul === 0 ? -60 : 20 * Math.log10(mul))
  return (
    <>
      <div
        className="flex"
        style={{ display: node.id === Dhwani.mixerNodeId ? "none" : "" }}
      >
        <Button
          disabled={node.data.muls.length <= 1}
          onClick={() => {
            setNChannels(node.data.muls.length - 1)
          }}
        >
          <MinusIcon size={16} />
        </Button>
        <Button
          onClick={() => {
            setNChannels(node.data.muls.length + 1)
          }}
        >
          <PlusIcon size={16} />
        </Button>
      </div>
      <div className="pt-2">
        {ports.map((port, i) => (
          <div className="mt-8" key={`${port.nodeId}-${port.id}`}>
            <Slider
              defaultValue={[mulToDb(node.data.muls[i])]}
              min={-60}
              max={12}
              step={1}
              className="w-full"
              onValueChange={(value) => onVolumeChange(i, value[0])}
            />
          </div>
        ))}
      </div>
    </>
  )
}
