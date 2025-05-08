from enum import Enum
import numpy as np
from pathlib import Path
from typing import Union


class ImgColor(Enum):
    GRAY = 0
    RGB = 1
    RGBA = 2
    GRAYA = 3
    DYNAMIC = 4


class ImgFormat(Enum):
    F32 = 0
    U8 = 1
    U16 = 2
    DYNAMIC = 3


class CVTColor(Enum):
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


def read(path: Union[str | Path], color_mode: ImgColor = ImgColor.DYNAMIC,
         img_format: ImgFormat = ImgFormat.DYNAMIC) -> np.ndarray: ...


def save(img: np.ndarray, path: Union[str | Path]): ...


def cvt_color(img: np.ndarray, cvt_mode: CVTColor): ...


__all__ = [
    'ImgColor',
    'read',
    'save',
    'cvt_color',
    'CVTColor',
    'ImgFormat'
]
