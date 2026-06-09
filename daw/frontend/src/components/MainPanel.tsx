import { Dhwani, type Track, type TrackInfo } from "@/lib/core"
import { trackName } from "@/lib/utils"
import { TrashIcon } from "lucide-react"
import { useCallback, useEffect, useRef, useState } from "react"
import { NodeEditor } from "./NodeEditor"
import { SamplerNode } from "@/lib/nodes/sampler"

interface TimelineProps {
  className?: string
  dhwani: Dhwani
  time: number
  tempo: number
  sigNum: number
  sigDen: number
  tracks: Track[]
  isSnapEnabled: boolean
  zoomLevel: number
  selectedTrackInfo?: TrackInfo
  onTrackSelected: (trackId?: number) => void
  onUpdateTrackDuration: (
    trackId: number,
    start: number,
    end: number,
    finalize: boolean
  ) => void
}

interface DragData {
  startTime: number
  isResizeHandle: boolean
  trackId: number
  startPos: number
  start: number
  end: number
  moved: boolean
}

function TrackComponenet({
  dhwani,
  track,
  marginTopAuto,
  zoomLevel,
  handleMouseDown,
}: {
  dhwani: Dhwani
  track: Track
  marginTopAuto: boolean
  zoomLevel: number
  handleMouseDown: (
    e: React.MouseEvent<HTMLDivElement>,
    isResizeHandle: boolean
  ) => void
}) {
  return (
    <div className="shrink" style={{ marginTop: marginTopAuto ? "auto" : "" }}>
      <div className="pointer-events-none absolute left-0 z-track-header flex h-track-height w-track-header-width flex-col justify-between border-y bg-secondary py-2 shadow-md transition-colors">
        <div className="flex w-full items-center justify-between px-2">
          <span className="truncate px-2 text-sm font-semibold text-foreground">
            {trackName(track.order)}
          </span>
          <div className="flex space-x-1 px-2">
            <button className="pointer-events-auto h-6 w-6 cursor-pointer rounded bg-background/50 text-[10px] font-bold text-neutral-300 shadow-sm transition-colors hover:bg-primary hover:text-foreground">
              M
            </button>
            <button className="pointer-events-auto h-6 w-6 cursor-pointer rounded bg-background/50 text-[10px] font-bold text-neutral-300 shadow-sm transition-colors hover:bg-primary hover:text-foreground">
              S
            </button>
            {track.id !== 0 ? (
              <button
                onClick={() => {
                  dhwani.removeTrack(track.id)
                }}
                className="pointer-events-auto flex h-6 w-6 cursor-pointer items-center justify-center rounded bg-background/50 text-[10px] font-bold text-neutral-300 shadow-sm transition-colors hover:bg-primary hover:text-foreground"
              >
                <TrashIcon size={12} className="text-destructive" />
              </button>
            ) : (
              <></>
            )}
          </div>
        </div>
        <div className="flex h-max px-4">
          <div className="w-full rounded-full bg-background/50 shadow-inner">
            {/* Replaced Green with Gold */}
            <div className="h-1 w-10 rounded-full bg-amber-500/80 shadow-[0_0_5px_rgba(245,158,11,0.5)]" />
          </div>
        </div>
      </div>
      <div
        data-track-id={track.id}
        onMouseDown={(e) => handleMouseDown(e, false)}
        className={`relative flex h-track-height rounded-md border border-chart-3 bg-muted/40`}
        style={{
          cursor: track.id !== 0 ? "move" : "pointer",
          width: (track.data.end - track.data.start) * zoomLevel,
          marginLeft: track.data.start * zoomLevel,
        }}
      >
        <div className="truncate p-2 font-mono text-xs font-bold text-foreground/80 select-none">
          Clip: {(track.data.end - track.data.start).toFixed(1)}s
        </div>
        {/* Resize Handle */}
        {!(track.baseNode instanceof SamplerNode) ? (
          <div
            data-track-id={track.id}
            onMouseDown={(e) => handleMouseDown(e, true)}
            className="resize-handle absolute top-0 right-0 bottom-0 w-2 cursor-ew-resize rounded-r-md bg-transparent hover:bg-foreground/20"
          />
        ) : (
          <></>
        )}
      </div>
    </div>
  )
}

