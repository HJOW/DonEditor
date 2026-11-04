"""Don Editor 임시(placeholder) 아이콘과 로고를 생성한다.

실제 디자인이 준비되면 이 스크립트가 만드는 파일들을 직접 교체하면 된다.
생성 대상:
  - crates/gram/resources/windows/app-icon.ico      (stable 채널, 파란색)
  - crates/gram/resources/windows/app-icon-dev.ico  (dev 채널, 주황색)
  - assets/images/logo_dark.svg, logo_light.svg     (시작 화면 로고)

사용법 (저장소 루트에서): python script/make_placeholder_icon.py
필요: Pillow (pip install pillow)
"""
from pathlib import Path

from PIL import Image, ImageDraw

# 저장소 루트 (이 파일은 script/ 폴더에 있다)
ROOT = Path(__file__).resolve().parent.parent

# 채널별 배경색 (RGB)
STABLE_COLOR = (47, 111, 237)  # 파란색
DEV_COLOR = (230, 126, 34)  # 주황색
WHITE = (255, 255, 255)

# ICO 에 담을 크기 목록 (Windows 탐색기·작업 표시줄·알림 영역에서 쓰는 크기)
ICO_SIZES = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]

# 고해상도로 그린 뒤 줄여서 가장자리를 부드럽게 만든다
SUPERSAMPLE = 1024


def draw_icon(color):
    """둥근 사각형 배경 위에 흰색 'D' 글자를 그린 정사각형 이미지를 반환한다."""
    s = SUPERSAMPLE
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle((0, 0, s - 1, s - 1), radius=int(s * 0.22), fill=color + (255,))

    # 'D' 바깥 모양: 세로 막대 + 오른쪽 반원
    left, top, right, bottom = 0.30 * s, 0.24 * s, 0.52 * s, 0.76 * s
    radius = 0.26 * s
    cx, cy = right, 0.5 * s
    d.rectangle((left, top, right, bottom), fill=WHITE)
    d.pieslice((cx - radius, cy - radius, cx + radius, cy + radius), -90, 90, fill=WHITE)

    # 'D' 안쪽 구멍(배경색으로 덮는다)
    in_left, in_top, in_bottom = 0.40 * s, 0.34 * s, 0.66 * s
    in_radius = 0.16 * s
    d.rectangle((in_left, in_top, right, in_bottom), fill=color + (255,))
    d.pieslice((cx - in_radius, cy - in_radius, cx + in_radius, cy + in_radius), -90, 90, fill=color + (255,))
    return img


def save_ico(path, color):
    """여러 크기를 담은 .ico 파일을 저장한다."""
    base = draw_icon(color)
    path.parent.mkdir(parents=True, exist_ok=True)
    base.resize((256, 256), Image.LANCZOS).save(path, format="ICO", sizes=ICO_SIZES)


def svg_logo(color):
    """시작 화면에서 쓰는 SVG 로고 문자열을 만든다 (아이콘과 같은 도형)."""
    hex_color = "#%02X%02X%02X" % color
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">'
        f'<rect width="100" height="100" rx="22" fill="{hex_color}"/>'
        '<path fill="#FFFFFF" fill-rule="evenodd" '
        'd="M30 24H52A26 26 0 0 1 52 76H30Z M40 34V66H52A16 16 0 0 0 52 34Z"/>'
        "</svg>\n"
    )


def main():
    save_ico(ROOT / "crates/gram/resources/windows/app-icon.ico", STABLE_COLOR)
    save_ico(ROOT / "crates/gram/resources/windows/app-icon-dev.ico", DEV_COLOR)
    # 어두운/밝은 테마 모두에서 보이도록 같은 파란 로고를 사용한다
    for name in ("logo_dark.svg", "logo_light.svg"):  # 로고 파일을 순회하며 저장
        (ROOT / "assets/images" / name).write_text(svg_logo(STABLE_COLOR), encoding="utf-8")
    print("임시 아이콘·로고 생성 완료")


if __name__ == "__main__":
    main()
