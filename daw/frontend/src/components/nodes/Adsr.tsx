import type { Dhwani } from "@/lib/core"
import { AdsrNode } from "@/lib/nodes/adsr"
import { Input } from "../ui/input"
import { useCallback } from "react"

export function Adsr({ dhwani, node }: { dhwani: Dhwani; node: AdsrNode }) {
  const onAttackChanged = useCallback(
    (aStr: string) => {
      let a
      try {
        a = parseFloat(aStr)
      } catch {
        return
      }
      if (!Number.isFinite(a) && a < 0) {
        return
      }
      dhwani.replaceNode(node.id, AdsrNode, { ...node.data, a })
    },
    [dhwani, node]
  )
  const onDecayChanged = useCallback(
    (dStr: string) => {
      let d
      try {
        d = parseFloat(dStr)
      } catch {
        return
      }
      if (!Number.isFinite(d) && d < 0) {
        return
      }
      dhwani.replaceNode(node.id, AdsrNode, { ...node.data, d })
    },
    [dhwani, node]
  )
  const onSustainChanged = useCallback(
    (sStr: string) => {
      let s
      try {
        s = parseFloat(sStr)
      } catch {
        return
      }
      if (!Number.isFinite(s) && s < 0) {
        return
      }
      dhwani.replaceNode(node.id, AdsrNode, { ...node.data, s })
    },
    [dhwani, node]
  )
  const onReleaseChanged = useCallback(
    (rStr: string) => {
      let r
      try {
        r = parseFloat(rStr)
      } catch {
        return
      }
      if (!Number.isFinite(r) && r < 0) {
        return
      }
      dhwani.replaceNode(node.id, AdsrNode, { ...node.data, r })
    },
    [dhwani, node]
  )
  return (
    <>
      <Input
        placeholder="Attack"
        value={node.data.a}
        inputMode="numeric"
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
          onAttackChanged(e.target.value)
        }}
      />
      <Input
        placeholder="Decay"
        value={node.data.d}
        inputMode="numeric"
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
          onDecayChanged(e.target.value)
        }}
      />
      <Input
        placeholder="Sustain"
        value={node.data.s}
        inputMode="numeric"
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
          onSustainChanged(e.target.value)
        }}
      />
      <Input
        placeholder="Release"
        value={node.data.r}
        inputMode="numeric"
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
          onReleaseChanged(e.target.value)
        }}
      />
    </>
  )
}
