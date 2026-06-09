import type { Dhwani } from "@/lib/core"
import { OscNode, type OscMode } from "@/lib/nodes/osc"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "../ui/dropdown-menu"
import { Button } from "../ui/button"
import { useCallback } from "react"
import { Input } from "../ui/input"

export function Osc({ dhwani, node }: { dhwani: Dhwani; node: OscNode }) {
  const modes: OscMode[] = ["Sine", "Square", "Saw"]
  const onModeSelect = useCallback(
    (mode: OscMode) => {
      dhwani.replaceNode(node.id, OscNode, { ...node.data, mode })
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
      dhwani.replaceNode(node.id, OscNode, { ...node.data, freq })
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
