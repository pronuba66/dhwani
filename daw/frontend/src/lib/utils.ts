import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function cloneInstance<T>(obj: T): T {
  // WARN: private memebers are not cloned. Do not use for instances having
  // private members
  return Object.assign(Object.create(Object.getPrototypeOf(obj)), obj)
}

export function trackName(order: number): string {
  return order === 0 ? "Master" : `Track #${order}`
}
