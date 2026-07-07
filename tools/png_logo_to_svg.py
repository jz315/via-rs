#!/usr/bin/env python3
"""Vectorize a flat logo PNG into a compact-ish filled-path SVG.

This is intentionally aimed at simple generated logo art, not photos. It removes
near-white backgrounds, clusters foreground colors, traces each cluster with
OpenCV contours, simplifies large contours aggressively enough to recover clean
logo edges, and writes SVG paths using even-odd fill rules.
"""

from __future__ import annotations

import argparse
import html
from dataclasses import dataclass
from pathlib import Path

import cv2
import numpy as np
from PIL import Image


@dataclass(frozen=True)
class SvgPath:
    color: tuple[int, int, int]
    area: float
    data: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Vectorize a flat logo PNG into an SVG.",
    )
    parser.add_argument("input", type=Path, help="input PNG")
    parser.add_argument("output", type=Path, help="output SVG")
    parser.add_argument(
        "--colors",
        type=int,
        default=4,
        help="maximum foreground color clusters to trace (default: 4)",
    )
    parser.add_argument(
        "--background",
        type=int,
        default=246,
        help="RGB threshold for near-white background removal (default: 246)",
    )
    parser.add_argument(
        "--alpha",
        type=int,
        default=16,
        help="alpha threshold below which pixels are background (default: 16)",
    )
    parser.add_argument(
        "--min-area",
        type=float,
        default=18.0,
        help="minimum contour area in source pixels (default: 18)",
    )
    parser.add_argument(
        "--simplify",
        type=float,
        default=0.85,
        help="base contour simplification epsilon in pixels (default: 0.85)",
    )
    parser.add_argument(
        "--adaptive-simplify",
        type=float,
        default=0.00045,
        help=(
            "extra simplification as a fraction of contour perimeter; "
            "higher values make large shapes more compact (default: 0.00045)"
        ),
    )
    parser.add_argument(
        "--no-straighten",
        action="store_true",
        help="disable the long-straight-edge cleanup pass",
    )
    parser.add_argument(
        "--straighten-epsilon",
        type=float,
        default=2.8,
        help="maximum pixel deviation when collapsing long runs into one line (default: 2.8)",
    )
    parser.add_argument(
        "--straighten-min-length",
        type=float,
        default=72.0,
        help="minimum chord length eligible for straight-line cleanup (default: 72)",
    )
    parser.add_argument(
        "--coordinate-step",
        type=float,
        default=0.25,
        help="snap output coordinates to this pixel step, 0 disables (default: 0.25)",
    )
    parser.add_argument(
        "--blur",
        type=int,
        default=1,
        help="median blur kernel for cluster masks; use odd values, 0 disables (default: 1)",
    )
    return parser.parse_args()


def foreground_mask(rgba: np.ndarray, background: int, alpha: int) -> np.ndarray:
    rgb = rgba[:, :, :3]
    a = rgba[:, :, 3]
    near_white = (
        (rgb[:, :, 0] >= background)
        & (rgb[:, :, 1] >= background)
        & (rgb[:, :, 2] >= background)
    )
    return (a > alpha) & ~near_white


def cluster_foreground(
    rgba: np.ndarray,
    mask: np.ndarray,
    colors: int,
) -> tuple[np.ndarray, list[tuple[int, int, int]]]:
    pixels = rgba[:, :, :3][mask].astype(np.float32)
    if pixels.size == 0:
        raise ValueError("input image has no foreground pixels after background removal")

    k = max(1, min(colors, len(pixels)))
    criteria = (
        cv2.TERM_CRITERIA_EPS + cv2.TERM_CRITERIA_MAX_ITER,
        60,
        0.4,
    )
    _, labels, centers = cv2.kmeans(
        pixels,
        k,
        None,
        criteria,
        6,
        cv2.KMEANS_PP_CENTERS,
    )

    label_image = np.full(mask.shape, -1, dtype=np.int32)
    label_image[mask] = labels.reshape(-1)
    palette = [tuple(int(round(v)) for v in center) for center in centers]
    return label_image, palette


