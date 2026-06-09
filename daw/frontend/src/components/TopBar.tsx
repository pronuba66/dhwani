import {
  PlayIcon,
  PauseIcon,
  SquareIcon,
  SkipBackIcon,
  SkipForwardIcon,
  MagnetIcon,
  CircleQuestionMarkIcon,
  PlusIcon,
  XIcon,
} from "lucide-react"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import type { Dhwani } from "@/lib/core"

interface TopBarProps {
  className?: string
  dhwani: Dhwani
  tempo: number
  onTempoChange: (val: number) => void
  sigNum: number
  onSigNumChange: (val: number) => void
  sigDen: number
  onSigDenChange: (val: number) => void
  onToggleSnap: () => void
  onAddTrack: () => void
  isSnapEnabled: boolean
  openHelp: boolean
  setOpenHelp: (open: boolean) => void
}

function ShortcutItem({
  shortcut,
  label,
}: {
  shortcut: string
  label: string
}) {
  return (
    <div className="flex items-center">
      <span className="mr-2 cursor-pointer rounded border-border bg-input p-2 text-xs font-bold text-muted-foreground transition-colors hover:bg-ring hover:text-primary">
        {shortcut}
      </span>
      <span>{label}</span>
    </div>
  )
}

export function TopBar({
  className,
  dhwani,
  tempo,
  sigNum,
  sigDen,
  isSnapEnabled,
  onToggleSnap,
  onAddTrack,
  openHelp,
  setOpenHelp,
}: TopBarProps) {
  const time = dhwani.time() / 1000
  const secondsPerBeat = (4 * (60 / tempo)) / sigDen
  const beatStr = (Math.floor(time / secondsPerBeat) % sigDen)
    .toString()
    .padStart(2, "0")
  const barStr = Math.floor(time / (sigNum * secondsPerBeat))
    .toString()
    .padStart(2, "0")

  const msStr = (Math.round(time * 1000) % 1000).toString().padStart(3, "0")
  const seconds = Math.floor(time)
  const secondsStr = (seconds % 60).toString().padStart(2, "0")
  const minutes = Math.floor(seconds / 60)
  const minutesStr = (minutes % 60).toString().padStart(2, "0")
  const hours = Math.floor(minutes / 60)
  const hoursStr = hours.toString()

  return (
    <div className={className}>
      <div className="flex h-full w-full items-center justify-start border-b border-border bg-muted text-muted-foreground shadow-sm">
        {/* Logo */}
        <div className="mx-2 flex h-full">
          <img
            className="h-full w-auto object-contain"
            src="/logo.svg"
            alt="Dhwani"
          />
        </div>
        {/* Time/Step */}
        <div className="mx-2 flex flex-col">
          <span className="text-xl leading-none font-bold tracking-tight text-chart-2">
            {barStr}:{beatStr}
          </span>
          <span className="min-w-32 leading-none font-medium tracking-wider">
            {hoursStr}:{minutesStr}:{secondsStr}.{msStr}
          </span>
        </div>
        {/* Controls */}
        <div className="mx-2 flex space-x-1">
          <button
            onClick={() => {
              dhwani.seek("current", -2)
            }}
            className="cursor-pointer p-2.5 transition-all hover:text-foreground active:scale-95"
          >
            <SkipBackIcon size={20} className="fill-current" />
          </button>
          <button
            onClick={() => {
              dhwani.stop()
            }}
            className="cursor-pointer p-2.5 transition-all hover:text-foreground active:scale-95"
          >
            <SquareIcon size={20} className="fill-current" />
          </button>
          <button
            onClick={() => {
              dhwani.play(!dhwani.isPlaying())
            }}
            className="cursor-pointer rounded-full bg-chart-2 p-2 text-muted shadow-sm transition-all hover:text-muted-foreground active:scale-95"
          >
            {dhwani.isPlaying() ? (
              <PauseIcon size={24} className="fill-current" />
            ) : (
              <PlayIcon size={24} className="fill-current" />
            )}
          </button>
          <button
            onClick={() => {
              dhwani.seek("current", 2)
            }}
            className="cursor-pointer p-2.5 transition-all hover:text-foreground active:scale-95"
          >
            <SkipForwardIcon size={20} className="fill-current" />
          </button>
        </div>
        {/* Tempo */}
        <div className="mx-2 ml-auto flex overflow-hidden rounded-lg border border-border bg-accent py-1 shadow-inner">
          <div className="flex items-center space-x-1 border-r px-2">
            <span className="font-semibold text-chart-2">{tempo}</span>
            <span className="text-xs font-bold text-muted-foreground">BPM</span>
          </div>
          <div className="flex space-x-1 px-2 text-center font-semibold text-chart-2 outline-none">
            <span>{sigNum}</span>
            <span className="font-bold text-muted-foreground">/</span>
            <span>{sigDen}</span>
          </div>
        </div>
        {/* Snap*/}
        <div className="mx-2 flex items-center">
          <button
            title="Snap"
            onClick={onToggleSnap}
            className={`flex cursor-pointer items-center space-x-2 rounded p-2 transition-colors ${isSnapEnabled ? "bg-chart-2 text-muted" : "bg-input text-chart-2 hover:bg-muted-foreground hover:text-background"}`}
          >
            <MagnetIcon size={16} />
          </button>
        </div>
        {/* Add Track */}
        <div className="mx-2 flex items-center">
          <button
            title="Add Track"
            onClick={onAddTrack}
            className="flex cursor-pointer items-center space-x-2 rounded border-border bg-input p-2 text-xs font-bold text-chart-2 hover:bg-muted-foreground hover:text-background"
          >
            <PlusIcon size={16} />
            <span>Track</span>
          </button>
        </div>
        <div className="mx-2 flex items-center">
          <Dialog open={openHelp}>
            <DialogTrigger
              onClick={() => setOpenHelp(true)}
              title="Shortcuts"
              className="flex cursor-pointer items-center rounded border-border bg-input p-2 text-xs font-bold text-chart-2 hover:bg-muted-foreground hover:text-background"
            >
              <CircleQuestionMarkIcon size={16} />
            </DialogTrigger>
            <DialogContent showCloseButton={false}>
              <DialogHeader>
                <DialogTitle>Shortcuts</DialogTitle>
                <DialogDescription>Keyboard shortcuts</DialogDescription>
              </DialogHeader>
              <div className="flex flex-col gap-2">
                <ShortcutItem shortcut="?" label="Show/hide help" />
                <ShortcutItem shortcut="[space]" label="Play/pause" />
                <ShortcutItem shortcut="+" label="Zoom in timeline" />
                <ShortcutItem shortcut="-" label="Zoom out timeline" />
              </div>
              <Button
                variant="ghost"
                className="absolute top-5 right-5 bg-secondary"
                size="icon-sm"
                onClick={() => {
                  setOpenHelp(false)
                }}
              >
                <XIcon />
                <span className="sr-only">Close</span>
              </Button>
            </DialogContent>
          </Dialog>
        </div>
      </div>
    </div>
  )
}
