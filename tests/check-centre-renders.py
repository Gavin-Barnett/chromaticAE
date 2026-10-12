from pathlib import Path
from PIL import Image
root = Path(__file__).resolve().parents[1]
for mode, scale in [('full', 1), ('half', 2)]:
    path = root / f'test-output/centre-{mode}.jpg00000'
    im = Image.open(path).convert('RGB')
    assert im.size == (256//scale, 192//scale), (mode, im.size)
    def pixel(x, y):
        return im.getpixel((x//scale, y//scale))
    centre = pixel(128, 96)
    edge = pixel(8, 96)
    assert max(abs(a-b) for a,b in zip(centre, (0,255,0))) < 12, (mode, centre)
    assert max(abs(a-b) for a,b in zip(edge, (51,102,115))) < 12, (mode, edge)
    print(f'PASS {mode}: original green centre={centre}; recoloured outer edge={edge}')