def mask_for_label(label_image: np.ndarray, label: int, blur: int) -> np.ndarray:
    mask = (label_image == label).astype(np.uint8) * 255
    if blur and blur > 1:
        kernel = blur if blur % 2 == 1 else blur + 1
        mask = cv2.medianBlur(mask, kernel)
    return mask


def simplify_epsilon(contour: np.ndarray, simplify: float, adaptive_simplify: float) -> float:
    perimeter = cv2.arcLength(contour, closed=True)
    return max(0.0, simplify + perimeter * adaptive_simplify)


def quantize(value: float, step: float) -> float:
    if step <= 0:
        return value
    return round(value / step) * step


def format_coord(value: float) -> str:
    text = f"{value:.2f}".rstrip("0").rstrip(".")
    return "0" if text in {"", "-0"} else text


def distance(point_a: np.ndarray, point_b: np.ndarray) -> float:
    return float(np.linalg.norm(point_b - point_a))


def line_distances(points: np.ndarray, start: np.ndarray, end: np.ndarray) -> np.ndarray:
    line = end - start
    length = float(np.linalg.norm(line))
    if length == 0:
        return np.linalg.norm(points - start, axis=1)
    relative = points - start
    return np.abs(line[0] * relative[:, 1] - line[1] * relative[:, 0]) / length


def rotate_to_sharpest_corner(points: np.ndarray) -> np.ndarray:
    if len(points) < 4:
        return points

    best_index = 0
    best_cosine = 1.0
    for index, point in enumerate(points):
        previous_point = points[index - 1]
        next_point = points[(index + 1) % len(points)]
        previous_vector = previous_point - point
        next_vector = next_point - point
        denominator = float(np.linalg.norm(previous_vector) * np.linalg.norm(next_vector))
        if denominator == 0:
            continue
        cosine = float(np.dot(previous_vector, next_vector) / denominator)
        if cosine < best_cosine:
            best_cosine = cosine
            best_index = index

    return np.vstack([points[best_index:], points[:best_index]])


def simplify_open_points(
    points: np.ndarray,
    curve_epsilon: float,
    straight_epsilon: float,
    straight_min_length: float,
) -> list[np.ndarray]:
    if len(points) <= 2:
        return [points[0], points[-1]]

    start = points[0]
    end = points[-1]
    distances = line_distances(points, start, end)
    split_index = int(np.argmax(distances))
    max_distance = float(distances[split_index])
    chord_length = distance(start, end)

    can_collapse_curve = max_distance <= curve_epsilon
    can_collapse_straight = (
        chord_length >= straight_min_length and max_distance <= straight_epsilon
    )
    if can_collapse_curve or can_collapse_straight:
        return [start, end]

    left = simplify_open_points(
        points[: split_index + 1],
        curve_epsilon,
        straight_epsilon,
        straight_min_length,
    )
    right = simplify_open_points(
        points[split_index:],
        curve_epsilon,
        straight_epsilon,
        straight_min_length,
    )
    return left[:-1] + right


def straighten_long_lines(
    points: np.ndarray,
    curve_epsilon: float,
    straight_epsilon: float,
    straight_min_length: float,
) -> np.ndarray:
    if len(points) < 4 or straight_epsilon <= curve_epsilon:
        return points

    rotated = rotate_to_sharpest_corner(points.astype(np.float64))
    closed = np.vstack([rotated, rotated[0]])
    simplified = simplify_open_points(
        closed,
        curve_epsilon,
        straight_epsilon,
        straight_min_length,
    )
    if len(simplified) > 1 and np.array_equal(simplified[0], simplified[-1]):
        simplified.pop()
    return np.array(simplified, dtype=np.float64)


def dedupe_points(points: np.ndarray) -> list[tuple[float, float]]:
    deduped: list[tuple[float, float]] = []
    for x, y in points:
        point = (float(x), float(y))
        if not deduped or point != deduped[-1]:
            deduped.append(point)
    if len(deduped) > 1 and deduped[0] == deduped[-1]:
        deduped.pop()
    return deduped


