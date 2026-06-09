import type { Dhwani, MidiEvent } from "@/lib/core"
import { PianoRollNode } from "@/lib/nodes/piano-roll"
import { useCallback, useEffect, useRef, useState } from "react"

const keyWidth = 64
const keyHeight = 16
const totalNotes = 128
const noteNames = [
  "C", // 0
  "C#", // 1
  "D", // 2
  "D#", // 3
  "E", // 4
  "F", // 5
  "F#", // 6
  "G", // 7
  "G#", // 8
  "A", // 9
  "A#", // 10
  "B", // 11
] as const
const sharpNotes: number[] = [1, 3, 6, 8, 10]
const keyInRenderingOrder = Array.from(
  { length: totalNotes },
  (_, i) => i
).sort((a, b) => {
  const aIsSharp = sharpNotes.includes(a)
  const bIsSharp = sharpNotes.includes(b)
  if (aIsSharp === bIsSharp) {
    return 0
  } else if (aIsSharp) {
    return -1
  } else {
    return 1
  }
})
const height = totalNotes * keyHeight

interface DragData {
  isDragging: boolean
  hasMoved: boolean
  timestamp: number
  startPos: number
  start: number
  end?: number
  event: MidiEvent
}

function KeyComponent({ note }: { note: number }) {
  const top = height - (note + 1) * keyHeight
  const isSharp = sharpNotes.includes(note % 12)
  const octave = Math.floor(note / 12) - 1
  return (
    <div
      className="absolute flex justify-end border align-middle"
      style={{
        left: 0,
        top,
        width: keyWidth,
        height: keyHeight,
        borderColor: !isSharp ? "var(--background)" : "var(--foreground)",
        backgroundColor: !isSharp ? "var(--foreground)" : "var(--background)",
        color: !isSharp ? "var(--background)" : "var(--foreground)",
      }}
    >
      <div className="px-2 text-xs">
        {noteNames[note % 12]}
        {octave}
      </div>
    </div>
  )
}

function NoteComponent({
  event,
  zoomLevel,
}: {
  event: MidiEvent
  zoomLevel: number
}) {
  return (
    <div
      className="absolute border border-secondary bg-primary"
      style={{
        top: height - (event.note + 1) * keyHeight,
        left: event.start * zoomLevel,
        width: (event.end - event.start) * zoomLevel,
        height: keyHeight,
      }}
    />
  )
}

const newId = (events: MidiEvent[]): number => {
  let id = 0
  for (const event of events) {
    if (id < event.id) {
      id = event.id
    }
  }
  return id + 1
}

