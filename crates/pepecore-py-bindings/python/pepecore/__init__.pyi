from enum import Enum
import numpy as np
from pathlib import Path
from typing import Union


class ColorMode(Enum):
    GRAY = 0
    RGB = 1
    RGBA = 2
    GRAYA = 3
    DYNAMIC = 4


def read(path: Union[str | Path], color_mode: ColorMode = ColorMode.DYNAMIC) -> np.ndarray: ...


def save(img: np.ndarray, path: Union[str | Path]): ...


class ResizeInterpolation(Enum):
    NearestNeighbour = 0


def resize(img: np.ndarray, height: int, width: int, interpolation: ResizeInterpolation) -> np.ndarray: ...


__all__ = [
    'ColorMode',
    'ResizeInterpolation',
    'read',
    'resize'
]
