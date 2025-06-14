from collections.abc import Sequence
from enum import IntEnum
from pathlib import Path

import numpy as np

class ImgColor(IntEnum):
    GRAY = 0
    RGB = 1
    RGBA = 2
    GRAYA = 3
    DYNAMIC = 4

    def __reduce__(self): ...

class ImgFormat(IntEnum):
    F32 = 0
    U8 = 1
    U16 = 2
    DYNAMIC = 3

    def __reduce__(self): ...

class DotType(IntEnum):
    CIRCLE = 0
    CROSS = 1
    ELLIPSE = 2
    LINE = 3
    INVLINE = 4

    def __reduce__(self): ...

class CVTColor(IntEnum):
    RGB2Gray_2020 = 0
    RGB2Gray_601 = 1
    RGB2Gray_709 = 2
    RGB2YCbCR_2020 = 3
    RGB2YCbCR_601 = 4
    RGB2YCbCR_709 = 5
    YCbCR2RGB_2020 = 6
    YCbCR2RGB_601 = 7
    YCbCR2RGB_709 = 8
    RGB2CMYK = 9
    CMYK2RGB = 10
    RGB2BGR = 11
    BGR2RGB = 12
    Gray2RGB = 13

    def __reduce__(self): ...

class TypeNoise(Enum):
    PERLIN = (0,)
    SIMPLEX = (1,)
    OPENSIMPLEX = (2,)
    SUPERSIMPLEX = (3,)
    PERLINSURFLET = 4

    def __reduce__(self): ...

def read(path: str | Path, color_mode: ImgColor = ..., img_format: ImgFormat = ...) -> np.ndarray: ...
def save(img: np.ndarray, path: str | Path): ...
def cvt_color(img: np.ndarray, cvt_mode: CVTColor): ...
def crop(img: np.ndarray, x: int, y: int, w: int, h: int) -> np.ndarray: ...
def color_levels(
    img: np.ndarray,
    in_low: int | None = 0,
    in_high: int | None = 255,
    out_low: int | None = 0,
    out_high: int | None = 255,
    gamma: float | None = 1.0,
) -> np.ndarray: ...
def screentone(img: np.ndarray, dot_size: int, angle: int | None = 0, dot_type: DotType | None = ...) -> np.ndarray: ...
def halftone(
    img: np.ndarray,
    dot_sizes: Sequence[int],
    angles: Sequence[float] | None = None,
    dot_types: Sequence[DotType] | None = None,
) -> np.ndarray: ...
def best_tile(img: np.ndarray, tile_size: int) -> np.ndarray: ...
def noise_generate(
    size: tuple[int, int] | tuple[int, int, int],
    type_noise: TypeNoise,
    octaves: int,
    frequency: float,
    lacunarity: float,
    seed: Optional[int] = ...,
) -> np.ndarray: ...

__all__ = [
    "CVTColor",
    "DotType",
    "ImgColor",
    "ImgFormat",
    "TypeNoise",
    "best_tile",
    "color_levels",
    "crop",
    "cvt_color",
    "halftone",
    "noise_generate",
    "read",
    "save",
    "screentone",
]
