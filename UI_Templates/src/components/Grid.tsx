import { useEffect, useMemo, useRef, useState } from "react"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
    DEFAULT_CELL_COLOUR,
    distanceToColour,
    roiBorder,
    type Point as LidarPoint,
    type RangePoint,
} from "@/lib/Modules/LIDAR"

const MIN_ANGLE = -90
const MAX_ANGLE = 90

export const W = MAX_ANGLE - MIN_ANGLE + 1
export const H = MAX_ANGLE - MIN_ANGLE + 1

export type Point = LidarPoint

type GridCoordinate = {
    column: number
    row: number
}

type HoveredCell = GridCoordinate & {
    pivot: Point
    left: number
    top: number
    containerWidth: number
}

type SelectedPoint = {
    point: Point
    selected: boolean
}

type GridProps = {
    setRoi: (min: Point, max: Point) => void
    move_point: (point: Point) => void
    min: Point
    max: Point
    map: RangePoint[]
}

function pivotToGrid(point: Point): GridCoordinate {
    return {
        column: point.x - MIN_ANGLE,
        row: MAX_ANGLE - point.y,
    }
}

function gridToPivot(column: number, row: number): Point {
    return {
        x: MIN_ANGLE + column,
        y: MAX_ANGLE - row,
    }
}

function pivotToIndex(point: Point): number | null {
    const { column, row } = pivotToGrid(point)
    if (column < 0 || column >= W || row < 0 || row >= H) return null
    return row * W + column
}

function clampAngle(value: number): number {
    return Math.min(MAX_ANGLE, Math.max(MIN_ANGLE, Math.round(value)))
}

function normalizeRoi(first: Point, second: Point): { min: Point; max: Point } {
    return {
        min: {
            x: clampAngle(Math.min(first.x, second.x)),
            y: clampAngle(Math.min(first.y, second.y)),
        },
        max: {
            x: clampAngle(Math.max(first.x, second.x)),
            y: clampAngle(Math.max(first.y, second.y)),
        },
    }
}

function drawRoiBoundary(
    context: CanvasRenderingContext2D,
    min: Point,
    max: Point,
) {
    context.fillStyle = roiBorder

    for (let x = min.x; x <= max.x; x++) {
        const top = pivotToGrid({ x, y: max.y })
        const bottom = pivotToGrid({ x, y: min.y })
        context.fillRect(top.column, top.row, 1, 1)
        context.fillRect(bottom.column, bottom.row, 1, 1)
    }

    for (let y = min.y; y <= max.y; y++) {
        const left = pivotToGrid({ x: min.x, y })
        const right = pivotToGrid({ x: max.x, y })
        context.fillRect(left.column, left.row, 1, 1)
        context.fillRect(right.column, right.row, 1, 1)
    }
}

