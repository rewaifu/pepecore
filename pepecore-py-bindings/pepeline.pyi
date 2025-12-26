from collections.abc import Sequence, Union
from enum import Enum, IntEnum
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
    RGB2Bayer_BGGR = 14
    RGB2Bayer_RGGB = 15
    RGB2Bayer_GBRG = 16
    RGB2Bayer_GRBG = 17
    Bayer2RGB_RGGB = 18
    Bayer2RGB_BGGR = 19
    Bayer2RGB_GRBG = 20
    Bayer2RGB_GBRG = 21
    def __reduce__(self): ...

class TypeNoise(Enum):
    PERLIN = 0
    OPENSIMPLEX2 = 1
    SUPERSIMPLEX2S = 3
    CELLULAR = 4
    VALUECUBIC = 5
    VALUE = 6

    def __reduce__(self): ...

class ResizesFilter(Enum):
    Box = 0
    Bilinear = 1
    Hamming = 2
    CatmullRom = 3
    Mitchell = 4
    Gaussian = 5
    Lanczos3 = 6

    def __reduce__(self): ...

class ResizesFilter(Enum):
    Box = 0
    Bilinear = 1
    Hamming = 2
    CatmullRom = 3
    Mitchell = 4
    Gaussian = 5
    Lanczos3 = 6

    def __reduce__(self): ...

class ResizesAlg:
    @staticmethod
    def Nearest() -> ResizesAlg: ...
    @staticmethod
    def Conv(filter: ResizesFilter) -> ResizesAlg: ...
    @staticmethod
    def Interpolation(filter: ResizesFilter) -> ResizesAlg: ...
    @staticmethod
    def SuperSampling(filter: ResizesFilter, passes: int) -> ResizesAlg: ...

def read(path: str | Path, color_mode: ImgColor = ..., img_format: ImgFormat = ...) -> np.ndarray: ...
def buff_read(buffer: Union[bytes, bytearray, memoryview], color_mode: ImgColor = ..., img_format: ImgFormat = ...) -> np.ndarray: ...
def save(img: np.ndarray, path: str | Path): ...
def cvt_color(img: np.ndarray, cvt_mode: CVTColor): ...
def crop(img: np.ndarray, x: int, y: int, w: int, h: int) -> np.ndarray: ...
def color_levels(
    img: np.ndarray, in_low: int | None = 0, in_high: int | None = 255, out_low: int | None = 0, out_high: int | None = 255, gamma: float | None = 1.0
) -> np.ndarray: ...
def screentone(
    img: np.ndarray, dot_size: int, angle: int | None = 0, dot_type: DotType | None = ..., scale: float | None = None, resize_alg: ResizesAlg = ...
) -> np.ndarray: ...
def halftone(
    img: np.ndarray,
    dot_sizes: Sequence[int],
    angles: Sequence[float] | None = None,
    dot_types: Sequence[DotType] | None = None,
    scale: float | None = None,
    resize_alg: ResizesAlg = ...,
) -> np.ndarray: ...
def best_tile(img: np.ndarray, tile_size: int) -> tuple[int, int] : ...


def noise(
    shape: tuple[int, int] | tuple[int, int, int],
    octaves: int,
    amplitudes: Sequence[float],
    frequency: Sequence[float],
    noise_type: Sequence[TypeNoise]
) -> np.ndarray: ...

class JpegSamplingFactor(IntEnum):
    R444 = 0
    R440 = 1
    R441 = 2
    R422 = 3
    R420 = 4
    R411 = 5
    R410 = 6
class QuantizeTable(IntEnum):
        Default = 0
        Flat = 1
        CustomMsSsim = 2
        CustomPsnrHvs = 3
        ImageMagick = 4
        KleinSilversteinCarney = 5
        DentalXRays = 6
        VisualDetectionModel = 7
        ImprovedDetectionModel = 8
class PaletteAlg(IntEnum):
    OcTree=0
    MedianCut=1
    Wu=2
    MinMaxUniform=3
def jpeg_encode(img: np.ndarray, quality: int = 100, qt: QuantizeTable=..., sampling_factor: JpegSamplingFactor = ...) -> np.ndarray: ...
def resize(img: np.ndarray, h: int, w: int, resize_alg: ResizesAlg = ..., alpha: bool = True) -> np.ndarray: ...
def rayon_mode(on:bool = True) -> None:...
def normalize(img: np.ndarray, scale: float) -> np.ndarray: ...
def real_hw(img:np.ndarray)->tuple[int, int] :...
def real_h(img:np.ndarray)->int:...
def real_w(img:np.ndarray)->int:...

def get_palette(img:np.ndarray,num_ch:int,alg:PaletteAlg = PaletteAlg.OcTree)->np.ndarray:...
class Point:
    def __init__(self,x:int,y:int,size:int):...
class Bresenham:
    def __init__(self,p0:Point,p1:Point):...
class Bezier:
    def __init__(self,p0:Point,p1:Point,p2:Point,p3:Point,step:float):...

def line(lines:Sequence[Bresenham|Bezier],h:int,w:int)
def read_tiler(path: str | Path, color_mode: ImgColor = ..., img_format: ImgFormat = ...,tile_size: usize):...
__all__ = [
    'CVTColor',
    'DotType',
    'ImgColor',
    'ImgFormat',
    'ResizesAlg',
    'ResizesFilter',
    'TypeNoise',
    'best_tile',
    'buff_read',
    'color_levels',
    'crop',
    'cvt_color',
    'halftone',
    'jpeg_encode',
    'noise',
    'read',
    'resize',
    'save',
    'screentone',
    'rayon_mode',
    'normalize',
    'real_hw',
    'real_h',
    'real_w',
    'Bresenham',
    'Bezier',
    'Point',
    'line',
    'read_tiler',
    'get_palette',
    'PaletteAlg'
]
