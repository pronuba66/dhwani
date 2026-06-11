import type { Dhwani } from "@/lib/core"
import { DelayNode } from "@/lib/nodes/delay"
import { Input } from "../ui/input"
import { useCallback } from "react"

export function Delay({ dhwani, node }: { dhwani: Dhwani; node: DelayNode }) {
  const onDelayChanged = useCallback(
    (delayStr: string) => {
      let delay
      try {
        delay = parseFloat(delayStr)
      } catch {
        return
      }
      if (!Number.isFinite(delay) && delay < 0) {
        return
      }
      dhwani.replaceNode(node.id, DelayNode, { ...node.data, delay })
    },
    [dhwani, node]
  )
  const onMulChanged = useCallback(
    (mulStr: string) => {
      let mul
      try {
        mul = parseFloat(mulStr)
      } catch {
        return
      }
      if (!Number.isFinite(mul)) {
        return
      }
      dhwani.replaceNode(node.id, DelayNode, { ...node.data, mul })
    },
    [dhwani, node]
  )
  return (
    <>
      <Input
        placeholder="Delay"
        value={node.data.delay}
        inputMode="numeric"
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
          onDelayChanged(e.target.value)
        }}
      />
      <Input
        placeholder="Multiplier"
        value={node.data.mul}
        inputMode="numeric"
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
          onMulChanged(e.target.value)
        }}
      />
    </>
  )
}