export function PianoRoll({
  dhwani,
  nodeId,
  zoomLevel,
  tempo,
  sigNum,
  sigDen,
}: {
  dhwani: Dhwani
  nodeId: number
  zoomLevel: number
  tempo: number
  sigNum: number
  sigDen: number
}) {
  const node = dhwani.getNode(nodeId) as PianoRollNode | undefined
  if (!node) {
    throw "Invalid node id"
  }
  const secondsPerBeat = (4 * (60 / tempo)) / sigDen
  const gridSpan = secondsPerBeat * sigNum * zoomLevel
  const width = (node.track.data.end - node.track.data.start) * zoomLevel
  const [gridImage, setGridImage] = useState<string | undefined>(undefined)
  const dragData = useRef<DragData>({
    isDragging: false,
    hasMoved: false,
    timestamp: 0,
    startPos: 0,
    start: 0,
    end: undefined,
    event: {
      id: 0,
      note: 0,
      vel: 0,
      start: 0,
      end: 0,
    },
  })
  const [events, setEvents] = useState<MidiEvent[]>(node.data.events)
  const [currentEvent, setCurrentEvent] = useState<MidiEvent | undefined>(
    undefined
  )
  const scrollViewRef = useRef<HTMLDivElement>(null)
  const pianoRollViewRef = useRef<HTMLDivElement>(null)

  const handleMouseDown = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (e.button !== 0) {
        // Only allow left mouse button
        return
      }
      if (!pianoRollViewRef.current) {
        return
      }
      const data = dragData.current
      e.preventDefault()
      if (data.timestamp !== undefined && Date.now() - data.timestamp <= 250) {
        // Double click, delete note
        const newEvents = [...events]
        const index = newEvents.findIndex((event) => event.id === data.event.id)
        if (index >= 0) {
          newEvents.splice(index, 1)
        }
        data.timestamp = 0
        setEvents(newEvents)
        return
      }
      const rect = pianoRollViewRef.current.getBoundingClientRect()
      const note = Math.floor((height - (e.clientY - rect.top)) / keyHeight)
      const div = gridSpan / sigNum
      const start =
        (Math.floor((e.clientX - rect.left) / div) * div) / zoomLevel
      const event = events.find(
        (event) =>
          event.note === note && start >= event.start && start < event.end
      )
      const current: DragData = {
        isDragging: true,
        hasMoved: false,
        timestamp: 0,
        startPos: e.clientX,
        start: event !== undefined ? event.start : start,
        end: event !== undefined ? event.end : undefined,
        event:
          event === undefined
            ? {
                id: newId(events),
                note,
                vel: 1,
                start,
                end: start + div / zoomLevel,
              }
            : event,
      }
      dragData.current = current
      setCurrentEvent({ ...current.event })
    },
    [events, gridSpan, sigNum, zoomLevel]
  )

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      const data = dragData.current
      if (!data.isDragging) {
        return
      }
      e.preventDefault()
      const div = gridSpan / sigNum
      let delta =
        (Math.round((e.clientX - data.startPos) / div) * div) / zoomLevel
      if (data.end !== undefined) {
        // Is move
        const start = data.start + delta
        if (start < 0) {
          return
        }
        data.hasMoved = true
        const end = data.end + delta
        data.event.start = start
        data.event.end = end
      } else {
        // Is new
        if (delta <= div / zoomLevel) {
          delta = div / zoomLevel
        }
        data.event.end = data.event.start + delta
      }
      setCurrentEvent({ ...data.event })
    },
    [gridSpan, sigNum, zoomLevel]
  )

  const handleMouseUp = useCallback(
    (e: MouseEvent) => {
      e.preventDefault()
      const data = dragData.current
      if (!data.isDragging) {
        return
      }
      data.isDragging = false
      data.timestamp = Date.now()
      setCurrentEvent(undefined)
      const newEvents = [...events]
      if (data.end !== undefined) {
        // move
        if (!data.hasMoved) {
          return
        }
        const index = newEvents.findIndex((event) => event.id === data.event.id)
        if (index >= 0) {
          newEvents[index] = data.event
        }
      } else {
        // new
        newEvents.push(data.event)
      }
      setEvents(newEvents)
      dhwani.replaceNode(node.id, PianoRollNode, {
        ...node.data,
        events: newEvents,
      })
    },
    [dhwani, node, events]
  )

  useEffect(() => {
    const width = gridSpan
    // Border 2 px
    const height = keyHeight
    const strokeWidth = 0.25
    let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" preserveAspectRatio="none" viewBox="0 0 ${width} ${height}">`
    svg += `<rect x="0" y="0" width="${width}" height="${height}" stroke="white" stroke-width="${strokeWidth}" />`
    svg += `</svg>`
    const url = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setGridImage(url)
  }, [gridSpan, zoomLevel])

  useEffect(() => {
    document.addEventListener("mousemove", handleMouseMove)
    document.addEventListener("mouseup", handleMouseUp)
    return () => {
      document.removeEventListener("mousemove", handleMouseMove)
      document.removeEventListener("mouseup", handleMouseUp)
    }
  }, [handleMouseMove, handleMouseUp])

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setEvents(node.data.events)
  }, [node])

  useEffect(() => {
    if (!scrollViewRef.current) {
      throw "Getting scroll view failed"
    }
    scrollViewRef.current.scrollTop = height / 2
  }, [])

  return (
    <div className="h-full w-full bg-background/40 p-8">
      <div ref={scrollViewRef} className="relative h-full w-full overflow-auto">
        <div
          className="relative"
          style={{
            paddingLeft: keyWidth,
          }}
        >
          {/* Reverse to have auto z-index */}
          {keyInRenderingOrder.map((i) => (
            <KeyComponent key={i} note={i} />
          ))}
          <div
            ref={pianoRollViewRef}
            className="relative"
            style={{
              height: height,
              width: width,
              backgroundSize: `${gridSpan}px ${keyHeight}px`,
              backgroundImage: gridImage ? `url('${gridImage}')` : "",
            }}
            onMouseDown={(e) => handleMouseDown(e)}
          >
            {events.map((event) => (
              <NoteComponent
                key={event.id}
                event={event}
                zoomLevel={zoomLevel}
              />
            ))}
            {currentEvent !== undefined ? (
              <NoteComponent
                key={currentEvent.id}
                event={currentEvent}
                zoomLevel={zoomLevel}
              />
            ) : (
              <></>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
