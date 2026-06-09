import type { Dhwani } from "@/lib/core"
import {
  SimpleFilterNode,
  type SimpleFilterMode,
} from "@/lib/nodes/simple-filter"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "../ui/dropdown-menu"
import { Button } from "../ui/button"
import { Input } from "../ui/input"
import { useCallback } from "react"

export function SimpleFilter({
  dhwani,
  node,
}: {
  dhwani: Dhwani
  node: SimpleFilterNode
}) {
  const modes: SimpleFilterMode[] = ["LPF", "HPF", "BPF", "BSF"]
  const onModeSelect = useCallback(
    (mode: SimpleFilterMode) => {
      dhwani.replaceNode(node.id, SimpleFilterNode, { ...node.data, mode })
    },
    [dhwani, node]
  )
  const onFreqChanged = useCallback(
    (freqStr: string) => {
      let freq
      try {
        freq = parseFloat(freqStr)
      } catch {
        return
      }
      if (!Number.isFinite(freq)) {
        return
      }
      dhwani.replaceNode(node.id, SimpleFilterNode, { ...node.data, freq })
    },
    [dhwani, node]
  )
  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline">Mode: {node.data.mode}</Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent>
          {modes.map((mode) => (
            <DropdownMenuItem
              key={mode}
              onClick={() => {
                onModeSelect(mode)
              }}
            >
              {mode}
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
      <Input
        placeholder="Frequency"
        value={node.data.freq}
        inputMode="numeric"
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
          onFreqChanged(e.target.value)
        }}
      />
    </>
  )
}
