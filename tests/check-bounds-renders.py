from pathlib import Path
from PIL import Image
root=Path('test-output')
for name,scale in [('full',1),('half',2)]:
    image=Image.open(root/('bounds-'+name+'.jpg00000'))
    for point,want in [((72,128),(255,0,0)),((128,128),(255,255,255)),((182,128),(0,0,255)),((50,128),(0,0,0))]:
        p=image.getpixel((point[0]//scale,point[1]//scale))
        assert all(abs(p[i]-want[i])<12 for i in range(3)), (name,point,p,want)
    print('PASS',name,image.size,'expanded red and blue fringes, original centre and clear exterior')
