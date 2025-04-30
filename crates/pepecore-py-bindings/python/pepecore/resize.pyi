from enum import Enum
import numpy as np

class ResizeInterpolation(Enum):
    NearestNeighbour = 0


def resize(img: np.ndarray, height: int, width: int, interpolation: ResizeInterpolation) -> np.ndarray: ...

__all__ = [
    'ResizeInterpolation',
    'resize'
]