export function MainPanel({
  className,
  dhwani,
  time,
  tempo,
  sigNum,
  sigDen,
  tracks,
  isSnapEnabled,
  zoomLevel,
  selectedTrackInfo,
  onTrackSelected,
  onUpdateTrackDuration,
}: TimelineProps) {
  const secondsPerBeat = (4 * (60 / tempo)) / sigDen
  const gridSpan = secondsPerBeat * zoomLevel * sigNum
  const trackRulerRef = useRef<HTMLDivElement>(null)
  const [rulerImage, setRulerImage] = useState<string | undefined>(undefined)
  const [gridImage, setGridImage] = useState<string | undefined>(undefined)
  const dragData = useRef<DragData>(null)

  const isDark = document.documentElement.classList.contains("dark")

  // const beatLength = 64 * zoomLevel
  let minDuration = 0
  for (const track of tracks) {
    minDuration = Math.max(minDuration, track.data.end)
  }
  minDuration += 30 / zoomLevel // 30 pixel always gap

  const handleRulerClick = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (!e.currentTarget) {
        return
      }
      console.log(e.clientX)
    },
    []
  )

  const handleCloseDrawer = useCallback(() => {
    onTrackSelected(undefined)
  }, [onTrackSelected])

  const handleMouseDown = useCallback(
    (e: React.MouseEvent<HTMLDivElement>, isResizeHandle: boolean) => {
      if (e.button !== 0) {
        // Only allow left mouse button
        return
      }
      if (!e.currentTarget) {
        return
      }
      if (!e.currentTarget.dataset) {
        return
      }
      if (dragData.current) {
        return
      }
      const idStr = e.currentTarget.dataset.trackId
      if (!idStr) {
        return
      }
      const id = parseInt(idStr)
      const track = tracks.find((item) => item.id === id)
      if (!track || (isResizeHandle && track.baseNode instanceof SamplerNode)) {
        return
      }
      dragData.current = {
        startTime: Date.now(),
        isResizeHandle,
        trackId: track.id,
        startPos: e.clientX,
        start: track.data.start,
        end: track.data.end,
        moved: false,
      }
    },
    [tracks]
  )

  const updateTrackDuration = useCallback(
    (e: MouseEvent, finalize: boolean) => {
      const data = dragData.current
      if (!data) {
        return
      }
      if (!data.moved && data.startPos != e.clientX) {
        data.moved = true
      }
      let start = data.start
      if (!data.isResizeHandle) {
        if (data.trackId === Dhwani.rootTrackId) {
          // Root track should not be moved
          return
        }
        start += (e.clientX - data.startPos) / zoomLevel
      }
      let end = data.end + (e.clientX - data.startPos) / zoomLevel
      if (isSnapEnabled) {
        start = Math.round(start / secondsPerBeat) * secondsPerBeat
        end = Math.round(end / secondsPerBeat) * secondsPerBeat
      }
      if (end <= start) {
        return
      }
      onUpdateTrackDuration(data.trackId, start, end, finalize)
    },
    [isSnapEnabled, onUpdateTrackDuration, zoomLevel, secondsPerBeat]
  )

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      updateTrackDuration(e, false)
    },
    [updateTrackDuration]
  )

  const handleMouseUp = useCallback(
    (e: MouseEvent) => {
      if (!dragData.current) {
        return
      }
      const data = dragData.current
      const delta = Date.now() - data.startTime
      if (delta < 250 && !dragData.current.moved) {
        onTrackSelected(data.trackId)
      } else {
        updateTrackDuration(e, true)
      }

      dragData.current = null
    },
    [onTrackSelected, updateTrackDuration]
  )

  useEffect(() => {
    document.addEventListener("mousemove", handleMouseMove)
    document.addEventListener("mouseup", handleMouseUp)
    return () => {
      document.removeEventListener("mousemove", handleMouseMove)
      document.removeEventListener("mouseup", handleMouseUp)
    }
  }, [tempo, onUpdateTrackDuration, handleMouseMove, handleMouseUp])

  useEffect(() => {
    const width = gridSpan
    const spacing = width / sigDen
    {
      // Border 2 px
      const height = 32
      const color = isDark ? "rgba(255,255,255,0.05)" : "rgba(0,0,0,0.05)"
      let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" preserveAspectRatio="none" viewBox="0 0 ${width} ${height}">`
      for (let i = 0; i < sigDen; ++i) {
        svg += `<rect x="${i * spacing}" y="0" width="1" height="${height}" fill="${color}" />`
      }
      svg += `</svg>`
      const url = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setGridImage(url)
    }
    if (trackRulerRef && trackRulerRef.current) {
      // Border 1 px
      const height =
        trackRulerRef.current.clientHeight > 0
          ? trackRulerRef.current.clientHeight + 1
          : 32
      const primarycolor = isDark ? "rgba(255,255,255,0.5)" : "rgba(0,0,0,0.5)"
      const secondarycolor = isDark
        ? "rgba(255,255,255,0.25)"
        : "rgba(0,0,0,0.25)"
      let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" preserveAspectRatio="none" viewBox="0 0 ${width} ${height}">`
      svg += `<rect x="0" y="0" width="1" height="${height}" fill="${primarycolor}" />`
      for (let i = 1; i < sigDen; ++i) {
        svg += `<rect x="${i * spacing}" y="${height / 2}" width="1" height="${height / 2}" fill="${secondarycolor}" />`
      }
      svg += `</svg>`
      const url = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setRulerImage(url)
    }
  }, [dhwani, gridSpan, sigDen, isDark, tracks])

  return (
    <>
      {/* Main panel container */}
      <div className={className}>
        <div className="relative h-full w-full">
          <div className="flex min-h-full flex-col overflow-auto pl-track-header-width">
            {/* Track header mask */}
            <div className="absolute left-0 z-track-header-mask h-full w-track-header-width bg-secondary" />
            {/* Ruler */}
            <div
              className="relative h-track-ruler-height min-w-full shrink border-b border-b-foreground/20 bg-background bg-repeat-x"
              ref={trackRulerRef}
              onClick={handleRulerClick}
              style={{
                width: `${minDuration * zoomLevel}px`,
                backgroundSize: `${gridSpan}px 100%`,
                backgroundImage: rulerImage ? `url('${rulerImage}')` : "",
              }}
            >
              <div
                className="absolute top-0 left-0 h-track-ruler-height w-px bg-orange-600"
                style={{ left: `${(time / 1000) * zoomLevel}px` }}
              />
            </div>
            {/* Tracks */}
            <div
              className="flex min-w-full flex-1 flex-col bg-repeat"
              style={{
                backgroundSize: `${gridSpan}px var(--spacing-track-height)`,
                backgroundImage: gridImage ? `url('${gridImage}')` : "",
              }}
            >
              {[...tracks].reverse().map((track, i) => (
                <TrackComponenet
                  key={track.id}
                  dhwani={dhwani}
                  track={track}
                  marginTopAuto={i == tracks.length - 1}
                  zoomLevel={zoomLevel}
                  handleMouseDown={handleMouseDown}
                />
              ))}
            </div>
            {/* Tracks ends */}
          </div>
          <NodeEditor
            dhwani={dhwani}
            onClose={handleCloseDrawer}
            trackInfo={selectedTrackInfo}
            zoomLevel={zoomLevel}
            tempo={tempo}
            sigNum={sigNum}
            sigDen={sigDen}
          />
        </div>
      </div>
    </>
  )
}
