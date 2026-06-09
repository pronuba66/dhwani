import clsx from "clsx"
import { LoaderCircleIcon } from "lucide-react"

export function Loader({
  enable,
  className,
  ...props
}: React.ComponentProps<"div"> & { enable: boolean }) {
  return (
    <div
      className={clsx(
        enable ? "" : "hidden",
        "absolute top-0 left-0 z-loader h-full w-full items-center justify-center bg-background",
        className
      )}
      {...props}
    >
      <LoaderCircleIcon
        size={48}
        className="absolute top-[50%] left-[50%] translate-[-50%] animate-spin"
      />
    </div>
  )
}