export default function Grid({ setRoi, move_point, max, min, map }: GridProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null)
    const lastHoveredCell = useRef<string | null>(null)
    const [hoveredCell, setHoveredCell] = useState<HoveredCell | null>(null)
    const [selectedPoint, setSelectedPoint] = useState<{
        p1: SelectedPoint
        p2: SelectedPoint
    }>({
        p1: { point: min, selected: false },
        p2: { point: max, selected: false },
    })

    useEffect(() => {
        setSelectedPoint({
            p1: { point: min, selected: false },
            p2: { point: max, selected: false },
        })
    }, [min.x, min.y, max.x, max.y])

    const rangeByIndex = useMemo(() => {
        const ranges = new Float64Array(W * H)

        for (const point of map) {
            const index = pivotToIndex(point)
            if (index === null || point.distant <= 0) continue
            ranges[index] = point.distant
        }

        return ranges
    }, [map])

    const normalizedRoi = normalizeRoi(
        selectedPoint.p1.point,
        selectedPoint.p2.point,
    )
    const selectionIsIncomplete =
        selectedPoint.p1.selected !== selectedPoint.p2.selected

    useEffect(() => {
        const canvas = canvasRef.current
        const context = canvas?.getContext("2d")
        if (!canvas || !context) return

        context.fillStyle = DEFAULT_CELL_COLOUR
        context.fillRect(0, 0, W, H)

        for (let index = 0; index < rangeByIndex.length; index++) {
            const range = rangeByIndex[index]
            if (range <= 0) continue

            context.fillStyle = distanceToColour(range)
            context.fillRect(index % W, Math.floor(index / W), 1, 1)
        }

        if (!selectionIsIncomplete) {
            drawRoiBoundary(context, normalizedRoi.min, normalizedRoi.max)
        }
    }, [normalizedRoi.max.x, normalizedRoi.max.y, normalizedRoi.min.x, normalizedRoi.min.y, rangeByIndex, selectionIsIncomplete])

    function getCellFromPointer(event: React.MouseEvent<HTMLCanvasElement>) {
        const canvas = event.currentTarget
        const bounds = canvas.getBoundingClientRect()
        const canvasX = ((event.clientX - bounds.left) / bounds.width) * canvas.width
        const canvasY = ((event.clientY - bounds.top) / bounds.height) * canvas.height
        const column = Math.floor(canvasX)
        const row = Math.floor(canvasY)
        const isInside = column >= 0 && column < W && row >= 0 && row < H

        return {
            cell: isInside ? { column, row } : null,
            bounds,
        }
    }

    function handlePointerMove(event: React.PointerEvent<HTMLCanvasElement>) {
        const { cell, bounds } = getCellFromPointer(event)
        if (!cell) {
            handlePointerLeave()
            return
        }

        const cellKey = `${cell.column}:${cell.row}`
        if (cellKey === lastHoveredCell.current) return

        lastHoveredCell.current = cellKey
        setHoveredCell({
            ...cell,
            pivot: gridToPivot(cell.column, cell.row),
            left: event.clientX - bounds.left,
            top: event.clientY - bounds.top,
            containerWidth: bounds.width,
        })
    }

    function handlePointerLeave() {
        lastHoveredCell.current = null
        setHoveredCell(null)
    }

    function handleClick(event: React.MouseEvent<HTMLCanvasElement>) {
        const { cell } = getCellFromPointer(event)
        if (!cell) return

        const point = gridToPivot(cell.column, cell.row)
        setSelectedPoint((previous) => {
            if (!previous.p1.selected || previous.p2.selected) {
                return {
                    p1: { point, selected: true },
                    p2: { point, selected: false },
                }
            }

            return {
                ...previous,
                p2: { point, selected: true },
            }
        })
    }

    function resetSelection() {
        setSelectedPoint({
            p1: { point: min, selected: false },
            p2: { point: max, selected: false },
        })
    }

    const hoveredIndex = hoveredCell ? pivotToIndex(hoveredCell.pivot) : null
    const hoveredRange = hoveredIndex === null ? 0 : rangeByIndex[hoveredIndex]

    return (
        <div className="flex flex-1 flex-col items-center justify-center gap-3 p-2 sm:p-3">
            <div className="flex w-full flex-wrap items-center justify-center gap-2">
                <div className="flex flex-wrap items-center justify-center gap-2">
                    <div className="flex flex-row items-center gap-1.5">
                        <p>Point 1</p>
                        <Badge variant={selectedPoint.p1.selected ? "secondary" : "outline"}>
                            ({selectedPoint.p1.point.x}, {selectedPoint.p1.point.y})
                        </Badge>
                    </div>

                    <div className="flex flex-row items-center gap-1.5">
                        <p>Point 2</p>
                        <Badge variant={selectedPoint.p2.selected ? "secondary" : "outline"}>
                            ({selectedPoint.p2.point.x}, {selectedPoint.p2.point.y})
                        </Badge>
                    </div>
                </div>

                <div className="flex flex-row gap-3 p-1">
                    <Button onClick={resetSelection}>Clear</Button>
                    <Button
                        disabled={selectionIsIncomplete}
                        onClick={() => setRoi(normalizedRoi.min, normalizedRoi.max)}
                    >
                        Set ROI
                    </Button>
                    <Button onClick={() => move_point(selectedPoint.p1.point)}>
                        Move to point
                    </Button>
                </div>
            </div>

            <div className="relative aspect-square w-full max-w-190 overflow-hidden rounded-md border bg-muted shadow-sm">
                <canvas
                    ref={canvasRef}
                    width={W}
                    height={H}
                    aria-label={`${W} by ${H} LiDAR grid`}
                    className="block size-full cursor-crosshair [image-rendering:pixelated]"
                    onClick={handleClick}
                    onPointerLeave={handlePointerLeave}
                    onPointerMove={handlePointerMove}
                />

                {hoveredCell && (
                    <>
                        <div
                            className="pointer-events-none absolute z-10 ring-1 ring-amber-400"
                            style={{
                                left: `${(hoveredCell.column * 100) / W}%`,
                                top: `${(hoveredCell.row * 100) / H}%`,
                                width: `${100 / W}%`,
                                height: `${100 / H}%`,
                            }}
                        />
                        <div
                            className="pointer-events-none absolute z-20 flex flex-col gap-1.5 rounded-md bg-popover px-2 py-1 text-xs text-popover-foreground shadow-md ring-1 ring-foreground/10"
                            style={{
                                left: Math.max(
                                    4,
                                    Math.min(hoveredCell.left + 12, hoveredCell.containerWidth - 90),
                                ),
                                top: Math.max(hoveredCell.top - 52, 4),
                            }}
                        >
                            <Badge>
                                ({hoveredCell.pivot.x}, {hoveredCell.pivot.y})
                            </Badge>
                            <Badge variant="outline">
                                {hoveredRange > 0 ? `${hoveredRange} mm` : "No reading"}
                            </Badge>
                        </div>
                    </>
                )}
            </div>
        </div>
    )
}
