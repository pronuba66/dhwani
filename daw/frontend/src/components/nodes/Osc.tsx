import type { Dhwani } from "@/lib/core"
import { OscNode, type OscMode } from "@/lib/nodes/osc"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "../ui/dropdown-menu"
import { Button } from "../ui/button"
import { useCallback, useState } from "react"
import { Input } from "../ui/input"

export function Osc({ dhwani, node }: { dhwani: Dhwani; node: OscNode }) {
  const modes: OscMode[] = ["Sine", "Square", "Saw"]
  const [freq, setFreq] = useState<string>(node.data.freq.toString())
  const [mul, setMul] = useState<string>(node.data.mul.toString())
  const onModeSelect = useCallback(
    (mode: OscMode) => {
      dhwani.replaceNode(node.id, OscNode, { ...node.data, mode })
    },
    [dhwani, node]
  )
  const onFreqChanged = useCallback(
    (freqStr: string) => {
      setFreq(freqStr)
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
  const onMulChanged = useCallback(
    (mulStr: string) => {
      setMul(mulStr)
      let mul
      try {
        mul = parseFloat(mulStr)
      } catch {
        return
      }
      if (!Number.isFinite(mul)) {
        return
      }
      dhwani.replaceNode(node.id, OscNode, { ...node.data, mul })
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
        value={freq}
        inputMode="decimal"
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
          onFreqChanged(e.target.value)
        }}
      />
      <Input
        placeholder="Multiplier"
        value={mul}
        inputMode="decimal"
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
          onMulChanged(e.target.value)
        }}
      />
    </>
  )
}
