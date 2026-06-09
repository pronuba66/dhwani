import { useCallback, useEffect, useState } from "react"
import { TopBar } from "@/components/TopBar"
import { MainPanel } from "@/components/MainPanel"
import { Dhwani, type Track, type TrackInfo } from "@/lib/core"
import { Loader } from "@/components/ui/loader"
import { getCurrentWebview } from "@tauri-apps/api/webview"
import type { UnlistenFn } from "@tauri-apps/api/event"
import { isTauri } from "@tauri-apps/api/core"
import { SamplerNode } from "./lib/nodes/sampler"
import { SimpleMixerNode } from "./lib/nodes/simple-mixer"
import { PianoRollNode, type PianoRollNodeProps } from "./lib/nodes/piano-roll"

// For debug
declare global {
  interface Window {
    dhwani: Dhwani
  }
}

const initialDhwani = new Dhwani()

function App() {
  const [dhwani, setDhwani] = useState(initialDhwani)
  const [isLoading, setIsLoading] = useState<boolean>(true)
  const [isPlaying, setIsPlaying] = useState<boolean>(false)
  const [openHelp, setOpenHelp] = useState<boolean>(false)
  const [isSnapEnabled, setIsSnapEnabled] = useState(true)
  // Tempo and Signature State
  const [time, setTime] = useState<number>(0)
  const [tempo, setTempo] = useState<number>(120)
  const [sigNum, setSigNum] = useState<number>(4)
  const [sigDen, setSigDen] = useState<number>(4)
  // Zoom State
  const [zoomLevel, setZoomLevel] = useState<number>(16) // 1s = 16px
  // Track info
  const [selectedTrackInfo, setSelectedTrackInfo] = useState<
    TrackInfo | undefined
  >(undefined)
  // Tracks
  const [tracks, setTracks] = useState<Track[]>([])

  const handleTrackSelected = useCallback(
    (trackId?: number) => {
      const track = trackId !== undefined ? dhwani.getTrack(trackId) : undefined
      setSelectedTrackInfo(
        track !== undefined
          ? ({
              id: track.id,
              order: track.order,
            } satisfies TrackInfo)
          : undefined
      )
    },
    [dhwani]
  )

  const handleUpdateTrackDuration = useCallback(
    (trackId: number, start: number, end: number, finalize: boolean) => {
      if (finalize) {
        dhwani.setTrackDuration(trackId, start, end).then(() => {
          setTracks(dhwani.tracks())
        })
      } else {
        setTracks((prev) =>
          prev.map((t) => {
            if (t.id === trackId) {
              t.setDuration(start, end)
            }
            return t
          })
        )
      }
    },
    [dhwani, setTracks]
  )

  const handleToggleSnap = () => {
    setIsSnapEnabled(!isSnapEnabled)
  }

  const handleAddTrack = useCallback(() => {
    dhwani.addTrack({ start: 10, end: 30 }).then((track) => {
      setTracks(dhwani.tracks())
      const data: PianoRollNodeProps = {
        events: [],
      }
      dhwani.addNode(track.id, PianoRollNode, data).then((node) => {
        track.setBaseNode(node.id)
      })
    })
  }, [dhwani, setTracks])

  useEffect(() => {
    if (dhwani.released()) {
      console.debug("Dhwani released, creating new...")
      // This will re trigger this useEffect
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setDhwani(new Dhwani())
      return
    }
    if (import.meta.env.DEV) {
      window.dhwani = dhwani
    }
    setIsLoading(true)
    dhwani.setCb({
      onReleased: () => {
        console.info("Released")
      },
      onStateChanged: (state) => {
        if (state === "playing") {
          setIsPlaying(true)
        } else if (state === "paused") {
          setIsPlaying(false)
        }
      },
      tick: (time) => {
        setTime(time)
      },
      onNodeAdded: (node) => {
        console.info(`Node with id #${node.id} added`)
        setTracks(dhwani.tracks())
        setSelectedTrackInfo((prev) => (prev ? { ...prev } : undefined))
      },
      onNodeReplaced: (node) => {
        console.info(`Node with id #${node.id} replaced`)
        setTracks(dhwani.tracks())
        setSelectedTrackInfo((prev) => (prev ? { ...prev } : undefined))
      },
      onNodeRemoved: (id) => {
        console.info(`Node with id #${id} removed`)
        setTracks(dhwani.tracks())
      },
      onTrackRemoved: (id) => {
        console.info(`Track with id ${id} removed`)
        setTracks(dhwani.tracks())
      },
      onTrackDurationUpdated: (track, start, end) => {
        console.info(
          `Time updated from [${start}, ${end}] for track with id ${track.id}`
        )
        setTracks(dhwani.tracks())
      },
      onOutputPortChanged: (port) => {
        console.log("Output port changed to ", port)
        setSelectedTrackInfo((prev) => (prev ? { ...prev } : undefined))
      },
      onPortsConnected: (source, target) => {
        console.log("Ports connected, source: ", source, "target: ", target)
        setSelectedTrackInfo((prev) => (prev ? { ...prev } : undefined))
      },
    })
    dhwani
      .clear()
      .then(() => dhwani.defaults())
      .then(() => {
        setTracks(dhwani.tracks())
        setIsLoading(false)
      })
      .catch((e) => {
        console.error(e)
      })

    let unmount = false
    let dragDropEventUnlistenFn: UnlistenFn | undefined = undefined
    if (isTauri()) {
      getCurrentWebview()
        .onDragDropEvent((event) => {
          if (event.payload.type === "drop") {
            if (event.payload.paths.length > 0) {
              setIsLoading(true)
              dhwani
                .storeFile(event.payload.paths[0])
                .then(async (info) => {
                  const track = await dhwani.addTrack({
                    start: 0,
                    end: info.nSamplesPerCh / 44100,
                  })
                  const node = await dhwani.addNode(
                    track.id,
                    SamplerNode,
                    SamplerNode.new(info.id)
                  )
                  track.setBaseNode(node.id)
                  const source = node.ports.get(SamplerNode.portIdOutput)
                  if (!source) {
                    throw "Invalid source port"
                  }
                  const target = dhwani
                    .getMixerNode()
                    ?.ports.get(SimpleMixerNode.getInputPortId(track.order - 1))
                  if (!target) {
                    throw "Invalid target port"
                  }
                  await dhwani.connectPorts(source, target)
                })
                .catch((e) => {
                  console.error("Failed to read file")
                  console.error(e)
                })
                .finally(() => {
                  setIsLoading(false)
                })
            }
          } else if (event.payload.type === "over") {
            setIsLoading(true)
          } else if (event.payload.type === "leave") {
            setIsLoading(false)
          }
        })
        .then((unlistenFn) => {
          dragDropEventUnlistenFn = unlistenFn
          if (unmount) {
            unlistenFn()
          }
        })
    }
    return () => {
      unmount = true
      if (dragDropEventUnlistenFn) {
        dragDropEventUnlistenFn()
      }
      dhwani.release()
    }
  }, [dhwani])

  useEffect(() => {
    const listenerKeydown = (e: KeyboardEvent) => {
      if (e.key === " ") {
        e.preventDefault()
      }
    }
    const listenerKeyup = (e: KeyboardEvent) => {
      if (e.key === " ") {
        dhwani.play(!dhwani.isPlaying())
      } else if (e.key === "=" || e.key === "+") {
        let z = Math.round(Math.log2(zoomLevel))
        z += 1
        if (z > 8) {
          z = 8
        }
        z = Math.pow(2, z)
        setZoomLevel(z)
      } else if (e.key == "-" || e.key == "_") {
        let z = Math.round(Math.log2(zoomLevel))
        z -= 1
        if (z < -2) {
          z = -2
        }
        z = Math.pow(2, z)
        setZoomLevel(z)
      } else if (e.key === "?") {
        setOpenHelp(!openHelp)
      } else if (e.key === "Escape") {
        if (selectedTrackInfo) {
          setSelectedTrackInfo(undefined)
        }
      }
    }
    document.addEventListener("keydown", listenerKeydown)
    document.addEventListener("keyup", listenerKeyup)
    return () => {
      document.removeEventListener("keydown", listenerKeydown)
      document.removeEventListener("keyup", listenerKeyup)
    }
  }, [dhwani, isPlaying, zoomLevel, openHelp, selectedTrackInfo])

  return (
    <div className="relative h-screen min-h-window-min-height w-screen min-w-window-min-width overflow-x-hidden overflow-y-auto bg-background select-none">
      <TopBar
        className="fixed top-0 left-0 z-topbar h-topbar-height w-full"
        dhwani={dhwani}
        tempo={tempo}
        onTempoChange={setTempo}
        sigNum={sigNum}
        onSigNumChange={setSigNum}
        sigDen={sigDen}
        onSigDenChange={setSigDen}
        isSnapEnabled={isSnapEnabled}
        onToggleSnap={handleToggleSnap}
        onAddTrack={handleAddTrack}
        openHelp={openHelp}
        setOpenHelp={setOpenHelp}
      />
      <MainPanel
        className="h-full w-full pt-topbar-height"
        dhwani={dhwani}
        time={time}
        tempo={tempo}
        sigNum={sigNum}
        sigDen={sigDen}
        tracks={tracks}
        isSnapEnabled={isSnapEnabled}
        zoomLevel={zoomLevel}
        selectedTrackInfo={selectedTrackInfo}
        onTrackSelected={handleTrackSelected}
        onUpdateTrackDuration={handleUpdateTrackDuration}
      />
      <Loader enable={isLoading} />
    </div>
  )
}

export default App