def contour_path(
    contour: np.ndarray,
    simplify: float,
    adaptive_simplify: float,
    straighten: bool,
    straighten_epsilon: float,
    straighten_min_length: float,
    coordinate_step: float,
) -> str:
    epsilon = simplify_epsilon(contour, simplify, adaptive_simplify)
    approx = cv2.approxPolyDP(contour, epsilon, closed=True)
    points = approx.reshape(-1, 2)
    if straighten:
        points = straighten_long_lines(
            points,
            curve_epsilon=epsilon,
            straight_epsilon=max(epsilon, straighten_epsilon),
            straight_min_length=straighten_min_length,
        )
    if coordinate_step > 0:
        points = np.array(
            [[quantize(x, coordinate_step), quantize(y, coordinate_step)] for x, y in points],
            dtype=np.float64,
        )
    point_list = dedupe_points(points)
    if len(point_list) < 3 and epsilon > simplify:
        fallback = cv2.approxPolyDP(contour, simplify, closed=True).reshape(-1, 2)
        point_list = dedupe_points(fallback)
    if len(point_list) < 3:
        return ""
    parts = [f"M{format_coord(point_list[0][0])},{format_coord(point_list[0][1])}"]
    parts.extend(f"L{format_coord(x)},{format_coord(y)}" for x, y in point_list[1:])
    parts.append("Z")
    return " ".join(parts)


def trace_color(
    color: tuple[int, int, int],
    mask: np.ndarray,
    min_area: float,
    simplify: float,
    adaptive_simplify: float,
    straighten: bool,
    straighten_epsilon: float,
    straighten_min_length: float,
    coordinate_step: float,
) -> SvgPath | None:
    contours, _ = cv2.findContours(mask, cv2.RETR_TREE, cv2.CHAIN_APPROX_NONE)
    paths: list[str] = []
    total_area = 0.0

    for contour in contours:
        area = abs(cv2.contourArea(contour))
        if area < min_area:
            continue
        path = contour_path(
            contour,
            simplify,
            adaptive_simplify,
            straighten,
            straighten_epsilon,
            straighten_min_length,
            coordinate_step,
        )
        if not path:
            continue
        paths.append(path)
        total_area += area

    if not paths:
        return None
    return SvgPath(color=color, area=total_area, data=" ".join(paths))


def hex_color(color: tuple[int, int, int]) -> str:
    return "#{:02x}{:02x}{:02x}".format(*color)


def write_svg(
    output: Path,
    width: int,
    height: int,
    paths: list[SvgPath],
    title: str,
) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        (
            f'<svg xmlns="http://www.w3.org/2000/svg" '
            f'viewBox="0 0 {width} {height}" width="{width}" height="{height}" '
            f'role="img" aria-labelledby="title">'
        ),
        f"  <title>{html.escape(title)}</title>",
    ]
    for path in sorted(paths, key=lambda item: item.area, reverse=True):
        lines.append(
            f'  <path fill="{hex_color(path.color)}" fill-rule="evenodd" '
            f'd="{path.data}"/>'
        )
    lines.append("</svg>")
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    args = parse_args()
    image = Image.open(args.input).convert("RGBA")
    rgba = np.array(image)
    mask = foreground_mask(rgba, args.background, args.alpha)
    label_image, palette = cluster_foreground(rgba, mask, args.colors)

    paths: list[SvgPath] = []
    for label, color in enumerate(palette):
        traced = trace_color(
            color,
            mask_for_label(label_image, label, args.blur),
            args.min_area,
            args.simplify,
            args.adaptive_simplify,
            not args.no_straighten,
            args.straighten_epsilon,
            args.straighten_min_length,
            args.coordinate_step,
        )
        if traced is not None:
            paths.append(traced)

    if not paths:
        raise ValueError("no vector paths were generated")

    write_svg(args.output, image.width, image.height, paths, args.input.stem)
    print(f"wrote {args.output} ({len(paths)} color paths)")


if __name__ == "__main__":
    main()
